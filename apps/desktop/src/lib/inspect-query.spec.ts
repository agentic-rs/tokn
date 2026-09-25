import assert from "node:assert/strict";
import { inspectQuery } from "./inspect-query";
assert.deepEqual(
  inspectQuery(
    "/api/request?day=2026-09-25&request_id=req%2Fone&row_id=9223372036854775807",
  ),
  {
    kind: "request",
    day: "2026-09-25",
    request_id: "req/one",
    row_id: "9223372036854775807",
  },
);
assert.deepEqual(
  inspectQuery("/api/session-node?session_id=a%2Fb&node_id=x%2By"),
  { kind: "session_node", session_id: "a/b", node_id: "x+y" },
);
assert.equal(
  inspectQuery("/api/requests?limit=100&errors_only=true").kind,
  "requests",
);
for (const locator of [
  "https://example.com/api/info",
  "//example.com/api/info",
  "/api/unknown",
  "/api/request",
  "/api/sessions?limit=NaN",
  "/api/sessions?limit=9007199254740993",
])
  assert.throws(() => inspectQuery(locator));
console.log("Inspector query transport tests passed");
