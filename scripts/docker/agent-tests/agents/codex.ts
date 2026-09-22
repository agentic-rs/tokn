import { fixturePath, preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedAgentTest, AgentTestCase, AgentTestOutput, AgentTestResult, AgentTestToolCall } from "./types";

function tomlString(value: string): string {
  return JSON.stringify(value);
}

function prepare(testCase: AgentTestCase, options: { router_url: string; marker?: string }): PreparedAgentTest {
  if (testCase.mode !== "api") throw new Error(`Codex does not support agent-test mode '${testCase.mode}'`);
  if (testCase.api !== "responses") throw new Error("Codex agent tests require the Responses API");
  const { expected_text: expectedText, prompt, fixture } = preparePrompt(testCase, options.marker);
  const model = testCase.upstream_model ?? testCase.model;
  const baseUrl = `${options.router_url.replace(/\/$/, "")}${testCase.base_path}`;
  const provider = `{ name = "Tokn integration test", base_url = ${tomlString(baseUrl)}, ` +
    'env_key = "TOKN_AGENT_TEST_API_KEY", wire_api = "responses" }';
  return {
    files: [
      { path: "prompt.txt", content: `${prompt}\n` },
      ...(fixture ? [fixture] : []),
    ],
    command: [
      "exec", "--json", "--skip-git-repo-check", "--ephemeral", "--ignore-user-config", "--ignore-rules",
      "--sandbox", "read-only", "--ask-for-approval", "never", "--cd", "/workspace", "--model", model,
      "--config", 'model_provider="tokn"', "--config", `model_providers.tokn=${provider}`, prompt,
    ],
    environment: { CODEX_HOME: "/tmp/codex-home" },
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

function evaluate(testCase: AgentTestCase, prepared: PreparedAgentTest, output: AgentTestOutput): AgentTestResult {
  const toolCalls: AgentTestToolCall[] = [];
  let threadId: string | undefined;
  let threadStarted = false;
  let turnStarted = false;
  let completedSteps = 0;
  let parseError: string | undefined;
  let agentError = false;
  let text = "";

  for (const [index, line] of output.stdout.split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    let event: Record<string, unknown>;
    try {
      event = record(JSON.parse(line));
      if (typeof event.type !== "string") throw new Error("missing event type");
    } catch {
      parseError ??= `Invalid Codex JSON event at line ${index + 1}`;
      continue;
    }
    const eventThreadId = string(event.thread_id);
    if (event.type === "thread.started" && !eventThreadId) {
      parseError ??= `Codex thread.started event at line ${index + 1} is missing thread_id`;
    }
    if (eventThreadId) {
      if (threadId === undefined) threadId = eventThreadId;
      else if (threadId !== eventThreadId) {
        parseError ??= `Codex event at line ${index + 1} has a conflicting thread_id`;
      }
    }
    if (event.type === "thread.started" && eventThreadId) threadStarted = true;
    if (event.type === "turn.started") turnStarted = true;
    if (event.type === "turn.completed") completedSteps += 1;
    if (event.type === "turn.failed" || event.type === "error") agentError = true;
    if (event.type !== "item.completed") continue;
    const item = record(event.item);
    if (item.type === "agent_message") {
      if (typeof item.text !== "string") parseError ??= `Invalid Codex agent message at line ${index + 1}`;
      else text = item.text.trim();
    }
    if (item.type === "command_execution") {
      const command = string(item.command) ?? "";
      toolCalls.push({
        name: "command_execution",
        status: string(item.status) ?? "unknown",
        ...(prepared.fixture_path && command.includes(prepared.fixture_path) ? { file_path: prepared.fixture_path } : {}),
        ...(typeof item.aggregated_output === "string" ? { output: item.aggregated_output } : {}),
      });
    } else if (["file_change", "mcp_tool_call", "web_search"].includes(string(item.type) ?? "")) {
      toolCalls.push({ name: string(item.type) ?? "unknown", status: string(item.status) ?? "unknown" });
    }
  }

  let error: string | undefined;
  if (output.timed_out) error = "Codex agent test timed out";
  else if (output.exit_code !== 0) error = `Codex exited with code ${output.exit_code ?? "unknown"}`;
  else if (agentError) error = "Codex reported an error event; inspect stdout.jsonl for details";
  else if (parseError) error = parseError;
  else if (!threadStarted || !turnStarted || completedSteps !== 1) error = "Codex did not emit one complete turn";
  else if (text !== prepared.expected_text) error = "Codex response did not exactly match the expected marker";
  else if (testCase.probe === "text" && toolCalls.length !== 0) error = "Text probe unexpectedly invoked a tool";
  else if (testCase.probe === "read_tool") {
    const call = toolCalls[0];
    if (toolCalls.length !== 1 || call?.name !== "command_execution" || call.status !== "completed") {
      error = "Read probe requires exactly one completed command and no other tools";
    } else if (call.file_path !== prepared.fixture_path) {
      error = "Read probe did not read the expected fixture path";
    } else if (!call.output?.includes(`verification_token=${prepared.expected_text}`)) {
      error = "Read command output did not contain the expected fixture token";
    }
  }
  return {
    success: error === undefined,
    text,
    tool_calls: toolCalls,
    completed_steps: completedSteps,
    session_ids: threadId ? [threadId] : [],
    ...(error ? { error } : {}),
  };
}

export const codex: AgentAdapter = {
  id: "codex",
  version: "0.154.0",
  image: "tokn-codex-agent-test:0.154.0",
  dockerfile: "scripts/docker/Dockerfile.codex",
  prepare,
  evaluate,
};
