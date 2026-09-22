import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { defaultCases } from "./cases";
import type { CommandOptions, CommandResult, ContainerEngine } from "./engine";
import { runSuite } from "./runner";
import { parseSuite, type AgentTestSuite } from "./suite";

const private_key = "tokn_test_private_key_not_for_artifacts";
const marker = "TOKN_OFFLINE_PROBE_OK";
const completed_output = [
  { type: "text", sessionID: "session-offline", part: { text: marker } },
  { type: "step_finish", part: { reason: "stop" } },
].map((event) => JSON.stringify(event)).join("\n");

type FakeOptions = {
  agent_timeout?: boolean;
  agent_start_failure?: boolean;
  agent_wait_failure?: boolean;
  agent_wait_stdout?: string;
  network_create_failure?: boolean;
  gateway_exit_code?: number;
  shutdown_marker?: boolean;
  copy_failure?: boolean;
  cancel_on_wait?: AbortController;
  cancel_on_network_create?: AbortController;
};
type FakeState = { kind: "gateway" | "agent"; running: boolean; exit_code: number };

// The fake keeps resources alive until the runner actually removes/stops them.
// It also makes cp fail for a running gateway, matching the export contract.
class FakeEngine implements ContainerEngine {
  calls: { args: string[]; options: CommandOptions }[] = [];
  containers = new Map<string, FakeState>();
  volumes = new Set<string>();
  networks = new Set<string>();

  constructor(private behavior: FakeOptions = {}) {}

  async run(args: string[], options: CommandOptions = {}): Promise<CommandResult> {
    options.signal?.throwIfAborted();
    this.calls.push({ args: [...args], options: { ...options, env: options.env ? { ...options.env } : undefined } });
    const result = (stdout = "", exit_code = 0, stderr = "", timed_out = false): CommandResult => ({
      stdout, exit_code, stderr, timed_out,
    });
    const name = args.at(-1)!;
    if (args[0] === "image" && args[1] === "inspect") return result("{}\n");
    if (args[0] === "volume" && args[1] === "create") {
      this.volumes.add(name);
      return result(`${name}\n`);
    }
    if (args[0] === "network") {
      if (args[1] === "create") {
        this.networks.add(name);
        if (this.behavior.cancel_on_network_create) {
          this.behavior.cancel_on_network_create.abort(new Error("network creation interrupted"));
          options.signal?.throwIfAborted();
        }
        if (this.behavior.network_create_failure) return result("", 1, "network creation interrupted after allocation");
      }
      else if (args[1] === "rm") this.networks.delete(name);
      else throw new Error(`Unexpected network command: ${args[1]}`);
      return result();
    }
    if (args[0] === "run") {
      if (args.includes("api-key")) return result(`name: offline-agent-test\nkey: ${private_key}\n`);
      if (args.includes("--rm")) return result();
      const container_name = args[args.indexOf("--name") + 1];
      const kind = args.includes("serve") ? "gateway" : "agent";
      this.containers.set(container_name, { kind, running: true, exit_code: 0 });
      if (kind === "agent" && this.behavior.agent_start_failure) {
        return result("", 125, "agent creation failed after resource allocation");
      }
      return result(`${container_name}\n`);
    }
    if (args[0] === "wait") {
      const state = this.state(name);
      if (this.behavior.cancel_on_wait) {
        this.behavior.cancel_on_wait.abort(new Error("agent test cancelled"));
        options.signal?.throwIfAborted();
      }
      if (this.behavior.agent_wait_failure) return result("", 125, "wait failed");
      if (this.behavior.agent_timeout) return result("", 137, "", true);
      state.running = false;
      return result(this.behavior.agent_wait_stdout ?? `${state.exit_code}\n`);
    }
    if (args[0] === "inspect") {
      const state = this.state(name);
      return result(JSON.stringify({ Running: state.running, ExitCode: state.exit_code }));
    }
    if (args[0] === "logs") {
      const state = this.state(name);
      if (state.kind === "agent") return result(completed_output);
      const shutdown = !state.running && this.behavior.shutdown_marker !== false
        ? "\nshutdown persistence cleanup complete"
        : "";
      return result(`tokn-router listening on 127.0.0.1:4141${shutdown}\n`);
    }
    if (args[0] === "stop") {
      const state = this.state(name);
      state.running = false;
      state.exit_code = state.kind === "gateway" ? this.behavior.gateway_exit_code ?? 0 : 143;
      return result(`${name}\n`);
    }
    if (args[0] === "cp") {
      const separator = args[1].indexOf(":");
      const state = this.state(args[1].slice(0, separator));
      if (state.running) throw new Error("Attempted database export before shutdown");
      if (this.behavior.copy_failure && args[1].endsWith("/sessions.db")) {
        return result("", 1, "simulated database copy failure");
      }
      if (args[1].endsWith("/requests")) {
        mkdirSync(args[2]);
        writeFileSync(join(args[2], "2026-09-21.db"), "offline request database");
      } else {
        writeFileSync(args[2], "offline database");
      }
      return result();
    }
    if (args[0] === "rm") {
      this.state(name);
      this.containers.delete(name);
      return result();
    }
    throw new Error(`Unexpected fake engine command: ${args[0]}`);
  }

