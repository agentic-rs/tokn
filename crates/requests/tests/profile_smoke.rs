//! End-to-end smoke test for the pre-Send pipeline.
//!
//! Assembles a [`Profile::without_send`] with [`DefaultExtract`], a fake
//! [`AccountSelector`], and the [`NoopBuildHeaders`]/[`NoopConvertRequest`]
//! stages (real impls land in PR2 follow-ups). Runs against a synthetic
//! [`RawInbound`] and asserts the event sequence. The pipeline halts at
//! Send via `PipelineError::stop`; subscribers fold the per-stage events
//! to reconstruct the partial outputs.
//!
//! The PR3b full-pipeline test (`full_pipeline_buffered_happy_path`)
//! additionally exercises the real `DefaultSend` + `DefaultConvertResponse`
//! against a canned `reqwest::Response`.

use async_trait::async_trait;
use bytes::Bytes;
use serde_json::Value;
use smol_str::SmolStr;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokn_accounts::AccountHandle;
use tokn_core::account::AccountConfig;
use tokn_core::provider::{
  AuthKind, Endpoint, ModelCache, Provider, ProviderInfo, RequestCtx, Result as ProviderResult,
};
use tokn_core::util::secret::Secret;
use tokn_core::AgentId;
use tokn_headers::HeaderMap;
use tokn_mock_server::{MockAuthConfig, MockLlmConfig, MockLlmServer};
use tokn_provider_copilot::CopilotProvider;
use tokn_provider_openai::CodexProvider;
use tokn_requests::event::{EventPayload, RecordEvent, Stage, StageEvent};
use tokn_requests::pipeline::stages::ConvertedBody;
use tokn_requests::stage_traits::{BuildHeadersStage, ExtractStage, Resolved};
use tokn_requests::stages::{
  AccountSelector, DefaultBuildHeaders, DefaultConvertRequest, DefaultConvertResponse, DefaultExtract, DefaultSend,
  NoopBuildHeaders, NoopConvertRequest, PoolResolve, SelectorOutcome,
};
use tokn_requests::{Event, EventBus, PipelineError, PipelineRunner, Profile, RawInbound, RetryPolicy};

const CODEX_CLI_OPENAI_SEND_HEADERS_YAML: &str = include_str!("fixtures/agent_id_headers/codex-cli_openai_send.yaml");
const OPENCODE_OPENAI_SEND_HEADERS_YAML: &str = include_str!("fixtures/agent_id_headers/opencode_openai_send.yaml");
const CLAUDE_CODE_OPENAI_SEND_HEADERS_YAML: &str =
  include_str!("fixtures/agent_id_headers/claude-code_openai_send.yaml");
const CLINE_OPENAI_SEND_HEADERS_YAML: &str = include_str!("fixtures/agent_id_headers/cline_openai_send.yaml");
const COPILOT_CLI_OPENAI_SEND_HEADERS_YAML: &str =
  include_str!("fixtures/agent_id_headers/copilot-cli_openai_send.yaml");
const CODEX_HEADERS_INPUT_YAML: &str = include_str!("fixtures/provider_headers/codex/input.yaml");
const CODEX_HEADERS_OUTPUT_YAML: &str = include_str!("fixtures/provider_headers/codex/output.yaml");
const COPILOT_HEADERS_INPUT_YAML: &str = include_str!("fixtures/provider_headers/copilot/input.yaml");
const COPILOT_HEADERS_OUTPUT_YAML: &str = include_str!("fixtures/provider_headers/copilot/output.yaml");

#[derive(Debug, PartialEq, Eq)]
struct HeaderFixtureEntry {
  name: String,
  value: String,
}

struct AgentHeaderCase {
  name: &'static str,
  agent_id: AgentId,
  provider_id: &'static str,
  fixture_yaml: &'static str,
}

struct ProviderHeaderCase {
  name: &'static str,
  provider_id: &'static str,
  upstream_endpoint: Endpoint,
  upstream_model: &'static str,
  input_yaml: &'static str,
  output_yaml: &'static str,
  handle: Arc<AccountHandle>,
  bearer_token: Option<&'static str>,
}

/// Minimal `Provider` used only to satisfy the new typed
/// `AccountHandle` requirement on `SelectorOutcome::Selected`.
struct StubProvider {
  info: ProviderInfo,
}

#[async_trait]
impl Provider for StubProvider {
  fn id(&self) -> &str {
    &self.info.id
  }
  fn info(&self) -> &ProviderInfo {
    &self.info
  }
  async fn list_models(&self, _http: &reqwest::Client) -> ProviderResult<Value> {
    Ok(Value::Null)
  }
  async fn chat(&self, _ctx: RequestCtx<'_>) -> ProviderResult<reqwest::Response> {
    unreachable!("smoke test never reaches Send")
  }
}

fn stub_handle(provider_id: &str, account_id: &str) -> Arc<AccountHandle> {
  let info = ProviderInfo {
    id: provider_id.into(),
    aliases: &[],
    display_name: "stub",
    upstream_url: String::new(),
    auth_kind: AuthKind::StaticApiKey,
    default_models: vec![],
    default_endpoints: &[Endpoint::ChatCompletions],
    model_cache: Arc::new(ModelCache::default()),
  };
  let cfg = AccountConfig {
    id: account_id.to_string(),
    provider: provider_id.to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
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
  };
  Arc::new(AccountHandle::new(Arc::new(cfg), Arc::new(StubProvider { info })))
}

struct OkSelector;

#[async_trait]
impl AccountSelector for OkSelector {
  async fn select(
    &self,
    _ctx: &tokn_requests::pipeline::ctx::PipelineCtx,
    _ex: &tokn_requests::stage_traits::Extracted,
  ) -> Result<SelectorOutcome, PipelineError> {
    Ok(SelectorOutcome::Selected {
      account_id: SmolStr::new("acct-1"),
      provider_id: SmolStr::new("zai-coding-plan"),
      upstream_endpoint: Some(Endpoint::ChatCompletions),
      upstream_model: SmolStr::new("glm-4"),
      account_handle: stub_handle("zai-coding-plan", "acct-1"),
    })
  }
}

