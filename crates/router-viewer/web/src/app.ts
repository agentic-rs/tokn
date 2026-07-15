import { LitElement, html, nothing } from "lit";
import { HttpError, fetchJson, isAbortError } from "./api";
import { eventDetail, requestKey } from "./format";
import "./request-detail";
import "./request-list";
import "./session-views";
import type {
  DetailTab,
  LatestRequests,
  LoadState,
  RequestDay,
  RequestDetail,
  RequestFilters,
  RequestPage,
  RequestSummary,
  SessionDetail,
  SessionSummary,
  TimezoneMode,
  ViewerInfo,
  ViewName
} from "./types";

const PAGE_LIMIT = 100;

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function isDetailTab(value: string | null): value is DetailTab {
  return value === "overview" || value === "client" || value === "provider" || value === "raw";
}

function emptyFilters(): RequestFilters {
  return { query: "", provider_id: "", status: "", errors_only: false };
}

type HistoryMode = "push" | "replace" | null;

class ViewerApp extends LitElement {
  static properties = {
    active_view: { type: String },
    info: { attribute: false },
    requests: { attribute: false },
    request_days: { attribute: false },
    selected_day: { type: String },
    selected_request: { attribute: false },
    selected_request_id: { type: String },
    selected_request_row_id: { type: String },
    selected_request_detail: { attribute: false },
    request_list_state: { type: String },
    request_list_error: { type: String },
    request_detail_state: { type: String },
    request_detail_error: { type: String },
    next_cursor: { type: String },
    loading_more: { type: Boolean },
    load_more_error: { type: String },
    search_query: { type: String },
    provider_id: { type: String },
    status_filter: { type: String },
    errors_only: { type: Boolean },
    applied_filters: { attribute: false },
    active_detail_tab: { type: String },
    timezone: { type: String },
    request_days_loading: { type: Boolean },
    request_days_error: { type: String },
    sessions: { attribute: false },
    selected_session: { attribute: false },
    selected_session_detail: { attribute: false },
    sessions_loading: { type: Boolean },
    sessions_error: { type: String },
    session_detail_loading: { type: Boolean },
    session_detail_error: { type: String }
  };

  declare active_view: ViewName;
  declare info: ViewerInfo | undefined;
  declare requests: RequestSummary[];
  declare request_days: RequestDay[];
  declare selected_day: string | undefined;
  declare selected_request: RequestSummary | undefined;
  declare selected_request_id: string | undefined;
  declare selected_request_row_id: string | undefined;
  declare selected_request_detail: RequestDetail | undefined;
  declare request_list_state: LoadState;
  declare request_list_error: string | undefined;
  declare request_detail_state: LoadState;
  declare request_detail_error: string | undefined;
  declare next_cursor: string | undefined;
  declare loading_more: boolean;
  declare load_more_error: string | undefined;
  declare search_query: string;
  declare provider_id: string;
  declare status_filter: string;
  declare errors_only: boolean;
  declare applied_filters: RequestFilters;
  declare active_detail_tab: DetailTab;
  declare timezone: TimezoneMode;
  declare request_days_loading: boolean;
  declare request_days_error: string | undefined;
  declare sessions: SessionSummary[];
  declare selected_session: SessionSummary | undefined;
  declare selected_session_detail: SessionDetail | undefined;
  declare sessions_loading: boolean;
  declare sessions_error: string | undefined;
  declare session_detail_loading: boolean;
  declare session_detail_error: string | undefined;

  private request_load_id = 0;
  private request_detail_load_id = 0;
  private session_detail_load_id = 0;
  private request_days_load_id = 0;
  private sessions_loaded = false;
  private requested_request_id: string | undefined;
  private requested_request_row_id: string | undefined;
  private request_rows_context: string | undefined;
  private request_controller: AbortController | undefined;
  private request_detail_controller: AbortController | undefined;
  private navigation_workflow_id = 0;
  private readonly popstate_handler = () => void this.restoreFromHistory();

  constructor() {
    super();
    this.active_view = "requests";
    this.requests = [];
    this.request_days = [];
    this.sessions = [];
    this.request_list_state = "idle";
    this.request_detail_state = "idle";
    this.search_query = "";
    this.provider_id = "";
    this.status_filter = "";
    this.errors_only = false;
    this.applied_filters = emptyFilters();
    this.active_detail_tab = "overview";
    this.timezone = "local";
    this.loading_more = false;
    this.request_days_loading = false;
    this.sessions_loading = false;
    this.session_detail_loading = false;
  }

