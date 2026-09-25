import { invoke } from "@tauri-apps/api/core";
import type {
  GatewayStatus,
  RoutingDocument,
  AccountSummary,
  UsageSummary,
  RequestHistory,
  RequestDetail,
  RequestSummary,
} from "./types";
export const api = {
  status: () => invoke<GatewayStatus>("gateway_status"),
  start: () => invoke<void>("start_gateway"),
  stop: () => invoke<void>("stop_gateway"),
  reload: () => invoke<string>("reload_gateway"),
  routing: () => invoke<RoutingDocument>("read_routing"),
  saveRouting: (document: RoutingDocument, routing_toml: string) =>
    invoke<RoutingDocument>("save_routing", {
      revision: document.revision,
      routing_toml,
    }),
  accounts: () => invoke<AccountSummary[]>("list_accounts"),
  usage: () => invoke<UsageSummary[]>("read_usage"),
  history: () => invoke<RequestHistory>("read_history"),
  detail: (request: RequestSummary) =>
    invoke<RequestDetail | null>("request_detail", {
      day: request.day,
      request_id: request.request_id,
      row_id: request.row_id,
    }),
};