struct EmptySelector;

#[async_trait]
impl AccountSelector for EmptySelector {
  async fn select(
    &self,
    _ctx: &tokn_requests::pipeline::ctx::PipelineCtx,
    _ex: &tokn_requests::stage_traits::Extracted,
  ) -> Result<SelectorOutcome, PipelineError> {
    Ok(SelectorOutcome::NoAccount)
  }
}

struct PendingSend;

#[async_trait]
impl tokn_requests::stage_traits::SendStage for PendingSend {
  async fn send(
    &self,
    ctx: &tokn_requests::PipelineCtx,
    _extracted: &tokn_requests::stage_traits::Extracted,
    _resolved: &tokn_requests::stage_traits::Resolved,
    _headers: &tokn_requests::stage_traits::BuiltHeaders,
    body: &tokn_requests::stage_traits::ConvertedRequest,
  ) -> Result<tokn_requests::stage_traits::SentResponse, PipelineError> {
    ctx.emit_record(RecordEvent::UpstreamReq {
      method: SmolStr::new("POST"),
      url: SmolStr::new("https://example.test/pending"),
      headers: HeaderMap::new(),
      body: body.upstream_wire_body.clone(),
    });
    std::future::pending::<Result<tokn_requests::stage_traits::SentResponse, PipelineError>>().await
  }
}

fn capture_bus() -> (Arc<EventBus>, Arc<Mutex<Vec<Event>>>) {
  let bus = Arc::new(EventBus::new(256));
  let log: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
  {
    let log = log.clone();
    let mut rx = bus.subscribe();
    tokio::spawn(async move {
      loop {
        match rx.recv().await {
          Ok(arc) => {
            if let tokn_core::event::Event::Requests(ev) = &*arc {
              log.lock().unwrap().push(ev.clone());
            }
          }
          Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
          Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
        }
      }
    });
  }
  (bus, log)
}

fn raw_chat(model: &str) -> RawInbound {
  let body = serde_json::json!({"model": model, "messages": []});
  let decoded = Bytes::from(serde_json::to_vec(&body).unwrap());
  RawInbound {
    request_endpoint: Endpoint::ChatCompletions.into(),
    headers: HeaderMap::new(),
    raw_body: decoded.clone(),
    decoded_body: decoded,
    body_json: body,
    request_id: Some(SmolStr::new("req-smoke-1")),
  }
}

fn raw_responses(model: &str, headers: HeaderMap, stream: bool) -> RawInbound {
  let body = serde_json::json!({
    "model": model,
    "input": "hello from codex",
    "stream": stream,
    "max_output_tokens": 128,
  });
  let decoded = Bytes::from(serde_json::to_vec(&body).unwrap());
  RawInbound {
    request_endpoint: Endpoint::Responses.into(),
    headers,
    raw_body: decoded.clone(),
    decoded_body: decoded,
    body_json: body,
    request_id: Some(SmolStr::new("req-codex-headers")),
  }
}

async fn drain_until_completed(log: &Arc<Mutex<Vec<Event>>>) -> std::sync::MutexGuard<'_, Vec<Event>> {
  // Subscribers run on a spawned tokio task; yield until a `Completed`
  // event is observed (every pipeline run ends with one).
  for _ in 0..1000 {
    {
      let guard = log.lock().unwrap();
      let done = guard
        .iter()
        .any(|e| matches!(&e.payload, EventPayload::Stage(StageEvent::Completed { .. })));
      if done {
        return guard;
      }
    }
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
  }
  panic!("timed out waiting for Completed event");
}

async fn drain_until_completed_attempts(
  log: &Arc<Mutex<Vec<Event>>>,
  expected_attempts: u32,
) -> std::sync::MutexGuard<'_, Vec<Event>> {
  for _ in 0..1000 {
    {
      let guard = log.lock().unwrap();
      let done = guard.iter().any(|e| {
        matches!(
          &e.payload,
          EventPayload::Stage(StageEvent::Completed {
            attempts,
            ..
          }) if *attempts == expected_attempts
        )
      });
      if done {
        return guard;
      }
    }
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
  }
  panic!("timed out waiting for Completed event with attempts={expected_attempts}");
}

async fn drain_until_upstream_req(log: &Arc<Mutex<Vec<Event>>>) {
  for _ in 0..1000 {
    {
      let guard = log.lock().unwrap();
      let saw = guard
        .iter()
        .any(|e| matches!(&e.payload, EventPayload::Record(RecordEvent::UpstreamReq { .. })));
      if saw {
        return;
      }
    }
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
  }
  panic!("timed out waiting for UpstreamReq event");
}

fn known_kinds(events: &[Event]) -> Vec<&'static str> {
  events
    .iter()
    .map(|e| match &e.payload {
      EventPayload::Stage(k) => match k {
        StageEvent::Started { .. } => "started",
        StageEvent::Extract(_) => "extract",
        StageEvent::Resolve(_) => "resolve",
        StageEvent::BuildHeaders(_) => "build_headers",
        StageEvent::ConvertRequest(_) => "convert_request",
        StageEvent::Send(_) => "send",
        StageEvent::ConvertResponse(_) => "convert_response",
        StageEvent::Error { .. } => "error",
        StageEvent::Completed { .. } => "completed",
      },
      EventPayload::Record(_) => "record",
      EventPayload::Custom(c) => c.kind,
    })
    .collect()
}

