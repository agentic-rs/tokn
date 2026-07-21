import { LitElement, html, nothing } from "lit";
import { displayPath, formatTimestamp, numberField } from "./format";
import { buildLlmRequestOverview, cacheReadPercent } from "./llm-request";
import type { TimezoneMode } from "./types";

function displayValue(value: string | undefined): string {
  return value && value.length > 0 ? value : "—";
}

function formatTokens(value: number | undefined): string {
  return value === undefined ? "—" : value.toLocaleString();
}

function formatMilliseconds(value: number | undefined): string {
  if (value === undefined) {
    return "—";
  }
  if (value < 1_000) {
    return `${Math.round(value).toLocaleString()} ms`;
  }
  const seconds = value / 1_000;
  return `${seconds >= 10 ? seconds.toFixed(1) : seconds.toFixed(2)} s`;
}

function statusTone(status: number | undefined): string {
  if (status === undefined) {
    return "neutral";
  }
  if (status >= 400) {
    return "error";
  }
  if (status >= 300) {
    return "warning";
  }
  return "success";
}

function statusLabel(status: number | undefined): string {
  return status === undefined ? "—" : String(status);
}

export class LlmRequestOverview extends LitElement {
  static properties = {
    request: { attribute: false },
    day: { type: String },
    timezone: { type: String }
  };

  declare request: Record<string, unknown>;
  declare day: string;
  declare timezone: TimezoneMode;

  createRenderRoot() {
    return this;
  }

  render() {
    const overview = buildLlmRequestOverview(this.request);
    if (!overview) {
      return nothing;
    }

    const ts = numberField(this.request, "ts");
    const cache_percent = cacheReadPercent(overview.usage);
    const timing_total = overview.latency_ms ?? 0;
    const first_response_percent = timing_total > 0 && overview.first_response_ms !== undefined
      ? Math.min(100, (overview.first_response_ms / timing_total) * 100)
      : 0;
    const pipeline = [overview.pipeline, overview.mode].filter(Boolean).join(" · ");

    return html`
      <section class="llm-overview" aria-label="LLM request overview">
        <section class="llm-route-flow" aria-label="Model request route">
          <div class="llm-route-step">
            <span class="eyebrow">Client</span>
            <strong>${displayValue(overview.client_method)} ${displayPath(overview.client_url)}</strong>
            <small>Response <span class="status-text ${statusTone(overview.client_status)}">${statusLabel(overview.client_status)}</span></small>
          </div>
          <span class="llm-route-arrow" aria-hidden="true">→</span>
          <div class="llm-route-step llm-route-model">
            <span class="eyebrow">${displayValue(overview.provider_id)}</span>
            <strong>${displayValue(overview.model)}</strong>
            <small>${displayValue(overview.endpoint)}${overview.stream === undefined ? "" : overview.stream ? " · streaming" : " · buffered"}</small>
          </div>
          <span class="llm-route-arrow" aria-hidden="true">→</span>
          <div class="llm-route-step">
            <span class="eyebrow">Provider</span>
            <strong>${displayValue(overview.provider_method)} ${displayPath(overview.provider_url)}</strong>
            <small>Response <span class="status-text ${statusTone(overview.provider_status)}">${statusLabel(overview.provider_status)}</span></small>
          </div>
        </section>

        <div class="llm-metrics-grid">
          <section class="llm-metric-panel llm-token-panel" aria-labelledby="llm-token-heading">
            <header>
              <div>
                <p class="eyebrow">${overview.usage.kind ? `${overview.usage.kind} usage` : "Usage"}</p>
                <h3 id="llm-token-heading">Token usage</h3>
              </div>
              ${cache_percent === undefined
                ? nothing
                : html`<span>${cache_percent.toFixed(1)}% of input cached</span>`}
            </header>
            <dl class="llm-token-grid">
              <div class="llm-primary-metric"><dt>Total</dt><dd>${formatTokens(overview.usage.total_tokens)}</dd></div>
              <div><dt>Input</dt><dd>${formatTokens(overview.usage.input_tokens)}</dd></div>
              <div><dt>Output</dt><dd>${formatTokens(overview.usage.output_tokens)}</dd></div>
              <div><dt>Cache read</dt><dd>${formatTokens(overview.usage.cache_read_tokens)}</dd></div>
              ${overview.usage.cache_write_tokens === undefined
                ? nothing
                : html`<div><dt>Cache write</dt><dd>${formatTokens(overview.usage.cache_write_tokens)}</dd></div>`}
              <div><dt>Reasoning</dt><dd>${formatTokens(overview.usage.reasoning_tokens)}</dd></div>
            </dl>
          </section>

          <section class="llm-metric-panel llm-timing-panel" aria-labelledby="llm-timing-heading">
            <header>
              <div>
                <p class="eyebrow">Performance</p>
                <h3 id="llm-timing-heading">Response timing</h3>
              </div>
              <span>${overview.stream ? "Streamed" : overview.stream === false ? "Buffered" : "Mode unknown"}</span>
            </header>
            ${overview.latency_ms !== undefined && overview.first_response_ms !== undefined
              ? html`
                  <div class="llm-timing-bar" title="First response ${formatMilliseconds(overview.first_response_ms)} of ${formatMilliseconds(overview.latency_ms)} total">
                    <span style=${`width: ${first_response_percent}%`}></span>
                  </div>
                `
              : nothing}
            <dl class="llm-timing-grid">
              <div><dt>First response</dt><dd>${formatMilliseconds(overview.first_response_ms)}</dd></div>
              ${overview.stream && overview.streamed_ms !== undefined
                ? html`<div><dt>Streaming</dt><dd>${formatMilliseconds(overview.streamed_ms)}</dd></div>`
                : nothing}
              <div class="llm-primary-metric"><dt>Total</dt><dd>${formatMilliseconds(overview.latency_ms)}</dd></div>
            </dl>
          </section>
        </div>

        <dl class="metadata-grid llm-metadata-grid">
          <div><dt>Timestamp</dt><dd>${ts === undefined ? "—" : formatTimestamp(ts, this.timezone)}</dd></div>
          <div><dt>Storage day</dt><dd>${this.day}</dd></div>
          <div><dt>Account</dt><dd title=${displayValue(overview.account_id)}>${displayValue(overview.account_id)}</dd></div>
          <div><dt>Pipeline</dt><dd title=${pipeline || "—"}>${pipeline || "—"}</dd></div>
        </dl>
      </section>
    `;
  }
}

customElements.define("llm-request-overview", LlmRequestOverview);
