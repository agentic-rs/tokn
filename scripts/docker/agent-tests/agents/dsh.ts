import { preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedAgentTest, AgentTestCase, AgentTestOutput, AgentTestResult } from "./types";

function yamlString(value: string): string {
  return JSON.stringify(value);
}

function prepare(testCase: AgentTestCase, options: { router_url: string; marker?: string }): PreparedAgentTest {
  if (testCase.mode !== "api") throw new Error(`DSH does not support agent-test mode '${testCase.mode}'`);
  if (testCase.probe !== "text") {
    throw new Error("DSH 0.1.5-rc.2 does not expose structured tool events; only text probes are supported");
  }
  const { expected_text: expectedText, prompt } = preparePrompt(testCase, options.marker);
  const model = testCase.upstream_model ?? testCase.model;
  const api = testCase.api === "responses" ? "openai-responses" : "openai-completions";
  const baseUrl = `${options.router_url.replace(/\/$/, "")}${testCase.base_path}`;
  const settings = [
    "agent-default-model:",
    `  provider: ${yamlString("tokn")}`,
    `  model: ${yamlString(model)}`,
    "llm-pi-ai:",
    "  providers:",
    "    tokn:",
    `      apiKeyEnv: ${yamlString("TOKN_AGENT_TEST_API_KEY")}`,
    `      api: ${yamlString(api)}`,
    `      baseURL: ${yamlString(baseUrl)}`,
    "      models:",
    `        - id: ${yamlString(model)}`,
    "",
  ].join("\n");
  return {
    files: [
      { path: "settings.yaml", content: settings },
      { path: "prompt.txt", content: `${prompt}\n` },
    ],
    command: ["--profile", "headless", prompt],
    environment: {
      DSH_HOME: "/tmp/dsh-home",
      DSH_PERMISSION_MODE: "workspace-write",
      DSH_TELEMETRY_DISABLED: "1",
    },
    working_dir: "/workspace",
    expected_text: expectedText,
  };
}

function evaluate(_testCase: AgentTestCase, prepared: PreparedAgentTest, output: AgentTestOutput): AgentTestResult {
  const text = output.stdout.trim();
  let error: string | undefined;
  if (output.timed_out) error = "DSH agent test timed out";
  else if (output.exit_code !== 0) error = `DSH exited with code ${output.exit_code ?? "unknown"}`;
  else if (text !== prepared.expected_text) error = "DSH response did not exactly match the expected marker";
  return {
    success: error === undefined,
    text,
    tool_calls: [],
    completed_steps: error ? 0 : 1,
    session_ids: [],
    ...(error ? { error } : {}),
  };
}

export const dsh: AgentAdapter = {
  id: "dsh",
  version: "0.1.5-rc.2",
  image: "tokn-dsh-agent-test:0.1.5-rc.2",
  dockerfile: "scripts/docker/Dockerfile.dsh",
  prepare,
  evaluate,
};