#[tokio::test]
async fn pre_send_happy_path_emits_expected_event_sequence() {
  let (bus, log) = capture_bus();
  let profile = Arc::new(Profile::without_send(
    "smoke",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(Arc::new(OkSelector))),
    Arc::new(NoopBuildHeaders),
    Arc::new(NoopConvertRequest),
  ));
  let runner = PipelineRunner::new(profile, bus);

  // `without_send` halts at the Send stage via `PipelineError::stop`.
  let err = runner
    .run(raw_chat("input-model"))
    .await
    .expect_err("without_send must return Err(stop) at Send");
  assert!(err.stop, "expected a stop error, got {err:?}");
  assert_eq!(err.stage, Stage::Send);

  let events = drain_until_completed(&log).await;
  let kinds = known_kinds(&events);
  assert_eq!(
    kinds,
    [
      "started",
      "extract",
      "resolve",
      "build_headers",
      "convert_request",
      "error",
      "completed",
    ]
  );

  // The Error event must carry the stop flag verbatim so subscribers can
  // distinguish a deliberate stop from a real failure.
  let (err_stage, stop_flag) = events
    .iter()
    .find_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Error { stage, stop, .. }) => Some((*stage, *stop)),
      _ => None,
    })
    .expect("Error event must be present");
  assert_eq!(err_stage, Stage::Send);
  assert!(stop_flag);

  // Spot-check the Resolve event carries the upstream model and provider.
  let resolve = events.iter().find_map(|e| match &e.payload {
    EventPayload::Stage(StageEvent::Resolve(r)) => Some((
      r.upstream_model.clone(),
      r.provider_id.clone(),
      r.account_id.clone(),
      r.agent_id.clone(),
    )),
    _ => None,
  });
  let (upstream, provider, account, client) = resolve.expect("Resolve event must be present");
  assert_eq!(upstream, "glm-4");
  assert_eq!(provider, "zai-coding-plan");
  assert_eq!(account, "acct-1");
  assert!(client.is_none());
}

#[tokio::test]
async fn pre_send_no_account_emits_error_then_completed_failure() {
  let (bus, log) = capture_bus();
  let profile = Arc::new(Profile::without_send(
    "smoke",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(Arc::new(EmptySelector))),
    Arc::new(NoopBuildHeaders),
    Arc::new(NoopConvertRequest),
  ));
  let runner = PipelineRunner::new(profile, bus);

  let err = runner
    .run(raw_chat("nope"))
    .await
    .expect_err("empty selector must fail at Resolve");
  assert_eq!(err.stage, Stage::Resolve);
  assert!(!err.recoverable);
  assert!(!err.stop, "no-account is a real failure, not a stop");

  let events = drain_until_completed(&log).await;
  let kinds = known_kinds(&events);
  assert_eq!(kinds, ["started", "extract", "error", "completed"]);

  // The error event must mirror the returned error's stage / flags.
  let (stage, recoverable, stop) = events
    .iter()
    .find_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Error {
        stage,
        recoverable,
        stop,
        ..
      }) => Some((*stage, *recoverable, *stop)),
      _ => None,
    })
    .expect("Error event must be present");
  assert_eq!(stage, Stage::Resolve);
  assert!(!recoverable);
  assert!(!stop);

  // The terminal Completed event must report success=false.
  let success = events.iter().find_map(|e| match &e.payload {
    EventPayload::Stage(StageEvent::Completed { success, .. }) => Some(*success),
    _ => None,
  });
  assert_eq!(success, Some(false));
}

// ---------- PR3b: full-pipeline (all six default stages) ----------

/// Stub provider whose `chat` returns a single pre-armed `reqwest::Response`.
/// The trait method takes `&self`, so the response sits behind a `Mutex`.
struct RespondingProvider {
  info: ProviderInfo,
  resp: Mutex<Option<reqwest::Response>>,
}

#[async_trait]
impl Provider for RespondingProvider {
  fn id(&self) -> &str {
    &self.info.id
  }
  fn info(&self) -> &ProviderInfo {
    &self.info
  }
  async fn list_models(&self, _http: &reqwest::Client) -> ProviderResult<Value> {
    Ok(Value::Null)
  }
  async fn chat(&self, _ctx: RequestCtx<'_>) -> ProviderResult<reqwest::Response> {
    Ok(
      self
        .resp
        .lock()
        .unwrap()
        .take()
        .expect("RespondingProvider::chat: no canned response armed"),
    )
  }
}

fn responding_handle(provider_id: &str, account_id: &str, resp: reqwest::Response) -> Arc<AccountHandle> {
  let info = ProviderInfo {
    id: provider_id.into(),
    aliases: &[],
    display_name: "responding",
    upstream_url: String::new(),
    auth_kind: AuthKind::StaticApiKey,
    default_models: vec![],
    default_endpoints: &[Endpoint::ChatCompletions],
    model_cache: Arc::new(ModelCache::default()),
  };
  let cfg = AccountConfig {
    id: account_id.to_string(),
    provider: provider_id.to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
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
  };
  let provider = RespondingProvider {
    info,
    resp: Mutex::new(Some(resp)),
  };
  Arc::new(AccountHandle::new(Arc::new(cfg), Arc::new(provider)))
}

struct CannedSelector {
  handle: Arc<AccountHandle>,
}

#[async_trait]
impl AccountSelector for CannedSelector {
  async fn select(
    &self,
    _ctx: &tokn_requests::pipeline::ctx::PipelineCtx,
    _ex: &tokn_requests::stage_traits::Extracted,
  ) -> Result<SelectorOutcome, PipelineError> {
    Ok(SelectorOutcome::Selected {
      account_id: SmolStr::new(self.handle.config.load().id.clone()),
      provider_id: SmolStr::new(self.handle.provider.id()),
      upstream_endpoint: Some(Endpoint::ChatCompletions),
      upstream_model: SmolStr::new("glm-4"),
      account_handle: self.handle.clone(),
    })
  }
}

struct CodexSelector {
  handle: Arc<AccountHandle>,
}

