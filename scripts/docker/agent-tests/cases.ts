import type { AgentTestCase } from "./agents/types";

// Routes are explicit so suites can target existing host profiles without
// changing host configuration. Add agent adapters independently of test modes.
export const defaultCases: AgentTestCase[] = [
  {
    id: "opencode-deepseek-text",
    agent: "opencode",
    mode: "api",
    model: "deepseek-v4-flash",
    base_path: "/opencode-deepseek/v1",
    api: "chat_completions",
    probe: "text",
  },
  {
    id: "opencode-deepseek-read",
    agent: "opencode",
    mode: "api",
    model: "deepseek-v4-flash",
    base_path: "/opencode-deepseek/v1",
    api: "chat_completions",
    probe: "read_tool",
  },
  {
    id: "opencode-luna-text",
    agent: "opencode",
    mode: "api",
    model: "gpt-5.6-luna",
    upstream_model: "codex/gpt-5.6-luna",
    base_path: "/opencode-codex/v1",
    api: "responses",
    probe: "text",
  },
  {
    id: "opencode-luna-read",
    agent: "opencode",
    mode: "api",
    model: "gpt-5.6-luna",
    upstream_model: "codex/gpt-5.6-luna",
    base_path: "/opencode-codex/v1",
    api: "responses",
    probe: "read_tool",
  },
];

const fields = new Set([
  "id", "agent", "mode", "model", "base_path", "api", "probe", "upstream_model", "display_name", "expected_text",
]);

export function parseCases(value: unknown): AgentTestCase[] {
  if (!Array.isArray(value) || value.length === 0) throw new Error("Agent-test cases must be a nonempty JSON array");
  const ids = new Set<string>();
  return value.map((entry, index) => {
    const location = `Agent-test case ${index + 1}`;
    if (entry === null || typeof entry !== "object" || Array.isArray(entry)) throw new Error(`${location} must be an object`);
    const item = entry as Record<string, unknown>;
    for (const key of Object.keys(item)) {
      if (!fields.has(key)) throw new Error(`${location} has unknown field '${key}'`);
      if (typeof item[key] !== "string" || item[key] === "" || /[\r\n\0]/.test(item[key] as string)) {
        throw new Error(`${location}.${key} must be a nonempty single-line string`);
      }
    }
    for (const key of ["id", "agent", "mode", "model", "base_path", "api", "probe"]) {
      if (typeof item[key] !== "string") throw new Error(`${location} requires '${key}'`);
    }
    const testCase = item as AgentTestCase;
    if (!/^[a-z0-9][a-z0-9_-]*$/.test(testCase.id)) {
      throw new Error(`${location}.id must contain only lowercase letters, digits, '_' or '-'`);
    }
    if (ids.has(testCase.id)) throw new Error(`Duplicate agent-test case id '${testCase.id}'`);
    ids.add(testCase.id);
    if (testCase.api !== "responses" && testCase.api !== "chat_completions" && testCase.api !== "messages") {
      throw new Error(`${location}.api must be 'responses', 'chat_completions', or 'messages'`);
    }
    if (testCase.probe !== "text" && testCase.probe !== "read_tool") {
      throw new Error(`${location}.probe must be 'text' or 'read_tool'`);
    }
    if (!/^\/(?:[a-zA-Z0-9_-]+\/)*v1$/.test(testCase.base_path)) {
      throw new Error(`${location}.base_path must be an absolute API path ending in '/v1'`);
    }
    if (!/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(testCase.model)) {
      throw new Error(`${location}.model must be an unqualified model name; use upstream_model for a provider-qualified ID`);
    }
    if (testCase.expected_text !== undefined && testCase.expected_text.trim() !== testCase.expected_text) {
      throw new Error(`${location}.expected_text must not have surrounding whitespace`);
    }
    return { ...testCase };
  });
}

export function selectCases(cases: AgentTestCase[], ids: string[]): AgentTestCase[] {
  if (ids.length === 0) return [...cases];
  const requested = new Set(ids);
  const selected = cases.filter((testCase) => requested.delete(testCase.id));
  if (requested.size > 0) throw new Error(`Unknown agent-test case(s): ${[...requested].join(", ")}`);
  return selected;
}