  createRenderRoot() {
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    this.restoreUrlState();
    window.addEventListener("popstate", this.popstate_handler);
    void this.loadInitialData();
  }

  disconnectedCallback() {
    window.removeEventListener("popstate", this.popstate_handler);
    this.request_controller?.abort();
    this.request_detail_controller?.abort();
    super.disconnectedCallback();
  }

  private restoreUrlState() {
    const params = new URLSearchParams(window.location.search);
    this.active_view = params.get("view") === "sessions" ? "sessions" : "requests";
    const day = params.get("day");
    this.selected_day = day && /^\d{4}-\d{2}-\d{2}$/.test(day) ? day : undefined;
    this.search_query = params.get("query") ?? "";
    this.provider_id = params.get("provider_id") ?? "";
    const status = params.get("status") ?? "";
    this.status_filter = /^\d{3}$/.test(status) ? status : "";
    this.errors_only = params.get("errors_only") === "true" || params.get("errors_only") === "1";
    this.applied_filters = {
      query: this.search_query,
      provider_id: this.provider_id,
      status: this.status_filter,
      errors_only: this.errors_only
    };
    this.requested_request_id = params.get("request_id") ?? undefined;
    const row_id = params.get("row_id");
    this.requested_request_row_id = row_id && /^-?\d+$/.test(row_id) ? row_id : undefined;
    const tab = params.get("tab");
    this.active_detail_tab = isDetailTab(tab) ? tab : "overview";
    this.timezone = params.get("timezone") === "utc" ? "utc" : "local";
  }

  private selectedRequestDay(): string | undefined {
    return this.selected_request_detail?.day ?? this.selected_request?.day ?? this.selected_day;
  }

  private syncUrl(mode: Exclude<HistoryMode, null> = "replace") {
    const params = new URLSearchParams();
    if (this.active_view !== "requests") {
      params.set("view", this.active_view);
    }
    const day = this.selected_request_id ? this.selectedRequestDay() : this.selected_day;
    if (day) {
      params.set("day", day);
    }
    if (this.applied_filters.query) {
      params.set("query", this.applied_filters.query);
    }
    if (this.applied_filters.provider_id) {
      params.set("provider_id", this.applied_filters.provider_id);
    }
    if (this.applied_filters.status) {
      params.set("status", this.applied_filters.status);
    }
    if (this.applied_filters.errors_only) {
      params.set("errors_only", "true");
    }
    if (this.selected_request_id) {
      params.set("request_id", this.selected_request_id);
      if (this.selected_request_row_id) {
        params.set("row_id", this.selected_request_row_id);
      }
      params.set("tab", this.active_detail_tab);
    }
    params.set("timezone", this.timezone);
    const query = params.toString();
    const url = `${window.location.pathname}${query ? `?${query}` : ""}`;
    if (`${window.location.pathname}${window.location.search}` === url) {
      return;
    }
    if (mode === "push") {
      window.history.pushState(null, "", url);
    } else {
      window.history.replaceState(null, "", url);
    }
  }

  private async loadInitialData() {
    const workflow_id = ++this.navigation_workflow_id;
    void this.loadInfo();
    void this.loadRequestDays();
    await this.loadUrlState(workflow_id);
  }

  private async restoreFromHistory() {
    const workflow_id = ++this.navigation_workflow_id;
    this.request_controller?.abort();
    this.request_detail_controller?.abort();
    this.resetRequestSelection();
    this.restoreUrlState();
    this.requests = [];
    this.next_cursor = undefined;
    this.request_rows_context = undefined;
    await this.loadUrlState(workflow_id);
  }

  private async loadUrlState(workflow_id: number) {
    const requested_request_id = this.requested_request_id;
    const requested_request_row_id = this.requested_request_row_id;
    if (this.active_view === "sessions") {
      void this.ensureSessionsLoaded();
    }
    let loaded: boolean;
    if (this.selected_day) {
      loaded = await this.loadRequests();
    } else {
      loaded = await this.loadLatestRequests();
      if (loaded && this.selected_day && this.hasAppliedFilters()) {
        loaded = await this.loadRequests();
      }
    }
    if (!loaded || workflow_id !== this.navigation_workflow_id) {
      return;
    }
    if (requested_request_id && this.selected_day) {
      const summary = this.requests.find((request) =>
        request.request_id === requested_request_id
        && (!requested_request_row_id || request.row_id === requested_request_row_id)
      );
      await this.loadRequestDetail(
        this.selected_day,
        requested_request_id,
        requested_request_row_id ?? summary?.row_id,
        summary,
        false,
        null
      );
    }
  }

