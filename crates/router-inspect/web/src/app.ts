import { LitElement, html, nothing } from "lit";

interface RequestSummary {
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

interface RequestDetail {
  day: string;
  request: Record<string, unknown>;
}

interface SessionSummary {
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

interface SessionDetail {
  session: SessionSummary;
  requests: RequestSummary[];
}

interface ViewerInfo {
  requests_dir: string;
  sessions_db: string;
}

interface LatestRequests {
  day: string | null;
  requests: RequestSummary[];
}

type RequestDayState = "available" | "empty" | "unavailable";

interface RequestDay {
  day: string;
  state: RequestDayState;
}

type ViewName = "requests" | "sessions";

class HttpError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "HttpError";
    this.status = status;
  }
}

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as { error?: string };
    throw new HttpError(response.status, body.error ?? `Request failed (${response.status})`);
  }
  return response.json() as Promise<T>;
}

function formatTime(ts: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium"
  }).format(new Date(ts));
}

function formatStatus(status: number | null): string {
  return status === null ? "—" : String(status);
}

function requestKey(request: RequestSummary): string {
  return `${request.day}:${request.request_id}`;
}

function eventDetail<T>(event: Event): T {
  return (event as CustomEvent<T>).detail;
}

class JsonViewer extends LitElement {
  static properties = {
    label: { type: String },
    value: { attribute: false }
  };

  declare label: string;
  declare value: unknown;

  createRenderRoot() {
    return this;
  }

  render() {
    if (this.value === null || this.value === undefined || this.value === "") {
      return nothing;
    }
    const value = typeof this.value === "string" ? this.value : JSON.stringify(this.value, null, 2);
    return html`
      <section class="payload-section">
        <h3>${this.label}</h3>
        <pre>${value}</pre>
      </section>
    `;
  }
}

class RequestList extends LitElement {
  static properties = {
    requests: { attribute: false },
    selected_key: { type: String }
  };

  declare requests: RequestSummary[];
  declare selected_key: string | undefined;

  createRenderRoot() {
    return this;
  }

  private selectRequest(request: RequestSummary) {
    this.dispatchEvent(new CustomEvent<RequestSummary>("request-select", { detail: request, bubbles: true, composed: true }));
  }

  render() {
    const requests = this.requests ?? [];
    if (requests.length === 0) {
      return html`<p class="empty">No persisted requests match this view.</p>`;
    }
    return html`
      <div class="list" role="list">
        ${requests.map(
          (request) => html`
            <button
              class="list-row ${this.selected_key === requestKey(request) ? "selected" : ""}"
              @click=${() => this.selectRequest(request)}
              role="listitem"
            >
              <span class="status ${request.status !== null && request.status >= 400 ? "error" : ""}">${formatStatus(request.status)}</span>
              <span class="list-row-main">
                <strong>${request.model ?? request.endpoint ?? "unknown request"}</strong>
                <small>${request.provider_id ?? "unknown provider"} · ${formatTime(request.ts)}</small>
              </span>
              <span class="list-row-meta">${request.session_id ?? request.request_id}</span>
            </button>
          `
        )}
      </div>
    `;
  }
}

class SessionList extends LitElement {
  static properties = {
    sessions: { attribute: false },
    selected_session_id: { type: String }
  };

  declare sessions: SessionSummary[];
  declare selected_session_id: string | undefined;

  createRenderRoot() {
    return this;
  }

  private selectSession(session: SessionSummary) {
    this.dispatchEvent(new CustomEvent<SessionSummary>("session-select", { detail: session, bubbles: true, composed: true }));
  }

  render() {
    const sessions = this.sessions ?? [];
    if (sessions.length === 0) {
      return html`<p class="empty">No stored sessions yet.</p>`;
    }
    return html`
      <div class="list" role="list">
        ${sessions.map(
          (session) => html`
            <button
              class="list-row ${this.selected_session_id === session.session_id ? "selected" : ""}"
              @click=${() => this.selectSession(session)}
              role="listitem"
            >
              <span class="session-count">${session.request_count}</span>
              <span class="list-row-main">
                <strong>${session.model ?? session.endpoint ?? "session"}</strong>
                <small>${session.provider_id ?? "unknown provider"} · ${formatTime(session.last_ts)}</small>
              </span>
              <span class="list-row-meta">${session.session_id}</span>
            </button>
          `
        )}
      </div>
    `;
  }
}

class RequestDetailView extends LitElement {
  static properties = {
    detail: { attribute: false },
    selected_session_id: { type: String }
  };