#[async_trait]
impl AccountSelector for CodexSelector {
  async fn select(
    &self,
    _ctx: &tokn_requests::pipeline::ctx::PipelineCtx,
    _ex: &tokn_requests::stage_traits::Extracted,
  ) -> Result<SelectorOutcome, PipelineError> {
    Ok(SelectorOutcome::Selected {
      account_id: SmolStr::new(self.handle.config.load().id.clone()),
      provider_id: SmolStr::new("codex"),
      upstream_endpoint: Some(Endpoint::Responses),
      upstream_model: SmolStr::new("gpt-5-codex"),
      account_handle: self.handle.clone(),
    })
  }
}

struct FixedAgentExtract {
  agent_id: AgentId,
}

#[async_trait]
impl tokn_requests::stage_traits::ExtractStage for FixedAgentExtract {
  async fn extract(
    &self,
    ctx: &tokn_requests::PipelineCtx,
    raw: RawInbound,
  ) -> Result<tokn_requests::stage_traits::Extracted, PipelineError> {
    let mut extracted = DefaultExtract.extract(ctx, raw).await?;
    extracted.agent_id = Some(self.agent_id.clone());
    Ok(extracted)
  }
}

fn ok_response(status: u16, body: &'static str) -> reqwest::Response {
  let resp = http::Response::builder()
    .status(status)
    .header("content-type", "application/json")
    .body(body)
    .unwrap();
  reqwest::Response::from(resp)
}

fn headers_from_fixture(yaml: &str) -> HeaderMap {
  let entries = load_agent_id_header_fixture(yaml);
  let mut headers = HeaderMap::new();
  for HeaderFixtureEntry { name, value } in entries {
    headers.insert(
      tokn_headers::HeaderName::new(name),
      tokn_headers::HeaderValue::from_string(value),
    );
  }
  headers
}

fn codex_account(base_url: &str) -> Arc<AccountConfig> {
  Arc::new(AccountConfig {
    id: "codex-acct".to_string(),
    provider: "codex".to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: Some(base_url.to_string()),
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
    api_key_expires_at: None,
    access_token: Some(Secret::new("atk-codex".to_string())),
    access_token_expires_at: None,
    id_token: None,
    refresh_token: None,
    provider_account_id: Some("acct-from-provider".to_string()),
    extra: Default::default(),
    refresh_url: None,
    last_refresh: None,
    settings: Default::default(),
  })
}

fn codex_handle(base_url: &str) -> Arc<AccountHandle> {
  let config = codex_account(base_url);
  let provider = CodexProvider::from_account(config.clone()).expect("codex test account should build");
  Arc::new(AccountHandle::new(config, Arc::new(provider)))
}

fn copilot_account() -> Arc<AccountConfig> {
  Arc::new(AccountConfig {
    id: "copilot-acct".to_string(),
    provider: "github-copilot".to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
    api_key_expires_at: None,
    access_token: Some(Secret::new("api-copilot".to_string())),
    access_token_expires_at: Some(4_102_444_800),
    id_token: None,
    refresh_token: Some(Secret::new("gh-copilot".to_string())),
    provider_account_id: None,
    extra: Default::default(),
    refresh_url: None,
    last_refresh: None,
    settings: Default::default(),
  })
}

fn copilot_handle() -> Arc<AccountHandle> {
  let config = copilot_account();
  let provider = CopilotProvider::from_account(config.clone()).expect("copilot test account should build");
  Arc::new(AccountHandle::new(config, Arc::new(provider)))
}

struct RecordingProvider {
  info: ProviderInfo,
  resp: Mutex<Option<reqwest::Response>>,
  seen_client_headers: Arc<Mutex<Option<HeaderMap>>>,
}

#[async_trait]
impl Provider for RecordingProvider {
  fn id(&self) -> &str {
    &self.info.id
  }

  fn info(&self) -> &ProviderInfo {
    &self.info
  }

  async fn list_models(&self, _http: &reqwest::Client) -> ProviderResult<Value> {
    Ok(Value::Null)
  }

  async fn chat(&self, ctx: RequestCtx<'_>) -> ProviderResult<reqwest::Response> {
    *self.seen_client_headers.lock().unwrap() = ctx.client_headers.clone();
    Ok(
      self
        .resp
        .lock()
        .unwrap()
        .take()
        .expect("RecordingProvider::chat: no canned response armed"),
    )
  }
}

fn recording_handle(
  provider_id: &str,
  account_id: &str,
  resp: reqwest::Response,
) -> (Arc<AccountHandle>, Arc<Mutex<Option<HeaderMap>>>) {
  let info = ProviderInfo {
    id: provider_id.into(),
    aliases: &[],
    display_name: "recording",
    upstream_url: String::new(),
    auth_kind: AuthKind::StaticApiKey,
    default_models: vec![],
    default_endpoints: &[Endpoint::ChatCompletions],
    model_cache: Arc::new(ModelCache::default()),
  };
  let cfg = AccountConfig {
    id: account_id.to_string(),
    provider: provider_id.to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
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
  };
  let seen_client_headers = Arc::new(Mutex::new(None));
  let provider = RecordingProvider {
    info,
    resp: Mutex::new(Some(resp)),
    seen_client_headers: seen_client_headers.clone(),
  };
  (
    Arc::new(AccountHandle::new(Arc::new(cfg), Arc::new(provider))),
    seen_client_headers,
  )
}

fn load_agent_id_header_fixture(yaml: &str) -> Vec<HeaderFixtureEntry> {
  let entries: Vec<serde_json::Map<String, Value>> = serde_yaml::from_str(yaml).expect("fixture is a YAML array");
  let mut headers = Vec::with_capacity(entries.len());
  for entry in entries {
    assert_eq!(entry.len(), 1, "each fixture row must contain exactly one header");
    let (name, value) = entry.into_iter().next().expect("fixture row must contain one header");
    let value = value
      .as_str()
      .expect("fixture header value must be a string")
      .to_string();
    headers.push(HeaderFixtureEntry { name, value });
  }
  headers
}