  private state(name: string): FakeState {
    const state = this.containers.get(name);
    if (!state) throw new Error(`Unknown fake container: ${name}`);
    return state;
  }
}

let directory: string;
let output_dir: string;
let suite: AgentTestSuite;

beforeEach(() => {
  directory = mkdtempSync(join(tmpdir(), "tokn-agent-test-runner-"));
  const host_dir = join(directory, "host-router");
  mkdirSync(host_dir);
  mkdirSync(join(host_dir, "config.d"));
  mkdirSync(join(host_dir, "auth.d"));
  writeFileSync(join(host_dir, "config.toml"), "schema_version = 2\n");
  writeFileSync(join(host_dir, "auth.yaml"), "version: 1\naccounts: []\n");
  writeFileSync(join(host_dir, "usage.db"), "host database must remain isolated");
  output_dir = join(directory, "capture");
  suite = parseSuite({
    schema_version: 1,
    gateway_image: "tokn-gateway-cli:offline-test",
    config_file: join(host_dir, "config.toml"),
    timeout_secs: 7,
    cases: [{ ...defaultCases[0], expected_text: marker }],
  }, directory);
});

afterEach(() => {
  rmSync(directory, { recursive: true, force: true });
});

function fileContents(path: string): string[] {
  return readdirSync(path, { withFileTypes: true }).flatMap((entry) => {
    const child = join(path, entry.name);
    return entry.isDirectory() ? fileContents(child) : [readFileSync(child, "utf8")];
  });
}

function expectCleaned(engine: FakeEngine): void {
  expect(engine.containers.size).toBe(0);
  expect(engine.networks.size).toBe(0);
  expect(engine.volumes.size).toBe(1);
}