  private async loadInfo() {
    try {
      this.info = await fetchJson<ViewerInfo>("/api/info");
    } catch {
      this.info = undefined;
    }
  }

  private async loadLatestRequests(): Promise<boolean> {
    this.request_controller?.abort();
    const controller = new AbortController();
    this.request_controller = controller;
    const load_id = ++this.request_load_id;
    this.requests = [];
    this.next_cursor = undefined;
    this.request_rows_context = undefined;
    this.request_list_state = "loading";
    this.request_list_error = undefined;
    try {
      const page = await fetchJson<LatestRequests>(`/api/requests/latest?limit=${PAGE_LIMIT}`, controller.signal);
      if (load_id !== this.request_load_id || this.request_controller !== controller) {
        return false;
      }
      this.selected_day = page.day ?? undefined;
      this.requests = page.requests;
      this.next_cursor = page.next_cursor ?? undefined;
      this.request_rows_context = this.requestContext(this.selected_day, emptyFilters());
      this.request_list_state = "ready";
      this.syncUrl();
      return true;
    } catch (error) {
      if (load_id === this.request_load_id && !isAbortError(error)) {
        this.request_list_state = "error";
        this.request_list_error = errorMessage(error, "Unable to load recent requests");
      }
      return false;
    } finally {
      if (this.request_controller === controller) {
        this.request_controller = undefined;
      }
    }
  }

  private requestContext(day = this.selected_day, filters = this.applied_filters): string | undefined {
    return day
      ? JSON.stringify([day, filters.query, filters.provider_id, filters.status, filters.errors_only])
      : undefined;
  }

  private requestParams(day: string, filters: RequestFilters, cursor?: string): URLSearchParams {
    const params = new URLSearchParams({ day, limit: String(PAGE_LIMIT) });
    if (filters.query) {
      params.set("query", filters.query);
    }
    if (filters.provider_id) {
      params.set("provider_id", filters.provider_id);
    }
    if (filters.status) {
      params.set("status", filters.status);
    }
    if (filters.errors_only) {
      params.set("errors_only", "true");
    }
    if (cursor) {
      params.set("cursor", cursor);
    }
    return params;
  }

  private async loadRequests(append = false): Promise<boolean> {
    const day = this.selected_day;
    if (!day) {
      this.request_list_state = "idle";
      this.requests = [];
      this.next_cursor = undefined;
      this.request_rows_context = undefined;
      return false;
    }
    const filters = { ...this.applied_filters };
    const context = this.requestContext(day, filters);
    const cursor = append ? this.next_cursor : undefined;
    if (append && (!cursor || this.request_rows_context !== context)) {
      return false;
    }
    this.request_controller?.abort();
    const controller = new AbortController();
    this.request_controller = controller;
    const load_id = ++this.request_load_id;
    if (append) {
      this.loading_more = true;
      this.load_more_error = undefined;
    } else {
      this.loading_more = false;
      if (this.request_rows_context !== context) {
        this.requests = [];
        this.next_cursor = undefined;
        this.request_rows_context = undefined;
      }
      this.request_list_state = "loading";
      this.request_list_error = undefined;
      this.load_more_error = undefined;
    }
    try {
      const page = await fetchJson<RequestPage>(
        `/api/requests?${this.requestParams(day, filters, cursor).toString()}`,
        controller.signal
      );
      if (load_id !== this.request_load_id || this.request_controller !== controller || this.requestContext() !== context) {
        return false;
      }
      if (append) {
        const existing = new Set(this.requests.map((request) => requestKey(request)));
        this.requests = [...this.requests, ...page.requests.filter((request) => !existing.has(requestKey(request)))];
      } else {
        this.requests = page.requests;
      }
      this.next_cursor = page.next_cursor ?? undefined;
      this.request_rows_context = context;
      this.request_list_state = "ready";
      return true;
    } catch (error) {
      if (load_id !== this.request_load_id || isAbortError(error)) {
        return false;
      }
      if (error instanceof HttpError && error.status === 503) {
        this.markRequestDayUnavailable(day);
      }
      if (append) {
        this.load_more_error = errorMessage(error, "Unable to load more requests");
      } else {
        this.request_list_state = "error";
        this.request_list_error = errorMessage(error, "Unable to load requests");
      }
      return false;
    } finally {
      if (load_id === this.request_load_id) {
        this.loading_more = false;
      }
      if (this.request_controller === controller) {
        this.request_controller = undefined;
      }
    }
  }

