export type AgentTestCase = {
  id: string;
  agent: string;
  mode: string;
  model: string;
  base_path: string;
  api: "responses" | "chat_completions" | "messages";
  probe: "text" | "read_tool";
  upstream_model?: string;
  display_name?: string;
  expected_text?: string;
};

export type PreparedAgentTest = {
  files: { path: string; content: string }[];
  command: string[];
  environment: Record<string, string>;
  working_dir: string;
  expected_text: string;
  fixture_path?: string;
};

export type AgentTestOutput = {
  stdout: string;
  stderr: string;
  exit_code: number | null;
  timed_out?: boolean;
};

export type AgentTestToolCall = {
  name: string;
  status: string;
  file_path?: string;
  output?: string;
};

export type AgentTestResult = {
  success: boolean;
  text: string;
  tool_calls: AgentTestToolCall[];
  completed_steps: number;
  session_ids: string[];
  error?: string;
};

export type AgentAdapter = {
  id: string;
  version: string;
  image: string;
  dockerfile: string;
  prepare: (testCase: AgentTestCase, options: { router_url: string; marker?: string }) => PreparedAgentTest;
  evaluate: (testCase: AgentTestCase, prepared: PreparedAgentTest, output: AgentTestOutput) => AgentTestResult;
};
