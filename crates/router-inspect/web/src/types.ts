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

export interface SessionDetail {
  session: SessionSummary;
  requests: RequestSummary[];
}

export interface ViewerInfo {
  requests_dir: string;
  sessions_db: string;
}

export type RequestDayState = "available" | "empty" | "unavailable";

export interface RequestDay {
  day: string;
  state: RequestDayState;
}

export type ViewName = "requests" | "sessions";
export type LoadState = "idle" | "loading" | "ready" | "error";
export type DetailTab = "overview" | "client" | "provider" | "raw";
export type TimezoneMode = "local" | "utc";

export interface RequestFilters {
  query: string;
  provider_id: string;
  status: string;
  errors_only: boolean;
}
