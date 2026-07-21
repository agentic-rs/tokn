import { LitElement, html, nothing } from "lit";
import type { PropertyValues } from "lit";
import { fetchJson, isAbortError } from "./api";
import type { LlmItemDetail, LoadState } from "./types";

export class LlmExpandableItem extends LitElement {
  static properties = {
    title: { type: String },
    meta: { type: String },
    preview: { type: String },
    size_label: { type: String },
    load_url: { type: String },
    open: { type: Boolean, state: true },
    load_state: { type: String, state: true },
    value: { attribute: false, state: true },
    error_message: { type: String, state: true }
  };

  declare title: string;
  declare meta: string;
  declare preview: string | undefined;
  declare size_label: string;
  declare load_url: string;
  declare open: boolean;
  declare load_state: LoadState;
  declare value: unknown;
  declare error_message: string | undefined;

  private load_controller: AbortController | undefined;

  constructor() {
    super();
    this.title = "Item";
    this.meta = "";
    this.size_label = "";
    this.load_url = "";
    this.open = false;
    this.load_state = "idle";
  }

  createRenderRoot() {
    return this;
  }

  disconnectedCallback() {
    this.load_controller?.abort();
    super.disconnectedCallback();
  }

  protected willUpdate(changed_properties: PropertyValues<this>) {
    if (!changed_properties.has("load_url")) {
      return;
    }
    this.load_controller?.abort();
    this.load_controller = undefined;
    this.open = false;
    this.load_state = "idle";
    this.value = undefined;
    this.error_message = undefined;
  }

  private toggle(event: Event) {
    this.open = (event.currentTarget as HTMLDetailsElement).open;
    if (this.open && this.load_state === "idle") {
      void this.load();
    }
  }

  private async load() {
    if (!this.load_url) {
      return;
    }
    const load_url = this.load_url;
    this.load_controller?.abort();
    const controller = new AbortController();
    this.load_controller = controller;
    this.load_state = "loading";
    this.error_message = undefined;
    try {
      const detail = await fetchJson<LlmItemDetail>(load_url, controller.signal);
      const expected_index = Number(new URL(load_url, window.location.origin).searchParams.get("index"));
      if (this.load_controller !== controller || this.load_url !== load_url) {
        return;
      }
      if (!Number.isInteger(expected_index) || detail.index !== expected_index) {
        throw new Error("LLM item response did not match the requested index");
      }
      this.value = detail.value;
      this.load_state = "ready";
    } catch (error) {
      if (this.load_controller !== controller || isAbortError(error)) {
        return;
      }
      this.load_state = "error";
      this.error_message = error instanceof Error ? error.message : "Unable to load item";
    } finally {
      if (this.load_controller === controller) {
        this.load_controller = undefined;
      }
    }
  }

  render() {
    return html`
      <details class="llm-expandable-item" ?open=${this.open} @toggle=${this.toggle}>
        <summary>
          <span class="llm-expandable-chevron" aria-hidden="true">›</span>
          <span class="llm-expandable-heading"><strong>${this.title}</strong><small>${this.meta}</small></span>
          <span class="llm-expandable-preview">${this.preview ?? "Non-text content"}</span>
          <span class="llm-expandable-size">${this.size_label}</span>
        </summary>
        <div class="llm-expandable-content">
          ${this.load_state === "loading"
            ? html`<div class="llm-item-loading"><span class="spinner" aria-hidden="true"></span>Loading full content…</div>`
            : this.load_state === "error"
              ? html`<div class="llm-item-error" role="alert"><span>${this.error_message}</span><button type="button" @click=${() => void this.load()}>Retry</button></div>`
              : this.load_state === "ready"
                ? html`<pre>${JSON.stringify(this.value, null, 2)}</pre>`
                : nothing}
        </div>
      </details>
    `;
  }
}

customElements.define("llm-expandable-item", LlmExpandableItem);
