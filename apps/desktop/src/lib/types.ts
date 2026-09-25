export interface GatewayStatus {
  state: string;
  ownership: string;
  address: string;
  pid: number | null;
  last_error: string | null;
}
export interface RoutingDocument {
  config_path: string;
  revision: string;
  routing_toml: string;
  schema: string;
  overlay_paths: string[];
}
export interface AccountSummary {
  id: string;
  provider: string;
  label: string | null;
  enabled: boolean;
  tier: string;
}
export interface UsageSummary {
  account: string | null;
  provider: string | null;
  model: string;
  requests: number;
  input_tokens: number;
  output_tokens: number;
  cached_tokens: number;
}
export interface RequestSummary {
  row_id: string;
  day: string;
  request_id: string;
  ts: number;
  endpoint: string | null;
  status: number | null;
  request_error: string | null;
  provider_id: string | null;
  model: string | null;
}
export interface RequestHistory {
  day: string | null;
  requests: RequestSummary[];
  next_cursor: string | null;
}
export interface RequestDetail {
  day: string;
  row_id: string;
  request: Record<string, unknown>;
}
