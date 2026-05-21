use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use bytes::Bytes;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};
use tokn_config::{Config, ModelFamily, RouteMode};
use tokn_core::account::{AccountConfig, AccountTier, AuthType, Secret};
use tokn_core::event::{spawn_event_loop, EventBus};
use tokn_mock_server::{MockEndpoint, MockLlmConfig, MockLlmServer, MockResponse, MockRoute};
use tokn_persistence::{read_request_row, RequestEventHandler};
use tokn_router::api::{build_state, router};
use tokn_router::proxy::passthrough_pipeline::{proxy_passthrough_via_pipeline_inner, proxy_switch_via_pipeline_inner};
use tower::ServiceExt;

struct RequestsHarness {
  _tmp: TempDir,
  requests_dir: PathBuf,
  events: Arc<EventBus>,
  event_thread: Option<std::thread::JoinHandle<()>>,
}

impl RequestsHarness {
  fn new() -> Self {
    let tmp = tempfile::tempdir().expect("create temp db dir");
    let requests_dir = tmp.path().join("requests");
    let events = Arc::new(EventBus::new(1024));
    let receiver = events.subscribe();
    let handler = RequestEventHandler::new(requests_dir.clone()).expect("create requests event handler");
    let event_thread = spawn_event_loop(receiver, vec![Box::new(handler)]);
    Self {
      _tmp: tmp,
      requests_dir,
      events,
      event_thread: Some(event_thread),
    }
  }

  async fn row(&self, request_id: &str) -> Map<String, Value> {
    for _ in 0..100 {
      if let Some(row) = read_request_row(&self.requests_dir, request_id).expect("read request row") {
        if row.get("latency_ms").and_then(Value::as_i64).is_some() {
          return row;
        }
      }
      sleep(Duration::from_millis(20)).await;
    }
    panic!("completed request row was not written for {request_id}");
  }

  async fn shutdown(&mut self) {
    self.events.shutdown().await;
    if let Some(thread) = self.event_thread.take() {
      thread.join().expect("join event loop");
    }
  }
}

fn cfg_for(requests_dir: &Path, route_mode: RouteMode) -> Config {
  let mut cfg = Config::default();
  cfg.server.route_mode = route_mode;
  cfg.db.enabled = true;
  cfg.db.requests_dir = Some(requests_dir.to_path_buf());
  cfg.db.archive_extension = None;
  cfg.model_families = vec![ModelFamily {
    name: "glm-family".into(),
    members: vec!["glm-4.7".into(), "glm-5.1".into()],
  }];
  cfg
}

fn zai_account(base_url: &str) -> AccountConfig {
  AccountConfig {
    id: "zai-test-acct".into(),
    provider: "zai-coding-plan".into(),
    enabled: true,
    tier: AccountTier::Active,
    tags: Vec::new(),
    label: None,
    base_url: Some(base_url.to_string()),
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: Some(Secret::new("sk-router-test".into())),
    api_key_expires_at: None,
    access_token: None,
    access_token_expires_at: None,
    id_token: None,
    refresh_token: None,
    provider_account_id: None,
    extra: Default::default(),
    refresh_url: None,
    last_refresh: None,
    settings: Default::default(),
  }
}