  declare detail: RequestDetail | undefined;
  declare selected_session_id: string | undefined;

  createRenderRoot() {
    return this;
  }

  private openSession(session_id: string) {
    this.dispatchEvent(new CustomEvent<string>("open-session", { detail: session_id, bubbles: true, composed: true }));
  }

  render() {
    if (!this.detail) {
      return html`<section class="empty-detail"><p>Select a request to inspect its persisted metadata and bodies.</p></section>`;
    }
    const request = this.detail.request;
    const metadata = [
      ["request_id", request.request_id],
      ["day", this.detail.day],
      ["timestamp", typeof request.ts === "number" ? formatTime(request.ts) : request.ts],
      ["endpoint", request.endpoint],
      ["status", request.status],
      ["provider", request.provider_id],
      ["account", request.account_id],
      ["model", request.model]
    ];
    const session_id = typeof request.session_id === "string" ? request.session_id : undefined;

    return html`
      <section class="detail-header">
        <div>
          <p class="eyebrow">request</p>
          <h2>${String(request.model ?? request.endpoint ?? "request")}</h2>
          <p class="muted">${String(request.request_id ?? "unknown id")}</p>
        </div>
        ${session_id
          ? html`<button class="link-button" @click=${() => this.openSession(session_id)}>Open session</button>`
          : nothing}
      </section>
      <dl class="metadata-grid">
        ${metadata.map(
          ([label, value]) => html`
            <div>
              <dt>${label}</dt>
              <dd>${value === null || value === undefined ? "—" : String(value)}</dd>
            </div>
          `
        )}
      </dl>
      ${request.request_error ? html`<section class="error-message">${String(request.request_error)}</section>` : nothing}
      <json-viewer label="Inbound request headers" .value=${request.inbound_req_headers}></json-viewer>
      <json-viewer label="Inbound request" .value=${request.inbound_req_body}></json-viewer>
      <json-viewer label="Outbound request headers" .value=${request.outbound_req_headers}></json-viewer>
      <json-viewer label="Outbound request" .value=${request.outbound_req_body}></json-viewer>
      <json-viewer label="Outbound response headers" .value=${request.outbound_resp_headers}></json-viewer>
      <json-viewer label="Outbound response" .value=${request.outbound_resp_body}></json-viewer>
      <json-viewer label="Inbound response headers" .value=${request.inbound_resp_headers}></json-viewer>
      <json-viewer label="Inbound response" .value=${request.inbound_resp_body}></json-viewer>
      <json-viewer label="Request parameters" .value=${request.params_json}></json-viewer>
      <json-viewer label="Usage" .value=${request.usage_json}></json-viewer>
      <json-viewer label="Request context" .value=${request.ctx_json}></json-viewer>
    `;
  }
}

class SessionTimeline extends LitElement {
  static properties = {
    detail: { attribute: false }
  };

  declare detail: SessionDetail | undefined;

  createRenderRoot() {
    return this;
  }

  private selectRequest(request: RequestSummary) {
    this.dispatchEvent(new CustomEvent<RequestSummary>("request-select", { detail: request, bubbles: true, composed: true }));
  }

  render() {
    if (!this.detail) {
      return html`<section class="empty-detail"><p>Select a session to see its request timeline.</p></section>`;
    }
    const { session, requests } = this.detail;
    return html`
      <section class="detail-header">
        <div>
          <p class="eyebrow">request-history timeline</p>
          <h2>${session.model ?? session.endpoint ?? "session"}</h2>
          <p class="muted">${session.session_id}</p>
        </div>
        <span class="session-count">${session.request_count}</span>
      </section>
      <dl class="metadata-grid">
        <div><dt>first seen</dt><dd>${formatTime(session.first_ts)}</dd></div>
        <div><dt>last seen</dt><dd>${formatTime(session.last_ts)}</dd></div>
        <div><dt>provider</dt><dd>${session.provider_id ?? "—"}</dd></div>
        <div><dt>account</dt><dd>${session.account_id ?? "—"}</dd></div>
      </dl>
      <section class="timeline">
        <h3>Request timeline</h3>
        ${requests.map(
          (request) => html`
            <button class="timeline-row" @click=${() => this.selectRequest(request)}>
              <time>${formatTime(request.ts)}</time>
              <span class="status ${request.status !== null && request.status >= 400 ? "error" : ""}">${formatStatus(request.status)}</span>
              <span>${request.model ?? request.endpoint ?? request.request_id}</span>
              <small>${request.request_id}</small>
            </button>
          `
        )}
      </section>
    `;
  }
}

