import { buildSessionTree } from "./session-tree.js";
import type { SessionNodeSummary } from "./types.js";

function node(node_id: string, parent_node_id: string | null, ts: number, is_head = false): SessionNodeSummary {
  return {
    node_id,
    parent_node_id,
    request_id: node_id,
    ts,
    endpoint: "responses",
    status: 200,
    account_id: null,
    provider_id: "openai",
    model: "gpt-test",
    reduction_kind: parent_node_id ? "suffix_append" : "root_snapshot",
    parent_source: parent_node_id ? "inferred_head" : "none",
    common_prefix_messages: 0,
    request_message_count: 1,
    response_message_count: 1,
    is_head
  };
}

function equal(actual: unknown, expected: unknown, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}\nexpected ${JSON.stringify(expected)}\nreceived ${JSON.stringify(actual)}`);
  }
}

function truthy(value: unknown, message: string) {
  if (!value) {
    throw new Error(message);
  }
}

{
  const model = buildSessionTree([node("root", null, 30), node("middle", "root", 10), node("head", "middle", 20, true)]);
  equal(model.rows.map((row) => row.node.node_id), ["head", "middle", "root"], "chain follows topology, not time");
  equal(model.max_lane_count, 1, "a chain needs one graph lane");
  equal(model.remaining_lanes, [], "a complete chain has no open ancestry");
}

{
  const model = buildSessionTree([
    node("root", null, 1),
    node("alternate", "root", 30),
    node("head", "root", 10, true)
  ]);
  equal(model.rows.map((row) => row.node.node_id), ["head", "alternate", "root"], "the head branch is ordered first");
  equal(model.rows.map((row) => row.child_count), [0, 0, 2], "fork metadata reports both children");
  equal(model.max_lane_count, 2, "a fork opens a second lane");
  const alternate_row = model.rows[1];
  equal(
    alternate_row.connections.map((connection) => [connection.kind, connection.active]),
    [["continuation", true], ["parent", false]],
    "the head lane remains active while an alternate branch crosses it"
  );
  equal(alternate_row.bottom_lane_is_active, [true], "expanded content preserves the active head lane");
}

{
  const model = buildSessionTree([node("b", "missing-b", 10), node("a", "missing-a", 10, true)]);
  equal(model.rows.map((row) => row.node.node_id), ["a", "b"], "head and IDs make detached roots deterministic");
  equal(model.missing_parent_ids, ["missing-a", "missing-b"], "missing ancestry is explicit");
  equal(model.remaining_lanes, ["missing-a", "missing-b"], "missing parents remain open at the boundary");
  truthy(model.rows.every((row) => row.parent_is_missing), "detached rows are marked");
}

{
  const model = buildSessionTree([node("b", null, 1), node("a", null, 1)]);
  equal(model.rows.map((row) => row.node.node_id), ["a", "b"], "tied roots use node ID ordering");
}

{
  const model = buildSessionTree([node("parent", null, 1, true), node("child", "parent", 2)]);
  equal(model.rows.map((row) => row.node.node_id), ["child", "parent"], "an internal head still follows child-before-parent order");
  equal(model.rows.map((row) => row.is_on_head_path), [false, true], "head path metadata is exact");
}

{
  const detached_nodes = Array.from({ length: 8 }, (_, index) =>
    node(`node-${index}`, `missing-${index}`, 1, index === 0)
  );
  const model = buildSessionTree(detached_nodes);
  equal(model.max_lane_count, 8, "logical lanes remain distinct beyond the compact rendering threshold");
  equal(model.rows.map((row) => row.node_lane), [0, 1, 2, 3, 4, 5, 6, 7], "detached lanes never merge");
}

{
  const model = buildSessionTree([node("self", "self", 1, true)]);
  equal(model.cycle_node_ids, ["self"], "a self-parent is reported as a cycle");
  equal(model.rows[0].connections, [], "a self-parent edge is detached");
  equal(model.remaining_lanes, [], "a self-parent does not look like omitted ancestry");
}

{
  const model = buildSessionTree([
    node("a", "b", 1, true),
    node("b", "c", 2),
    node("c", "a", 3)
  ]);
  equal(model.rows.length, 3, "cycles do not loop or drop rows");
  equal(model.cycle_node_ids, ["a", "b", "c"], "every member of a multi-node cycle is reported");
  truthy(model.rows.every((row) => row.has_topology_warning), "every cycle member is marked");
  truthy(model.rows.every((row) => row.connections.length === 0), "all cyclic parent edges are detached");
  equal(model.remaining_lanes, [], "cycle edges do not become omitted ancestry");
}

console.log("session tree model specs passed");
