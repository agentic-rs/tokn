export type AgentTestMode = {
  id: string;
  network_args: (gateway_name: string) => string[];
};

const modes = new Map<string, AgentTestMode>([
  ["api", { id: "api", network_args: (gateway_name) => ["--network", `container:${gateway_name}`] }],
]);

export function resolveMode(id: string): AgentTestMode {
  const mode = modes.get(id);
  if (!mode) throw new Error(`Unsupported agent-test mode '${id}' (available: ${[...modes.keys()].join(", ")})`);
  return mode;
}