class InspectApp extends LitElement {
  static properties = {
    active_view: { type: String },
    info: { attribute: false },
    requests: { attribute: false },
    request_days: { attribute: false },
    selected_day: { type: String },
    sessions: { attribute: false },
    selected_request: { attribute: false },
    selected_request_detail: { attribute: false },
    selected_session: { attribute: false },
    selected_session_detail: { attribute: false },
    search_query: { type: String },
    loading: { type: Boolean },
    request_days_loading: { type: Boolean },
    request_days_error: { type: String },
    sessions_loading: { type: Boolean },
    sessions_error: { type: String },
    error_message: { type: String }
  };

  declare active_view: ViewName;
  declare info: ViewerInfo | undefined;
  declare requests: RequestSummary[];
  declare request_days: RequestDay[];
  declare selected_day: string | undefined;
  declare sessions: SessionSummary[];
  declare selected_request: RequestSummary | undefined;
  declare selected_request_detail: RequestDetail | undefined;
  declare selected_session: SessionSummary | undefined;
  declare selected_session_detail: SessionDetail | undefined;
  declare search_query: string;
  declare loading: boolean;
  declare request_days_loading: boolean;
  declare request_days_error: string | undefined;
  declare sessions_loading: boolean;
  declare sessions_error: string | undefined;
  declare error_message: string | undefined;

  private request_load_id = 0;
  private request_detail_load_id = 0;
  private session_detail_load_id = 0;
  private request_days_load_id = 0;
  private sessions_loaded = false;

  constructor() {
    super();
    this.active_view = "requests";
    this.requests = [];
    this.request_days = [];
    this.sessions = [];
    this.search_query = "";
    this.loading = true;
    this.request_days_loading = false;
    this.sessions_loading = false;
  }

  createRenderRoot() {
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    void this.loadInitialData();
  }

