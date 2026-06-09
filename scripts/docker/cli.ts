import { mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const tmpRoot = "/tmp";
const engine = process.env.TOKN_CONTAINER_ENGINE ?? "podman";
const defaultTag = "ci";
const gatewayImageRepo = "tokn-gateway-cli";
const agentImage = "tokn-agent-runner:ci";
const gatewayArtifactName = "tokn-gateway-cli-image";
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
  tag: string;
  forwarded: string[];
};

type TaggedArgs = {
  tag: string;
  rest: string[];
};

type HarnessNames = {
  gatewayImage: string;
  gatewayContainer: string;
  networkName: string;
  projectName: string;
  routerStateVolume: string;
};

type UpArgs = {
  copyLocal: "none" | "config" | "accounts";
  forceCopyLocal: boolean;
  tag: string;
  port?: number;
  proxyPort?: number;
};

function usage(): never {
  console.error(`Usage:
  bun --cwd scripts docker load [--tag <tag>] <image.tar>
  bun --cwd scripts docker load --pr <number>
  bun --cwd scripts docker up [--tag <tag>] [--copy-local-config|--copy-local-accounts] [--force-copy-local] [--port <host-port>] [--proxy-port <host-port>]
  bun --cwd scripts docker agent [--tag <tag>] --agent codex|opencode|pi --mode api-route|proxy-switch|api-passthrough|proxy-passthrough [-- <args>]
  bun --cwd scripts docker down [--tag <tag>]
  bun --cwd scripts docker reset [--tag <tag>] --yes
  bun --cwd scripts docker status [--tag <tag>]
  bun --cwd scripts docker logs [--tag <tag>]
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

function gh(args: string[], options: RunOptions = {}): string {
  return run("gh", args, options);
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

function namesForTag(tag: string): HarnessNames {
  const projectName = `tokn-router-${tag}`;
  return {
    gatewayImage: `${gatewayImageRepo}:${tag}`,
    gatewayContainer: `${projectName}-gateway`,
    networkName: `${projectName}-net`,
    projectName,
    routerStateVolume: `${projectName}-router-state`,
  };
}

function ensureNetwork(names: HarnessNames): void {
  if (!resourceExists("network", names.networkName)) {
    container(["network", "create", names.networkName]);
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

function requireTag(raw: string): string {
  if (!/^[a-z0-9][a-z0-9_.-]*$/.test(raw)) {
    throw new Error("--tag must start with a lowercase letter or digit and contain only lowercase letters, digits, '.', '_', or '-'");
  }
  return raw;
}

function requirePositiveInteger(raw: string, label: string): number {
  if (!/^[1-9][0-9]*$/.test(raw)) {
    throw new Error(`${label} must be a positive integer`);
  }
  return Number(raw);
}

function requirePort(raw: string, label: string): number {
  const port = requirePositiveInteger(raw, label);
  if (port > 65535) {
    throw new Error(`${label} must be between 1 and 65535`);
  }
  return port;
}

function localRouterHome(): string {
  const home = process.env.HOME;
  if (!home) {
    throw new Error("HOME is not set; cannot resolve local ~/.tokn/router");
  }
  return join(home, ".tokn", "router");
}

function parseTaggedArgs(args: string[]): TaggedArgs {
  let tag = process.env.TOKN_TAG ?? defaultTag;
  const rest: string[] = [];
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--tag") {
      tag = requireTag(requireValue(args, i, "--tag"));
      i += 1;
    } else if (arg.startsWith("--tag=")) {
      tag = requireTag(arg.slice("--tag=".length));
    } else {
      rest.push(arg);
    }
  }
  return { tag: requireTag(tag), rest };
}

function parseUpArgs(args: string[]): UpArgs {
  const tagged = parseTaggedArgs(args);
  let copyLocal: UpArgs["copyLocal"] = "none";
  let forceCopyLocal = false;
  let port: number | undefined;
  let proxyPort: number | undefined;
  for (let i = 0; i < tagged.rest.length; i += 1) {
    const arg = tagged.rest[i];
    if (arg === "--port") {
      port = requirePort(requireValue(tagged.rest, i, "--port"), "--port");
      i += 1;
    } else if (arg.startsWith("--port=")) {
      port = requirePort(arg.slice("--port=".length), "--port");
    } else if (arg === "--proxy-port") {
      proxyPort = requirePort(requireValue(tagged.rest, i, "--proxy-port"), "--proxy-port");
      i += 1;
    } else if (arg.startsWith("--proxy-port=")) {
      proxyPort = requirePort(arg.slice("--proxy-port=".length), "--proxy-port");
    } else if (arg === "--copy-local-config") {
      if (copyLocal !== "none") {
        throw new Error("--copy-local-config and --copy-local-accounts are mutually exclusive");
      }
      copyLocal = "config";
    } else if (arg === "--copy-local-accounts") {
      if (copyLocal !== "none") {
        throw new Error("--copy-local-config and --copy-local-accounts are mutually exclusive");
      }
      copyLocal = "accounts";
    } else if (arg === "--force-copy-local") {
      forceCopyLocal = true;
    } else {
      throw new Error(`unknown up option: ${arg}`);
    }
  }
  if (forceCopyLocal && copyLocal === "none") {
    throw new Error("--force-copy-local requires --copy-local-config or --copy-local-accounts");
  }
  return { copyLocal, forceCopyLocal, tag: tagged.tag, port, proxyPort };
}

function parseAgentArgs(args: string[]): ParsedAgentArgs {
  const rawForwardedAt = args.indexOf("--");
  const rawOptionArgs = rawForwardedAt >= 0 ? args.slice(0, rawForwardedAt) : args;
  const tagged = parseTaggedArgs(rawOptionArgs);
  let agent = process.env.TOKN_AGENT ?? "codex";
  let mode = process.env.TOKN_MODE ?? "api-route";
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
  return { agent, mode, tag: tagged.tag, forwarded };
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

function loadTar(tarPath: string, targetImage: string): void {
  console.log(`Loading ${basename(tarPath)}...`);
  const output = container(["load", "-i", tarPath], { capture: true });
  process.stdout.write(output);
  const loadedRef = imageRefFromDockerLoad(output);
  if (loadedRef !== targetImage) {
    container(["tag", loadedRef, targetImage]);
    console.log(`Tagged ${loadedRef} as ${targetImage}`);
  }
}

function loadImage(args: string[]): void {
  const tagged = parseTaggedArgs(args);
  if (tagged.rest.length === 1) {
    loadTar(resolve(tagged.rest[0]), namesForTag(tagged.tag).gatewayImage);
    return;
  }
  if (tagged.rest.length === 2 && tagged.rest[0] === "--pr") {
    const prNumber = requirePositiveInteger(tagged.rest[1], "--pr");
    const targetTag = tagged.tag === defaultTag ? `pr-${prNumber}` : tagged.tag;
    loadPrImage(prNumber, namesForTag(targetTag).gatewayImage);
    return;
  }
  usage();
}

type PullRequestView = {
  headRefName: string;
};

type WorkflowRun = {
  databaseId: number;
  headBranch: string;
  status: string;
  conclusion: string;
  url: string;
};

function loadPrImage(prNumber: number, targetImage: string): void {
  const pr = JSON.parse(gh(["pr", "view", String(prNumber), "--json", "headRefName"], { capture: true })) as PullRequestView;
  if (!pr.headRefName) {
    throw new Error(`could not resolve head branch for PR #${prNumber}`);
  }

  const runs = JSON.parse(
    gh(
      [
        "run",
        "list",
        "--branch",
        pr.headRefName,
        "--status",
        "success",
        "--json",
        "databaseId,headBranch,status,conclusion,url",
        "--limit",
        "20",
      ],
      { capture: true },
    ),
  ) as WorkflowRun[];
  const run = runs.find((candidate) => candidate.conclusion === "success");
  if (!run) {
    throw new Error(`could not find a successful workflow run for PR #${prNumber} branch ${pr.headRefName}`);
  }

  const downloadDir = join(tmpRoot, `tokn-pr-${prNumber}-${run.databaseId}`);
  rmSync(downloadDir, { recursive: true, force: true });
  mkdirSync(downloadDir, { recursive: true });
  console.log(`Downloading ${gatewayArtifactName} from ${run.url}...`);
  gh(["run", "download", String(run.databaseId), "--name", gatewayArtifactName, "--dir", downloadDir]);
  const tarPath = findFirstFile(downloadDir, (path) => path.endsWith(".tar"));
  if (!tarPath) {
    throw new Error(`downloaded artifact did not contain a .tar file under ${downloadDir}`);
  }
  loadTar(tarPath, targetImage);
}

