import type { InspectQuery, RequestIdentity } from "./types";

// Existing lazy payload locators are local identifiers, never network URLs.
// Convert them to a closed set of native queries at the transport boundary.
export function inspectQuery(locator: string): InspectQuery {
  if (!locator.startsWith("/api/") || locator.startsWith("//"))
    throw new Error("Invalid inspector locator");
  const url = new URL(locator, "https://inspector.invalid");
  const params = url.searchParams;
  const text = (key: string) => params.get(key) ?? undefined;
  const required = (key: string) => {
    const value = text(key);
    if (!value) throw new Error(`Missing ${key}`);
    return value;
  };
  const number = (key: string) => {
    const value = text(key);
    if (value === undefined) return undefined;
    if (!/^\d+$/.test(value) || !Number.isSafeInteger(Number(value)))
      throw new Error(`Invalid ${key}`);
    return Number(value);
  };
  const identity = (): RequestIdentity => ({
    day: required("day"),
    request_id: required("request_id"),
    row_id: text("row_id"),
  });
  switch (url.pathname) {
    case "/api/info":
      return { kind: "info" };
    case "/api/request-days":
      return { kind: "request_days" };
    case "/api/request-url-paths":
      return { kind: "request_url_paths", day: required("day") };
    case "/api/requests":
      return {
        kind: "requests",
        day: text("day"),
        limit: number("limit"),
        cursor: text("cursor"),
        session_id: text("session_id"),
        provider_id: text("provider_id"),
        url_path: text("url_path"),
        status: number("status"),
        errors_only: text("errors_only") === "true",
        query: text("query"),
      };
    case "/api/requests/latest":
      return {
        kind: "latest_requests",
        limit: number("limit"),
        cursor: text("cursor"),
      };
    case "/api/request":
      return { kind: "request", ...identity() };
    case "/api/request-llm-summary":
      return { kind: "request_llm_summary", ...identity() };
    case "/api/request-payload":
      return {
        kind: "request_payload",
        ...identity(),
        field: required("field"),
      };
    case "/api/request-llm-message":
    case "/api/request-llm-tool-definition": {
      const index = number("index");
      if (index === undefined) throw new Error("Missing index");
      return {
        kind:
          url.pathname === "/api/request-llm-message"
            ? "request_llm_message"
            : "request_llm_tool_definition",
        ...identity(),
        index,
      };
    }
    case "/api/sessions":
      return { kind: "sessions", limit: number("limit") };
    case "/api/session":
      return {
        kind: "session",
        session_id: required("session_id"),
        limit: number("limit"),
      };
    case "/api/session-usage":
      return { kind: "session_usage", session_id: required("session_id") };
    case "/api/session-node":
      return {
        kind: "session_node",
        session_id: required("session_id"),
        node_id: required("node_id"),
      };
    default:
      throw new Error("Unsupported inspector query");
  }
}
