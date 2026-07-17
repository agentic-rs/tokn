import { LitElement, html, nothing } from "lit";
import { formatCompactTimestamp, formatDuration, formatTimestamp, sessionOutcome, shortId } from "./format";
import { buildSessionTree } from "./session-tree";
import type { SessionTreeModel, SessionTreeRow } from "./session-tree";
import type {
  LoadState,
  SessionDetail,
  SessionMessage,
  SessionNodeDetail,
  SessionNodeSummary,
  SessionPart,
  SessionSummary,
  TimezoneMode
} from "./types";

const COMPACT_TREE_LANE_THRESHOLD = 6;
const TREE_LANE_WIDTH = 16;
const TREE_NODE_Y = 25;

function statusOutcome(status: number | null) {
  if (status === null) {
    return { label: "—", tone: "neutral", title: "No response status stored" } as const;
  }
  if (status >= 400) {
    return { label: String(status), tone: "error", title: `Response status: ${status}` } as const;
  }
  if (status >= 300) {
    return { label: String(status), tone: "warning", title: `Response status: ${status}` } as const;
  }
  return { label: String(status), tone: "success", title: `Response status: ${status}` } as const;
}

function roleTone(role: string): string {
  switch (role.toLowerCase()) {
    case "assistant":
      return "assistant";
    case "system":
    case "developer":
      return "system";
    case "tool":
    case "function":
      return "tool";
    default:
      return "user";
  }
}

function displayJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2) ?? String(value);
  } catch {
    return String(value);
  }
}

function formatByteSize(bytes: number): string {
  if (bytes < 1_024) {
    return `${bytes.toLocaleString()} B`;
  }
  const units = ["KiB", "MiB", "GiB"];
  let value = bytes / 1_024;
  let unit = units[0];
  for (const candidate of units.slice(1)) {
    if (value < 1_024) {
      break;
    }
    value /= 1_024;
    unit = candidate;
  }
  return `${value >= 10 ? value.toFixed(0) : value.toFixed(1)} ${unit}`;
}

function requestSectionCopy(reduction_kind: string) {
  switch (reduction_kind) {
    case "message_tree":
      return {
        direction: "Complete",
        title: "Input prefix",
        empty_message: "No semantic input was stored for this observation."
      };
    case "suffix_append":
      return {
        direction: "Appended",
        title: "Input delta",
        empty_message: "No new semantic input was stored for this node."
      };
    case "root_snapshot":
      return {
        direction: "Initial",
        title: "Input snapshot",
        empty_message: "No semantic input was stored for this root snapshot."
      };
    case "conflict_snapshot":
      return {
        direction: "Replaced",
        title: "Replacement snapshot",
        empty_message: "No semantic input was stored for this replacement snapshot."
      };
    default:
      return {
        direction: "Stored",
        title: "Node input",
        empty_message: "No semantic input was stored for this node."
      };
  }
}

function treeLaneX(lane: number): number {
  return (lane + 0.5) * TREE_LANE_WIDTH;
}

function treeLaneClass(lane_count: number): string {
  return `session-tree-lanes-${Math.min(lane_count, COMPACT_TREE_LANE_THRESHOLD)}`;
}

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
    return html`
      <ul class="session-list" aria-label="Sessions">
        ${sessions.map((session) => {
          const selected = this.selected_session_id === session.session_id;
          const outcome = sessionOutcome(session);
          return html`
            <li>
              <button
                type="button"
                class="session-row ${selected ? "selected" : ""}"
                data-session-id=${session.session_id}
                aria-current=${selected ? "true" : "false"}
                @click=${() => this.selectSession(session)}
              >
                <time datetime=${new Date(session.last_ts).toISOString()}>
                  ${formatCompactTimestamp(session.last_ts, this.timezone)}
                </time>
                <span class="status ${outcome.tone}" title=${outcome.title}>${outcome.label}</span>
                <span class="session-row-main">
                  <span class="session-row-title">
                    <strong>${session.model ?? "Unknown model"}</strong>
                    <span>${session.endpoint ?? "unknown endpoint"}</span>
                  </span>
                  <span class="session-row-context">
                    <span>${session.provider_id ?? "unknown provider"}</span>
                    <span aria-hidden="true">·</span>
                    <span>${session.request_count.toLocaleString()} ${session.request_count === 1 ? "node" : "nodes"}</span>
                  </span>
                  <span class="session-row-id" title=${session.session_id}>
                    session ${shortId(session.session_id)}
                  </span>
                </span>
                <span class="session-row-chevron" aria-hidden="true">›</span>
              </button>
            </li>
          `;
        })}
      </ul>
    `;
  }
}