fn header_entries(headers: &HeaderMap) -> Vec<HeaderFixtureEntry> {
  headers
    .iter()
    .map(|(name, value)| HeaderFixtureEntry {
      name: name.original().to_string(),
      value: value.as_str().to_string(),
    })
    .collect()
}

fn assert_headers_match_fixture(headers: &HeaderMap, fixture_yaml: &str, label: &str) {
  assert_eq!(
    header_entries(headers),
    load_agent_id_header_fixture(fixture_yaml),
    "{label}: header fixture mismatch"
  );
}

fn assert_headers_include_fixture(headers: &HeaderMap, fixture_yaml: &str, label: &str) {
  for HeaderFixtureEntry { name, value } in load_agent_id_header_fixture(fixture_yaml) {
    assert_eq!(
      headers.get(&name).map(|value| value.as_str()),
      Some(value.as_str()),
      "{label}: header mismatch for {name}"
    );
  }
}

#[tokio::test]
async fn full_pipeline_buffered_happy_path() {
  let (bus, log) = capture_bus();

  // Canned upstream payload: a tiny chat-completions response.
  let resp = ok_response(
    200,
    r#"{"id":"resp-1","choices":[{"message":{"role":"assistant","content":"hi"}}]}"#,
  );
  let handle = responding_handle("zai-coding-plan", "acct-1", resp);
  let selector = Arc::new(CannedSelector { handle });

  let profile = Arc::new(Profile::full(
    "smoke-full",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new(profile, bus);

  let converted = runner
    .run(raw_chat("glm-4"))
    .await
    .expect("happy-path pipeline must succeed");

  // Full happy-path event sequence: every stage fires exactly once,
  // followed by the terminal Completed marker. The `record` entries are
  // wire-truth captures — the mock provider bypasses
  // `tokn_core::util::http::send`, so `Record::UpstreamReq` is skipped
  // and only `Record::UpstreamResp` (from Send) and
  // `Record::UpstreamBody` (from ConvertResponse) appear.
  let events = drain_until_completed(&log).await;
  let kinds = known_kinds(&events);
  assert_eq!(
    kinds,
    [
      "started",
      "extract",
      "resolve",
      "build_headers",
      "convert_request",
      "record",
      "send",
      "record",
      "convert_response",
      "record",
      "completed",
    ]
  );

  // Completed must report success=true with attempts=1.
  let (success, attempts) = events
    .iter()
    .find_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Completed { success, attempts }) => Some((*success, *attempts)),
      _ => None,
    })
    .expect("Completed event must be present");
  assert!(success);
  assert_eq!(attempts, 1);

  // The converted response must round-trip the canned upstream payload.
  assert_eq!(converted.status, 200);
  match converted.body {
    ConvertedBody::Buffered { body_json, .. } => {
      let body_json = body_json.unwrap();
      assert_eq!(body_json["id"], "resp-1");
      assert_eq!(body_json["choices"][0]["message"]["content"], "hi");
    }
    other => panic!("expected Buffered, got {other:?}"),
  }
}