  private async loadInitialData() {
    const load_id = ++this.request_load_id;
    this.loading = true;
    this.error_message = undefined;
    const [info_result, latest_result] = await Promise.allSettled([
      fetchJson<ViewerInfo>("/api/info"),
      fetchJson<LatestRequests>("/api/requests/latest?limit=100")
    ]);

    if (info_result.status === "fulfilled") {
      this.info = info_result.value;
    }
    if (latest_result.status === "fulfilled" && load_id === this.request_load_id) {
      this.selected_day = latest_result.value.day ?? undefined;
      this.requests = latest_result.value.requests;
      this.clearRequestSelection();
    }

    const error =
      info_result.status === "rejected"
        ? info_result.reason
        : latest_result.status === "rejected"
          ? latest_result.reason
          : undefined;
    if (error) {
      this.error_message = error instanceof Error ? error.message : "Unable to load persisted history";
    }
    if (load_id === this.request_load_id) {
      this.loading = false;
    }

    void this.loadRequestDays();
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
        this.request_days_error = error instanceof Error ? error.message : "Unable to load request day states";
      }
    } finally {
      if (load_id === this.request_days_load_id) {
        this.request_days_loading = false;
      }
    }
  }

  private markRequestDayUnavailable(day: string) {
    const request_day = this.request_days.find((candidate) => candidate.day === day);
    if (request_day) {
      this.request_days = this.request_days.map((candidate) =>
        candidate.day === day ? { ...candidate, state: "unavailable" } : candidate
      );
      return;
    }
    this.request_days = [{ day, state: "unavailable" }, ...this.request_days];
  }

  private clearRequestSelection() {
    this.request_detail_load_id += 1;
    this.selected_request = undefined;
    this.selected_request_detail = undefined;
  }

  private async loadRequests() {
    const day = this.selected_day;
    if (!day) {
      this.requests = [];
      this.clearRequestSelection();
      return;
    }

    const load_id = ++this.request_load_id;
    this.loading = true;
    this.error_message = undefined;
    this.clearRequestSelection();
    this.requests = [];
    try {
      const params = new URLSearchParams({ day, limit: "100" });
      const search = this.search_query.trim();
      if (search) {
        params.set("query", search);
      }
      const requests = await fetchJson<RequestSummary[]>(`/api/requests?${params.toString()}`);
      if (load_id !== this.request_load_id) {
        return;
      }
      this.requests = requests;
    } catch (error) {
      if (load_id === this.request_load_id) {
        if (error instanceof HttpError && error.status === 503 && this.selected_day === day) {
          this.markRequestDayUnavailable(day);
        }
        this.error_message = error instanceof Error ? error.message : "Unable to load requests";
      }
    } finally {
      if (load_id === this.request_load_id) {
        this.loading = false;
      }
    }
  }

  private selectDay(day: string) {
    this.selected_day = day;
    void this.loadRequests();
  }

  private async selectRequest(request: RequestSummary) {
    const load_id = ++this.request_detail_load_id;
    this.selected_request = request;
    this.selected_request_detail = undefined;
    this.error_message = undefined;
    try {
      const detail = await fetchJson<RequestDetail>(
        `/api/request?day=${encodeURIComponent(request.day)}&request_id=${encodeURIComponent(request.request_id)}`
      );
      if (load_id === this.request_detail_load_id) {
        this.selected_request_detail = detail;
      }
    } catch (error) {
      if (load_id === this.request_detail_load_id) {
        this.error_message = error instanceof Error ? error.message : "Unable to load request details";
      }
    }
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
      this.sessions_error = error instanceof Error ? error.message : "Unable to load sessions";
    } finally {
      this.sessions_loading = false;
    }
  }

  private async loadSession(session_id: string, session: SessionSummary | undefined) {
    const load_id = ++this.session_detail_load_id;
    this.selected_session = session;
    this.selected_session_detail = undefined;
    this.error_message = undefined;
    try {
      const detail = await fetchJson<SessionDetail>(
        `/api/session?session_id=${encodeURIComponent(session_id)}&limit=500`
      );
      if (load_id === this.session_detail_load_id) {
        this.selected_session = detail.session;
        this.selected_session_detail = detail;
      }
    } catch (error) {
      if (load_id === this.session_detail_load_id) {
        this.error_message = error instanceof Error ? error.message : "Unable to load session timeline";
      }
    }
  }

  private async selectSession(session: SessionSummary) {
    await this.loadSession(session.session_id, session);
  }

  private async openSession(session_id: string) {
    this.setActiveView("sessions", false);
    const session = this.sessions.find((candidate) => candidate.session_id === session_id);
    await this.loadSession(session_id, session);
  }

  private async openRequest(request: RequestSummary) {
    this.setActiveView("requests");
    const needs_day_switch = this.selected_day !== request.day;
    const needs_unfiltered_list = Boolean(this.search_query.trim());
    if (needs_day_switch || needs_unfiltered_list) {
      this.selected_day = request.day;
      this.search_query = "";
      await this.loadRequests();
    }
    await this.selectRequest(request);
  }

  private setActiveView(active_view: ViewName, load_sessions = true) {
    this.active_view = active_view;
    if (active_view === "sessions" && load_sessions) {
      void this.ensureSessionsLoaded();
    }
  }

  private submitSearch(event: SubmitEvent) {
    event.preventDefault();
    void this.loadRequests();
  }

  private updateSearch(event: Event) {
    this.search_query = (event.target as HTMLInputElement).value;
  }

  private pickerDays(): RequestDay[] {
    if (!this.selected_day || this.request_days.some((request_day) => request_day.day === this.selected_day)) {
      return this.request_days;
    }
    return [{ day: this.selected_day, state: "available" }, ...this.request_days];
  }

  private renderDayPicker() {
    const request_days = this.pickerDays();
    return html`
      <div class="day-picker-group">
        <div class="day-picker-heading">
          <span class="day-picker-label">Request day (UTC)</span>
          <button
            class="day-refresh"
            ?disabled=${this.request_days_loading}
            title="Refresh request day availability"
            @click=${() => void this.loadRequestDays()}
          >
            Refresh
          </button>
        </div>
        <div class="day-picker" role="group" aria-label="Request day">
          ${request_days.length > 0
            ? request_days.map((request_day) => {
                const available = request_day.state === "available";
                const selected = request_day.day === this.selected_day;
                const state_label =
                  request_day.state === "empty" ? "Empty" : request_day.state === "unavailable" ? "Unavailable" : undefined;
                const title =
                  request_day.state === "empty"
                    ? "No persisted requests for this day"
                    : request_day.state === "unavailable"
                      ? "This request day could not be read"
                      : `Show requests from ${request_day.day}`;
                return html`
                  <button
                    class="day-button ${request_day.state} ${selected ? "selected" : ""}"
                    ?disabled=${!available}
                    aria-pressed=${String(selected)}
                    title=${title}
                    @click=${() => this.selectDay(request_day.day)}
                  >
                    <span>${request_day.day}</span>${state_label ? html`<small>${state_label}</small>` : nothing}
                  </button>
                `;
              })
            : html`<span class="day-picker-empty">${this.request_days_loading ? "Checking request days…" : "No persisted request days."}</span>`}
          ${this.request_days_loading && request_days.length > 0
            ? html`<span class="day-picker-status">Checking days…</span>`
            : nothing}
        </div>
        ${this.request_days_error ? html`<p class="day-picker-error">${this.request_days_error}</p>` : nothing}
      </div>
    `;
  }

  private renderSessionsSidebar() {
    if (this.sessions_loading) {
      return html`<p class="empty">Loading sessions…</p>`;
    }
    if (this.sessions_error) {
      return html`
        <section class="sidebar-message">
          <p class="sidebar-warning">${this.sessions_error}</p>
          <button class="link-button" @click=${() => void this.ensureSessionsLoaded()}>Retry loading sessions</button>
        </section>
      `;
    }
    if (!this.sessions_loaded) {
      return html`
        <section class="sidebar-message">
          <p class="empty">The session list has not been loaded.</p>
          <button class="link-button" @click=${() => void this.ensureSessionsLoaded()}>Load session list</button>
        </section>
      `;
    }
    return html`<session-list
      .sessions=${this.sessions}
      .selected_session_id=${this.selected_session?.session_id}
      @session-select=${(event: Event) => void this.selectSession(eventDetail<SessionSummary>(event))}
    ></session-list>`;
  }

  render() {
    const selected_key = this.selected_request ? requestKey(this.selected_request) : undefined;
    const has_selected_day = Boolean(this.selected_day);
    const data_path = this.active_view === "sessions" ? this.info?.sessions_db : this.info?.requests_dir;
    const data_path_label = this.active_view === "sessions" ? "Loading sessions database…" : "Loading request history…";
    return html`
      <header class="app-header">
        <div>
          <p class="eyebrow">local, read-only viewer</p>
          <h1>tokn inspect</h1>
        </div>
        <p class="sensitive-notice">History may contain sensitive prompts and responses.</p>
      </header>
      <main class="app-shell">
        <nav class="tabs" aria-label="Inspector views">
          <button class=${this.active_view === "requests" ? "active" : ""} @click=${() => this.setActiveView("requests")}>Requests</button>
          <button class=${this.active_view === "sessions" ? "active" : ""} @click=${() => this.setActiveView("sessions")}>Sessions</button>
        </nav>
        <section class="toolbar">
          <div class="toolbar-controls">
            ${this.active_view === "requests"
              ? html`
                  ${this.renderDayPicker()}
                  <form class="request-search" @submit=${this.submitSearch}>
                    <input
                      aria-label="Search requests"
                      .value=${this.search_query}
                      @input=${this.updateSearch}
                      ?disabled=${!has_selected_day}
                      placeholder=${has_selected_day ? "Search request, session, or model" : "Choose an available request day"}
                    />
                    <button type="submit" ?disabled=${!has_selected_day}>Filter</button>
                  </form>
                `
              : html`<p class="muted">Session lists come from sessions.db; timelines use request history.</p>`}
          </div>
          <span class="data-path" title=${data_path ?? ""}>${data_path ?? data_path_label}</span>
        </section>
        ${this.error_message ? html`<section class="error-banner">${this.error_message}</section>` : nothing}
        <section class="viewer-grid ${this.loading ? "loading" : ""}" aria-busy=${String(this.loading)}>
          <aside class="sidebar">
            ${this.active_view === "requests"
              ? this.loading
                ? html`<p class="empty">Loading requests…</p>`
                : html`<request-list
                    .requests=${this.requests}
                    .selected_key=${selected_key}
                    @request-select=${(event: Event) => void this.selectRequest(eventDetail<RequestSummary>(event))}
                  ></request-list>`
              : this.renderSessionsSidebar()}
          </aside>
          <article class="detail-pane">
            ${this.active_view === "requests"
              ? html`<request-detail-view
                  .detail=${this.selected_request_detail}
                  @open-session=${(event: Event) => void this.openSession(eventDetail<string>(event))}
                ></request-detail-view>`
              : html`<session-timeline
                  .detail=${this.selected_session_detail}
                  @request-select=${(event: Event) => void this.openRequest(eventDetail<RequestSummary>(event))}
                ></session-timeline>`}
          </article>
        </section>
      </main>
    `;
  }
}

customElements.define("json-viewer", JsonViewer);
customElements.define("request-list", RequestList);
customElements.define("session-list", SessionList);
customElements.define("request-detail-view", RequestDetailView);
customElements.define("session-timeline", SessionTimeline);
customElements.define("inspect-app", InspectApp);
