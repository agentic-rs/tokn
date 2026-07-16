import type { SessionNodeSummary } from "./types";

export type SessionTreeConnectionKind = "continuation" | "parent";

export interface SessionTreeConnection {
  from_lane: number;
  to_lane: number;
  kind: SessionTreeConnectionKind;
  active: boolean;
}

export interface SessionTreeRow {
  node: SessionNodeSummary;
  top_lanes: string[];
  bottom_lanes: string[];
  node_lane: number;
  starts_here: boolean;
  connections: SessionTreeConnection[];
  bottom_lane_is_active: boolean[];
  child_count: number;
  parent_is_missing: boolean;
  is_on_head_path: boolean;
  has_topology_warning: boolean;
}

export interface SessionTreeModel {
  rows: SessionTreeRow[];
  max_lane_count: number;
  missing_parent_ids: string[];
  remaining_lanes: string[];
  cycle_node_ids: string[];
}

type VisitState = "visiting" | "done";

function collectCycleNodes(
  nodes: SessionNodeSummary[],
  node_by_id: Map<string, SessionNodeSummary>
): Set<string> {
  const cycle_node_ids = new Set<string>();
  const resolved = new Set<string>();

  for (const start of nodes) {
    if (resolved.has(start.node_id)) {
      continue;
    }
    const path: string[] = [];
    const path_index = new Map<string, number>();
    let current: SessionNodeSummary | undefined = start;
    while (current && !resolved.has(current.node_id)) {
      const cycle_start = path_index.get(current.node_id);
      if (cycle_start !== undefined) {
        for (const node_id of path.slice(cycle_start)) {
          cycle_node_ids.add(node_id);
        }
        break;
      }
      path_index.set(current.node_id, path.length);
      path.push(current.node_id);
      current = current.parent_node_id ? node_by_id.get(current.parent_node_id) : undefined;
    }
    for (const node_id of path) {
      resolved.add(node_id);
    }
  }

  return cycle_node_ids;
}

function compareNodes(left: SessionNodeSummary, right: SessionNodeSummary, head_path: Set<string>): number {
  const head_order = Number(head_path.has(right.node_id)) - Number(head_path.has(left.node_id));
  if (head_order !== 0) {
    return head_order;
  }
  if (left.ts !== right.ts) {
    return right.ts - left.ts;
  }
  return left.node_id.localeCompare(right.node_id);
}

function collectHeadPath(
  nodes: SessionNodeSummary[],
  node_by_id: Map<string, SessionNodeSummary>,
  cycle_node_ids: Set<string>
): Set<string> {
  const head = [...nodes]
    .filter((node) => node.is_head)
    .sort((left, right) => right.ts - left.ts || left.node_id.localeCompare(right.node_id))[0];
  const path = new Set<string>();
  let current: SessionNodeSummary | undefined = head;
  while (current) {
    if (path.has(current.node_id)) {
      cycle_node_ids.add(current.node_id);
      break;
    }
    path.add(current.node_id);
    current = current.parent_node_id ? node_by_id.get(current.parent_node_id) : undefined;
  }
  return path;
}

function appendPostorder(
  root: SessionNodeSummary,
  children_by_id: Map<string, SessionNodeSummary[]>,
  state: Map<string, VisitState>,
  cycle_node_ids: Set<string>,
  output: SessionNodeSummary[]
) {
  const stack: Array<{ node: SessionNodeSummary; next_child: number }> = [{ node: root, next_child: 0 }];
  while (stack.length > 0) {
    const frame = stack[stack.length - 1];
    const node_state = state.get(frame.node.node_id);
    if (node_state === "done") {
      stack.pop();
      continue;
    }
    if (node_state === undefined) {
      state.set(frame.node.node_id, "visiting");
    }

    const children = children_by_id.get(frame.node.node_id) ?? [];
    if (frame.next_child < children.length) {
      const child = children[frame.next_child];
      frame.next_child += 1;
      const child_state = state.get(child.node_id);
      if (child_state === undefined) {
        stack.push({ node: child, next_child: 0 });
      } else if (child_state === "visiting") {
        cycle_node_ids.add(frame.node.node_id);
        cycle_node_ids.add(child.node_id);
      }
      continue;
    }

    state.set(frame.node.node_id, "done");
    output.push(frame.node);
    stack.pop();
  }
}

function orderedNodes(
  nodes: SessionNodeSummary[],
  node_by_id: Map<string, SessionNodeSummary>,
  children_by_id: Map<string, SessionNodeSummary[]>,
  head_path: Set<string>,
  cycle_node_ids: Set<string>
): SessionNodeSummary[] {
  const compare = (left: SessionNodeSummary, right: SessionNodeSummary) => compareNodes(left, right, head_path);
  for (const children of children_by_id.values()) {
    children.sort(compare);
  }

  const roots = nodes
    .filter((node) =>
      node.parent_node_id === null
      || !node_by_id.has(node.parent_node_id)
      || cycle_node_ids.has(node.node_id)
    )
    .sort(compare);
  const state = new Map<string, VisitState>();
  const output: SessionNodeSummary[] = [];
  for (const root of roots) {
    appendPostorder(root, children_by_id, state, cycle_node_ids, output);
  }

  for (const node of [...nodes].sort(compare)) {
    if (!state.has(node.node_id)) {
      cycle_node_ids.add(node.node_id);
      appendPostorder(node, children_by_id, state, cycle_node_ids, output);
    }
  }
  return output;
}

