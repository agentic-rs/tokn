import { dsh } from "./dsh";
import { opencode } from "./opencode";
import { pi } from "./pi";
import type { AgentAdapter } from "./types";

const adapters = new Map<string, AgentAdapter>([
  [opencode.id, opencode],
  [pi.id, pi],
  [dsh.id, dsh],
]);

export function resolveAgent(id: string): AgentAdapter {
  const adapter = adapters.get(id);
  if (!adapter) throw new Error(`Unsupported agent-test adapter '${id}' (available: ${[...adapters.keys()].join(", ")})`);
  return adapter;
}

export type { AgentAdapter, PreparedAgentTest, AgentTestCase, AgentTestOutput, AgentTestResult, AgentTestToolCall } from "./types";