  private async loadRequestDays() {
    const load_id = ++this.request_days_load_id;
    this.request_days_loading = true;
    this.request_days_error = undefined;
    try {
      const request_days = await fetchJson<RequestDay[]>("/api/request-days");
      if (load_id === this.request_days_load_id) {
        this.request_days = request_days;
      }
    } catch (error) {
      if (load_id === this.request_days_load_id) {
        this.request_days_error = errorMessage(error, "Unable to load request day states");
      }
    } finally {
      if (load_id === this.request_days_load_id) {
        this.request_days_loading = false;
      }
    }
  }

  private markRequestDayUnavailable(day: string) {
    if (this.request_days.some((candidate) => candidate.day === day)) {
      this.request_days = this.request_days.map((candidate) =>
        candidate.day === day ? { ...candidate, state: "unavailable" } : candidate
      );
    } else {
      this.request_days = [{ day, state: "unavailable" }, ...this.request_days];
    }
  }

  private resetRequestSelection() {
    this.request_detail_controller?.abort();
    this.request_detail_controller = undefined;
    this.request_detail_load_id += 1;
    this.selected_request = undefined;
    this.selected_request_id = undefined;
    this.selected_request_row_id = undefined;
    this.selected_request_detail = undefined;
    this.request_detail_state = "idle";
    this.request_detail_error = undefined;
    this.active_detail_tab = "overview";
  }

  private async closeRequestDetail() {
    const selected_key = this.selected_request_row_id && this.selectedRequestDay()
      ? requestKey({ day: this.selectedRequestDay()!, row_id: this.selected_request_row_id })
      : undefined;
    ++this.navigation_workflow_id;
    this.resetRequestSelection();
    this.syncUrl("push");
    if (!selected_key || !window.matchMedia("(max-width: 680px)").matches) {
      return;
    }
    await this.updateComplete;
    const rows = this.querySelectorAll<HTMLButtonElement>("request-list [data-request-key]");
    [...rows].find((row) => row.dataset.requestKey === selected_key)?.focus();
  }

  private async loadRequestDetail(
    day: string,
    request_id: string,
    row_id: string | undefined,
    summary: RequestSummary | undefined,
    preserve: boolean,
    history_mode: HistoryMode = "replace"
  ): Promise<boolean> {
    this.request_detail_controller?.abort();
    const controller = new AbortController();
    this.request_detail_controller = controller;
    const load_id = ++this.request_detail_load_id;
    this.selected_day = day;
    this.selected_request = summary;
    this.selected_request_id = request_id;
    this.selected_request_row_id = row_id;
    if (!preserve) {
      this.selected_request_detail = undefined;
    }
    this.request_detail_state = "loading";
    this.request_detail_error = undefined;
    if (history_mode) {
      this.syncUrl(history_mode);
    }
    try {
      const params = new URLSearchParams({ day, request_id });
      if (row_id) {
        params.set("row_id", row_id);
      }
      const detail = await fetchJson<RequestDetail>(`/api/request?${params.toString()}`, controller.signal);
      if (load_id === this.request_detail_load_id && this.request_detail_controller === controller) {
        const locator_changed = this.selected_request_row_id !== detail.row_id;
        this.selected_request_detail = detail;
        this.selected_request_row_id = detail.row_id;
        this.request_detail_state = "ready";
        if (history_mode || locator_changed) {
          this.syncUrl("replace");
        }
        return true;
      }
      return false;
    } catch (error) {
      if (load_id === this.request_detail_load_id && !isAbortError(error)) {
        this.request_detail_state = "error";
        this.request_detail_error = errorMessage(error, "Unable to load request detail");
      }
      return false;
    } finally {
      if (this.request_detail_controller === controller) {
        this.request_detail_controller = undefined;
      }
    }
  }