fn text(row: &Map<String, Value>, key: &str) -> Option<String> {
  row.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn int(row: &Map<String, Value>, key: &str) -> Option<i64> {
  row.get(key).and_then(Value::as_i64)
}

fn body_json(row: &Map<String, Value>, key: &str) -> Value {
  let Some(value) = row.get(key) else {
    panic!("{key} missing");
  };
  match value {
    Value::String(body) => serde_json::from_str(body).unwrap_or_else(|err| panic!("{key} is not JSON: {err}: {body}")),
    other => other.clone(),
  }
}

fn missing_or_null(row: &Map<String, Value>, key: &str) -> bool {
  row.get(key).map(Value::is_null).unwrap_or(true)
}

fn body_text(row: &Map<String, Value>, key: &str) -> String {
  let Some(value) = row.get(key) else {
    panic!("{key} missing");
  };
  match value {
    Value::String(body) => body.clone(),
    other => panic!("{key} is not text: {other:?}"),
  }
}

#[derive(Clone, Copy)]
struct RouterCase {
  name: &'static str,
  mode: &'static str,
  model: &'static str,
  expected_status: StatusCode,
  expected_upstream_model: Option<&'static str>,
  expected_error: Option<&'static str>,
}

fn assert_router_row(row: &Map<String, Value>, case: RouterCase) {
  assert_eq!(text(row, "mode").as_deref(), Some(case.mode), "{}", case.name);
  assert_eq!(text(row, "method").as_deref(), Some("requests"), "{}", case.name);
  assert_eq!(
    text(row, "inbound_req_method").as_deref(),
    Some("POST"),
    "{}",
    case.name
  );
  assert_eq!(
    text(row, "endpoint").as_deref(),
    Some("chat_completions"),
    "{}",
    case.name
  );
  assert_eq!(text(row, "model").as_deref(), Some(case.model), "{}", case.name);

  if let Some(expected_error) = case.expected_error {
    let error = text(row, "request_error").unwrap_or_default();
    assert!(
      error.contains(expected_error),
      "{} persisted error {error:?} did not contain {expected_error:?}",
      case.name
    );
    assert!(
      int(row, "status").is_none(),
      "{} should not persist a success status",
      case.name
    );
    return;
  }

  assert_eq!(int(row, "status"), Some(200), "{}", case.name);
  assert_eq!(int(row, "outbound_resp_status"), Some(200), "{}", case.name);
  assert!(missing_or_null(row, "request_error"), "{}", case.name);
  assert_eq!(
    text(row, "account_id").as_deref(),
    Some("zai-test-acct"),
    "{}",
    case.name
  );
  assert_eq!(
    text(row, "provider_id").as_deref(),
    Some("zai-coding-plan"),
    "{}",
    case.name
  );
  let outbound = body_json(row, "outbound_req_body");
  assert_eq!(
    outbound["model"],
    Value::String(case.expected_upstream_model.unwrap().into()),
    "{}",
    case.name
  );
}

#[tokio::test]
async fn router_modes_return_expected_results_and_persist_request_rows() {
  let mock = MockLlmServer::start(MockLlmConfig::default()).await;
  let mut harness = RequestsHarness::new();
  let cfg = cfg_for(&harness.requests_dir, RouteMode::Route);
  let state = build_state(&cfg, &[zai_account(mock.base_url())], harness.events.clone()).unwrap();
  let app = router(state);

  let cases = [
    RouterCase {
      name: "route_success",
      mode: "route",
      model: "glm-4.7",
      expected_status: StatusCode::OK,
      expected_upstream_model: Some("glm-4.7"),
      expected_error: None,
    },
    RouterCase {
      name: "fuzzy_success",
      mode: "fuzzy",
      model: "glm-family",
      expected_status: StatusCode::OK,
      expected_upstream_model: Some("glm-family"),
      expected_error: None,
    },
    RouterCase {
      name: "exact_success",
      mode: "exact",
      model: "zai-coding-plan/glm-4.7",
      expected_status: StatusCode::OK,
      expected_upstream_model: Some("glm-4.7"),
      expected_error: None,
    },
    RouterCase {
      name: "passthrough_success",
      mode: "passthrough",
      model: "glm-4.7",
      expected_status: StatusCode::OK,
      expected_upstream_model: Some("glm-4.7"),
      expected_error: None,
    },
    RouterCase {
      name: "exact_failure",
      mode: "exact",
      model: "glm-4.7",
      expected_status: StatusCode::BAD_REQUEST,
      expected_upstream_model: None,
      expected_error: Some("exact mode requires model in 'provider/model' form"),
    },
  ];

  for case in cases {
    let request_id = format!("router-{}", case.name);
    let inbound_body = json!({
      "model": case.model,
      "messages": [{"role": "user", "content": "hello"}],
      "stream": false
    });
    let raw_body = serde_json::to_vec(&inbound_body).unwrap();
    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/v1/chat/completions")
          .header("content-type", "application/json")
          .header("x-request-id", &request_id)
          .header("x-route-mode", case.mode)
          .body(Body::from(raw_body.clone()))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), case.expected_status, "{}", case.name);
    let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  }

  assert_eq!(
    mock.requests().len(),
    4,
    "only successful router cases should hit upstream"
  );
  harness.shutdown().await;
  for case in cases {
    let request_id = format!("router-{}", case.name);
    let row = harness.row(&request_id).await;
    assert_router_row(&row, case);
  }
  mock.shutdown().await;
}

