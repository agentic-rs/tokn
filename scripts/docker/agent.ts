import { parseTaggedArgs, requireValue } from "./args";
import { ensureImage, ensureNetwork, namesForTag, resourceExists } from "./containers";
import { agentImage, defaultTag, engine, repoRoot, runAttached } from "./runtime";

const agents = new Set(["codex", "opencode", "pi"]);
const modes = new Set(["api-route", "proxy-switch", "api-passthrough", "proxy-passthrough"]);

type ParsedAgentArgs = {
  agent: string;
  mode: string;
  noTty: boolean;
  tag: string;
  forwarded: string[];
};

export async function agent(args: string[]): Promise<void> {
  const parsed = parseAgentArgs(args);
  const names = namesForTag(parsed.tag);
  ensureImage(agentImage, "run `bun --cwd scripts docker build-agent` first");
  if (!resourceExists("container", names.gatewayContainer)) {
    throw new Error(`gateway is not running; run \`bun --cwd scripts docker up --tag ${parsed.tag}\` first`);
  }
  ensureNetwork(names);
  const interactive = !parsed.noTty && process.stdin.isTTY === true && process.stdout.isTTY === true;
  await runAttached(
    engine,
    [
      "run",
      "--rm",
      "--sig-proxy=true",
      ...(interactive ? ["--interactive", "--tty"] : []),
      "--network",
      names.networkName,
      "-e",
      `TOKN_AGENT=${parsed.agent}`,
      "-e",
      `TOKN_MODE=${parsed.mode}`,
      "-e",
      `TOKN_GATEWAY_API_URL=http://${names.gatewayContainer}:4141`,
      "-e",
      `TOKN_GATEWAY_PROXY_URL=http://${names.gatewayContainer}:4142`,
      "-v",
      `${repoRoot}:/workspace`,
      "-w",
      "/workspace",
      agentImage,
      ...parsed.forwarded,
    ],
    { stdin: interactive ? "inherit" : "ignore" },
  );
}

function parseAgentArgs(args: string[]): ParsedAgentArgs {
  const rawForwardedAt = args.indexOf("--");
  const rawOptionArgs = rawForwardedAt >= 0 ? args.slice(0, rawForwardedAt) : args;
  const tagged = parseTaggedArgs(rawOptionArgs, defaultTag);
  let agent = process.env.TOKN_AGENT ?? "codex";
  let mode = process.env.TOKN_MODE ?? "api-route";
  let noTty = false;
  const optionArgs = tagged.rest;
  const forwarded = rawForwardedAt >= 0 ? args.slice(rawForwardedAt + 1) : [];

  for (let i = 0; i < optionArgs.length; i += 1) {
    const arg = optionArgs[i];
    if (arg === "--agent") {
      agent = requireValue(optionArgs, i, "--agent");
      i += 1;
    } else if (arg.startsWith("--agent=")) {
      agent = arg.slice("--agent=".length);
    } else if (arg === "--mode") {
      mode = requireValue(optionArgs, i, "--mode");
      i += 1;
    } else if (arg.startsWith("--mode=")) {
      mode = arg.slice("--mode=".length);
    } else if (arg === "--no-tty") {
      noTty = true;
    } else {
      throw new Error(`unknown agent option: ${arg}`);
    }
  }

  if (!agents.has(agent)) {
    throw new Error(`unsupported agent '${agent}' (expected codex, opencode, or pi)`);
  }
  if (!modes.has(mode)) {
    throw new Error(
      `unsupported mode '${mode}' (expected api-route, proxy-switch, api-passthrough, or proxy-passthrough)`,
    );
  }
  return { agent, mode, noTty, tag: tagged.tag, forwarded };
}
