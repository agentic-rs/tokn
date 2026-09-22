import { chmodSync, mkdirSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { randomUUID } from "node:crypto";

import { resolveAgent, type AgentTestCase, type AgentTestResult } from "./agents";
import { checked, directNetworkEnv, type ContainerEngine } from "./engine";
import { resolveMode } from "./modes";
import { validateInputs, type AgentTestSuite } from "./suite";

const state_root = "/root/.tokn/router";
const config_path = `${state_root}/config.toml`;
const shutdown_marker = "shutdown persistence cleanup complete";

export type CaseReport = { test_case: AgentTestCase; result: AgentTestResult; exit_code: number | null; timed_out: boolean };
export type RunReport = {
  schema_version: 1;
  run_id: string;
  volume_name: string;
  gateway_image: string;
  started_at: string;
  finished_at?: string;
  success: boolean;
  export_complete: boolean;
  gateway_exit_code?: number;
  cases: CaseReport[];
  errors: string[];
};
export type RunOptions = {
  output_dir: string;
  signal?: AbortSignal;
  on_update?: (message: string) => void;
};

function writePrivate(path: string, content: string): void {
  writeFileSync(path, content, { mode: 0o600, flag: "wx" });
}

function protectTree(path: string): void {
  chmodSync(path, 0o700);
  for (const entry of readdirSync(path, { withFileTypes: true })) {
    const child = join(path, entry.name);
    if (entry.isDirectory()) protectTree(child);
    else if (entry.isFile()) chmodSync(child, 0o600);
    else throw new Error("Unexpected non-file in database export");
  }
}

function bind(source: string, target: string): string[] {
  return ["--mount", `type=bind,src=${realpathSync(source)},dst=${target},readonly`];
}

export function gatewayMounts(suite: AgentTestSuite, volume_name: string): string[] {
  return [
    "--mount", `type=volume,src=${volume_name},dst=${state_root}`,
    ...bind(suite.config_file, config_path),
    ...bind(suite.auth_file, `${state_root}/auth.yaml`),
    ...(suite.config_dir ? bind(suite.config_dir, `${state_root}/config.d`) : []),
    ...(suite.auth_dir ? bind(suite.auth_dir, `${state_root}/auth.d`) : []),
  ];
}

async function containerState(engine: ContainerEngine, name: string): Promise<{ Running: boolean; ExitCode: number }> {
  return JSON.parse(await checked(engine, ["inspect", "--format", "{{json .State}}", name]));
}

async function waitReady(engine: ContainerEngine, name: string, signal?: AbortSignal): Promise<void> {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    signal?.throwIfAborted();
    const logs = await engine.run(["logs", name], { signal });
    if (!(await containerState(engine, name)).Running) throw new Error("Gateway exited before becoming ready; see gateway logs");
    if (`${logs.stdout}\n${logs.stderr}`.includes("tokn-router listening")) return;
    await Bun.sleep(200);
  }
  throw new Error("Gateway startup timed out; see gateway logs");
}

