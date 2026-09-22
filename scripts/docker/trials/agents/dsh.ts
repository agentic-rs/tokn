import { preparePrompt } from "./prompt";
import type { AgentAdapter, PreparedTrial, TrialCase, TrialOutput, TrialResult } from "./types";

function yamlString(value: string): string {
  return JSON.stringify(value);
}

function prepare(trial: TrialCase, options: { router_url: string; marker?: string }): PreparedTrial {
  if (trial.mode !== "api") throw new Error(`DSH does not support trial mode '${trial.mode}'`);
  if (trial.probe !== "text") {
    throw new Error("DSH 0.1.5-rc.2 does not expose structured tool events; only text probes are supported");
  }
  const { expected_text: expectedText, prompt } = preparePrompt(trial, options.marker);
  const model = trial.upstream_model ?? trial.model;
  const api = trial.api === "responses" ? "openai-responses" : "openai-completions";
  const baseUrl = `${options.router_url.replace(/\/$/, "")}${trial.base_path}`;
  const settings = [
    "agent-default-model:",
    `  provider: ${yamlString("tokn")}`,
    `  model: ${yamlString(model)}`,
    "llm-pi-ai:",
    "  providers:",
    "    tokn:",
    `      apiKeyEnv: ${yamlString("TOKN_TRIAL_API_KEY")}`,
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

function evaluate(_trial: TrialCase, prepared: PreparedTrial, output: TrialOutput): TrialResult {
  const text = output.stdout.trim();
  let error: string | undefined;
  if (output.timed_out) error = "DSH trial timed out";
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
  image: "tokn-dsh-trials:0.1.5-rc.2",
  dockerfile: "scripts/docker/Dockerfile.dsh",
  prepare,
  evaluate,
};