#[tokio::test]
async fn router_stream_returns_sse_and_persists_drained_stream_row() {
  let mock = MockLlmServer::start(MockLlmConfig::default().with_route(MockRoute::chat_completions_stream())).await;
  let mut harness = RequestsHarness::new();
  let cfg = cfg_for(&harness.requests_dir, RouteMode::Route);
  let state = build_state(&cfg, &[zai_account(mock.base_url())], harness.events.clone()).unwrap();
  let app = router(state);
  let request_id = "router-stream-success";
  let inbound_body = json!({
    "model": "glm-4.7",
    "messages": [{"role": "user", "content": "stream please"}],
    "stream": true
  });
  let raw_body = serde_json::to_vec(&inbound_body).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/v1/chat/completions")
        .header("accept", "text/event-stream")
        .header("content-type", "application/json")
        .header("x-request-id", request_id)
        .body(Body::from(raw_body.clone()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response
      .headers()
      .get("content-type")
      .and_then(|value| value.to_str().ok()),
    Some("text/event-stream")
  );
  let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body = std::str::from_utf8(&body).unwrap();
  assert!(body.contains("\"content\":\"hel\""), "{body}");
  assert!(body.contains("\"content\":\"lo\""), "{body}");
  assert!(body.contains("\"completion_tokens\":2"), "{body}");
  assert!(body.contains("data: [DONE]"), "{body}");

  let captured = mock.last_request().expect("upstream request captured");
  assert_eq!(captured.header("accept"), Some("text/event-stream"));
  let outbound: Value = serde_json::from_slice(&captured.body).unwrap();
  assert_eq!(outbound["model"], "glm-4.7");
  assert_eq!(outbound["stream"], true);

  harness.shutdown().await;

  let row = harness.row(request_id).await;
  assert_eq!(text(&row, "mode").as_deref(), Some("route"));
  assert_eq!(text(&row, "method").as_deref(), Some("requests"));
  assert_eq!(text(&row, "endpoint").as_deref(), Some("chat_completions"));
  assert_eq!(text(&row, "model").as_deref(), Some("glm-4.7"));
  assert_eq!(int(&row, "stream"), Some(1));
  assert_eq!(int(&row, "status"), Some(200));
  assert_eq!(int(&row, "outbound_resp_status"), Some(200));
  assert_eq!(int(&row, "inbound_resp_status"), Some(200));
  assert_eq!(int(&row, "input_tok"), Some(3));
  assert_eq!(int(&row, "output_tok"), Some(2));
  let persisted_outbound = body_json(&row, "outbound_req_body");
  assert_eq!(persisted_outbound["model"], "glm-4.7");
  assert_eq!(persisted_outbound["stream"], true);
  assert!(body_text(&row, "outbound_resp_body").contains("\"completion_tokens\":2"));
  assert!(body_text(&row, "inbound_resp_body").contains("\"completion_tokens\":2"));
  assert!(body_text(&row, "inbound_resp_body").contains("data: [DONE]"));
  assert!(missing_or_null(&row, "request_error"));

  mock.shutdown().await;
}

#[derive(Clone, Copy)]
struct ProxyCase {
  name: &'static str,
  upstream_status: StatusCode,
  expected_status: StatusCode,
  expected_error: Option<&'static str>,
}

fn assert_proxy_row(row: &Map<String, Value>, case: ProxyCase, inbound_body: &Bytes) {
  assert_eq!(text(row, "mode").as_deref(), Some("passthrough"), "{}", case.name);
  assert_eq!(text(row, "method").as_deref(), Some("proxy"), "{}", case.name);
  assert_eq!(
    text(row, "endpoint").as_deref(),
    Some("chat_completions"),
    "{}",
    case.name
  );
  assert_eq!(
    text(row, "inbound_req_method").as_deref(),
    Some("POST"),
    "{}",
    case.name
  );
  assert!(
    text(row, "inbound_req_url")
      .as_deref()
      .is_some_and(|url| url.starts_with("http://127.0.0.1:") && url.ends_with("/v1/chat/completions")),
    "{}",
    case.name
  );
  assert_eq!(text(row, "provider_id").as_deref(), Some("127.0.0.1"), "{}", case.name);
  assert_eq!(
    int(row, "outbound_resp_status"),
    Some(case.upstream_status.as_u16() as i64)
  );

  if let Some(expected_error) = case.expected_error {
    let error = text(row, "request_error").unwrap_or_default();
    assert!(
      error.contains(expected_error),
      "{} persisted error {error:?} did not contain {expected_error:?}",
      case.name
    );
    assert!(
      int(row, "status").is_none(),
      "{} should not persist success status",
      case.name
    );
  } else {
    assert_eq!(int(row, "status"), Some(200), "{}", case.name);
    assert!(missing_or_null(row, "request_error"), "{}", case.name);
    assert_eq!(
      body_json(row, "outbound_req_body"),
      serde_json::from_slice::<Value>(inbound_body).unwrap()
    );
    assert_eq!(body_json(row, "inbound_resp_body"), json!({"proxy": true}));
  }
}

#[tokio::test]
async fn proxy_passthrough_modes_return_expected_results_and_persist_request_rows() {
  let mut harness = RequestsHarness::new();
  let cfg = cfg_for(&harness.requests_dir, RouteMode::Passthrough);
  let state = build_state(&cfg, &[], harness.events.clone()).unwrap();

  let cases = [
    ProxyCase {
      name: "proxy_passthrough_success",
      upstream_status: StatusCode::OK,
      expected_status: StatusCode::OK,
      expected_error: None,
    },
    ProxyCase {
      name: "proxy_passthrough_failure",
      upstream_status: StatusCode::INTERNAL_SERVER_ERROR,
      expected_status: StatusCode::INTERNAL_SERVER_ERROR,
      expected_error: Some("upstream 500"),
    },
  ];

  for case in cases {
    let mock = MockLlmServer::start(MockLlmConfig::default().with_route(MockRoute::new(
      MockEndpoint::Custom {
        method: Method::POST,
        path: "/v1/chat/completions".into(),
      },
      MockResponse {
        status: case.upstream_status,
        headers: vec![("content-type".into(), "application/json".into())],
        body: Bytes::from_static(br#"{"proxy":true}"#),
      },
    )))
    .await;
    let mock_addr = mock
      .base_url()
      .strip_prefix("http://")
      .expect("mock uses http")
      .to_string();
    let (host, port) = mock_addr.rsplit_once(':').expect("host:port");
    let port = port.parse::<u16>().expect("mock port");
    let request_id = case.name;
    let inbound_body = Bytes::from_static(br#"{"model":"glm-4.7","messages":[{"role":"user","content":"via proxy"}]}"#);
    let req = Request::builder()
      .method(Method::POST)
      .uri("/v1/chat/completions")
      .header("content-type", "application/json")
      .header("authorization", "Bearer client-token")
      .header("x-request-id", request_id)
      .body(())
      .unwrap();
    let (parts, ()) = req.into_parts();

    let response = proxy_passthrough_via_pipeline_inner(
      &state,
      host,
      port,
      "http",
      Some("127.0.0.1:10000".into()),
      Some("127.0.0.1:4142".into()),
      parts,
      inbound_body.clone(),
    )
    .await;

    assert_eq!(response.status(), case.expected_status, "{}", case.name);
    let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    mock.shutdown().await;
  }

  harness.shutdown().await;
  let inbound_body = Bytes::from_static(br#"{"model":"glm-4.7","messages":[{"role":"user","content":"via proxy"}]}"#);
  for case in cases {
    let row = harness.row(case.name).await;
    assert_proxy_row(&row, case, &inbound_body);
  }
}

#[tokio::test]
async fn proxy_switch_failure_returns_bad_request_and_persists_error_row() {
  let mut harness = RequestsHarness::new();
  let cfg = cfg_for(&harness.requests_dir, RouteMode::Passthrough);
  let state = build_state(&cfg, &[], harness.events.clone()).unwrap();
  let request_id = "proxy-switch-unrecognized";
  let req = Request::builder()
    .method(Method::POST)
    .uri("/v1/chat/completions")
    .header("content-type", "application/json")
    .header("x-request-id", request_id)
    .body(())
    .unwrap();
  let (parts, ()) = req.into_parts();

  let response = proxy_switch_via_pipeline_inner(
    &state,
    "unrecognized.local",
    443,
    "https",
    Some("127.0.0.1:10001".into()),
    Some("127.0.0.1:4142".into()),
    parts,
    Bytes::from_static(br#"{"model":"glm-4.7","messages":[]}"#),
  )
  .await;

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
  let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body: Value = serde_json::from_slice(&body).unwrap();
  assert!(body["error"]["message"]
    .as_str()
    .unwrap_or_default()
    .contains("switch mode requires a recognized provider URL"));

  harness.shutdown().await;

  let row = harness.row(request_id).await;
  assert_eq!(text(&row, "mode").as_deref(), Some("switch"));
  assert_eq!(text(&row, "method").as_deref(), Some("proxy"));
  assert_eq!(text(&row, "endpoint").as_deref(), Some("chat_completions"));
  assert_eq!(text(&row, "inbound_req_method").as_deref(), Some("POST"));
  assert_eq!(
    text(&row, "inbound_req_url").as_deref(),
    Some("https://unrecognized.local/v1/chat/completions")
  );
  assert_eq!(int(&row, "status"), Some(400));
  let error = text(&row, "request_error").unwrap_or_default();
  assert!(
    error.contains("switch mode requires a recognized provider URL"),
    "persisted error was {error:?}"
  );
}