  private async selectRequest(request: RequestSummary) {
    ++this.navigation_workflow_id;
    const preserve = this.selected_request_id === request.request_id
      && this.selected_request_detail?.day === request.day
      && this.selected_request_detail.row_id === request.row_id;
    const loading = this.loadRequestDetail(request.day, request.request_id, request.row_id, request, preserve, "push");
    if (window.matchMedia("(max-width: 680px)").matches) {
      await this.updateComplete;
      this.querySelector<HTMLButtonElement>("request-detail-view .mobile-back-button")?.focus();
    }
    const loaded = await loading;
    if (loaded && window.matchMedia("(max-width: 680px)").matches) {
      await this.updateComplete;
      this.querySelector<HTMLButtonElement>("request-detail-view .mobile-back-button")?.focus();
    }
  }

  private retryRequestDetail() {
    const day = this.selected_request_detail?.day ?? this.selected_request?.day ?? this.selected_day;
    if (day && this.selected_request_id) {
      void this.loadRequestDetail(
        day,
        this.selected_request_id,
        this.selected_request_row_id,
        this.selected_request,
        Boolean(this.selected_request_detail),
        null
      );
    }
  }

  private selectDay(day: string) {
    if (day === this.selected_day) {
      return;
    }
    ++this.navigation_workflow_id;
    this.selected_day = day;
    this.resetRequestSelection();
    this.syncUrl("push");
    void this.loadRequests();
  }

  private pickerDays(): RequestDay[] {
    if (!this.selected_day || this.request_days.some((request_day) => request_day.day === this.selected_day)) {
      return this.request_days;
    }
    return [{ day: this.selected_day, state: "available" }, ...this.request_days];
  }

  private adjacentAvailableDay(offset: -1 | 1): string | undefined {
    const available = this.pickerDays()
      .filter((request_day) => request_day.state === "available")
      .map((request_day) => request_day.day)
      .sort();
    if (!this.selected_day) {
      return undefined;
    }
    const index = available.indexOf(this.selected_day);
    return index < 0 ? undefined : available[index + offset];
  }

  private submitFilters(event: SubmitEvent) {
    event.preventDefault();
    ++this.navigation_workflow_id;
    this.applied_filters = {
      query: this.search_query.trim(),
      provider_id: this.provider_id.trim(),
      status: this.status_filter.trim(),
      errors_only: this.errors_only
    };
    this.resetRequestSelection();
    this.syncUrl("push");
    void this.loadRequests();
  }

  private clearFilters() {
    this.search_query = "";
    this.provider_id = "";
    this.status_filter = "";
    this.errors_only = false;
    this.applied_filters = emptyFilters();
    ++this.navigation_workflow_id;
    this.resetRequestSelection();
    this.syncUrl("push");
    void this.loadRequests();
  }

  private hasAppliedFilters(): boolean {
    return Boolean(
      this.applied_filters.query
      || this.applied_filters.provider_id
      || this.applied_filters.status
      || this.applied_filters.errors_only
    );
  }

  private filtersChanged(): boolean {
    return this.search_query.trim() !== this.applied_filters.query
      || this.provider_id.trim() !== this.applied_filters.provider_id
      || this.status_filter.trim() !== this.applied_filters.status
      || this.errors_only !== this.applied_filters.errors_only;
  }

  private providerOptions(): string[] {
    const providers = new Set(this.requests.flatMap((request) => (request.provider_id ? [request.provider_id] : [])));
    if (this.applied_filters.provider_id) {
      providers.add(this.applied_filters.provider_id);
    }
    return [...providers].sort();
  }

  private async ensureSessionsLoaded() {
    if (this.sessions_loaded || this.sessions_loading) {
      return;
    }
    this.sessions_loading = true;
    this.sessions_error = undefined;
    try {
      this.sessions = await fetchJson<SessionSummary[]>("/api/sessions?limit=100");
      this.sessions_loaded = true;
    } catch (error) {
      this.sessions_error = errorMessage(error, "Unable to load sessions");
    } finally {
      this.sessions_loading = false;
    }
  }

  private retrySessions() {
    this.sessions_loaded = false;
    void this.ensureSessionsLoaded();
  }

  private async loadSession(session_id: string, session: SessionSummary | undefined) {
    const load_id = ++this.session_detail_load_id;
    this.selected_session = session;
    this.selected_session_detail = undefined;
    this.session_detail_loading = true;
    this.session_detail_error = undefined;
    try {
      const detail = await fetchJson<SessionDetail>(`/api/session?session_id=${encodeURIComponent(session_id)}&limit=500`);
      if (load_id === this.session_detail_load_id) {
        this.selected_session = detail.session;
        this.selected_session_detail = detail;
      }
    } catch (error) {
      if (load_id === this.session_detail_load_id) {
        this.session_detail_error = errorMessage(error, "Unable to load session timeline");
      }
    } finally {
      if (load_id === this.session_detail_load_id) {
        this.session_detail_loading = false;
      }
    }
  }

