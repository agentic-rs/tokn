export const CODEX_WEB_SEARCH_ENDPOINT = "/backend-api/codex/alpha/search";

export type WebSearchOperation =
  | { kind: "search_query"; value: string; domains: string[]; recency_days?: number }
  | { kind: "open"; value: string; line_number?: number }
  | { kind: "click"; value: string; link_id: number }
  | { kind: "find"; value: string; pattern: string };

export interface WebSearchResult {
  type?: string;
  domain?: string;
  ref_id?: string;
  snippet?: string;
  title?: string;
  url?: string;
}

export interface WebSearchInspection {
  operations: WebSearchOperation[];
  response_length?: string;
  allowed_callers: string[];
  external_web_access?: boolean;
  prompt?: string;
  output?: string;
  results: WebSearchResult[];
  encrypted_output_bytes?: number;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

function stringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === "string") : [];
}

function finiteNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function operationFromValue(kind: string, value: unknown): WebSearchOperation | undefined {
  const item = record(value);
  switch (kind) {
    case "search_query": {
      const query = stringValue(item?.q);
      return query ? {
        kind,
        value: query,
        domains: stringArray(item?.domains),
        recency_days: finiteNumber(item?.recency)
      } : undefined;
    }
    case "open": {
      const ref_id = stringValue(item?.ref_id);
      return ref_id ? { kind, value: ref_id, line_number: finiteNumber(item?.lineno) } : undefined;
    }
    case "click": {
      const ref_id = stringValue(item?.ref_id);
      const link_id = finiteNumber(item?.id);
      return ref_id && link_id !== undefined ? { kind, value: ref_id, link_id } : undefined;
    }
    case "find": {
      const ref_id = stringValue(item?.ref_id);
      const pattern = stringValue(item?.pattern);
      return ref_id && pattern ? { kind, value: ref_id, pattern } : undefined;
    }
    default:
      return undefined;
  }
}

function operationsFromCommands(value: unknown): WebSearchOperation[] {
  const commands = record(value);
  if (!commands) {
    return [];
  }
  return Object.entries(commands).flatMap(([kind, command]) => {
    if (!Array.isArray(command)) {
      return [];
    }
    return command.flatMap((item) => {
      const operation = operationFromValue(kind, item);
      return operation ? [operation] : [];
    });
  });
}

export function webSearchOperationSummary(operations: WebSearchOperation[]): string {
  if (operations.length === 0) {
    return "No operations";
  }
  const kinds = new Set(operations.map((operation) => operation.kind));
  if (kinds.size !== 1) {
    return `${operations.length} operations`;
  }
  const label = {
    search_query: ["query", "queries"],
    open: ["page open", "page opens"],
    click: ["link click", "link clicks"],
    find: ["find", "finds"]
  }[operations[0].kind];
  return `${operations.length} ${label[operations.length === 1 ? 0 : 1]}`;
}

function resultFromValue(value: unknown): WebSearchResult | undefined {
  const item = record(value);
  if (!item) {
    return undefined;
  }
  const result = {
    type: stringValue(item.type),
    domain: stringValue(item.domain),
    ref_id: stringValue(item.ref_id),
    snippet: stringValue(item.snippet),
    title: stringValue(item.title),
    url: stringValue(item.url)
  };
  return Object.values(result).some((field) => field !== undefined) ? result : undefined;
}

function promptFromInput(value: unknown): string | undefined {
  if (!Array.isArray(value)) {
    return undefined;
  }
  for (const candidate of value) {
    const message = record(candidate);
    const content = message?.content;
    if (!Array.isArray(content)) {
      continue;
    }
    for (const candidate_part of content) {
      const part = record(candidate_part);
      const text = stringValue(part?.text) ?? stringValue(part?.input_text);
      if (text) {
        return text;
      }
    }
  }
  return undefined;
}

export function decodedBase64ByteLength(value: string): number | undefined {
  const normalized = value.replace(/\s/g, "");
  if (!normalized || !/^[A-Za-z0-9_\-+/]*={0,2}$/.test(normalized)) {
    return undefined;
  }
  const unpadded_length = normalized.replace(/=+$/, "").length;
  const remainder = unpadded_length % 4;
  if (remainder === 1) {
    return undefined;
  }
  return Math.floor(unpadded_length * 3 / 4);
}

export function inspectWebSearch(request: unknown, response: unknown): WebSearchInspection {
  const request_record = record(request);
  const response_record = record(response);
  const commands = record(request_record?.commands);
  const settings = record(request_record?.settings);
  const result_values = Array.isArray(response_record?.results) ? response_record.results : [];
  const encrypted_output = stringValue(response_record?.encrypted_output);

  return {
    operations: operationsFromCommands(commands),
    response_length: stringValue(commands?.response_length),
    allowed_callers: stringArray(settings?.allowed_callers),
    external_web_access: typeof settings?.external_web_access === "boolean"
      ? settings.external_web_access
      : undefined,
    prompt: promptFromInput(request_record?.input),
    output: stringValue(response_record?.output),
    results: result_values.flatMap((value) => {
      const result = resultFromValue(value);
      return result ? [result] : [];
    }),
    encrypted_output_bytes: encrypted_output ? decodedBase64ByteLength(encrypted_output) : undefined
  };
}

export function isCodexWebSearchEndpoint(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  try {
    return new URL(value, "http://localhost").pathname === CODEX_WEB_SEARCH_ENDPOINT;
  } catch {
    return value.split("?", 1)[0] === CODEX_WEB_SEARCH_ENDPOINT;
  }
}

export function safeHttpUrl(value: string | undefined): string | undefined {
  if (!value) {
    return undefined;
  }
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:" ? url.href : undefined;
  } catch {
    return undefined;
  }
}