function findFirstFile(root: string, predicate: (path: string) => boolean): string | undefined {
  for (const entry of readdirSync(root)) {
    const path = join(root, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      const found = findFirstFile(path, predicate);
      if (found) return found;
    } else if (stat.isFile() && predicate(path)) {
      return path;
    }
  }
  return undefined;
}

function copyLocalRouterFiles(names: HarnessNames, parsed: UpArgs): void {
  if (parsed.copyLocal === "none") {
    return;
  }
  const localHome = localRouterHome();
  const files = parsed.copyLocal === "config" ? ["config.toml", "auth.yaml"] : ["auth.yaml"];
  const script = [
    "set -eu",
    "copied=0",
    ...files.flatMap((file) => [
      `if [ -f /src/${file} ]; then`,
      `  if [ -e /dst/${file} ] && [ "${parsed.forceCopyLocal ? "1" : "0"}" != "1" ]; then`,
      `    echo 'tokn-copy: target already exists: /dst/${file}' >&2`,
      "    echo 'tokn-copy: rerun with --force-copy-local to overwrite selected files' >&2",
      "    exit 1",
      "  fi",
      `  cp "/src/${file}" "/dst/${file}"`,
      "  copied=$((copied + 1))",
      `  echo 'tokn-copy: copied ${file}'`,
      "else",
      `  echo 'tokn-copy: skipped missing ${file}'`,
      "fi",
    ]),
    "if [ \"$copied\" -eq 0 ]; then",
    "  echo 'tokn-copy: no selected local files were found' >&2",
    "  exit 1",
    "fi",
  ].join("\n");

  container([
    "run",
    "--rm",
    "-v",
    `${localHome}:/src:ro`,
    "-v",
    `${names.routerStateVolume}:/dst`,
    "--entrypoint",
    "sh",
    names.gatewayImage,
    "-c",
    script,
  ]);
}

