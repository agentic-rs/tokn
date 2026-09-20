import { opencode } from "./opencode";
import type { AgentAdapter } from "./types";

const adapters = new Map<string, AgentAdapter>([[opencode.id, opencode]]);

export function resolveAgent(id: string): AgentAdapter {
  const adapter = adapters.get(id);
  if (!adapter) throw new Error(`Unsupported trial agent '${id}' (available: ${[...adapters.keys()].join(", ")})`);
  return adapter;
}

export type { AgentAdapter, PreparedTrial, TrialCase, TrialOutput, TrialResult, TrialToolCall } from "./types";