#[tokio::test]
async fn cancelled_attempt_after_upstream_req_emits_terminal_events() {
  let (bus, log) = capture_bus();
  let selector = Arc::new(OkSelector);
  let profile = Arc::new(Profile::full(
    "smoke-cancelled-at-send",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(NoopBuildHeaders),
    Arc::new(NoopConvertRequest),
    Arc::new(PendingSend),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new(profile, bus);

  let task = tokio::spawn(async move { runner.run(raw_chat("glm-4")).await });
  drain_until_upstream_req(&log).await;
  task.abort();
  let _ = task.await;

  let events = drain_until_completed(&log).await;
  let saw_cancel_error = events.iter().any(|e| {
    matches!(
      &e.payload,
      EventPayload::Stage(StageEvent::Error {
        stage: Stage::Send,
        message,
        ..
      }) if message.as_str().contains("cancelled")
    )
  });
  assert!(saw_cancel_error, "cancelled send must emit a terminal error");

  let completed = events.iter().find_map(|e| match &e.payload {
    EventPayload::Stage(StageEvent::Completed { success, attempts }) => Some((*success, *attempts)),
    _ => None,
  });
  assert_eq!(completed, Some((false, 1)));
}

#[tokio::test]
async fn full_pipeline_agent_id_shapes_headers_seen_by_send() {
  let cases = [
    AgentHeaderCase {
      name: "opencode_openai",
      agent_id: AgentId::Opencode,
      provider_id: "openai",
      fixture_yaml: OPENCODE_OPENAI_SEND_HEADERS_YAML,
    },
    AgentHeaderCase {
      name: "codex_cli_openai",
      agent_id: AgentId::CodexCli,
      provider_id: "openai",
      fixture_yaml: CODEX_CLI_OPENAI_SEND_HEADERS_YAML,
    },
    AgentHeaderCase {
      name: "claude_code_openai",
      agent_id: AgentId::ClaudeCode,
      provider_id: "openai",
      fixture_yaml: CLAUDE_CODE_OPENAI_SEND_HEADERS_YAML,
    },
    AgentHeaderCase {
      name: "cline_openai",
      agent_id: AgentId::Cline,
      provider_id: "openai",
      fixture_yaml: CLINE_OPENAI_SEND_HEADERS_YAML,
    },
    AgentHeaderCase {
      name: "copilot_cli_openai",
      agent_id: AgentId::CopilotCli,
      provider_id: "openai",
      fixture_yaml: COPILOT_CLI_OPENAI_SEND_HEADERS_YAML,
    },
  ];

  for case in cases {
    let (bus, _log) = capture_bus();
    let (handle, seen_client_headers) = recording_handle(
      case.provider_id,
      "acct-1",
      ok_response(
        200,
        r#"{"id":"resp-agent-id","choices":[{"message":{"role":"assistant","content":"hi"}}]}"#,
      ),
    );
    let selector = Arc::new(CannedSelector { handle });

    let profile = Arc::new(Profile::full(
      "smoke-agent-id-headers",
      Arc::new(FixedAgentExtract {
        agent_id: case.agent_id.clone(),
      }),
      Arc::new(PoolResolve::new(selector)),
      Arc::new(DefaultBuildHeaders::with_provider_defaults()),
      Arc::new(DefaultConvertRequest),
      Arc::new(DefaultSend::new(reqwest::Client::new())),
      Arc::new(DefaultConvertResponse::new()),
    ));
    let runner = PipelineRunner::new(profile, bus);

    runner
      .run(raw_chat("glm-4"))
      .await
      .unwrap_or_else(|err| panic!("{}: pipeline should succeed: {err}", case.name));

    let seen = seen_client_headers
      .lock()
      .unwrap()
      .clone()
      .unwrap_or_else(|| panic!("{}: provider should observe client headers", case.name));
    assert_headers_match_fixture(&seen, case.fixture_yaml, case.name);
  }
}

#[tokio::test]
async fn provider_headers_patch_from_fixtures() {
  let cases = vec![
    ProviderHeaderCase {
      name: "codex",
      provider_id: "codex",
      upstream_endpoint: Endpoint::Responses,
      upstream_model: "gpt-5-codex",
      input_yaml: CODEX_HEADERS_INPUT_YAML,
      output_yaml: CODEX_HEADERS_OUTPUT_YAML,
      handle: codex_handle("http://127.0.0.1"),
      bearer_token: None,
    },
    ProviderHeaderCase {
      name: "copilot",
      provider_id: "github-copilot",
      upstream_endpoint: Endpoint::Responses,
      upstream_model: "gpt-5",
      input_yaml: COPILOT_HEADERS_INPUT_YAML,
      output_yaml: COPILOT_HEADERS_OUTPUT_YAML,
      handle: copilot_handle(),
      bearer_token: Some("api-copilot"),
    },
  ];

  for case in cases {
    let ctx = tokn_requests::PipelineCtx::new(
      format!("req-{}-headers", case.name),
      Endpoint::Responses.into(),
      Arc::new(EventBus::new(64)),
    );
    let extracted = DefaultExtract
      .extract(
        &ctx,
        raw_responses(case.upstream_model, headers_from_fixture(case.input_yaml), false),
      )
      .await
      .unwrap_or_else(|err| panic!("{}: extract should succeed: {err}", case.name));
    let resolved = Resolved {
      agent_id: None,
      model: extracted.model.clone(),
      resolved_endpoint: Some(Endpoint::Responses),
      upstream_model: SmolStr::new(case.upstream_model),
      upstream_endpoint: Some(case.upstream_endpoint),
      account_id: SmolStr::new(case.handle.config.load().id.clone()),
      provider_id: SmolStr::new(case.provider_id),
      account_handle: case.handle.clone(),
    };
    let built = DefaultBuildHeaders::with_provider_defaults()
      .build_headers(&ctx, &extracted, &resolved)
      .await
      .unwrap_or_else(|err| panic!("{}: build_headers should succeed: {err}", case.name));
    let mut headers = built.headers.clone();
    resolved
      .account_handle
      .provider
      .patch_headers(
        &mut headers,
        &tokn_core::provider::HeaderPatchCtx {
          endpoint: case.upstream_endpoint,
          body: extracted.body_json.as_ref(),
          bearer_token: case.bearer_token,
          content_encoding: extracted.content_encoding.map(|encoding| encoding.as_str()),
          stream: extracted.stream,
          initiator: extracted.initiator.as_deref().unwrap_or("user"),
          inbound_headers: &extracted.headers,
          vars: &built.vars,
        },
      )
      .unwrap_or_else(|err| panic!("{}: patch_headers should succeed: {err}", case.name));

    assert_headers_include_fixture(&headers, case.output_yaml, case.name);
  }
}

#[tokio::test]
async fn full_pipeline_codex_headers_are_captured_after_build_and_patch() {
  let server = MockLlmServer::start(MockLlmConfig::default().with_auth(MockAuthConfig::bearer(["atk-codex"]))).await;
  let (bus, log) = capture_bus();
  let selector = Arc::new(CodexSelector {
    handle: codex_handle(server.base_url()),
  });

  let profile = Arc::new(Profile::full(
    "smoke-codex-headers",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new(profile, bus);

  let inbound_headers = headers_from_fixture(CODEX_HEADERS_INPUT_YAML);

  let converted = runner
    .run(raw_responses("gpt-5-codex", inbound_headers, false))
    .await
    .expect("codex responses pipeline must succeed");

  assert_eq!(converted.status, 200);
  let events = drain_until_completed(&log).await;
  let built_headers = events
    .iter()
    .find_map(|event| match &event.payload {
      EventPayload::Stage(StageEvent::BuildHeaders(headers)) => Some(headers.headers.clone()),
      _ => None,
    })
    .expect("BuildHeaders event should be emitted before Send");
  assert_eq!(
    built_headers.get("chatgpt-account-id").map(|value| value.as_str()),
    Some("acct-from-inbound"),
    "BuildHeaders should carry the inbound account correlation before provider auth patching"
  );
  assert_eq!(
    built_headers.get("OpenAI-Beta").map(|value| value.as_str()),
    Some("responses=v1"),
    "BuildHeaders should include the Codex overlay beta before provider normalization"
  );

  let captured = server
    .last_request()
    .expect("mock server should capture the upstream request");
  assert_eq!(captured.path, "/responses");
  for HeaderFixtureEntry { name, value } in load_agent_id_header_fixture(CODEX_HEADERS_OUTPUT_YAML) {
    assert_eq!(
      captured.header(&name),
      Some(value.as_str()),
      "captured Codex output header mismatch for {name}"
    );
  }
}

// ---------- PR3c: failure preserves partial outcome ----------

/// Provider whose `chat` always returns an upstream 401. Used to assert
/// that a Send-stage failure short-circuits the pipeline with the
/// matching `PipelineError`, that the terminal Error + Completed events
/// report the failing stage, and that subscribers can fold the prior
/// per-stage events to recover Resolve / BuildHeaders / ConvertRequest
/// outputs (the runner no longer carries partial state in its return
/// value).
struct FailingProvider {
  info: ProviderInfo,
}

#[async_trait]
impl Provider for FailingProvider {
  fn id(&self) -> &str {
    &self.info.id
  }
  fn info(&self) -> &ProviderInfo {
    &self.info
  }
  async fn list_models(&self, _http: &reqwest::Client) -> ProviderResult<Value> {
    Ok(Value::Null)
  }
  async fn chat(&self, _ctx: RequestCtx<'_>) -> ProviderResult<reqwest::Response> {
    let resp = http::Response::builder()
      .status(401)
      .header("content-type", "application/json")
      .body(r#"{"error":"unauthorized"}"#)
      .unwrap();
    Ok(reqwest::Response::from(resp))
  }
}

fn failing_handle(provider_id: &str, account_id: &str) -> Arc<AccountHandle> {
  let info = ProviderInfo {
    id: provider_id.into(),
    aliases: &[],
    display_name: "failing",
    upstream_url: String::new(),
    auth_kind: AuthKind::StaticApiKey,
    default_models: vec![],
    default_endpoints: &[Endpoint::ChatCompletions],
    model_cache: Arc::new(ModelCache::default()),
  };
  let cfg = AccountConfig {
    id: account_id.to_string(),
    provider: provider_id.to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
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
  };
  let provider = FailingProvider { info };
  Arc::new(AccountHandle::new(Arc::new(cfg), Arc::new(provider)))
}

enum ScriptedResponse {
  Http { status: u16, body: &'static str },
}

struct SequencedProvider {
  info: ProviderInfo,
  responses: Mutex<VecDeque<ScriptedResponse>>,
  calls: AtomicUsize,
}

#[async_trait]
impl Provider for SequencedProvider {
  fn id(&self) -> &str {
    &self.info.id
  }

  fn info(&self) -> &ProviderInfo {
    &self.info
  }

  async fn list_models(&self, _http: &reqwest::Client) -> ProviderResult<Value> {
    Ok(Value::Null)
  }

  async fn chat(&self, _ctx: RequestCtx<'_>) -> ProviderResult<reqwest::Response> {
    self.calls.fetch_add(1, Ordering::Relaxed);
    let next = self
      .responses
      .lock()
      .unwrap()
      .pop_front()
      .expect("scripted provider should have a queued response");
    match next {
      ScriptedResponse::Http { status, body } => Ok(ok_response(status, body)),
    }
  }
}

fn sequenced_handle(provider_id: &str, account_id: &str, responses: Vec<ScriptedResponse>) -> Arc<AccountHandle> {
  let info = ProviderInfo {
    id: provider_id.into(),
    aliases: &[],
    display_name: "sequenced",
    upstream_url: String::new(),
    auth_kind: AuthKind::StaticApiKey,
    default_models: vec![],
    default_endpoints: &[Endpoint::ChatCompletions],
    model_cache: Arc::new(ModelCache::default()),
  };
  let cfg = AccountConfig {
    id: account_id.to_string(),
    provider: provider_id.to_string(),
    enabled: true,
    tier: Default::default(),
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: None,
    username: None,
    api_key: None,
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
  };
  let provider = SequencedProvider {
    info,
    responses: Mutex::new(responses.into()),
    calls: AtomicUsize::new(0),
  };
  Arc::new(AccountHandle::new(Arc::new(cfg), Arc::new(provider)))
}

#[tokio::test]
async fn pipeline_send_failure_preserves_partial_outcome() {
  let (bus, log) = capture_bus();

  let handle = failing_handle("zai-coding-plan", "acct-1");
  let selector = Arc::new(CannedSelector { handle });

  let profile = Arc::new(Profile::full(
    "smoke-fail",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new(profile, bus);

  let err = runner
    .run(raw_chat("glm-4"))
    .await
    .expect_err("upstream 401 must surface as Err");

  // The pipeline failed at Send.
  assert_eq!(err.stage, Stage::Send);
  assert!(
    err.message().contains("401"),
    "error message should mention upstream status: {}",
    err.message()
  );
  assert!(!err.stop, "401 is a real failure, not a stop");

  // Subscribers fold prior per-stage events to recover the partial state
  // — the runner does not carry it on the return value any more. Every
  // earlier stage must have fired exactly once before the Error. The
  // mock bypasses `util::http::send`, so `Record::UpstreamReq` is
  // skipped, but buffered upstream status failures now emit both
  // `Record::UpstreamResp` and `Record::UpstreamBody` before the
  // pipeline surfaces the Send error.
  let events = drain_until_completed(&log).await;
  let kinds = known_kinds(&events);
  assert_eq!(
    kinds,
    [
      "started",
      "extract",
      "resolve",
      "build_headers",
      "convert_request",
      "record",
      "record",
      "error",
      "completed",
    ]
  );

  let mut saw_upstream_resp = false;
  let mut saw_upstream_body = false;
  for event in &*events {
    match &event.payload {
      EventPayload::Record(RecordEvent::UpstreamResp { status, .. }) => {
        saw_upstream_resp = true;
        assert_eq!(*status, 401);
      }
      EventPayload::Record(RecordEvent::UpstreamBody { body, error }) => {
        saw_upstream_body = true;
        assert_eq!(
          std::str::from_utf8(body.as_ref()).unwrap(),
          r#"{"error":"unauthorized"}"#
        );
        assert!(error.is_none());
      }
      _ => {}
    }
  }
  assert!(saw_upstream_resp, "expected UpstreamResp record on send failure");
  assert!(saw_upstream_body, "expected UpstreamBody record on send failure");

  // Spot-check that each pre-Send stage's event carries its full output.
  let resolved_seen = events
    .iter()
    .any(|e| matches!(&e.payload, EventPayload::Stage(StageEvent::Resolve(_))));
  let headers_seen = events
    .iter()
    .any(|e| matches!(&e.payload, EventPayload::Stage(StageEvent::BuildHeaders(_))));
  let req_seen = events
    .iter()
    .any(|e| matches!(&e.payload, EventPayload::Stage(StageEvent::ConvertRequest(_))));
  assert!(resolved_seen, "Resolve event must precede the Send failure");
  assert!(headers_seen, "BuildHeaders event must precede the Send failure");
  assert!(req_seen, "ConvertRequest event must precede the Send failure");

  // The terminal events mirror the failure: Error tags the originating
  // stage with stop=false; Completed reports success=false.
  let (err_stage, err_stop) = events
    .iter()
    .find_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Error { stage, stop, .. }) => Some((*stage, *stop)),
      _ => None,
    })
    .expect("Error event must be present");
  assert_eq!(err_stage, Stage::Send);
  assert!(!err_stop);
  let completed_success = events.iter().find_map(|e| match &e.payload {
    EventPayload::Stage(StageEvent::Completed { success, .. }) => Some(*success),
    _ => None,
  });
  assert_eq!(completed_success, Some(false));
}

#[tokio::test]
async fn pipeline_retries_recoverable_send_failures_and_succeeds() {
  let (bus, log) = capture_bus();

  let handle = sequenced_handle(
    "zai-coding-plan",
    "acct-1",
    vec![
      ScriptedResponse::Http {
        status: 503,
        body: "retry me",
      },
      ScriptedResponse::Http {
        status: 200,
        body: r#"{"id":"resp-retry","choices":[{"message":{"role":"assistant","content":"ok"}}]}"#,
      },
    ],
  );
  let selector = Arc::new(CannedSelector { handle });

  let profile = Arc::new(Profile::full(
    "smoke-retry-success",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new_with_retry(profile, bus, RetryPolicy::new(2, Duration::from_millis(1)));

  let converted = runner
    .run(raw_chat("glm-4"))
    .await
    .expect("second attempt should succeed");
  let events = drain_until_completed_attempts(&log, 2).await;
  let error_attempts: Vec<u32> = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Error {
        stage: Stage::Send,
        recoverable,
        ..
      }) if *recoverable => Some(e.attempt),
      _ => None,
    })
    .collect();
  assert_eq!(error_attempts, vec![0]);

  let completed = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Completed { success, attempts }) => Some((e.attempt, *success, *attempts)),
      _ => None,
    })
    .collect::<Vec<_>>();
  let started_attempts: Vec<u32> = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Started { .. }) => Some(e.attempt),
      _ => None,
    })
    .collect();
  assert_eq!(started_attempts, vec![0, 1]);
  assert_eq!(completed, vec![(0, false, 1), (1, true, 2)]);

  match converted.body {
    ConvertedBody::Buffered { body_json, .. } => {
      let body_json = body_json.unwrap();
      assert_eq!(body_json["id"], "resp-retry");
    }
    other => panic!("expected Buffered, got {other:?}"),
  }
}

