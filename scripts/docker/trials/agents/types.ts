export type TrialCase = {
  id: string;
  agent: string;
  mode: string;
  model: string;
  base_path: string;
  api: "responses" | "chat_completions";
  probe: "text" | "read_tool";
  upstream_model?: string;
  display_name?: string;
  expected_text?: string;
};

export type PreparedTrial = {
  files: { path: string; content: string }[];
  command: string[];
  environment: Record<string, string>;
  working_dir: string;
  expected_text: string;
  fixture_path?: string;
};

export type TrialOutput = {
  stdout: string;
  stderr: string;
  exit_code: number | null;
  timed_out?: boolean;
};

export type TrialToolCall = {
  name: string;
  status: string;
  file_path?: string;
  output?: string;
};

export type TrialResult = {
  success: boolean;
  text: string;
  tool_calls: TrialToolCall[];
  completed_steps: number;
  session_ids: string[];
  error?: string;
};

export type AgentAdapter = {
  id: string;
  version: string;
  image: string;
  dockerfile: string;
  prepare: (trial: TrialCase, options: { router_url: string; marker?: string }) => PreparedTrial;
  evaluate: (trial: TrialCase, prepared: PreparedTrial, output: TrialOutput) => TrialResult;
};
