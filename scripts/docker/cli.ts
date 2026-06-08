import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectName = "tokn-router-pr";
const engine = process.env.TOKN_CONTAINER_ENGINE ?? "podman";
const gatewayImage = "tokn-gateway-cli:ci";
const agentImage = "tokn-agent-runner:ci";
const networkName = `${projectName}-net`;
const gatewayContainer = `${projectName}-gateway`;
const routerStateVolume = `${projectName}-router-state`;
const repoRoot = resolve(scriptDir, "../..");
const agents = new Set(["codex", "opencode", "pi"]);
const modes = new Set(["api-route", "proxy-switch", "api-passthrough", "proxy-passthrough"]);

type RunOptions = {
  capture?: boolean;
  env?: Record<string, string>;
};

type ParsedAgentArgs = {
  agent: string;
  mode: string;
  forwarded: string[];
};

function usage(): never {
  console.error(`Usage:
  bun --cwd scripts docker load <image.tar>
  bun --cwd scripts docker up
  bun --cwd scripts docker agent --agent codex|opencode|pi --mode api-route|proxy-switch|api-passthrough|proxy-passthrough [-- <args>]
  bun --cwd scripts docker down
  bun --cwd scripts docker reset --yes
  bun --cwd scripts docker status
  bun --cwd scripts docker logs
  bun --cwd scripts docker build-agent

Environment:
  TOKN_CONTAINER_ENGINE=podman|docker  (default: podman)`);
  process.exit(64);
}

function run(program: string, args: string[], options: RunOptions = {}): string {
  const env = options.env ? { ...process.env, ...options.env } : process.env;
  const proc = Bun.spawnSync({
    cmd: [program, ...args],
    env,
    stdout: options.capture ? "pipe" : "inherit",
    stderr: options.capture ? "pipe" : "inherit",
  });
  const stdout = proc.stdout ? new TextDecoder().decode(proc.stdout) : "";
  const stderr = proc.stderr ? new TextDecoder().decode(proc.stderr) : "";
  if (!proc.success) {
    if (options.capture) {
      if (stdout.trim()) console.error(stdout.trimEnd());
      if (stderr.trim()) console.error(stderr.trimEnd());
    }
    const rendered = [program, ...args].join(" ");
    throw new Error(`command failed (${proc.exitCode}): ${rendered}`);
  }
  return stdout;
}

function container(args: string[], options: RunOptions = {}): string {
  return run(engine, args, options);
}

function containerOk(args: string[]): boolean {
  const proc = Bun.spawnSync({
    cmd: [engine, ...args],
    stdout: "ignore",
    stderr: "ignore",
  });
  return proc.success;
}

function resourceExists(kind: "container" | "image" | "network" | "volume", name: string): boolean {
  return containerOk([kind, "inspect", name]);
}

function ensureNetwork(): void {
  if (!resourceExists("network", networkName)) {
    container(["network", "create", networkName]);
  }
}

function ensureImage(image: string, hint: string): void {
  if (!resourceExists("image", image)) {
    throw new Error(`${image} is not available; ${hint}`);
  }
}