function up(args: string[] = []): void {
  const parsed = parseUpArgs(args);
  const names = namesForTag(parsed.tag);
  const loadHint = /^pr-[1-9][0-9]*$/.test(parsed.tag)
    ? `run \`bun --cwd scripts docker load --pr ${parsed.tag.slice("pr-".length)}\` first`
    : `run \`bun --cwd scripts docker load --tag ${parsed.tag} <image.tar>\` first`;
  ensureImage(names.gatewayImage, loadHint);
  ensureNetwork(names);
  if (resourceExists("container", names.gatewayContainer)) {
    container(["rm", "-f", names.gatewayContainer]);
  }
  copyLocalRouterFiles(names, parsed);
  const portArgs: string[] = [];
  if (parsed.port !== undefined) {
    portArgs.push("-p", `127.0.0.1:${parsed.port}:4141`);
  }
  if (parsed.proxyPort !== undefined) {
    portArgs.push("-p", `127.0.0.1:${parsed.proxyPort}:4142`);
  }
  container([
    "run",
    "-d",
    "--name",
    names.gatewayContainer,
    "--network",
    names.networkName,
    ...portArgs,
    "-v",
    `${names.routerStateVolume}:/root/.tokn/router`,
    names.gatewayImage,
    "serve",
    "--host",
    "0.0.0.0",
    "--with-proxy",
    "--insecure-allow-remote",
  ]);
  if (parsed.port !== undefined) {
    console.log(`API: http://127.0.0.1:${parsed.port}/v1`);
  }
  if (parsed.proxyPort !== undefined) {
    console.log(`Proxy: http://127.0.0.1:${parsed.proxyPort}`);
  }
}

function buildAgent(): void {
  container(["build", "-t", agentImage, "-f", resolve(scriptDir, "Dockerfile.agent"), scriptDir]);
}

function agent(args: string[]): void {
  const parsed = parseAgentArgs(args);
  const names = namesForTag(parsed.tag);
  ensureImage(agentImage, "run `bun --cwd scripts docker build-agent` first");
  if (!resourceExists("container", names.gatewayContainer)) {
    throw new Error(`gateway is not running; run \`bun --cwd scripts docker up --tag ${parsed.tag}\` first`);
  }
  ensureNetwork(names);
  run(engine, [
    "run",
    "--rm",
    ...(process.stdin.isTTY && process.stdout.isTTY ? ["-it"] : []),
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
  ]);
}

function down(args: string[] = []): void {
  const tagged = parseTaggedArgs(args);
  if (tagged.rest.length !== 0) usage();
  const names = namesForTag(tagged.tag);
  if (resourceExists("container", names.gatewayContainer)) {
    container(["rm", "-f", names.gatewayContainer]);
  }
  if (resourceExists("network", names.networkName)) {
    container(["network", "rm", names.networkName]);
  }
}

function reset(args: string[]): void {
  const tagged = parseTaggedArgs(args);
  if (tagged.rest.length !== 1 || tagged.rest[0] !== "--yes") {
    throw new Error("reset removes containers and volumes; pass --yes to confirm");
  }
  const names = namesForTag(tagged.tag);
  down(["--tag", tagged.tag]);
  if (resourceExists("volume", names.routerStateVolume)) {
    container(["volume", "rm", names.routerStateVolume]);
  }
}

function status(args: string[] = []): void {
  const tagged = parseTaggedArgs(args);
  if (tagged.rest.length !== 0) usage();
  const names = namesForTag(tagged.tag);
  container(["ps", "-a", "--filter", `name=${names.projectName}`]);
  container(["image", "ls", names.gatewayImage]);
  container(["image", "ls", agentImage]);
  container(["volume", "ls", "--filter", `name=${names.routerStateVolume}`]);
  container(["network", "ls", "--filter", `name=${names.networkName}`]);
}

function logs(args: string[] = []): void {
  const tagged = parseTaggedArgs(args);
  if (tagged.rest.length !== 0) usage();
  const names = namesForTag(tagged.tag);
  container(["logs", "-f", names.gatewayContainer]);
}

function main(): void {
  const [cmd, ...args] = process.argv.slice(2);
  try {
    switch (cmd) {
      case "load":
        loadImage(args);
        break;
      case "up":
        up(args);
        break;
      case "agent":
        agent(args);
        break;
      case "down":
        down(args);
        break;
      case "reset":
        reset(args);
        break;
      case "status":
        status(args);
        break;
      case "logs":
        logs(args);
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
