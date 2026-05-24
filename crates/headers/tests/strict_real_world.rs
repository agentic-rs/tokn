//! Real-world alignment tests for `build_strict` against captures in
//! `fixtures/inbound_real_world.json`.
//!
//! Strict mode targets the SSE-HTTP shape (the canonical superset). When a
//! capture is for a different transport (e.g. chatgpt.com WebSocket upgrade),
//! we still verify that every header the capture carries appears in strict
//! output with the same value — strict is allowed to add SSE-shape headers
//! that the WS capture omits (`Accept`, `Content-Type`, `Content-Length`).
//!
//! Redacted values (`<redacted>`) and ephemeral correlation IDs are checked
//! for presence + format, not byte-equality.

use serde_json::Value;
use tokn_headers::schemas::CodexCliHeaders;
use tokn_headers::{HeaderMap, HeaderName, HeaderSchema, HeaderValue, TemplateVars};

const FIXTURE_JSON: &str = include_str!("fixtures/inbound_real_world.json");

fn load_cell(key: &str) -> HeaderMap {
  let root: serde_json::Map<String, Value> = serde_json::from_str(FIXTURE_JSON).expect("fixture is valid JSON");
  let obj = root
    .get(key)
    .unwrap_or_else(|| panic!("fixture missing cell `{key}`"))
    .as_object()
    .expect("cell is a JSON object");
  let mut map = HeaderMap::with_capacity(obj.len());
  for (n, v) in obj {
    map.insert(
      HeaderName::new(n.as_str()),
      HeaderValue::from_string(v.as_str().expect("string value").to_string()),
    );
  }
  map
}

/// Headers we treat as transport-level and intentionally exclude from
/// codex-cli persona alignment (modelled by overlays or framework layers).
fn ignore(name: &str) -> bool {
  matches!(
    name,
    // WebSocket upgrade transport — modelled as Extra, inbound-only in
    // strict; capture provides them, strict passes them through. They DO
    // appear in output when inbound has them, so they aren't ignored.
    "" // no-op placeholder; kept for future opt-outs
  )
}

#[test]
fn strict_matches_codex_cli_sse_capture() {
  // SSE-HTTP capture: deepseek upstream, codex_exec/0.130.0.
  let inbound = load_cell("deepseek__responses__codex-cli");
  let out = CodexCliHeaders::build_strict(&TemplateVars::default(), &inbound)
    .expect("strict build")
    .dump();

  for (name, value) in inbound.iter() {
    let n = name.as_str();
    if ignore(n) {
      continue;
    }
    let probe = HeaderName::new(n);
    let got = out
      .get(&probe)
      .unwrap_or_else(|| panic!("strict output missing `{n}` (from capture); have keys {:?}", keys(&out)));
    assert_eq!(
      got.as_str(),
      value.as_str(),
      "value mismatch for `{n}`: capture={:?} strict={:?}",
      value.as_str(),
      got.as_str(),
    );
  }
}

#[test]
fn strict_matches_codex_cli_websocket_capture_as_superset() {
  // chatgpt.com WS upgrade capture. Strict is allowed to add SSE-shape
  // headers the WS capture omits (Accept, Content-Type, Content-Length).
  let inbound = load_cell("chatgpt.com__backend-api_codex_responses__codex-cli");
  let out = CodexCliHeaders::build_strict(&TemplateVars::default(), &inbound)
    .expect("strict build")
    .dump();

  // Every header in the capture must appear with the same value (subset check).
  for (name, value) in inbound.iter() {
    let n = name.as_str();
    if ignore(n) {
      continue;
    }
    let probe = HeaderName::new(n);
    let got = out
      .get(&probe)
      .unwrap_or_else(|| panic!("strict output missing `{n}` (from WS capture)"));
    assert_eq!(got.as_str(), value.as_str(), "value mismatch for `{n}`");
  }

  // Strict explicitly adds these SSE-shape headers even when the WS capture
  // lacks them; this is the intentional superset trade-off.
  for added in ["accept", "content-type", "content-length"] {
    assert!(
      out.contains_key(&HeaderName::new(added)),
      "strict should add `{added}` as part of SSE-shape superset"
    );
  }
}

#[test]
fn strict_with_empty_inbound_synthesizes_full_sse_shape() {
  let out = CodexCliHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new())
    .expect("strict build")
    .dump();

  // Required + Standard tier fields must all be populated by strict.
  for expected in [
    "user-agent",
    "authorization",
    "accept",
    "originator",
    "version",
    "content-type",
    "content-length",
    "session_id",
    "thread_id",
    "x-client-request-id",
  ] {
    assert!(
      out.contains_key(&HeaderName::new(expected)),
      "strict empty-inbound output missing `{expected}`"
    );
  }
  // Correlation triple shares a single synthesized UUID.
  let sid = out.get(&HeaderName::new("session_id")).unwrap().as_str();
  let tid = out.get(&HeaderName::new("thread_id")).unwrap().as_str();
  let crid = out.get(&HeaderName::new("x-client-request-id")).unwrap().as_str();
  assert_eq!(sid, tid);
  assert_eq!(sid, crid);
  assert_eq!(sid.len(), 36, "expected canonical UUID form");
}

fn keys(map: &HeaderMap) -> Vec<String> {
  let mut v: Vec<String> = map.iter().map(|(n, _)| n.as_str().to_string()).collect();
  v.sort();
  v
}