export class SessionDetailView extends LitElement {
  static properties = {
    detail: { attribute: false },
    node_detail: { attribute: false },
    state: { type: String },
    error_message: { type: String },
    node_state: { type: String },
    node_error_message: { type: String },
    selected_node_id: { type: String },
    timezone: { type: String }
  };

  declare detail: SessionDetail | undefined;
  declare node_detail: SessionNodeDetail | undefined;
  declare state: LoadState;
  declare error_message: string | undefined;
  declare node_state: LoadState;
  declare node_error_message: string | undefined;
  declare selected_node_id: string | undefined;
  declare timezone: TimezoneMode;

  createRenderRoot() {
    return this;
  }

  private close() {
    this.dispatchEvent(new CustomEvent("session-close", { bubbles: true, composed: true }));
  }

  private retryDetail() {
    this.dispatchEvent(new CustomEvent("session-retry", { bubbles: true, composed: true }));
  }

  private retryNode() {
    this.dispatchEvent(new CustomEvent("session-node-retry", { bubbles: true, composed: true }));
  }

  private selectNode(node: SessionNodeSummary) {
    this.dispatchEvent(new CustomEvent<SessionNodeSummary>("session-node-select", {
      detail: node,
      bubbles: true,
      composed: true
    }));
  }

  private renderPart(part: SessionPart) {
    switch (part.content.encoding) {
      case "text": {
        const content = part.content.value || html`<span class="faint">Empty text part</span>`;
        const truncation_note = part.content.truncated
          ? html`<p class="session-part-note">Preview truncated · ${formatByteSize(part.byte_length)} stored</p>`
          : nothing;
        return html`<div class="session-part-text">${content}${truncation_note}</div>`;
      }
      case "json":
        return html`
          <details class="session-structured-part">
            <summary>${part.part_type.replaceAll("_", " ")}</summary>
            <pre>${displayJson(part.content.value)}</pre>
          </details>
        `;
      case "binary":
        return html`
          <details class="session-structured-part">
            <summary>${part.part_type.replaceAll("_", " ")} · binary</summary>
            <p>${formatByteSize(part.content.byte_length)} stored. Binary bytes are not returned to the viewer.</p>
          </details>
        `;
      case "omitted":
        return html`
          <details class="session-structured-part">
            <summary>${part.part_type.replaceAll("_", " ")} · omitted</summary>
            <p>
              ${formatByteSize(part.byte_length)} ${part.content.original_encoding} content omitted after reaching the
              ${part.content.reason === "part_limit" ? "per-part byte preview" : "node content-size"} limit.
            </p>
          </details>
        `;
    }
  }

  private renderMessages(messages: SessionMessage[], empty_message: string) {
    if (messages.length === 0) {
      return html`<p class="session-message-empty">${empty_message}</p>`;
    }
    return html`
      <div class="session-message-stack">
        ${messages.map((message) => html`
          <article class="session-message ${roleTone(message.role)}">
            <header>
              <span>${message.role}</span>
              <span>
                ${message.parts.length.toLocaleString()}${message.parts.length === message.parts_total
                  ? ""
                  : ` of ${message.parts_total.toLocaleString()}`} parts
                ${message.status === null ? nothing : html` · status ${message.status}`}
              </span>
            </header>
            <div class="session-message-parts">
              ${message.parts.length > 0
                ? message.parts.map((part) => this.renderPart(part))
                : message.parts_total > 0
                  ? html`
                      <p class="session-message-empty">
                        ${message.parts_total.toLocaleString()} stored parts were omitted from this bounded preview.
                      </p>
                    `
                  : html`<p class="session-message-empty">No stored parts in this message.</p>`}
            </div>
          </article>
        `)}
      </div>
    `;
  }

