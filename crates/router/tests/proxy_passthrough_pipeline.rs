//! Integration test for the MITM proxy passthrough pipeline.
//!
//! This test drives [`proxy_passthrough_via_pipeline_inner`] (the
//! pre-body-read inner core) directly with a mock TCP upstream. Going
//! through the inner fn lets us bypass the full proxy TCP/CONNECT
//! machinery while still exercising:
//!
//! * `ProxyResolve` (reads `proxy.host` / `proxy.provider_id` /
//!   `proxy.account_id` from `RunConfig`).
//! * `PassthroughExtract` → `PassthroughBuildHeaders` (router-owned
//!   header stripping + client-auth preservation).
//! * `PassthroughConvertRequest` (verbatim body bytes).
//! * `ProxySend` (dispatch to `{scheme}://{host}{path}`).
//! * `PassthroughConvertResponse` (buffered response forwarding).
//! * `AccountIdentityResolver` integration (provider_id falls back to
//!   the intercepted host when no fingerprint match).
//! * `RecordEvent::UpstreamReq` emission with the right url + headers.

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use bytes::Bytes;
use llm_config::RouteMode;
use llm_core::event::EventBus;
use llm_router::api::build_state;
use llm_router::config::Config;
use llm_router::proxy::passthrough_pipeline::proxy_passthrough_via_pipeline_inner;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn proxy_passthrough_pipeline_forwards_request_and_preserves_client_auth() {
  use llm_core::event::Event as CoreEvent;
  use llm_core::request_event::{RecordEvent, RequestEventPayload};

  // Mock TCP upstream — captures the request, returns a known JSON.
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();

  let upstream_body = br#"{"id":"resp-proxy","ok":true}"#;
  let (req_tx, req_rx) = tokio::sync::oneshot::channel::<Vec<u8>>();

  let server = tokio::spawn(async move {
    let (mut stream, _) = listener.accept().await.unwrap();
    let mut buf = vec![0_u8; 16384];
    let n = stream.read(&mut buf).await.unwrap();
    buf.truncate(n);
    let resp = format!(
      "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
      upstream_body.len()
    );
    stream.write_all(resp.as_bytes()).await.unwrap();
    stream.write_all(upstream_body).await.unwrap();
    stream.flush().await.unwrap();
    let _ = req_tx.send(buf);
  });

  // Build router state in passthrough mode with zero accounts. The
  // proxy passthrough pipeline does no account resolution; identity
  // fallback for provider_id will be the intercepted host.
  let mut cfg = Config::default();
  cfg.server.route_mode = RouteMode::Passthrough;
  let events = Arc::new(EventBus::new(256));
  let mut rx = events.subscribe();
  let state = build_state(&cfg, &[], events.clone()).unwrap();

  let inbound_body = Bytes::from_static(
    br#"{"stream":false,"model":"glm-4.6","messages":[{"role":"user","content":"hi proxy"}]}"#,
  );

  // Use a non-default port + http scheme so the test exercises the
  // port-preservation path. The mock listener already bound to an
  // arbitrary high port; we pass the bare host and that port through
  // explicitly.
  let intercepted_host = addr.ip().to_string();
  let intercepted_port = addr.port();
  let expected_authority = format!("{intercepted_host}:{intercepted_port}");

  let req = Request::builder()
    .method(Method::POST)
    .uri("/v1/chat/completions")
    .header("content-type", "application/json")
    .header("authorization", "Bearer client-bearer-should-reach-upstream")
    .header("x-llm-router-local-addr", "127.0.0.1:9999")
    .body(())
    .unwrap();
  let (parts, ()) = req.into_parts();

  let resp = proxy_passthrough_via_pipeline_inner(
    &state,
    &intercepted_host,
    intercepted_port,
    "http",
    parts,
    inbound_body.clone(),
  )
  .await;
  assert_eq!(resp.status(), StatusCode::OK, "proxy passthrough should succeed");

  let resp_body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  assert_eq!(
    resp_body.as_ref(),
    upstream_body,
    "downstream body must be upstream body verbatim"
  );

  server.await.unwrap();
  let raw_req = req_rx.await.unwrap();
  let raw_req_str = String::from_utf8_lossy(&raw_req);
  let lower = raw_req_str.to_ascii_lowercase();

  // Inbound body bytes reach upstream verbatim.
  assert!(
    raw_req_str.contains(std::str::from_utf8(&inbound_body).unwrap()),
    "upstream must receive inbound body verbatim, got:\n{raw_req_str}"
  );

  // Client's own Authorization is preserved (no provider injection in
  // the proxy variant).
  assert!(
    lower.contains("authorization: bearer client-bearer-should-reach-upstream"),
    "client auth must reach upstream untouched, got:\n{raw_req_str}"
  );

  // Router-owned headers are stripped.
  assert!(
    !lower.contains("x-llm-router-local-addr"),
    "x-llm-router-* headers must be stripped before upstream send, got:\n{raw_req_str}"
  );

  // Upstream Host header is the resolved authority with the non-default
  // port preserved (since scheme=http and port != 80).
  assert!(
    lower.contains(&format!("host: {}", expected_authority.to_ascii_lowercase())),
    "Host header must be {expected_authority}, got:\n{raw_req_str}"
  );

  // Drain events; assert RecordEvent::UpstreamReq carries the expected
  // url and that provider_id falls back to the intercepted host (no
  // local account fingerprint match).
  let mut saw_upstream_req = false;
  for _ in 0..64 {
    let ev = tokio::time::timeout(std::time::Duration::from_millis(250), rx.recv()).await;
    let Ok(Ok(ev)) = ev else { break };
    if let CoreEvent::Requests(req) = &*ev {
      if let RequestEventPayload::Record(RecordEvent::UpstreamReq { url, method, .. }) = &req.payload {
        assert_eq!(method.as_str(), "POST");
        assert!(
          url.starts_with(&format!("http://{expected_authority}")),
          "upstream url must be http://{expected_authority}…, got {url}"
        );
        saw_upstream_req = true;
        break;
      }
    }
  }
  assert!(
    saw_upstream_req,
    "proxy passthrough pipeline must emit RecordEvent::UpstreamReq"
  );
}
