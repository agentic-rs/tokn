import { LitElement, html, nothing } from "lit";
import type { PropertyValues } from "lit";
import { fetchJson, isAbortError } from "./api";
import "./payload-panel";
import type { LoadState, RequestPayload } from "./types";
import type { WebSearchOperation } from "./web-search";
import { inspectWebSearch, safeHttpUrl, webSearchOperationSummary } from "./web-search";

function formatBytes(value: number): string {
  if (value < 1_000) {
    return `${value} B`;
  }
  if (value < 1_000_000) {
    return `${(value / 1_000).toFixed(1)} KB`;
  }
  return `${(value / 1_000_000).toFixed(1)} MB`;
}

function operationKindLabel(operation: WebSearchOperation): string {
  return {
    search_query: "Query",
    open: "Open",
    click: "Click",
    find: "Find"
  }[operation.kind];
}

function operationDetail(operation: WebSearchOperation): string | undefined {
  switch (operation.kind) {
    case "search_query": {
      const filters = [];
      if (operation.domains.length > 0) {
        filters.push(`Domains: ${operation.domains.join(", ")}`);
      }
      if (operation.recency_days !== undefined) {
        filters.push(`Last ${operation.recency_days} days`);
      }
      return filters.join(" · ") || undefined;
    }
    case "open":
      return operation.line_number === undefined ? undefined : `Starting at line ${operation.line_number}`;
    case "click":
      return `Link ${operation.link_id}`;
    case "find":
      return `Pattern: ${operation.pattern}`;
  }
}

export class WebSearchDetail extends LitElement {
  static properties = {
    request_url: { type: String },
    response_url: { type: String },
    load_state: { type: String, state: true },
    request_payload: { attribute: false, state: true },
    response_payload: { attribute: false, state: true },
    error_message: { type: String, state: true }
  };

  declare request_url: string;
  declare response_url: string;
  declare load_state: LoadState;
  declare request_payload: unknown;
  declare response_payload: unknown;
  declare error_message: string | undefined;

  private load_controller: AbortController | undefined;

  constructor() {
    super();
    this.request_url = "";
    this.response_url = "";
    this.load_state = "idle";
  }

  createRenderRoot() {
    return this;
  }

  disconnectedCallback() {
    this.load_controller?.abort();
    super.disconnectedCallback();
  }

  protected updated(changed_properties: PropertyValues<this>) {
    if (changed_properties.has("request_url") || changed_properties.has("response_url")) {
      void this.load();
    }
  }

  private async load() {
    if (!this.request_url || !this.response_url) {
      return;
    }
    const request_url = this.request_url;
    const response_url = this.response_url;
    this.load_controller?.abort();
    const controller = new AbortController();
    this.load_controller = controller;
    this.load_state = "loading";
    this.error_message = undefined;
    try {
      const [request, response] = await Promise.all([
        fetchJson<RequestPayload>(request_url, controller.signal),
        fetchJson<RequestPayload>(response_url, controller.signal)
      ]);
      if (this.load_controller !== controller
        || this.request_url !== request_url
        || this.response_url !== response_url) {
        return;
      }
      if (request.field !== "inbound_req_body" || response.field !== "inbound_resp_body") {
        throw new Error("Search payload response did not match the requested fields");
      }
      this.request_payload = request.value;
      this.response_payload = response.value;
      this.load_state = "ready";
    } catch (error) {
      if (this.load_controller !== controller || isAbortError(error)) {
        return;
      }
      this.load_state = "error";
      this.error_message = error instanceof Error ? error.message : "Unable to load web search";
    } finally {
      if (this.load_controller === controller) {
        this.load_controller = undefined;
      }
    }
  }

  render() {
    if (this.load_state === "loading" || this.load_state === "idle") {
      return html`
        <section class="web-search-inspection web-search-state" aria-label="Web search" aria-live="polite">
          <span class="spinner" aria-hidden="true"></span>
          <span>Loading web search…</span>
        </section>
      `;
    }
    if (this.load_state === "error") {
      return html`
        <section class="web-search-inspection web-search-state error-state" aria-label="Web search" role="alert">
          <div><strong>Web search could not be loaded</strong><span>${this.error_message}</span></div>
          <button type="button" @click=${() => void this.load()}>Retry</button>
        </section>
      `;
    }

    const inspection = inspectWebSearch(this.request_payload, this.response_payload);
    return html`
      <section class="web-search-inspection" aria-label="Web search">
        <header class="web-search-heading">
          <div>
            <p class="eyebrow">Codex web search</p>
            <h3>${webSearchOperationSummary(inspection.operations)}</h3>
          </div>
          <div class="web-search-metrics">
            <span><strong>${inspection.results.length}</strong> results</span>
            ${inspection.response_length
              ? html`<span><strong>${inspection.response_length}</strong> response</span>`
              : nothing}
            ${inspection.encrypted_output_bytes !== undefined
              ? html`<span title="Decoded encrypted payload size"><strong>${formatBytes(inspection.encrypted_output_bytes)}</strong> encrypted</span>`
              : nothing}
          </div>
        </header>

        <div class="web-search-operations">
          ${inspection.operations.length === 0
            ? html`<p class="web-search-empty">No supported web operation was persisted.</p>`
            : inspection.operations.map((operation, index) => {
                const detail = operationDetail(operation);
                const href = operation.kind === "open" ? safeHttpUrl(operation.value) : undefined;
                return html`
                  <article>
                    <span class="web-search-operation-index">${index + 1}</span>
                    <div>
                      <span class="web-search-operation-kind">${operationKindLabel(operation)}</span>
                      ${href
                        ? html`<a href=${href} target="_blank" rel="noopener noreferrer"><code>${operation.value}</code></a>`
                        : html`<code>${operation.value}</code>`}
                      ${detail ? html`<p>${detail}</p>` : nothing}
                    </div>
                  </article>
                `;
              })}
        </div>

        <dl class="web-search-settings">
          <div><dt>Caller</dt><dd>${inspection.allowed_callers.join(", ") || "—"}</dd></div>
          <div><dt>External web access</dt><dd>${inspection.external_web_access === undefined ? "—" : String(inspection.external_web_access)}</dd></div>
        </dl>

        <div class="web-search-results">
          <h4>Results</h4>
          ${inspection.results.length === 0
            ? html`<p class="web-search-empty">No structured results were returned.</p>`
            : inspection.results.map((result, index) => {
                const href = safeHttpUrl(result.url);
                return html`
                  <article class="web-search-result">
                    <span class="web-search-result-index">${index + 1}</span>
                    <div>
                      <div class="web-search-result-title">
                        ${href
                          ? html`<a href=${href} target="_blank" rel="noopener noreferrer">${result.title ?? result.url}</a>`
                          : html`<strong>${result.title ?? result.url ?? "Untitled result"}</strong>`}
                        <span>${result.domain ?? ""}</span>
                      </div>
                      ${result.snippet ? html`<p>${result.snippet}</p>` : nothing}
                      ${result.ref_id ? html`<code>${result.ref_id}</code>` : nothing}
                    </div>
                  </article>
                `;
              })}
        </div>

        <div class="payload-stack web-search-payloads">
          ${inspection.output
            ? html`<payload-panel label="Synthesized search output" .value=${inspection.output}></payload-panel>`
            : nothing}
          ${inspection.prompt
            ? html`<payload-panel label="Prompt context sent to search" .value=${inspection.prompt}></payload-panel>`
            : nothing}
        </div>
      </section>
    `;
  }
}

customElements.define("web-search-detail", WebSearchDetail);
