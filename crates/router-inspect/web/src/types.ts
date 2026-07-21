export interface RequestSummary {
  row_id: string;
  day: string;
  request_id: string;
  ts: number;
  endpoint: string | null;
  status: number | null;
  request_error: string | null;
  session_id: string | null;
  account_id: string | null;
  provider_id: string | null;
  model: string | null;
  inbound_req_method: string | null;
  inbound_req_url: string | null;
  outbound_resp_status: number | null;
  inbound_resp_status: number | null;
}

export interface RequestDetail {
  row_id: string;
  day: string;
  request: Record<string, unknown>;
}

export interface RequestPayload {
  field: string;
  value: unknown;
}

export interface RequestPage {
  requests: RequestSummary[];
  next_cursor: string | null;
}

export interface LatestRequests extends RequestPage {
  day: string | null;
}

export interface SessionSummary {
  session_id: string;
  source: string | null;
  first_ts: number;
  last_ts: number;
  request_count: number;
  last_request_day: string;
  last_request_id: string;
  endpoint: string | null;
  status: number | null;
  account_id: string | null;
  provider_id: string | null;
  model: string | null;
}

export interface SessionNodeSummary {
  node_id: string;
  parent_node_id: string | null;
  request_id: string;
  ts: number;
  endpoint: string;
  status: number | null;
  account_id: string | null;
  provider_id: string | null;
  model: string | null;
  reduction_kind: string;
  parent_source: string;
  common_prefix_messages: number;
  request_message_count: number;
  response_message_count: number;
  message_id: string | null;
  input_message_count: number;
  output_message_count: number;
  is_head: boolean;
}

export type SessionPartContent =
  | { encoding: "text"; value: string; truncated: boolean }
  | { encoding: "json"; value: unknown }
  | { encoding: "encrypted"; byte_length: number }
  | { encoding: "binary"; byte_length: number }
  | {
      encoding: "omitted";
      original_encoding: "text" | "json" | "binary" | "unknown";
      reason: "part_limit" | "aggregate_limit";
    };

export interface SessionPart {
  part_type: string;
  byte_length: number;
  content: SessionPartContent;
}

export interface SessionMessage {
  role: string;
  status: number | null;
  parts: SessionPart[];
  parts_total: number;
}

export interface SessionMessageTruncation {
  messages_total: number;
  messages_returned: number;
  messages_omitted_before: number;
  messages_omitted_after: number;
}

export interface SessionNodeTruncation {
  request_messages: SessionMessageTruncation;
  response_messages: SessionMessageTruncation;
  parts_total: number;
  parts_returned: number;
  parts_omitted: number;
  content_bytes_total: number;
  content_bytes_returned: number;
  content_parts_truncated: number;
  binary_parts_elided: number;
}

export interface SessionDetail {
  session: SessionSummary;
  head_node_id: string | null;
  nodes: SessionNodeSummary[];
  nodes_truncated: boolean;
}

export interface SessionUsage {
  session_id: string;
  request_count: number;
  requests_with_usage: number;
  input_tokens: number | null;
  output_tokens: number | null;
  total_tokens: number | null;
  cache_read_tokens: number | null;
  cache_write_tokens: number | null;
  reasoning_tokens: number | null;
  requests: SessionRequestUsage[];
}

export interface SessionRequestUsage {
  request_id: string;
  context_tokens: number | null;
  input_delta_tokens: number | null;
  output_tokens: number | null;
}

export interface SessionNodeDetail {
  node: SessionNodeSummary;
  request_messages: SessionMessage[];
  response_messages: SessionMessage[];
  truncation: SessionNodeTruncation;
}

export interface ViewerInfo {
  requests_dir: string;
  sessions_db: string;
  usage_db: string;
}

export type RequestDayState = "available" | "empty" | "unavailable";

export interface RequestDay {
  day: string;
  state: RequestDayState;
}

export interface RequestUrlPath {
  url_path: string;
  request_count: number;
}

export type ViewName = "requests" | "sessions";
export type LoadState = "idle" | "loading" | "ready" | "error";
export type DetailTab = "overview" | "client" | "provider" | "raw";
export type TimezoneMode = "local" | "utc";

export interface RequestFilters {
  query: string;
  provider_id: string;
  url_path: string;
  status: string;
  errors_only: boolean;
}
