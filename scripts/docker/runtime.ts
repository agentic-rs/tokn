import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const scriptDir = dirname(fileURLToPath(import.meta.url));
export const tmpRoot = "/tmp";
export const engine = process.env.TOKN_CONTAINER_ENGINE ?? "podman";
export const defaultTag = "ci";
export const gatewayImageRepo = "tokn-gateway-cli";
export const agentImage = "tokn-agent-runner:ci";
export const gatewayArtifactName = "tokn-gateway-cli-image";
export const repoRoot = resolve(scriptDir, "../..");

export type RunOptions = {
  capture?: boolean;
  env?: Record<string, string>;
  stdin?: "inherit" | "ignore";
};

export function run(program: string, args: string[], options: RunOptions = {}): string {
  const env = options.env ? { ...process.env, ...options.env } : process.env;
  const proc = Bun.spawnSync({
    cmd: [program, ...args],
    env,
    stdin: options.stdin ?? "ignore",
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

export async function runAttached(program: string, args: string[], options: RunOptions = {}): Promise<void> {
  const env = options.env ? { ...process.env, ...options.env } : process.env;
  const proc = Bun.spawn({
    cmd: [program, ...args],
    env,
    stdin: options.stdin ?? "ignore",
    stdout: "inherit",
    stderr: "inherit",
  });
  let forwardedSignalExitCode: number | undefined;
  const forwardSigint = () => {
    forwardedSignalExitCode = 130;
    proc.kill("SIGINT");
  };
  const forwardSigterm = () => {
    forwardedSignalExitCode = 143;
    proc.kill("SIGTERM");
  };
  process.on("SIGINT", forwardSigint);
  process.on("SIGTERM", forwardSigterm);
  const exitCode = await proc.exited;
  process.off("SIGINT", forwardSigint);
  process.off("SIGTERM", forwardSigterm);
  if (forwardedSignalExitCode !== undefined) {
    process.exit(forwardedSignalExitCode);
  }
  if (exitCode !== 0) {
    const rendered = [program, ...args].join(" ");
    throw new Error(`command failed (${exitCode}): ${rendered}`);
  }
}

export function gh(args: string[], options: RunOptions = {}): string {
  return run("gh", args, options);
}

export function container(args: string[], options: RunOptions = {}): string {
  return run(engine, args, options);
}

export function containerOk(args: string[]): boolean {
  const proc = Bun.spawnSync({
    cmd: [engine, ...args],
    stdout: "ignore",
    stderr: "ignore",
  });
  return proc.success;
}