export async function runSuite(engine: ContainerEngine, suite: AgentTestSuite, options: RunOptions): Promise<RunReport> {
  validateInputs(suite);
  const prepared = suite.cases.map((testCase) => {
    const adapter = resolveAgent(testCase.agent);
    const mode = resolveMode(testCase.mode);
    return { testCase, adapter, mode, input: adapter.prepare(testCase, { router_url: suite.router_url }) };
  });
  for (const agent of Object.keys(suite.agent_images)) resolveAgent(agent);
  const images = new Set([suite.gateway_image, ...prepared.map(({ adapter }) => suite.agent_images[adapter.id] ?? adapter.image)]);
  for (const image of images) await checked(engine, ["image", "inspect", image], { signal: options.signal });

  const output_dir = resolve(options.output_dir);
  if (output_dir.includes(",")) throw new Error("Output path cannot contain commas");
  mkdirSync(output_dir, { mode: 0o700 }); // Never reuse or overwrite a previous capture.
  const run_id = `tokn-agent-test-${randomUUID()}`;
  const gateway_name = `${run_id}-gateway`;
  const network_name = `${run_id}-net`;
  const volume_name = `${run_id}-state`;
  const report: RunReport = {
    schema_version: 1, run_id, volume_name, gateway_image: suite.gateway_image,
    started_at: new Date().toISOString(), success: false, export_complete: false, cases: [], errors: [],
  };
  let gateway_started = false;
  let network_attempted = false;
  const agents = new Set<string>();
  const note = options.on_update ?? (() => {});
  const error = (value: unknown) => report.errors.push(value instanceof Error ? value.message : String(value));
  const mounts = gatewayMounts(suite, volume_name);
  try {
    await checked(engine, ["volume", "create", volume_name], { signal: options.signal });
    // Creation can succeed in the engine before its client is interrupted.
    network_attempted = true;
    await checked(engine, ["network", "create", network_name], { signal: options.signal });
    await checked(engine, ["run", "--rm", "--network", "none", ...mounts, "--entrypoint", "/bin/sh", suite.gateway_image, "-c", `mkdir -p ${state_root}/requests`], { signal: options.signal });
    const key_output = await checked(engine, ["run", "--rm", "--network", "none", ...directNetworkEnv, ...mounts, suite.gateway_image, "--config", config_path, "api-key", "create", run_id], { signal: options.signal });
    const api_key = /^key: (\S+)$/m.exec(key_output)?.[1];
    if (!api_key) throw new Error("Unable to create a private agent-test API key");
    // Register before starting so cancellation during `run -d` still cleans up.
    gateway_started = true;
    await checked(engine, ["run", "-d", "--name", gateway_name, "--network", network_name, "--stop-timeout", "40", ...directNetworkEnv, "--env", "RUST_LOG=info", ...mounts, suite.gateway_image, "--config", config_path, "serve", ...suite.serve_args], { signal: options.signal });
    await waitReady(engine, gateway_name, options.signal);
    for (const { testCase, adapter, mode, input } of prepared) {
      options.signal?.throwIfAborted();
      note(`Running ${testCase.id}`);
      const case_dir = join(output_dir, testCase.id);
      mkdirSync(case_dir, { mode: 0o700 });
      for (const file of input.files) {
        if (!/^[a-zA-Z0-9_.-]+$/.test(file.path) || [".", ".."].includes(file.path)) throw new Error("Invalid adapter artifact path");
        writePrivate(join(case_dir, file.path), file.content);
      }
      const agent_name = `${run_id}-${testCase.id}`;
      agents.add(agent_name);
      await checked(engine, [
        "run", "-d", "--name", agent_name, ...mode.network_args(gateway_name), ...directNetworkEnv,
        ...Object.entries(input.environment).flatMap(([key, value]) => ["--env", `${key}=${value}`]),
        "--env", "TOKN_AGENT_TEST_API_KEY", ...bind(case_dir, "/agent-test"), "--tmpfs", "/workspace:rw", "--workdir", input.working_dir,
        suite.agent_images[adapter.id] ?? adapter.image, ...input.command,
      ], { signal: options.signal, env: { TOKN_AGENT_TEST_API_KEY: api_key } });
      const waited = await engine.run(["wait", agent_name], { timeout_ms: suite.timeout_secs * 1000, signal: options.signal });
      if (waited.timed_out) await checked(engine, ["stop", "--time", "5", agent_name]);
      else if (waited.exit_code !== 0) throw new Error(`Failed to wait for ${testCase.id}`);
      const exit_status = waited.stdout.trim();
      const exit_code = waited.timed_out ? null : Number(exit_status);
      if (exit_code !== null && (!/^[0-9]+$/.test(exit_status) || !Number.isSafeInteger(exit_code) || exit_code > 255)) {
        throw new Error(`Invalid container exit status for ${testCase.id}`);
      }
      const logs = await engine.run(["logs", agent_name]);
      if (logs.exit_code !== 0 || logs.timed_out) throw new Error(`Unable to collect logs for ${testCase.id}`);
      writePrivate(join(case_dir, "stdout.jsonl"), logs.stdout);
      writePrivate(join(case_dir, "stderr.log"), logs.stderr);
      const result = adapter.evaluate(testCase, input, { ...logs, exit_code, timed_out: waited.timed_out });
      report.cases.push({ test_case: testCase, result, exit_code, timed_out: waited.timed_out });
      note(`${testCase.id}: ${result.success ? "PASS" : `FAIL (${result.error})`}`);
      await checked(engine, ["rm", "-f", agent_name]);
      agents.delete(agent_name);
    }
  } catch (cause) {
    error(cause);
  } finally {
    for (const name of agents) {
      try { await checked(engine, ["rm", "-f", name]); } catch (cause) { error(cause); }
    }
    if (gateway_started) {
      try {
        note("Stopping gateway and exporting private databases");
        await checked(engine, ["stop", "--time", "40", gateway_name], { timeout_ms: 50_000 });
        const state = await containerState(engine, gateway_name);
        report.gateway_exit_code = state.ExitCode;
        const logs = await engine.run(["logs", gateway_name]);
        writePrivate(join(output_dir, "gateway.log"), `${logs.stdout}\n${logs.stderr}`);
        if (logs.exit_code !== 0 || logs.timed_out || state.Running || state.ExitCode !== 0 || !`${logs.stdout}\n${logs.stderr}`.includes(shutdown_marker)) {
          throw new Error("Gateway did not confirm clean persistence shutdown; private volume retained, export withheld");
        }
        const export_dir = join(output_dir, "export");
        mkdirSync(export_dir, { mode: 0o700 });
        for (const path of ["usage.db", "sessions.db", "requests"]) {
          await checked(engine, ["cp", `${gateway_name}:${state_root}/${path}`, join(export_dir, path)]);
        }
        protectTree(export_dir);
        report.export_complete = true;
      } catch (cause) { error(cause); }
      try { await checked(engine, ["rm", "-f", gateway_name]); } catch (cause) { error(cause); }
    }
    if (network_attempted) {
      try { await checked(engine, ["network", "rm", network_name]); } catch (cause) { error(cause); }
    }
    report.finished_at = new Date().toISOString();
    report.success = report.errors.length === 0 && report.export_complete && report.cases.length === prepared.length && report.cases.every(({ result }) => result.success);
    writePrivate(join(output_dir, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  }
  return report;
}