  private nodeDomId(kind: "trigger" | "content", node_id: string): string {
    return `session-node-${kind}-${encodeURIComponent(node_id)}`;
  }

  private renderNodeGraph(row: SessionTreeRow, lane_count: number) {
    const view_width = lane_count * TREE_LANE_WIDTH;
    const node_x = treeLaneX(row.node_lane);
    const node_path = `M ${node_x} ${TREE_NODE_Y} l 0 0.001`;
    const edge_paths = row.connections.map((connection) => {
      const from_x = treeLaneX(connection.from_lane);
      const to_x = treeLaneX(connection.to_lane);
      const from_y = connection.kind === "parent" ? TREE_NODE_Y : 0;
      return html`
        <path
          class="session-tree-edge ${connection.kind} ${connection.active ? "active" : ""}"
          d=${`M ${from_x} ${from_y} L ${to_x} 100`}
        ></path>
      `;
    });
    const dot_classes = [
      "session-tree-dot",
      row.node.is_head ? "head" : "",
      row.child_count > 1 ? "branch" : "",
      row.has_topology_warning ? "warning" : ""
    ].filter(Boolean).join(" ");
    return html`
      <svg
        viewBox=${`0 0 ${view_width} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${row.starts_here
          ? nothing
          : html`
              <path
                class="session-tree-edge incoming ${row.is_on_head_path ? "active" : ""}"
                d=${`M ${node_x} 0 L ${node_x} ${TREE_NODE_Y}`}
              ></path>
            `}
        ${edge_paths}
        <path class="${dot_classes} outline" d=${node_path}></path>
        <path class="${dot_classes} fill" d=${node_path}></path>
      </svg>
    `;
  }

  private renderNodeGraphContinuation(row: SessionTreeRow, lane_count: number) {
    const view_width = lane_count * TREE_LANE_WIDTH;
    return html`
      <svg
        viewBox=${`0 0 ${view_width} 100`}
        preserveAspectRatio="none"
        focusable="false"
        aria-hidden="true"
      >
        ${row.bottom_lanes.map((_, lane) => html`
          <path
            class="session-tree-edge continuation ${row.bottom_lane_is_active[lane] ? "active" : ""}"
            d=${`M ${treeLaneX(lane)} 0 L ${treeLaneX(lane)} 100`}
          ></path>
        `)}
      </svg>
    `;
  }

  private renderTreeBoundary(
    tree: SessionTreeModel,
    lane_count: number,
    nodes_truncated: boolean,
    omitted_nodes: number,
    loaded_parent?: SessionNodeSummary
  ) {
    if (tree.missing_parent_ids.length === 0) {
      return nothing;
    }
    const view_width = lane_count * TREE_LANE_WIDTH;
    const boundary_lanes = tree.remaining_lanes.length > 0
      ? tree.remaining_lanes.map((_, lane) => lane)
      : tree.missing_parent_ids.map((_, lane) => lane);
    const visible_lanes = [...new Set(boundary_lanes)];
    const label = loaded_parent
      ? "Connects to loaded tree"
      : nodes_truncated
        ? "Earlier ancestry omitted"
        : "Parent nodes unavailable";
    const detail = loaded_parent
      ? `Parent ${shortId(loaded_parent.node_id)} appears in the session tree below.`
      : nodes_truncated
        ? `${omitted_nodes.toLocaleString()} ${omitted_nodes === 1 ? "node falls" : "nodes fall"} outside this bounded tree snapshot.`
        : "The stored parent links point outside the returned session tree.";
    const link_detail = loaded_parent
      ? "Parent link resolved in the loaded snapshot"
      : `${tree.missing_parent_ids.length.toLocaleString()} parent ${tree.missing_parent_ids.length === 1 ? "link" : "links"} outside the snapshot`;
    return html`
      <li class="session-tree-boundary ${loaded_parent ? "loaded-parent" : ""} ${treeLaneClass(lane_count)}">
        <span class="session-tree-boundary-graph" aria-hidden="true">
          <svg viewBox=${`0 0 ${view_width} 100`} preserveAspectRatio="none" focusable="false">
            ${visible_lanes.map((lane) => html`
              <path class="session-tree-edge boundary" d=${`M ${treeLaneX(lane)} 0 L ${treeLaneX(lane)} 48`}></path>
              <path
                class="session-tree-boundary-dot outline"
                d=${`M ${treeLaneX(lane)} 52 l 0 0.001`}
              ></path>
              <path
                class="session-tree-boundary-dot fill"
                d=${`M ${treeLaneX(lane)} 52 l 0 0.001`}
              ></path>
            `)}
          </svg>
        </span>
        <div class="session-tree-boundary-card" role="note">
          <strong>${label}</strong>
          <span>${detail}</span>
          <span title=${loaded_parent?.node_id ?? tree.missing_parent_ids.join(", ")}>${link_detail}</span>
        </div>
      </li>
    `;
  }

  private renderLoadedNodeContent(detail: SessionNodeDetail) {
    const truncation = detail.truncation;
    const request_section = requestSectionCopy(detail.node.reduction_kind);
    const request_messages_omitted = truncation.request_messages.messages_total
      - truncation.request_messages.messages_returned;
    const response_messages_omitted = truncation.response_messages.messages_total
      - truncation.response_messages.messages_returned;
    const content_is_bounded = request_messages_omitted > 0
      || response_messages_omitted > 0
      || truncation.parts_omitted > 0
      || truncation.content_parts_truncated > 0
      || truncation.binary_parts_elided > 0;
    return html`
      ${content_is_bounded
        ? html`
            <div class="session-content-boundary" role="status">
              <strong>Bounded content preview</strong>
              <span>
                ${formatByteSize(truncation.content_bytes_returned)} of
                ${formatByteSize(truncation.content_bytes_total)} inline content returned
                ${request_messages_omitted + response_messages_omitted > 0
                  ? ` · ${(request_messages_omitted + response_messages_omitted).toLocaleString()} messages omitted`
                  : ""}
                ${truncation.parts_omitted > 0
                  ? ` · ${truncation.parts_omitted.toLocaleString()} parts omitted`
                  : ""}
                ${truncation.content_parts_truncated > 0
                  ? ` · ${truncation.content_parts_truncated.toLocaleString()} parts truncated`
                  : ""}
                ${truncation.binary_parts_elided > 0
                  ? ` · ${truncation.binary_parts_elided.toLocaleString()} binary parts represented as metadata`
                  : ""}
              </span>
            </div>
          `
        : nothing}
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">${request_section.direction}</span>
            <h3>${request_section.title}</h3>
          </div>
          <span>
            ${truncation.request_messages.messages_returned.toLocaleString()}
            ${truncation.request_messages.messages_returned === truncation.request_messages.messages_total
              ? ""
              : `of ${truncation.request_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(detail.request_messages, request_section.empty_message)}
      </div>
      <div class="session-conversation-section">
        <header>
          <div>
            <span class="direction-label">Captured</span>
            <h3>Model output</h3>
          </div>
          <span>
            ${truncation.response_messages.messages_returned.toLocaleString()}
            ${truncation.response_messages.messages_returned === truncation.response_messages.messages_total
              ? ""
              : `of ${truncation.response_messages.messages_total.toLocaleString()}`} messages
          </span>
        </header>
        ${this.renderMessages(detail.response_messages, "No semantic output was stored for this node.")}
      </div>
    `;
  }

  private renderNodeContent(node: SessionNodeSummary) {
    if (this.selected_node_id !== node.node_id) {
      return nothing;
    }
    const detail = this.node_detail?.node.node_id === node.node_id ? this.node_detail : undefined;
    const body = this.node_state === "loading"
      ? html`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Loading semantic content…</div>`
      : this.node_state === "error"
        ? html`
            <div class="inline-error" role="alert">
              <span>${this.node_error_message}</span>
              <button type="button" @click=${this.retryNode}>Retry</button>
            </div>
          `
        : detail
          ? this.renderLoadedNodeContent(detail)
          : nothing;
    return html`
      <section
        id=${this.nodeDomId("content", node.node_id)}
        class="session-node-content"
        aria-labelledby=${this.nodeDomId("trigger", node.node_id)}
        aria-live="polite"
        aria-busy=${String(this.node_state === "loading")}
      >
        ${body}
      </section>
    `;
  }

  private renderNode(row: SessionTreeRow, lane_count: number, loaded_parent?: SessionNodeSummary) {
    const node = row.node;
    const selected = node.node_id === this.selected_node_id;
    const outcome = statusOutcome(node.status);
    const parent_is_loaded = Boolean(loaded_parent && node.parent_node_id === loaded_parent.node_id);
    const parent_is_outside_snapshot = row.parent_is_missing && !parent_is_loaded;
    const node_classes = [
      "session-node",
      treeLaneClass(lane_count),
      selected ? "selected" : "",
      row.is_on_head_path ? "head-path" : "",
      parent_is_outside_snapshot ? "boundary-child" : "",
      row.has_topology_warning ? "topology-warning" : ""
    ].filter(Boolean).join(" ");
    const input_count = node.reduction_kind === "message_tree"
      ? node.input_message_count
      : node.request_message_count;
    const input_label = node.reduction_kind === "message_tree" ? "input" : "input delta";
    const output_count = node.reduction_kind === "message_tree"
      ? node.output_message_count
      : node.response_message_count;
    return html`
      <li class=${node_classes}>
        <span class="session-node-graph" aria-hidden="true">
          ${this.renderNodeGraph(row, lane_count)}
        </span>
        <button
          id=${this.nodeDomId("trigger", node.node_id)}
          type="button"
          class="session-node-trigger"
          data-node-id=${node.node_id}
          aria-expanded=${String(selected)}
          aria-controls=${selected ? this.nodeDomId("content", node.node_id) : nothing}
          aria-current=${node.is_head ? "true" : nothing}
          @click=${() => this.selectNode(node)}
        >
          <span class="session-node-primary">
            <time datetime=${new Date(node.ts).toISOString()}>${formatTimestamp(node.ts, this.timezone)}</time>
            <span class="status ${outcome.tone}" title=${outcome.title}>${outcome.label}</span>
            ${row.child_count > 1
              ? html`<span class="branch-badge">${row.child_count.toLocaleString()} branches</span>`
              : nothing}
            ${node.is_head ? html`<span class="head-badge">Current head</span>` : nothing}
          </span>
          <span class="session-node-title">
            <strong>${node.model ?? "Unknown model"}</strong>
            <span>${node.endpoint}</span>
          </span>
          <span class="session-node-context">
            <span>${node.provider_id ?? "unknown provider"}</span>
            <span aria-hidden="true">·</span>
            <span>${input_count.toLocaleString()} ${input_label}</span>
            <span aria-hidden="true">·</span>
            <span>${output_count.toLocaleString()} output</span>
          </span>
          <span class="session-node-id" title=${node.request_id}>
            request ${shortId(node.request_id)} · ${node.parent_node_id ? `parent ${shortId(node.parent_node_id)}` : "root"}
            ${parent_is_outside_snapshot ? " · outside snapshot" : ""}
          </span>
          <span class="visually-hidden">
            ${node.parent_node_id ? `Recorded child of ${node.parent_node_id}.` : "Recorded root node."}
            ${parent_is_outside_snapshot ? " Parent is outside this bounded snapshot." : ""}
            ${parent_is_loaded ? " Parent appears in the loaded session tree." : ""}
            ${row.has_topology_warning ? " Stored parent links contain a topology warning." : ""}
          </span>
        </button>
        ${selected
          ? html`
              <span class="session-node-content-graph" aria-hidden="true">
                ${this.renderNodeGraphContinuation(row, lane_count)}
              </span>
            `
          : nothing}
        ${this.renderNodeContent(node)}
      </li>
    `;
  }

  render() {
    if (!this.detail) {
      if (this.state === "loading") {
        return html`
          <section class="detail-state" aria-live="polite">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
            <span class="spinner" aria-hidden="true"></span>
            <p>Loading semantic session…</p>
          </section>
        `;
      }
      if (this.state === "error") {
        return html`
          <section class="detail-state error-state" role="alert">
            <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
            <strong>Session could not be loaded</strong>
            <p>${this.error_message}</p>
            <button type="button" class="primary-button" @click=${this.retryDetail}>Retry</button>
          </section>
        `;
      }
      return html`
        <section class="detail-state session-empty-state">
          <span class="session-empty-mark" aria-hidden="true">⌁</span>
          <strong>Choose a session</strong>
          <p>Inspect its semantic nodes and the conversation captured in <code>sessions.db</code>.</p>
        </section>
      `;
    }

    const { session, nodes } = this.detail;
    const session_tree = buildSessionTree(nodes);
    const lane_count = Math.max(1, session_tree.max_lane_count);
    const omitted_nodes = Math.max(0, session.request_count - nodes.length);
    const has_omitted_ancestry = session_tree.missing_parent_ids.length > 0;
    const selected_node_is_loaded = Boolean(
      this.selected_node_id && nodes.some((node) => node.node_id === this.selected_node_id)
    );
    const node_detail = this.node_detail;
    const linked_node = !selected_node_is_loaded
      && node_detail
      && node_detail.node.node_id === this.selected_node_id
      ? node_detail.node
      : undefined;
    const linked_tree = linked_node ? buildSessionTree([linked_node]) : undefined;
    const linked_lane_count = linked_tree ? Math.max(1, linked_tree.max_lane_count) : 1;
    const linked_parent = linked_node?.parent_node_id
      ? nodes.find((node) => node.node_id === linked_node.parent_node_id)
      : undefined;
    const session_model = session.model ?? "Unknown model";
    return html`
      <section class="detail-content session-detail-content">
        <header class="detail-header session-detail-header">
          <button type="button" class="mobile-back-button" @click=${this.close}>← Sessions</button>
          <div class="detail-title">
            <p class="eyebrow">session · ${shortId(session.session_id)}</p>
            <h2>${session_model}<span> on ${session.provider_id ?? "unknown provider"}</span></h2>
            <p class="muted" title=${session.session_id}>${session.session_id || "Missing session identifier"}</p>
          </div>
          <button
            type="button"
            class="icon-button"
            aria-label="Refresh session detail"
            title="Refresh session detail"
            @click=${this.retryDetail}
          >
            ↻
          </button>
        </header>
        ${this.state === "loading"
          ? html`<div class="inline-state"><span class="spinner" aria-hidden="true"></span>Refreshing session…</div>`
          : nothing}
        ${this.state === "error"
          ? html`
              <div class="inline-error" role="alert">
                <span>${this.error_message}</span>
                <button type="button" @click=${this.retryDetail}>Retry</button>
              </div>
            `
          : nothing}
        <dl class="session-metadata-grid">
          <div><dt>Semantic nodes</dt><dd>${session.request_count.toLocaleString()}</dd></div>
          <div><dt>Duration</dt><dd>${formatDuration(session.first_ts, session.last_ts)}</dd></div>
          <div><dt>First seen</dt><dd>${formatTimestamp(session.first_ts, this.timezone)}</dd></div>
          <div><dt>Last active</dt><dd>${formatTimestamp(session.last_ts, this.timezone)}</dd></div>
          <div><dt>Endpoint</dt><dd title=${session.endpoint ?? ""}>${session.endpoint ?? "—"}</dd></div>
          <div><dt>Account</dt><dd title=${session.account_id ?? ""}>${session.account_id ?? "—"}</dd></div>
        </dl>
        <section class="session-activity">
          <header class="session-section-header">
            <div>
              <p class="eyebrow">Recorded parent graph</p>
              <h3>Session tree</h3>
            </div>
            <span>
              ${nodes.length.toLocaleString()} loaded · head branch first${this.detail.nodes_truncated ? " · bounded" : ""}
              ${session_tree.max_lane_count > COMPACT_TREE_LANE_THRESHOLD ? " · compressed lanes" : ""}
            </span>
          </header>
          ${this.detail.nodes_truncated
            ? html`
                <p class="session-truncation-note">
                  ${omitted_nodes.toLocaleString()} older nodes are omitted.
                  ${has_omitted_ancestry
                    ? " Amber graph endpoints continue into the omitted ancestry."
                    : " The graph shows every parent link available in this snapshot."}
                </p>
              `
            : nothing}
          ${session_tree.cycle_node_ids.length > 0
            ? html`
                <p class="session-topology-warning" role="alert">
                  ${session_tree.cycle_node_ids.length.toLocaleString()} nodes contain cyclic parent links; their graph
                  edges were detached defensively.
                </p>
              `
            : nothing}
          ${nodes.length > 0
            ? html`
                <p class="session-tree-direction">
                  <span>Leaves and current-head branch</span>
                  <span aria-hidden="true">↓</span>
                  <span>recorded parents</span>
                </p>
              `
            : nothing}
          ${!this.selected_node_id
            ? html`<p class="session-content-hint">Open a node to load its conversation content from <code>sessions.db</code>.</p>`
            : nothing}
          ${this.selected_node_id && !selected_node_is_loaded
            ? html`
                <section class="session-linked-node" aria-label="Directly linked session node">
                  <header>
                    <div>
                      <p class="eyebrow">Direct link</p>
                      <h4>Node outside this activity snapshot</h4>
                    </div>
                    <span>${shortId(this.selected_node_id)}</span>
                  </header>
                  ${linked_tree
                    ? html`
                        <ol class="session-node-list linked-node-list">
                          ${linked_tree.rows.map((row) => this.renderNode(row, linked_lane_count, linked_parent))}
                          ${this.renderTreeBoundary(linked_tree, linked_lane_count, false, 0, linked_parent)}
                        </ol>
                      `
                    : this.node_state === "loading"
                      ? html`
                          <div class="inline-state" role="status" aria-live="polite">
                            <span class="spinner" aria-hidden="true"></span>Loading linked node…
                          </div>
                        `
                      : this.node_state === "error"
                        ? html`
                            <div class="inline-error" role="alert">
                              <span>${this.node_error_message}</span>
                              <button type="button" @click=${this.retryNode}>Retry</button>
                            </div>
                          `
                        : nothing}
                </section>
              `
            : nothing}
          ${nodes.length > 0
            ? html`
                <ol class="session-node-list">
                  ${session_tree.rows.map((row) => this.renderNode(row, lane_count))}
                  ${this.renderTreeBoundary(session_tree, lane_count, this.detail.nodes_truncated, omitted_nodes)}
                </ol>
              `
            : html`<p class="empty">This migrated session has no semantic nodes.</p>`}
        </section>
      </section>
    `;
  }
}

customElements.define("session-list", SessionList);
customElements.define("session-detail-view", SessionDetailView);
