export type CommandResult = {
  exit_code: number;
  stdout: string;
  stderr: string;
  timed_out: boolean;
};

export type CommandOptions = {
  timeout_ms?: number;
  env?: Record<string, string>;
  signal?: AbortSignal;
};

export interface ContainerEngine {
  run(args: string[], options?: CommandOptions): Promise<CommandResult>;
}

export function createEngine(program: string): ContainerEngine {
  return {
    async run(args, options = {}) {
      options.signal?.throwIfAborted();
      const child = Bun.spawn([program, ...args], {
        env: { ...process.env, ...options.env },
        stdin: "ignore",
        stdout: "pipe",
        stderr: "pipe",
      });
      let timed_out = false;
      const abort = () => child.kill("SIGKILL");
      options.signal?.addEventListener("abort", abort, { once: true });
      const timer = setTimeout(() => {
        timed_out = true;
        child.kill("SIGKILL");
      }, options.timeout_ms ?? 30_000);
      try {
        const [exit_code, stdout, stderr] = await Promise.all([
          child.exited,
          new Response(child.stdout).text(),
          new Response(child.stderr).text(),
        ]);
        options.signal?.throwIfAborted();
        return { exit_code, stdout, stderr, timed_out };
      } finally {
        clearTimeout(timer);
        options.signal?.removeEventListener("abort", abort);
      }
    },
  };
}

export async function checked(engine: ContainerEngine, args: string[], options?: CommandOptions): Promise<string> {
  const result = await engine.run(args, options);
  if (result.timed_out || result.exit_code !== 0) {
    // Command arguments can refer to secrets; avoid echoing the full invocation.
    throw new Error(`container ${args[0]} ${result.timed_out ? "timed out" : `failed (${result.exit_code})`}: ${result.stderr.trim()}`);
  }
  return result.stdout;
}

// Docker and Podman both accept explicit empty env values. This also overrides
// Podman's inherited proxy settings without relying on its --http-proxy flag.
export const directNetworkEnv = [
  "HTTP_PROXY=", "HTTPS_PROXY=", "ALL_PROXY=", "http_proxy=", "https_proxy=", "all_proxy=",
  "NO_PROXY=localhost,127.0.0.1,::1", "no_proxy=localhost,127.0.0.1,::1",
].flatMap((value) => ["--env", value]);