  private async openSession(session_id: string) {
    ++this.navigation_workflow_id;
    this.setActiveView("sessions", false, "push");
    const session = this.sessions.find((candidate) => candidate.session_id === session_id);
    await this.loadSession(session_id, session);
  }

  private async openRequest(request: RequestSummary) {
    const workflow_id = ++this.navigation_workflow_id;
    this.setActiveView("requests", true, null);
    const needs_reload = this.selected_day !== request.day || this.hasAppliedFilters();
    if (needs_reload) {
      this.selected_day = request.day;
      this.search_query = "";
      this.provider_id = "";
      this.status_filter = "";
      this.errors_only = false;
      this.applied_filters = emptyFilters();
      this.resetRequestSelection();
      const loaded = await this.loadRequests();
      if (!loaded || workflow_id !== this.navigation_workflow_id) {
        return;
      }
    }
    await this.selectRequest(request);
  }

  private setActiveView(active_view: ViewName, load_sessions = true, history_mode: HistoryMode = "push") {
    if (history_mode === "push") {
      ++this.navigation_workflow_id;
    }
    this.active_view = active_view;
    if (history_mode) {
      this.syncUrl(history_mode);
    }
    if (active_view === "sessions" && load_sessions) {
      void this.ensureSessionsLoaded();
    }
  }

  private setTimezone(timezone: TimezoneMode) {
    this.timezone = timezone;
    this.syncUrl("push");
  }

  private setDetailTab(tab: DetailTab) {
    this.active_detail_tab = tab;
    this.syncUrl("push");
  }

  private renderDayPicker() {
    const days = this.pickerDays();
    const previous_day = this.adjacentAvailableDay(-1);
    const next_day = this.adjacentAvailableDay(1);
    return html`
      <div class="day-control">
        <span class="control-label">UTC storage day</span>
        <div class="day-navigation">
          <button
            type="button"
            class="icon-button"
            title="Previous available day"
            aria-label="Previous available day"
            ?disabled=${!previous_day}
            @click=${() => previous_day && this.selectDay(previous_day)}
          >
            ←
          </button>
          <select
            aria-label="Request storage day"
            .value=${this.selected_day ?? ""}
            ?disabled=${days.length === 0}
            @change=${(event: Event) => this.selectDay((event.target as HTMLSelectElement).value)}
          >
            ${this.selected_day ? nothing : html`<option value="">No request day</option>`}
            ${days.map(
              (request_day) => html`
                <option value=${request_day.day} ?disabled=${request_day.state !== "available"}>
                  ${request_day.day}${request_day.state === "empty" ? " · empty" : request_day.state === "unavailable" ? " · unavailable" : ""}
                </option>
              `
            )}
          </select>
          <button
            type="button"
            class="icon-button"
            title="Next available day"
            aria-label="Next available day"
            ?disabled=${!next_day}
            @click=${() => next_day && this.selectDay(next_day)}
          >
            →
          </button>
        </div>
      </div>
    `;
  }

