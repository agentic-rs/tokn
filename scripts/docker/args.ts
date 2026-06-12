export type TaggedArgs = {
  tag: string;
  rest: string[];
};

export class UsageError extends Error {
  constructor(message = "invalid arguments") {
    super(message);
    this.name = "UsageError";
  }
}

export function requireValue(args: string[], index: number, flag: string): string {
  const value = args[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function requireTag(raw: string): string {
  if (!/^[a-z0-9][a-z0-9_.-]*$/.test(raw)) {
    throw new Error("--tag must start with a lowercase letter or digit and contain only lowercase letters, digits, '.', '_', or '-'");
  }
  return raw;
}

export function requirePositiveInteger(raw: string, label: string): number {
  if (!/^[1-9][0-9]*$/.test(raw)) {
    throw new Error(`${label} must be a positive integer`);
  }
  return Number(raw);
}

export function requireBranch(raw: string): string {
  if (!raw || raw.startsWith("-")) {
    throw new Error("--branch requires a branch name");
  }
  return raw;
}

export function requirePort(raw: string, label: string): number {
  const port = requirePositiveInteger(raw, label);
  if (port > 65535) {
    throw new Error(`${label} must be between 1 and 65535`);
  }
  return port;
}

export function parseTaggedArgs(args: string[], defaultTag: string): TaggedArgs {
  let tag = process.env.TOKN_TAG ?? defaultTag;
  const rest: string[] = [];
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--tag") {
      tag = requireTag(requireValue(args, i, "--tag"));
      i += 1;
    } else if (arg.startsWith("--tag=")) {
      tag = requireTag(arg.slice("--tag=".length));
    } else {
      rest.push(arg);
    }
  }
  return { tag: requireTag(tag), rest };
}
