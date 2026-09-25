import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import type {
  AccountProvider,
  AccountProbe,
  AccountImport,
  AccountEdit,
  LoginTicket,
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
  accountProviders: () => invoke<AccountProvider[]>("account_providers"),
  editAccount: (edit: AccountEdit) => invoke<void>("edit_account", { edit }),
  probeAccount: (id: string, force = false) =>
    invoke<AccountProbe>("probe_account", { id, force }),
  importAccount: (request: AccountImport) =>
    invoke<void>("import_account", { request }),
  beginAccountLogin: (id: string, provider: string) =>
    invoke<LoginTicket>("begin_account_login", { id, provider }),
  completeAccountLogin: (login_id: string) =>
    invoke<void>("complete_account_login", { login_id }),
  cancelAccountLogin: (login_id: string) =>
    invoke<void>("cancel_account_login", { login_id }),
  accounts: () => invoke<AccountSummary[]>("list_accounts"),
  usage: () => invoke<UsageSummary[]>("read_usage"),
  inspect: <T>(query: InspectQuery) => invoke<T>("inspect_query", { query }),
};
