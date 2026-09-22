import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { randomUUID } from "node:crypto";

import { engine as engine_program, repoRoot } from "../runtime";
import { resolveAgent } from "./agents";
import { selectCases } from "./cases";
import { checked, createEngine } from "./engine";
import { runSuite } from "./runner";
import { loadSuite } from "./suite";

const help = `Usage:
  bun --cwd scripts docker agent-test build-agent --agent <opencode|pi|dsh>
  bun --cwd scripts docker agent-test run --suite <suite.json> [--case <id>]... [--output <new-directory>]

TOKN_CONTAINER_ENGINE selects podman (default) or docker.
Runs retain their private volume and write report.json plus closed databases.
See scripts/docker/agent-tests/README.md for the separate history import step.`;

export async function agentTest(args: string[]): Promise<void> {
  if (args.length === 0 || args.includes("--help")) { console.log(help); return; }
  const [command, ...rest] = args;
  const fields = new Map<string, string[]>();
  for (let index = 0; index < rest.length; index += 2) {
    const flag = rest[index];
    const value = rest[index + 1];
    const allowed = command === "run" ? ["--suite", "--case", "--output"] : command === "build-agent" ? ["--agent"] : [];
    if (!flag || !allowed.includes(flag) || !value || value.startsWith("--")) throw new Error(help);
    if (flag !== "--case" && fields.has(flag)) throw new Error(`Duplicate option: ${flag}`);
    fields.set(flag, [...(fields.get(flag) ?? []), value]);
  }
  const engine = createEngine(engine_program);
  if (command === "build-agent") {
    const adapter = resolveAgent(fields.get("--agent")?.[0] ?? "opencode");
    console.log(`Building ${adapter.image}`);
    await checked(engine, ["build", "--tag", adapter.image, "--file", resolve(repoRoot, adapter.dockerfile), repoRoot], { timeout_ms: 600_000 });
    return;
  }
  const suite_path = fields.get("--suite")?.[0];
  if (command !== "run" || !suite_path) throw new Error(help);
  const suite = loadSuite(suite_path);
  suite.cases = selectCases(suite.cases, fields.get("--case") ?? []);
  const root = resolve(repoRoot, "tmp/docker-agent-tests");
  const requested_output = fields.get("--output")?.[0];
  if (!requested_output) mkdirSync(root, { recursive: true, mode: 0o700 });
  const output_dir = resolve(requested_output ?? resolve(root, `${Date.now()}-${randomUUID().slice(0, 8)}`));
  const controller = new AbortController();
  const cancel = () => controller.abort(new Error("Agent test interrupted; cleaning up containers"));
  process.on("SIGINT", cancel);
  process.on("SIGTERM", cancel);
  try {
    const report = await runSuite(engine, suite, { output_dir, signal: controller.signal, on_update: console.log });
    console.log(`Report: ${output_dir}/report.json`);
    console.log(`Private volume retained: ${report.volume_name}`);
    for (const error of report.errors) console.error(error);
    if (!report.success) process.exitCode = 1;
  } finally {
    process.off("SIGINT", cancel);
    process.off("SIGTERM", cancel);
  }
}
