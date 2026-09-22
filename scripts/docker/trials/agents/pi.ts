import { fixturePath, preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedTrial, TrialCase, TrialOutput, TrialResult, TrialToolCall } from "./types";

function prepare(trial: TrialCase, options: { router_url: string; marker?: string }): PreparedTrial {
  if (trial.mode !== "api") throw new Error(`Pi does not support trial mode '${trial.mode}'`);
  const { expected_text: expectedText, prompt, fixture } = preparePrompt(trial, options.marker);
  const model = trial.upstream_model ?? trial.model;
  const config = {
    providers: {
      tokn: {
        baseUrl: `${options.router_url.replace(/\/$/, "")}${trial.base_path}`,
        api: trial.api === "responses" ? "openai-responses" : "openai-completions",
        apiKey: "$TOKN_TRIAL_API_KEY",
        models: [{
          id: model,
          name: trial.display_name ?? trial.model,
          reasoning: false,
          input: ["text"],
          contextWindow: 128_000,
          maxTokens: 16_384,
        }],
      },
    },
  };
  return {
    files: [
      { path: "models.json", content: `${JSON.stringify(config, null, 2)}\n` },
      { path: "prompt.txt", content: `${prompt}\n` },
      ...(fixture ? [fixture] : []),
    ],
    command: [
      "--mode", "json", "--no-session", "--no-approve", "--provider", "tokn", "--model", model,
      ...(trial.probe === "read_tool" ? ["--tools", "read"] : ["--no-tools"]),
      prompt,
    ],
    environment: { PI_CODING_AGENT_DIR: "/tmp/pi-agent", PI_OFFLINE: "1" },
    working_dir: "/workspace",
    expected_text: expectedText,
    ...(fixture ? { fixture_path: fixturePath } : {}),
  };
}

function record(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}

function string(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function contentText(value: unknown): string {
  if (typeof value === "string") return value;
  if (!Array.isArray(value)) return "";
  return value.map((item) => {
    const block = record(item);
    return block.type === "text" && typeof block.text === "string" ? block.text : "";
  }).join("");
}

function evaluate(trial: TrialCase, prepared: PreparedTrial, output: TrialOutput): TrialResult {
  let sessionHeader = false;
  let agentStarted = false;
  let agentEnded = false;
  let completedSteps = 0;
  let parseError: string | undefined;
  let agentError = false;
  let text = "";
  const sessionIds = new Set<string>();
  const toolCalls: TrialToolCall[] = [];
  const toolArgs = new Map<string, Record<string, unknown>>();

  for (const [index, line] of output.stdout.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event: Record<string, unknown>;
    try {
      event = record(JSON.parse(line));
      if (typeof event.type !== "string") throw new Error("missing event type");
    } catch {
      parseError ??= `Invalid Pi JSON event at line ${index + 1}`;
      continue;
    }
    if (event.type === "session") {
      sessionHeader = true;
      const id = string(event.id);
      if (id) sessionIds.add(id);
      else parseError ??= `Invalid Pi session event at line ${index + 1}`;
    } else if (event.type === "agent_start") {
      agentStarted = true;
    } else if (event.type === "agent_end") {
      agentEnded = true;
    } else if (event.type === "turn_end") {
      completedSteps += 1;
    } else if (event.type === "message_end") {
      const message = record(event.message);
      if (message.role === "assistant") {
        text = contentText(message.content).trim();
        if (message.stopReason === "error" || typeof message.errorMessage === "string") agentError = true;
      }
    } else if (event.type === "tool_execution_start") {
      const id = string(event.toolCallId);
      if (id) toolArgs.set(id, record(event.args));
    } else if (event.type === "tool_execution_end") {
      const id = string(event.toolCallId);
      const args = Object.keys(record(event.args)).length > 0 ? record(event.args) : toolArgs.get(id ?? "") ?? {};
      const result = record(event.result);
      toolCalls.push({
        name: string(event.toolName) ?? "unknown",
        status: event.isError === false ? "completed" : "error",
        ...(typeof args.path === "string" ? { file_path: args.path } : {}),
        ...(contentText(result.content) ? { output: contentText(result.content) } : {}),
      });
    }
  }

  let error: string | undefined;
  if (output.timed_out) error = "Pi trial timed out";
  else if (output.exit_code !== 0) error = `Pi exited with code ${output.exit_code ?? "unknown"}`;
  else if (parseError) error = parseError;
  else if (agentError) error = "Pi reported an assistant error; inspect stdout.jsonl for details";
  else if (!sessionHeader || !agentStarted || !agentEnded || completedSteps === 0) error = "Pi did not emit a complete agent session";
  else if (text !== prepared.expected_text) error = "Pi response did not exactly match the expected marker";
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

export const pi: AgentAdapter = {
  id: "pi",
  version: "0.85.1",
  image: "tokn-pi-trials:0.85.1",
  dockerfile: "scripts/docker/Dockerfile.pi",
  prepare,
  evaluate,
};