  private renderRequestToolbar() {
    const has_day = Boolean(this.selected_day);
    return html`
      <section class="request-toolbar" aria-label="Request controls">
        <div class="toolbar-primary">
          ${this.renderDayPicker()}
          <button
            type="button"
            class="refresh-button"
            ?disabled=${!has_day || this.request_list_state === "loading"}
            @click=${() => {
              void this.loadRequests();
              void this.loadRequestDays();
            }}
          >
            <span aria-hidden="true">↻</span> Refresh requests
          </button>
          <div class="timezone-toggle" role="group" aria-label="Timestamp timezone">
            <button
              type="button"
              aria-pressed=${String(this.timezone === "local")}
              @click=${() => this.setTimezone("local")}
            >
              Local
            </button>
            <button
              type="button"
              aria-pressed=${String(this.timezone === "utc")}
              @click=${() => this.setTimezone("utc")}
            >
              UTC
            </button>
          </div>
        </div>
        <form class="filter-bar" @submit=${this.submitFilters}>
          <label class="search-field">
            <span class="visually-hidden">Search requests</span>
            <span class="search-icon" aria-hidden="true">⌕</span>
            <input
              type="search"
              .value=${this.search_query}
              ?disabled=${!has_day}
              placeholder="Search request, session, model…"
              @input=${(event: Event) => (this.search_query = (event.target as HTMLInputElement).value)}
            />
          </label>
          <label>
            <span class="visually-hidden">Provider ID</span>
            <input
              list="provider-options"
              .value=${this.provider_id}
              ?disabled=${!has_day}
              placeholder="Any provider"
              @input=${(event: Event) => (this.provider_id = (event.target as HTMLInputElement).value)}
            />
            <datalist id="provider-options">
              ${this.providerOptions().map((provider_id) => html`<option value=${provider_id}></option>`)}
            </datalist>
          </label>
          <label>
            <span class="visually-hidden">Exact response status</span>
            <input
              class="status-filter"
              type="number"
              min="100"
              max="599"
              step="1"
              .value=${this.status_filter}
              ?disabled=${!has_day}
              placeholder="Any status"
              @input=${(event: Event) => (this.status_filter = (event.target as HTMLInputElement).value)}
            />
          </label>
          <label class="errors-filter">
            <input
              type="checkbox"
              .checked=${this.errors_only}
              ?disabled=${!has_day}
              @change=${(event: Event) => (this.errors_only = (event.target as HTMLInputElement).checked)}
            />
            <span>Errors only</span>
          </label>
          <button type="submit" class="primary-button" ?disabled=${!has_day || !this.filtersChanged()}>Apply</button>
          ${this.hasAppliedFilters()
            ? html`<button type="button" class="text-button" @click=${this.clearFilters}>Clear</button>`
            : nothing}
        </form>
        ${this.request_days_error ? html`<p class="toolbar-warning" role="status">Day scan: ${this.request_days_error}</p>` : nothing}
      </section>
    `;
  }

  private renderRequestSidebar() {
    const has_content = this.requests.length > 0;
    return html`
      <div class="list-pane" aria-busy=${String(this.request_list_state === "loading")}>
        <header class="list-pane-header">
          <div>
            <strong>Requests</strong>
            <span>${this.requests.length.toLocaleString()} loaded${this.next_cursor ? " · more available" : ""}</span>
          </div>
          ${this.hasAppliedFilters() ? html`<span class="filter-indicator">Filtered</span>` : nothing}
        </header>
        ${this.request_list_state === "loading"
          ? html`
              <div class="inline-state" role="status">
                <span class="spinner" aria-hidden="true"></span>${has_content ? "Refreshing requests…" : "Loading requests…"}
              </div>
            `
          : nothing}
        ${this.request_list_state === "error"
          ? html`
              <div class="inline-error" role="alert">
                <span>${this.request_list_error}</span>
                <button type="button" @click=${() => void this.loadRequests()}>Retry</button>
              </div>
            `
          : nothing}
        ${has_content
          ? html`
              <request-list
                .requests=${this.requests}
                .selected_key=${this.selectedRequestDay() && this.selected_request_row_id
                  ? requestKey({ day: this.selectedRequestDay()!, row_id: this.selected_request_row_id })
                  : undefined}
                .timezone=${this.timezone}
                @request-select=${(event: Event) => void this.selectRequest(eventDetail<RequestSummary>(event))}
              ></request-list>
            `
          : this.request_list_state === "ready"
            ? html`<p class="empty">No persisted requests match these filters.</p>`
            : this.request_list_state === "idle"
              ? html`<p class="empty">Choose an available request day.</p>`
              : nothing}
        ${this.load_more_error
          ? html`
              <div class="inline-error load-more-error" role="alert">
                <span>${this.load_more_error}</span>
                <button type="button" @click=${() => void this.loadRequests(true)}>Retry</button>
              </div>
            `
          : nothing}
        ${this.next_cursor && has_content
          ? html`
              <div class="list-footer">
                <button type="button" class="secondary-button" ?disabled=${this.loading_more} @click=${() => void this.loadRequests(true)}>
                  ${this.loading_more ? "Loading…" : "Load more"}
                </button>
              </div>
            `
          : has_content && this.request_list_state === "ready"
            ? html`<p class="end-of-list">End of loaded day</p>`
            : nothing}
      </div>
    `;
  }

