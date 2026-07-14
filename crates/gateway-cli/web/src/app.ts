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
}

type ViewName = "requests" | "sessions";

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as { error?: string };
    throw new Error(body.error ?? `Request failed (${response.status})`);
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

  requests: RequestSummary[] = [];
  declare selected_key: string | undefined;

  createRenderRoot() {
    return this;
  }

  private selectRequest(request: RequestSummary) {
    this.dispatchEvent(new CustomEvent<RequestSummary>("request-select", { detail: request, bubbles: true, composed: true }));
  }

  render() {
    if (this.requests.length === 0) {
      return html`<p class="empty">No persisted requests match this view.</p>`;
    }
    return html`
      <div class="list" role="list">
        ${this.requests.map(
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

  sessions: SessionSummary[] = [];
  declare selected_session_id: string | undefined;

  createRenderRoot() {
    return this;
  }

  private selectSession(session: SessionSummary) {
    this.dispatchEvent(new CustomEvent<SessionSummary>("session-select", { detail: session, bubbles: true, composed: true }));
  }

  render() {
    if (this.sessions.length === 0) {
      return html`<p class="empty">No request records contain a session id yet.</p>`;
    }
    return html`
      <div class="list" role="list">
        ${this.sessions.map(
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
          <p class="eyebrow">inferred session</p>
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
    sessions: { attribute: false },
    selected_request: { attribute: false },
    selected_request_detail: { attribute: false },
    selected_session: { attribute: false },
    selected_session_detail: { attribute: false },
    search_query: { type: String },
    loading: { type: Boolean },
    error_message: { type: String }
  };

  declare active_view: ViewName;
  declare info: ViewerInfo | undefined;
  declare requests: RequestSummary[];
  declare sessions: SessionSummary[];
  declare selected_request: RequestSummary | undefined;
  declare selected_request_detail: RequestDetail | undefined;
  declare selected_session: SessionSummary | undefined;
  declare selected_session_detail: SessionDetail | undefined;
  declare search_query: string;
  declare loading: boolean;
  declare error_message: string | undefined;

  constructor() {
    super();
    this.active_view = "requests";
    this.requests = [];
    this.sessions = [];
    this.search_query = "";
    this.loading = true;
  }

  createRenderRoot() {
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    void this.loadInitialData();
  }

  private async loadInitialData() {
    this.loading = true;
    this.error_message = undefined;
    try {
      const [info, requests, sessions] = await Promise.all([
        fetchJson<ViewerInfo>("/api/info"),
        fetchJson<RequestSummary[]>("/api/requests?limit=100"),
        fetchJson<SessionSummary[]>("/api/sessions?limit=100")
      ]);
      this.info = info;
      this.requests = requests;
      this.sessions = sessions;
    } catch (error) {
      this.error_message = error instanceof Error ? error.message : "Unable to load persisted history";
    } finally {
      this.loading = false;
    }
  }

  private async loadRequests() {
    this.loading = true;
    this.error_message = undefined;
    try {
      const search = this.search_query.trim();
      const query = search ? `&query=${encodeURIComponent(search)}` : "";
      this.requests = await fetchJson<RequestSummary[]>(`/api/requests?limit=100${query}`);
      this.selected_request = undefined;
      this.selected_request_detail = undefined;
    } catch (error) {
      this.error_message = error instanceof Error ? error.message : "Unable to load requests";
    } finally {
      this.loading = false;
    }
  }

  private async selectRequest(request: RequestSummary) {
    this.selected_request = request;
    this.selected_request_detail = undefined;
    this.error_message = undefined;
    try {
      this.selected_request_detail = await fetchJson<RequestDetail>(
        `/api/request?day=${encodeURIComponent(request.day)}&request_id=${encodeURIComponent(request.request_id)}`
      );
    } catch (error) {
      this.error_message = error instanceof Error ? error.message : "Unable to load request details";
    }
  }

  private async selectSession(session: SessionSummary) {
    this.selected_session = session;
    this.selected_session_detail = undefined;
    this.error_message = undefined;
    try {
      this.selected_session_detail = await fetchJson<SessionDetail>(
        `/api/session?session_id=${encodeURIComponent(session.session_id)}&limit=500`
      );
    } catch (error) {
      this.error_message = error instanceof Error ? error.message : "Unable to load session timeline";
    }
  }

  private async openSession(session_id: string) {
    const session = this.sessions.find((candidate) => candidate.session_id === session_id);
    if (!session) {
      this.error_message = "This request references a session that is no longer available in the request history.";
      return;
    }
    this.active_view = "sessions";
    await this.selectSession(session);
  }

  private async openRequest(request: RequestSummary) {
    this.active_view = "requests";
    await this.selectRequest(request);
  }

  private setActiveView(active_view: ViewName) {
    this.active_view = active_view;
  }

  private submitSearch(event: SubmitEvent) {
    event.preventDefault();
    void this.loadRequests();
  }

  private updateSearch(event: Event) {
    this.search_query = (event.target as HTMLInputElement).value;
  }

  render() {
    const selected_key = this.selected_request ? requestKey(this.selected_request) : undefined;
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
          ${this.active_view === "requests"
            ? html`<form @submit=${this.submitSearch}>
                <input
                  aria-label="Search requests"
                  .value=${this.search_query}
                  @input=${this.updateSearch}
                  placeholder="Search request, session, or model"
                />
                <button type="submit">Filter</button>
              </form>`
            : html`<p class="muted">Sessions are inferred from persisted request session ids.</p>`}
          <span class="data-path">${this.info ? this.info.requests_dir : "Loading request history…"}</span>
        </section>
        ${this.error_message ? html`<section class="error-banner">${this.error_message}</section>` : nothing}
        <section class="viewer-grid ${this.loading ? "loading" : ""}">
          <aside class="sidebar">
            ${this.active_view === "requests"
              ? html`<request-list
                  .requests=${this.requests}
                  .selected_key=${selected_key}
                  @request-select=${(event: Event) => void this.selectRequest(eventDetail<RequestSummary>(event))}
                ></request-list>`
              : html`<session-list
                  .sessions=${this.sessions}
                  .selected_session_id=${this.selected_session?.session_id}
                  @session-select=${(event: Event) => void this.selectSession(eventDetail<SessionSummary>(event))}
                ></session-list>`}
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