function buildRows(
  nodes: SessionNodeSummary[],
  children_by_id: Map<string, SessionNodeSummary[]>,
  head_path: Set<string>,
  missing_parent_ids: Set<string>,
  cycle_node_ids: Set<string>
): Pick<SessionTreeModel, "rows" | "max_lane_count" | "remaining_lanes"> {
  const rows: SessionTreeRow[] = [];
  const lanes: string[] = [];
  const processed = new Set<string>();
  let max_lane_count = 0;

  for (const node of nodes) {
    let node_lane = lanes.indexOf(node.node_id);
    const starts_here = node_lane === -1;
    if (starts_here) {
      node_lane = lanes.length;
      lanes.push(node.node_id);
    }
    const top_lanes = [...lanes];
    const connections: SessionTreeConnection[] = [];
    let parent_lane: number | undefined;
    const stored_parent_id = node.parent_node_id;
    const parent_id = stored_parent_id
      && cycle_node_ids.has(node.node_id)
      && cycle_node_ids.has(stored_parent_id)
      ? null
      : stored_parent_id;

    if (parent_id && !processed.has(parent_id)) {
      const existing_parent_lane = lanes.findIndex((lane_id, index) => index !== node_lane && lane_id === parent_id);
      if (existing_parent_lane === -1) {
        lanes[node_lane] = parent_id;
        parent_lane = node_lane;
      } else {
        lanes.splice(node_lane, 1);
        parent_lane = existing_parent_lane - Number(node_lane < existing_parent_lane);
      }
    } else {
      if (parent_id && processed.has(parent_id)) {
        cycle_node_ids.add(node.node_id);
        cycle_node_ids.add(parent_id);
      }
      lanes.splice(node_lane, 1);
    }

    const bottom_lanes = [...lanes];
    for (let from_lane = 0; from_lane < top_lanes.length; from_lane += 1) {
      if (from_lane === node_lane) {
        continue;
      }
      const to_lane = bottom_lanes.indexOf(top_lanes[from_lane]);
      if (to_lane !== -1) {
        connections.push({
          from_lane,
          to_lane,
          kind: "continuation",
          active: head_path.has(top_lanes[from_lane])
        });
      }
    }
    if (parent_lane !== undefined) {
      connections.push({
        from_lane: node_lane,
        to_lane: parent_lane,
        kind: "parent",
        active: head_path.has(node.node_id)
      });
    }

    max_lane_count = Math.max(max_lane_count, top_lanes.length, bottom_lanes.length);
    rows.push({
      node,
      top_lanes,
      bottom_lanes,
      node_lane,
      starts_here,
      connections,
      bottom_lane_is_active: bottom_lanes.map((node_id) => head_path.has(node_id)),
      child_count: children_by_id.get(node.node_id)?.length ?? 0,
      parent_is_missing: Boolean(parent_id && missing_parent_ids.has(parent_id)),
      is_on_head_path: head_path.has(node.node_id),
      has_topology_warning: cycle_node_ids.has(node.node_id)
    });
    processed.add(node.node_id);
  }

  return { rows, max_lane_count, remaining_lanes: [...lanes] };
}

export function buildSessionTree(nodes: SessionNodeSummary[]): SessionTreeModel {
  const node_by_id = new Map<string, SessionNodeSummary>();
  for (const node of nodes) {
    if (!node_by_id.has(node.node_id)) {
      node_by_id.set(node.node_id, node);
    }
  }
  const unique_nodes = [...node_by_id.values()];
  const children_by_id = new Map(unique_nodes.map((node) => [node.node_id, [] as SessionNodeSummary[]]));
  const missing_parent_ids = new Set<string>();
  const cycle_node_ids = collectCycleNodes(unique_nodes, node_by_id);
  for (const node of unique_nodes) {
    const parent_id = node.parent_node_id;
    if (!parent_id) {
      continue;
    }
    if (node_by_id.has(parent_id) && !(cycle_node_ids.has(node.node_id) && cycle_node_ids.has(parent_id))) {
      children_by_id.get(parent_id)?.push(node);
    } else if (!node_by_id.has(parent_id)) {
      missing_parent_ids.add(parent_id);
    }
  }

  const head_path = collectHeadPath(unique_nodes, node_by_id, cycle_node_ids);
  const ordered = orderedNodes(unique_nodes, node_by_id, children_by_id, head_path, cycle_node_ids);
  const row_model = buildRows(ordered, children_by_id, head_path, missing_parent_ids, cycle_node_ids);
  for (const row of row_model.rows) {
    row.has_topology_warning = cycle_node_ids.has(row.node.node_id);
  }
  return {
    ...row_model,
    missing_parent_ids: [...missing_parent_ids].sort(),
    remaining_lanes: row_model.remaining_lanes.filter((lane_id) => missing_parent_ids.has(lane_id)),
    cycle_node_ids: [...cycle_node_ids].sort()
  };
}