function requireValue(args: string[], index: number, flag: string): string {
  const value = args[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

function parseAgentArgs(args: string[]): ParsedAgentArgs {
  let agent = process.env.TOKN_AGENT ?? "codex";
  let mode = process.env.TOKN_MODE ?? "api-route";
  let forwardedAt = args.indexOf("--");
  const optionArgs = forwardedAt >= 0 ? args.slice(0, forwardedAt) : args;
  const forwarded = forwardedAt >= 0 ? args.slice(forwardedAt + 1) : [];

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
  return { agent, mode, forwarded };
}

function imageRefFromDockerLoad(output: string): string {
  const lines = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  const loaded = lines
    .map((line) => line.match(/^Loaded image(?:\(s\))?(?: ID)?:\s+(.+)$/)?.[1])
    .filter((value): value is string => Boolean(value));
  const image = loaded.at(-1);
  if (!image) {
    throw new Error("could not find loaded image reference in docker load output");
  }
  return image;
}

function loadImage(args: string[]): void {
  if (args.length !== 1) usage();
  const tarPath = resolve(args[0]);
  console.log(`Loading ${basename(tarPath)}...`);
  const output = container(["load", "-i", tarPath], { capture: true });
  process.stdout.write(output);
  const loadedRef = imageRefFromDockerLoad(output);
  if (loadedRef !== gatewayImage) {
    container(["tag", loadedRef, gatewayImage]);
    console.log(`Tagged ${loadedRef} as ${gatewayImage}`);
  }
}

function up(): void {
  ensureImage(gatewayImage, "run `bun --cwd scripts docker load <image.tar>` first");
  ensureNetwork();
  if (resourceExists("container", gatewayContainer)) {
    container(["rm", "-f", gatewayContainer]);
  }
  container([
    "run",
    "-d",
    "--name",
    gatewayContainer,
    "--network",
    networkName,
    "-p",
    "127.0.0.1:4141:4141",
    "-p",
    "127.0.0.1:4142:4142",
    "-v",
    `${routerStateVolume}:/root/.tokn/router`,
    gatewayImage,
    "serve",
    "--host",
    "0.0.0.0",
    "--with-proxy",
    "--insecure-allow-remote",
  ]);
}

function buildAgent(): void {
  container(["build", "-t", agentImage, "-f", resolve(scriptDir, "Dockerfile.agent"), scriptDir]);
}

function agent(args: string[]): void {
  const parsed = parseAgentArgs(args);
  ensureImage(agentImage, "run `bun --cwd scripts docker build-agent` first");
  if (!resourceExists("container", gatewayContainer)) {
    throw new Error("gateway is not running; run `bun --cwd scripts docker up` first");
  }
  ensureNetwork();
  run(engine, [
    "run",
    "--rm",
    ...(process.stdin.isTTY && process.stdout.isTTY ? ["-it"] : []),
    "--network",
    networkName,
    "-e",
    `TOKN_AGENT=${parsed.agent}`,
    "-e",
    `TOKN_MODE=${parsed.mode}`,
    "-e",
    "TOKN_GATEWAY_API_URL=http://tokn-router-pr-gateway:4141",
    "-e",
    "TOKN_GATEWAY_PROXY_URL=http://tokn-router-pr-gateway:4142",
    "-v",
    `${repoRoot}:/workspace`,
    "-w",
    "/workspace",
    agentImage,
    ...parsed.forwarded,
  ]);
}

function down(): void {
  if (resourceExists("container", gatewayContainer)) {
    container(["rm", "-f", gatewayContainer]);
  }
  if (resourceExists("network", networkName)) {
    container(["network", "rm", networkName]);
  }
}

function reset(args: string[]): void {
  if (args.length !== 1 || args[0] !== "--yes") {
    throw new Error("reset removes containers and volumes; pass --yes to confirm");
  }
  down();
  if (resourceExists("volume", routerStateVolume)) {
    container(["volume", "rm", routerStateVolume]);
  }
}

function status(): void {
  container(["ps", "-a", "--filter", `name=${projectName}`]);
  container(["image", "ls", gatewayImage]);
  container(["image", "ls", agentImage]);
  container(["volume", "ls", "--filter", `name=${routerStateVolume}`]);
  container(["network", "ls", "--filter", `name=${networkName}`]);
}

function logs(): void {
  container(["logs", "-f", gatewayContainer]);
}

function main(): void {
  const [cmd, ...args] = process.argv.slice(2);
  try {
    switch (cmd) {
      case "load":
        loadImage(args);
        break;
      case "up":
        if (args.length !== 0) usage();
        up();
        break;
      case "agent":
        agent(args);
        break;
      case "down":
        if (args.length !== 0) usage();
        down();
        break;
      case "reset":
        reset(args);
        break;
      case "status":
        if (args.length !== 0) usage();
        status();
        break;
      case "logs":
        if (args.length !== 0) usage();
        logs();
        break;
      case "build-agent":
        if (args.length !== 0) usage();
        buildAgent();
        break;
      default:
        usage();
    }
  } catch (err) {
    console.error(err instanceof Error ? err.message : String(err));
    process.exit(1);
  }
}

main();
