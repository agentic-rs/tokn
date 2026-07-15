import { LitElement, html } from "lit";
import { formatTimestamp, requestOutcome, shortId } from "./format";
import type { RequestSummary, SessionDetail, SessionSummary, TimezoneMode } from "./types";

export class SessionList extends LitElement {
  static properties = {
    sessions: { attribute: false },
    selected_session_id: { type: String },
    timezone: { type: String }
  };

  declare sessions: SessionSummary[];
  declare selected_session_id: string | undefined;
  declare timezone: TimezoneMode;

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
      <ul class="session-list" aria-label="Sessions">
        ${sessions.map(
          (session) => html`
            <li>
              <button
                type="button"
                class="session-row ${this.selected_session_id === session.session_id ? "selected" : ""}"
                aria-current=${this.selected_session_id === session.session_id ? "true" : "false"}
                @click=${() => this.selectSession(session)}
              >
                <span class="session-count">${session.request_count}</span>
                <span class="session-row-main">
                  <strong>${session.model ?? session.endpoint ?? "session"}</strong>
                  <small>${session.provider_id ?? "unknown provider"} · ${formatTimestamp(session.last_ts, this.timezone)}</small>
                </span>
                <span class="session-row-id" title=${session.session_id}>${shortId(session.session_id)}</span>
              </button>
            </li>
          `
        )}
      </ul>
    `;
  }
}

export class SessionTimeline extends LitElement {
  static properties = {
    detail: { attribute: false },
    timezone: { type: String }
  };

  declare detail: SessionDetail | undefined;
  declare timezone: TimezoneMode;

  createRenderRoot() {
    return this;
  }

  private selectRequest(request: RequestSummary) {
    this.dispatchEvent(new CustomEvent<RequestSummary>("request-select", { detail: request, bubbles: true, composed: true }));
  }

  render() {
    if (!this.detail) {
      return html`<section class="detail-state"><p>Select a session to see its request timeline.</p></section>`;
    }
    const { session, requests } = this.detail;
    return html`
      <section class="detail-content">
        <header class="detail-header">
          <div class="detail-title">
            <p class="eyebrow">request-history timeline</p>
            <h2>${session.model ?? session.endpoint ?? "session"}</h2>
            <p class="muted">${session.session_id}</p>
          </div>
          <span class="session-count">${session.request_count}</span>
        </header>
        <dl class="metadata-grid">
          <div><dt>First seen</dt><dd>${formatTimestamp(session.first_ts, this.timezone)}</dd></div>
          <div><dt>Last seen</dt><dd>${formatTimestamp(session.last_ts, this.timezone)}</dd></div>
          <div><dt>Provider</dt><dd>${session.provider_id ?? "—"}</dd></div>
          <div><dt>Account</dt><dd>${session.account_id ?? "—"}</dd></div>
        </dl>
        <section class="timeline">
          <h3>Request timeline</h3>
          <ul>
            ${requests.map((request) => {
              const outcome = requestOutcome(request);
              return html`
                <li>
                  <button type="button" class="timeline-row" @click=${() => this.selectRequest(request)}>
                    <time>${formatTimestamp(request.ts, this.timezone)}</time>
                    <span class="status ${outcome.tone}" title=${outcome.title}>${outcome.label}</span>
                    <span>${request.model ?? request.endpoint ?? request.request_id}</span>
                    <small title=${request.request_id}>${shortId(request.request_id)}</small>
                  </button>
                </li>
              `;
            })}
          </ul>
        </section>
      </section>
    `;
  }
}

customElements.define("session-list", SessionList);
customElements.define("session-timeline", SessionTimeline);
