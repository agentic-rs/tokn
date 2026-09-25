import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import type {
  GatewayStatus,
  RoutingDocument,
  AccountSummary,
  UsageSummary,
  InspectQuery,
} from "./types";
export const api = {
  setTheme: (theme: "light" | "dark" | null) =>
    getCurrentWindow().setTheme(theme),
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
  inspect: <T>(query: InspectQuery) => invoke<T>("inspect_query", { query }),
};