#[tokio::test]
async fn pipeline_stops_after_retry_budget_exhausted() {
  let (bus, log) = capture_bus();

  let handle = sequenced_handle(
    "zai-coding-plan",
    "acct-1",
    vec![
      ScriptedResponse::Http {
        status: 503,
        body: "one",
      },
      ScriptedResponse::Http {
        status: 503,
        body: "two",
      },
      ScriptedResponse::Http {
        status: 503,
        body: "three",
      },
    ],
  );
  let selector = Arc::new(CannedSelector { handle });

  let profile = Arc::new(Profile::full(
    "smoke-retry-exhausted",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new_with_retry(profile, bus, RetryPolicy::new(2, Duration::from_millis(1)));

  let err = runner
    .run(raw_chat("glm-4"))
    .await
    .expect_err("retry budget should exhaust");
  assert_eq!(err.stage, Stage::Send);
  assert!(err.recoverable);

  let events = drain_until_completed_attempts(&log, 3).await;
  let completed = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Completed { success, attempts }) => Some((e.attempt, *success, *attempts)),
      _ => None,
    })
    .collect::<Vec<_>>();
  let started_attempts: Vec<u32> = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Started { .. }) => Some(e.attempt),
      _ => None,
    })
    .collect();
  assert_eq!(started_attempts, vec![0, 1, 2]);
  assert_eq!(completed, vec![(0, false, 1), (1, false, 2), (2, false, 3)]);
}