  private renderSessionsSidebar() {
    if (this.sessions_loading) {
      return html`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading sessions…</div>`;
    }
    if (this.sessions_error) {
      return html`
        <div class="inline-error" role="alert">
          <span>${this.sessions_error}</span>
          <button type="button" @click=${this.retrySessions}>Retry</button>
        </div>
      `;
    }
    if (!this.sessions_loaded) {
      return html`<button type="button" class="primary-button standalone-action" @click=${() => void this.ensureSessionsLoaded()}>
        Load session list
      </button>`;
    }
    return html`
      <session-list
        .sessions=${this.sessions}
        .selected_session_id=${this.selected_session?.session_id}
        .timezone=${this.timezone}
        @session-select=${(event: Event) => void this.loadSession(
          eventDetail<SessionSummary>(event).session_id,
          eventDetail<SessionSummary>(event)
        )}
      ></session-list>
    `;
  }

  private renderSessionDetail() {
    return html`
      ${this.session_detail_loading
        ? html`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading session timeline…</div>`
        : nothing}
      ${this.session_detail_error
        ? html`<div class="inline-error" role="alert"><span>${this.session_detail_error}</span></div>`
        : nothing}
      <session-timeline
        .detail=${this.selected_session_detail}
        .timezone=${this.timezone}
        @request-select=${(event: Event) => void this.openRequest(eventDetail<RequestSummary>(event))}
      ></session-timeline>
    `;
  }

  render() {
    const data_path = this.active_view === "sessions" ? this.info?.sessions_db : this.info?.requests_dir;
    const has_selection = this.active_view === "requests" && Boolean(this.selected_request_id);
    return html`
      <header class="app-header">
        <div class="brand">
          <span class="brand-mark" aria-hidden="true">t</span>
          <div><h1>tokn viewer</h1><p>Local · read only</p></div>
        </div>
        <p class="sensitive-notice">History may contain sensitive prompts and responses.</p>
      </header>
      <main class="app-shell">
        <div class="shell-navigation">
          <nav class="view-navigation" aria-label="Viewer sections">
            <button
              type="button"
              aria-current=${this.active_view === "requests" ? "page" : "false"}
              @click=${() => this.setActiveView("requests")}
            >
              Requests
            </button>
            <button
              type="button"
              aria-current=${this.active_view === "sessions" ? "page" : "false"}
              @click=${() => this.setActiveView("sessions")}
            >
              Sessions
            </button>
          </nav>
          <span class="data-path" title=${data_path ?? ""}>${data_path ?? "Loading data source…"}</span>
        </div>
        ${this.active_view === "requests"
          ? this.renderRequestToolbar()
          : html`
              <section class="session-toolbar">
                <p>Session list from <code>sessions.db</code>; timeline payloads from request history.</p>
                <div class="timezone-toggle" role="group" aria-label="Timestamp timezone">
                  <button type="button" aria-pressed=${String(this.timezone === "local")} @click=${() => this.setTimezone("local")}>Local</button>
                  <button type="button" aria-pressed=${String(this.timezone === "utc")} @click=${() => this.setTimezone("utc")}>UTC</button>
                </div>
              </section>
            `}
        <section class="viewer-grid ${this.active_view === "requests" ? "request-view" : "session-view"} ${has_selection ? "has-selection" : ""}">
          <aside class="sidebar" aria-label=${this.active_view === "requests" ? "Request list" : "Session list"}>
            ${this.active_view === "requests" ? this.renderRequestSidebar() : this.renderSessionsSidebar()}
          </aside>
          <article class="detail-pane" aria-label=${this.active_view === "requests" ? "Request detail" : "Session detail"}>
            ${this.active_view === "requests"
              ? html`
                  <request-detail-view
                    .detail=${this.selected_request_detail}
                    .summary=${this.selected_request}
                    .state=${this.request_detail_state}
                    .error_message=${this.request_detail_error}
                    .active_tab=${this.active_detail_tab}
                    .timezone=${this.timezone}
                    @detail-retry=${this.retryRequestDetail}
                    @detail-close=${() => void this.closeRequestDetail()}
                    @detail-tab-change=${(event: Event) => this.setDetailTab(eventDetail<DetailTab>(event))}
                    @open-session=${(event: Event) => void this.openSession(eventDetail<string>(event))}
                  ></request-detail-view>
                `
              : this.renderSessionDetail()}
          </article>
        </section>
      </main>
    `;
  }
}

customElements.define("viewer-app", ViewerApp);
