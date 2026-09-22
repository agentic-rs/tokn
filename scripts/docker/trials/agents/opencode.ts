import { fixturePath, preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedTrial, TrialCase, TrialOutput, TrialResult, TrialToolCall } from "./types";

// OpenCode 1.18.10 uses "/" as the worktree in our empty non-Git workspace.
// Its read permission checks paths relative to that worktree, while the
// external_directory permission checks absolute directory globs.
const fixtureReadPattern = fixturePath.slice(1);

function prepare(trial: TrialCase, options: { router_url: string; marker?: string }): PreparedTrial {
  if (trial.mode !== "api") throw new Error(`OpenCode does not support trial mode '${trial.mode}'`);
  const { expected_text: expectedText, prompt, fixture } = preparePrompt(trial, options.marker);
  const readTool = trial.probe === "read_tool";
  const config = {
    $schema: "https://opencode.ai/config.json",
    autoupdate: false,
    share: "disabled",
    snapshot: false,
    plugin: [],
    mcp: {},
    enabled_providers: ["tokn"],
    permission: readTool
      ? { "*": "deny", read: { "*": "deny", [fixtureReadPattern]: "allow" }, external_directory: { "*": "deny", "/trial/*": "allow" } }
      : "deny",
    tools: readTool ? { "*": false, read: true } : { "*": false },
    provider: {
      tokn: {
        npm: trial.api === "responses" ? "@ai-sdk/openai" : "@ai-sdk/openai-compatible",
        name: "Tokn integration trial",
        options: {
          // These property names belong to the OpenCode provider schema.
          baseURL: `${options.router_url.replace(/\/$/, "")}${trial.base_path}`,
          apiKey: "{env:TOKN_TRIAL_API_KEY}",
        },
        models: {
          [trial.model]: {
            name: trial.display_name ?? trial.model,
            id: trial.upstream_model ?? trial.model,
            tool_call: readTool,
          },
        },
      },
    },
  };
  return {
    files: [
      { path: "opencode.json", content: `${JSON.stringify(config, null, 2)}\n` },
      { path: "prompt.txt", content: `${prompt}\n` },
      ...(fixture ? [fixture] : []),
    ],
    command: ["--pure", "run", "--format", "json", "--model", `tokn/${trial.model}`, "--title", trial.id, "--dir", "/workspace", prompt],
    environment: {
      XDG_CONFIG_HOME: "/tmp/opencode-home/config",
      XDG_DATA_HOME: "/tmp/opencode-home/data",
      XDG_CACHE_HOME: "/tmp/opencode-home/cache",
      XDG_STATE_HOME: "/tmp/opencode-home/state",
      OPENCODE_CONFIG: "/trial/opencode.json",
      OPENCODE_DISABLE_PROJECT_CONFIG: "1",
      OPENCODE_DISABLE_AUTOUPDATE: "1",
      OPENCODE_DISABLE_MODELS_FETCH: "1",
      OPENCODE_DISABLE_CLAUDE_CODE: "1",
      OPENCODE_DISABLE_EXTERNAL_SKILLS: "1",
      OPENCODE_DISABLE_DEFAULT_PLUGINS: "1",
    },
    working_dir: "/workspace",
    expected_text: expectedText,
    ...(readTool ? { fixture_path: fixturePath } : {}),
  };
}

function record(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

function string(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function evaluate(trial: TrialCase, prepared: PreparedTrial, output: TrialOutput): TrialResult {
  const texts: string[] = [];
  const toolCalls: TrialToolCall[] = [];
  const sessionIds = new Set<string>();
  let completedSteps = 0;
  let terminalStep = false;
  let parseError: string | undefined;
  let agentError = false;

  for (const [index, line] of output.stdout.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event: Record<string, unknown>;
    try {
      const parsed: unknown = JSON.parse(line);
      event = record(parsed);
      if (typeof event.type !== "string") throw new Error("missing event type");
    } catch {
      // Do not copy unknown output or provider errors into the normalized report.
      parseError ??= `Invalid OpenCode JSON event at line ${index + 1}`;
      continue;
    }
    const part = record(event.part);
    const sessionId = string(event.sessionID) ?? string(part.sessionID);
    if (sessionId) sessionIds.add(sessionId);
    if (event.type === "error") agentError = true;
    if (event.type === "text") {
      if (typeof part.text !== "string") parseError ??= `Invalid OpenCode text event at line ${index + 1}`;
      else texts.push(part.text);
    }
    if (event.type === "step_start") terminalStep = false;
    if (event.type === "step_finish") {
      completedSteps += 1;
      terminalStep = part.reason === "stop";
    }
    if (event.type === "tool_use") {
      const state = record(part.state);
      const input = record(state.input);
      toolCalls.push({
        name: string(part.tool) ?? "unknown",
        status: string(state.status) ?? "unknown",
        ...(typeof input.filePath === "string" ? { file_path: input.filePath } : {}),
        ...(typeof state.output === "string" ? { output: state.output } : {}),
      });
    }
  }

  const text = texts.join("").trim();
  let error: string | undefined;
  if (output.timed_out) error = "OpenCode trial timed out";
  else if (output.exit_code !== 0) error = `OpenCode exited with code ${output.exit_code ?? "unknown"}`;
  else if (agentError) error = "OpenCode reported an error event; inspect stdout.jsonl for details";
  else if (parseError) error = parseError;
  else if (!terminalStep) error = "OpenCode did not finish a terminal step with reason 'stop'";
  else if (text !== prepared.expected_text) error = "OpenCode response did not exactly match the expected marker";
  else if (trial.probe === "text" && toolCalls.length !== 0) error = "Text probe unexpectedly invoked a tool";
  else if (trial.probe === "read_tool") {
    const call = toolCalls[0];
    if (toolCalls.length !== 1 || call?.name !== "read" || call.status !== "completed") {
      error = "Read probe requires exactly one completed read tool call and no other tools";
    } else if (call.file_path !== prepared.fixture_path) {
      error = "Read probe did not read the expected fixture path";
    } else if (!call.output?.includes(`verification_token=${prepared.expected_text}`)) {
      error = "Read tool output did not contain the expected fixture token";
    }
  }
  return {
    success: error === undefined,
    text,
    tool_calls: toolCalls,
    completed_steps: completedSteps,
    session_ids: [...sessionIds],
    ...(error ? { error } : {}),
  };
}

export const opencode: AgentAdapter = {
  id: "opencode",
  version: "1.18.10",
  image: "tokn-opencode-trials:1.18.10",
  dockerfile: "scripts/docker/Dockerfile.opencode",
  prepare,
  evaluate,
};
