import { fixturePath, preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedAgentTest, AgentTestCase, AgentTestOutput, AgentTestResult, AgentTestToolCall } from "./types";

function prepare(testCase: AgentTestCase, options: { router_url: string; marker?: string }): PreparedAgentTest {
  if (testCase.mode !== "api") throw new Error(`Claude Code does not support agent-test mode '${testCase.mode}'`);
  if (testCase.api !== "messages") throw new Error("Claude Code agent tests require the Messages API");
  const { expected_text: expectedText, prompt, fixture } = preparePrompt(testCase, options.marker);
  const model = testCase.upstream_model ?? testCase.model;
  const readTool = testCase.probe === "read_tool";
  const apiBaseUrl = `${options.router_url.replace(/\/$/, "")}${testCase.base_path}`;
  return {
    files: [
      { path: "prompt.txt", content: `${prompt}\n` },
      ...(fixture ? [fixture] : []),
    ],
    command: [
      "--print", "--output-format", "stream-json", "--verbose", "--no-session-persistence", "--bare", "--restricted",
      "--model", model, "--permission-mode", "dontAsk", "--permission-prompts", "none", "--max-turns", "3",
      "--tools", readTool ? "Read" : "", ...(readTool ? ["--allowedTools", "Read"] : []), prompt,
    ],
    environment: {
      // Claude Code appends /v1/messages to this origin itself.
      ANTHROPIC_BASE_URL: apiBaseUrl.replace(/\/v1$/, ""),
      CLAUDE_CONFIG_DIR: "/tmp/claude-home",
      CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: "1",
      CLAUDE_CODE_SKIP_PROMPT_HISTORY: "1",
      DISABLE_TELEMETRY: "1",
    },
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

function blocks(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.map(record) : [];
}

function blockText(value: unknown): string {
  if (typeof value === "string") return value;
  return blocks(value).map((block) => block.type === "text" && typeof block.text === "string" ? block.text : "").join("");
}

function evaluate(testCase: AgentTestCase, prepared: PreparedAgentTest, output: AgentTestOutput): AgentTestResult {
  const toolCalls: AgentTestToolCall[] = [];
  const toolIndexes = new Map<string, number>();
  let sessionId: string | undefined;
  let initialized = false;
  let completedSteps = 0;
  let parseError: string | undefined;
  let resultSeen = false;
  let agentError = false;
  let text = "";

  for (const [index, line] of output.stdout.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event: Record<string, unknown>;
    try {
      event = record(JSON.parse(line));
      if (typeof event.type !== "string") throw new Error("missing event type");
    } catch {
      parseError ??= `Invalid Claude Code JSON event at line ${index + 1}`;
      continue;
    }
    const eventSessionId = string(event.session_id);
    const isInit = event.type === "system" && event.subtype === "init";
    if (isInit && !eventSessionId) {
      parseError ??= `Claude Code init event at line ${index + 1} is missing session_id`;
    }
    if (eventSessionId) {
      if (sessionId === undefined) sessionId = eventSessionId;
      else if (sessionId !== eventSessionId) {
        parseError ??= `Claude Code event at line ${index + 1} has a conflicting session_id`;
      }
    }
    if (isInit && eventSessionId) initialized = true;
    if (event.type === "assistant") {
      const message = record(event.message);
      for (const block of blocks(message.content)) {
        if (block.type !== "tool_use") continue;
        const id = string(block.id);
        const input = record(block.input);
        const toolCall: AgentTestToolCall = {
          name: string(block.name) ?? "unknown",
          status: "started",
          ...(typeof input.file_path === "string" ? { file_path: input.file_path } : {}),
        };
        if (id) toolIndexes.set(id, toolCalls.length);
        toolCalls.push(toolCall);
      }
    }
    if (event.type === "user") {
      const message = record(event.message);
      for (const block of blocks(message.content)) {
        if (block.type !== "tool_result") continue;
        const toolIndex = toolIndexes.get(string(block.tool_use_id) ?? "");
        if (toolIndex === undefined) {
          parseError ??= `Claude Code tool result at line ${index + 1} has no matching tool use`;
          continue;
        }
        toolCalls[toolIndex].status = block.is_error === true ? "error" : "completed";
        const content = blockText(block.content);
        if (content) toolCalls[toolIndex].output = content;
      }
    }
    if (event.type === "result") {
      resultSeen = true;
      agentError = event.is_error === true || event.subtype !== "success";
      text = string(event.result)?.trim() ?? "";
      if (typeof event.num_turns === "number" && Number.isInteger(event.num_turns)) completedSteps = event.num_turns;
    }
  }

  let error: string | undefined;
  if (output.timed_out) error = "Claude Code agent test timed out";
  else if (output.exit_code !== 0) error = `Claude Code exited with code ${output.exit_code ?? "unknown"}`;
  else if (parseError) error = parseError;
  else if (!initialized || !resultSeen || completedSteps === 0) error = "Claude Code did not emit a complete agent session";
  else if (agentError) error = "Claude Code reported an error result; inspect stdout.jsonl for details";
  else if (text !== prepared.expected_text) error = "Claude Code response did not exactly match the expected marker";
  else if (testCase.probe === "text" && toolCalls.length !== 0) error = "Text probe unexpectedly invoked a tool";
  else if (testCase.probe === "read_tool") {
    const call = toolCalls[0];
    if (toolCalls.length !== 1 || call?.name !== "Read" || call.status !== "completed") {
      error = "Read probe requires exactly one completed Read tool call and no other tools";
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
    session_ids: sessionId ? [sessionId] : [],
    ...(error ? { error } : {}),
  };
}

export const claudeCode: AgentAdapter = {
  id: "claude-code",
  version: "2.1.272",
  image: "tokn-claude-code-agent-test:2.1.272",
  dockerfile: "scripts/docker/Dockerfile.claude-code",
  prepare,
  evaluate,
};
