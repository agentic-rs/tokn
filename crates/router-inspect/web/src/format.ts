import type { RequestSummary, SessionSummary, TimezoneMode } from "./types";

export interface RequestOutcome {
  label: string;
  tone: "success" | "warning" | "error" | "neutral";
  title: string;
}

export function formatTimestamp(ts: number, timezone: TimezoneMode, compact = false): string {
  const options: Intl.DateTimeFormatOptions = compact
    ? { hour: "2-digit", minute: "2-digit", second: "2-digit" }
    : { dateStyle: "medium", timeStyle: "medium" };
  if (timezone === "utc") {
    options.timeZone = "UTC";
  }
  return new Intl.DateTimeFormat(undefined, options).format(new Date(ts));
}

export function formatCompactTimestamp(ts: number, timezone: TimezoneMode): string {
  const date = new Date(ts);
  const now = new Date();
  const year = timezone === "utc" ? date.getUTCFullYear() : date.getFullYear();
  const current_year = timezone === "utc" ? now.getUTCFullYear() : now.getFullYear();
  const options: Intl.DateTimeFormatOptions = {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  };
  if (year !== current_year) {
    options.year = "numeric";
  }
  if (timezone === "utc") {
    options.timeZone = "UTC";
  }
  return new Intl.DateTimeFormat(undefined, options).format(date);
}

export function formatDuration(first_ts: number, last_ts: number): string {
  const duration_ms = Math.max(0, last_ts - first_ts);
  if (duration_ms < 1_000) {
    return `${duration_ms.toLocaleString()} ms`;
  }
  const seconds = Math.floor(duration_ms / 1_000);
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ${seconds % 60}s`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ${minutes % 60}m`;
  }
  const days = Math.floor(hours / 24);
  return `${days}d ${hours % 24}h`;
}

export function requestKey(request: Pick<RequestSummary, "day" | "row_id">): string {
  return `${request.day}:${request.row_id}`;
}

export function shortId(value: string | null | undefined, length = 10): string {
  if (!value) {
    return "—";
  }
  return value.length > length ? `…${value.slice(-length)}` : value;
}

export function requestPath(request: RequestSummary): string {
  const value = request.inbound_req_url ?? request.endpoint;
  return displayPath(value);
}

function isSensitiveParameter(name: string): boolean {
  const normalized = name.toLowerCase().replaceAll("_", "-");
  return normalized === "authorization"
    || normalized === "password"
    || normalized === "code"
    || normalized === "signature"
    || normalized === "sig"
    || normalized.includes("api-key")
    || normalized.includes("access-key")
    || normalized.includes("token")
    || normalized.includes("secret")
    || normalized.includes("credential");
}

export function displayPath(value: string | null | undefined): string {
  if (!value) {
    return "unknown endpoint";
  }
  try {
    const url = new URL(value, window.location.origin);
    for (const name of new Set(url.searchParams.keys())) {
      if (isSensitiveParameter(name)) {
        url.searchParams.set(name, "REDACTED");
      }
    }
    return `${url.pathname}${url.search}`;
  } catch {
    return value.replace(/([?&]([^=&]+)=)([^&]*)/g, (match, prefix: string, name: string) => {
      let decoded_name = name;
      try {
        decoded_name = decodeURIComponent(name);
      } catch {
        // Keep the encoded name; malformed URLs should still remain displayable.
      }
      return isSensitiveParameter(decoded_name) ? `${prefix}REDACTED` : match;
    });
  }
}

export function requestOutcome(request: RequestSummary): RequestOutcome {
  if (request.request_error) {
    return { label: "ERR", tone: "error", title: request.request_error };
  }
  const status = request.inbound_resp_status ?? request.outbound_resp_status ?? request.status;
  if (status === null) {
    return { label: "—", tone: "neutral", title: "No response status persisted" };
  }
  const source = request.inbound_resp_status !== null
    ? "Client response"
    : request.outbound_resp_status !== null
      ? "Provider response"
      : "Request";
  if (status >= 400) {
    return { label: String(status), tone: "error", title: `${source}: ${status}` };
  }
  if (status >= 300) {
    return { label: String(status), tone: "warning", title: `${source}: ${status}` };
  }
  return { label: String(status), tone: "success", title: `${source}: ${status}` };
}

export function sessionOutcome(session: SessionSummary): RequestOutcome {
  const status = session.status;
  if (status === null) {
    return { label: "—", tone: "neutral", title: "No status stored for the current session head" };
  }
  if (status >= 400) {
    return { label: String(status), tone: "error", title: `Current head status: ${status}` };
  }
  if (status >= 300) {
    return { label: String(status), tone: "warning", title: `Current head status: ${status}` };
  }
  return { label: String(status), tone: "success", title: `Current head status: ${status}` };
}

export function eventDetail<T>(event: Event): T {
  return (event as CustomEvent<T>).detail;
}

export function stringField(record: Record<string, unknown>, field: string): string | undefined {
  const value = record[field];
  return typeof value === "string" ? value : undefined;
}

export function numberField(record: Record<string, unknown>, field: string): number | undefined {
  const value = record[field];
  return typeof value === "number" ? value : undefined;
}
