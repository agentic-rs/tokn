export type TrialMode = {
  id: string;
  network_args: (gateway_name: string) => string[];
};

const modes = new Map<string, TrialMode>([
  ["api", { id: "api", network_args: (gateway_name) => ["--network", `container:${gateway_name}`] }],
]);

export function resolveMode(id: string): TrialMode {
  const mode = modes.get(id);
  if (!mode) throw new Error(`Unsupported trial mode '${id}' (available: ${[...modes.keys()].join(", ")})`);
  return mode;
}