describe("container agent-test runner", () => {
  test("isolates host state, exports after shutdown, and passes the key only via process environment", async () => {
    const engine = new FakeEngine();
    const report = await runSuite(engine, suite, { output_dir });
    expect(report.success).toBe(true);
    expect(report.export_complete).toBe(true);
    expect(report.cases[0].result.success).toBe(true);
    expect(report.gateway_exit_code).toBe(0);
    expectCleaned(engine);

    const gateway_runs = engine.calls.filter(({ args }) => args[0] === "run" && !args.includes("TOKN_AGENT_TEST_API_KEY"));
    const expected_sources = [suite.config_file, suite.auth_file, suite.config_dir!, suite.auth_dir!].map((path) => realpathSync(path));
    for (const { args } of gateway_runs) {
      const mounts = args.filter((_, index) => args[index - 1] === "--mount");
      expect(mounts.filter((mount) => mount.startsWith("type=volume,")))
        .toEqual([`type=volume,src=${report.volume_name},dst=/root/.tokn/router`]);
      const binds = mounts.filter((mount) => mount.startsWith("type=bind,"));
      expect(binds).toHaveLength(expected_sources.length);
      expect(binds.every((mount) => mount.endsWith(",readonly"))).toBe(true);
      expect(binds.map((mount) => /,src=([^,]+),/.exec(mount)![1]).sort()).toEqual([...expected_sources].sort());
    }
    const agent_run = engine.calls.find(({ args }) => args[0] === "run" && args.includes("TOKN_AGENT_TEST_API_KEY"))!;
    expect(agent_run.options.env).toEqual({ TOKN_AGENT_TEST_API_KEY: private_key });
    expect(agent_run.args[agent_run.args.indexOf("--network") + 1]).toBe(`container:${report.run_id}-gateway`);
    expect(agent_run.args).toContain(`/workspace:rw`);
    const all_args = engine.calls.flatMap(({ args }) => args);
    expect(all_args.some((arg) => ["-p", "-P", "--publish", "--publish-all", "--network=host"].includes(arg))).toBe(false);
    expect(JSON.stringify(all_args)).not.toContain(private_key);
    expect(JSON.stringify(report)).not.toContain(private_key);
    expect(fileContents(output_dir).every((content) => !content.includes(private_key))).toBe(true);
    expect(readFileSync(join(directory, "host-router", "usage.db"), "utf8")).toBe("host database must remain isolated");
    expect(existsSync(join(output_dir, "export", "requests", "2026-09-21.db"))).toBe(true);
    expect(statSync(join(output_dir, "export", "usage.db")).mode & 0o777).toBe(0o600);
    expect(statSync(join(output_dir, "export", "requests")).mode & 0o777).toBe(0o700);
    expect(JSON.parse(readFileSync(join(output_dir, "report.json"), "utf8"))).toEqual(report);
  });

  test("a bounded wait stops the actual agent container and records failure while exporting history", async () => {
    const engine = new FakeEngine({ agent_timeout: true });
    const report = await runSuite(engine, suite, { output_dir });
    expect(report.success).toBe(false);
    expect(report.export_complete).toBe(true);
    expect(report.cases[0].timed_out).toBe(true);
    expect(report.cases[0].exit_code).toBeNull();
    expect(report.cases[0].result.error).toContain("timed out");
    const wait = engine.calls.find(({ args }) => args[0] === "wait")!;
    expect(wait.options.timeout_ms).toBe(7000);
    const agent_stop = engine.calls.findIndex(({ args }) => args[0] === "stop" && args.at(-1) === wait.args[1]);
    const gateway_stop = engine.calls.findIndex(({ args }) => args[0] === "stop" && args.at(-1) === `${report.run_id}-gateway`);
    expect(agent_stop).toBeGreaterThan(-1);
    expect(agent_stop).toBeLessThan(gateway_stop);
    expect(engine.calls[gateway_stop].args.slice(0, 4)).toEqual(["stop", "--time", "40", `${report.run_id}-gateway`]);
    expect(engine.calls[gateway_stop].options.timeout_ms).toBeGreaterThanOrEqual(40_000);
    expectCleaned(engine);
  });

  test("withholds exports after nonzero shutdown or missing persistence confirmation", async () => {
    for (const behavior of [{ gateway_exit_code: 1 }, { shutdown_marker: false }]) {
      const engine = new FakeEngine(behavior);
      const run_output = join(directory, `capture-${behavior.gateway_exit_code ?? "missing-marker"}`);
      const report = await runSuite(engine, suite, { output_dir: run_output });
      expect(report.success).toBe(false);
      expect(report.export_complete).toBe(false);
      expect(report.errors.join("\n")).toContain("export withheld");
      expect(engine.calls.some(({ args }) => args[0] === "cp")).toBe(false);
      expect(existsSync(join(run_output, "gateway.log"))).toBe(true);
      expectCleaned(engine);
    }
  });

  test("a partial copy is marked incomplete and keeps the source volume for recovery", async () => {
    const engine = new FakeEngine({ copy_failure: true });
    const report = await runSuite(engine, suite, { output_dir });
    expect(report.success).toBe(false);
    expect(report.export_complete).toBe(false);
    expect(report.errors.join("\n")).toContain("simulated database copy failure");
    expect(existsSync(join(output_dir, "export", "usage.db"))).toBe(true);
    expect(existsSync(join(output_dir, "export", "sessions.db"))).toBe(false);
    expectCleaned(engine);
  });

  test("cancellation cleans up live agents and still gracefully captures gateway history", async () => {
    const cancellation = new AbortController();
    const engine = new FakeEngine({ cancel_on_wait: cancellation });
    const report = await runSuite(engine, suite, { output_dir, signal: cancellation.signal });
    expect(report.success).toBe(false);
    expect(report.export_complete).toBe(true);
    expect(report.errors).toContain("agent test cancelled");
    expect(report.cases).toHaveLength(0);
    expect(engine.calls.filter(({ args }) => args[0] === "rm")).toHaveLength(2);
    expectCleaned(engine);
  });

  test("agent start and wait failures clean up partially created resources", async () => {
    for (const behavior of [{ agent_start_failure: true }, { agent_wait_failure: true }]) {
      const engine = new FakeEngine(behavior);
      const report = await runSuite(engine, suite, {
        output_dir: join(directory, behavior.agent_start_failure ? "failed-start" : "failed-wait"),
      });
      expect(report.success).toBe(false);
      expect(report.errors).toHaveLength(1);
      expect(report.cases).toHaveLength(0);
      expect(report.export_complete).toBe(true);
      expectCleaned(engine);
    }
  });

  test("cleans up a network allocated just before create fails or is cancelled", async () => {
    const cancellation = new AbortController();
    const behaviors = [{ network_create_failure: true }, { cancel_on_network_create: cancellation }];
    for (const [index, behavior] of behaviors.entries()) {
      const engine = new FakeEngine(behavior);
      const report = await runSuite(engine, suite, {
        output_dir: join(directory, `network-failed-${index}`),
        signal: behavior.cancel_on_network_create?.signal,
      });
      expect(report.success).toBe(false);
      expect(report.export_complete).toBe(false);
      expect(report.errors.join("\n")).toContain("network creation interrupted");
      expect(engine.calls.some(({ args }) => args[0] === "run")).toBe(false);
      expectCleaned(engine);
    }
  });

  test("rejects missing or malformed wait status even when agent logs look successful", async () => {
    for (const [index, agent_wait_stdout] of ["", " \n", "0x0", "0.0", "0e0", "-1", "256"].entries()) {
      const engine = new FakeEngine({ agent_wait_stdout });
      const report = await runSuite(engine, suite, { output_dir: join(directory, `invalid-wait-${index}`) });
      expect(report.success).toBe(false);
      expect(report.cases).toHaveLength(0);
      expect(report.errors.join("\n")).toContain("Invalid container exit status");
      expect(report.export_complete).toBe(true);
      expectCleaned(engine);
    }
  });

  test("refuses unsupported cases and reused capture directories before creating resources", async () => {
    const engine = new FakeEngine();
    const unsupported = { ...suite, cases: [{ ...suite.cases[0], agent: "future-agent" }] };
    await expect(runSuite(engine, unsupported, { output_dir })).rejects.toThrow("Unsupported agent-test adapter");
    expect(engine.calls).toHaveLength(0);
    mkdirSync(output_dir);
    await expect(runSuite(engine, suite, { output_dir })).rejects.toThrow();
    expect(engine.calls.every(({ args }) => args[0] === "image" && args[1] === "inspect")).toBe(true);
    expect(engine.volumes.size).toBe(0);
    expect(engine.networks.size).toBe(0);
  });
});