#[tokio::test]
async fn pipeline_does_not_retry_permanent_send_failures() {
  let (bus, log) = capture_bus();

  let handle = sequenced_handle(
    "zai-coding-plan",
    "acct-1",
    vec![ScriptedResponse::Http {
      status: 401,
      body: "nope",
    }],
  );
  let selector = Arc::new(CannedSelector { handle });

  let profile = Arc::new(Profile::full(
    "smoke-retry-permanent",
    Arc::new(DefaultExtract),
    Arc::new(PoolResolve::new(selector)),
    Arc::new(DefaultBuildHeaders::with_provider_defaults()),
    Arc::new(DefaultConvertRequest),
    Arc::new(DefaultSend::new(reqwest::Client::new())),
    Arc::new(DefaultConvertResponse::new()),
  ));
  let runner = PipelineRunner::new_with_retry(profile, bus, RetryPolicy::new(2, Duration::from_millis(1)));

  let err = runner
    .run(raw_chat("glm-4"))
    .await
    .expect_err("401 should remain permanent");
  assert_eq!(err.stage, Stage::Send);
  assert!(!err.recoverable);

  let events = drain_until_completed(&log).await;
  let started_attempts: Vec<u32> = events
    .iter()
    .filter_map(|e| match &e.payload {
      EventPayload::Stage(StageEvent::Started { .. }) => Some(e.attempt),
      _ => None,
    })
    .collect();
  assert_eq!(started_attempts, vec![0]);
}
