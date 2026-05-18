//! Production [`SendStage`] implementation.
//!
//! Bridges the router2 stage pipeline to the legacy `Provider` trait
//! (`llm_core::provider::Provider`). Build a [`llm_core::provider::RequestCtx`]
//! from the upstream-shaped body (produced by ConvertRequest), persona
//! headers (produced by BuildHeaders), and a few inbound facts pulled from
//! `Extracted`; then dispatch on `resolved.upstream_endpoint` to the
//! provider's `chat` / `responses` / `messages` method.
//!
//! The provider is responsible for URL construction, auth injection, and
//! the actual HTTP call — `DefaultSend` only wires the data flow and
//! classifies failures into recoverable / permanent [`PipelineError`]s.
//!
//! The returned [`SentResponse`] carries the live [`reqwest::Response`];
//! draining or wrapping it as an SSE stream is the next stage's job.

use crate::event::Stage;
use crate::pipeline::ctx::PipelineCtx;
use crate::pipeline::error::PipelineError;
use crate::pipeline::stages::{BuiltHeaders, ConvertedRequest, Extracted, Resolved, SendStage, SentResponse};
use async_trait::async_trait;
use llm_core::provider::{Endpoint, RequestCtx};
use llm_headers::HeaderMap;
use smol_str::SmolStr;
use tracing::{debug, instrument};

pub struct DefaultSend {
  http: reqwest::Client,
}

impl DefaultSend {
  pub fn new(http: reqwest::Client) -> Self {
    Self { http }
  }
}

#[async_trait]
impl SendStage for DefaultSend {
  #[instrument(name = "default_send", skip_all, fields(
    account = %resolved.account_id,
    provider = %resolved.provider_id,
    endpoint = ?resolved.upstream_endpoint,
    stream = extracted.stream,
  ))]
  async fn send(
    &self,
    _ctx: &PipelineCtx,
    extracted: &Extracted,
    resolved: &Resolved,
    headers: &BuiltHeaders,
    body: &ConvertedRequest,
  ) -> Result<SentResponse, PipelineError> {
    let initiator: &str = extracted.initiator.as_str();
    // Persona headers are passed via `profile_headers`. The provider's
    // own `patch_headers` will run on top to inject auth + content-type;
    // `inbound_headers` therefore only needs to provide template-vars-
    // adjacent context — empty is fine because we already populated
    // `vars` in BuildHeaders.
    let inbound_headers = HeaderMap::new();
    let req_ctx = RequestCtx {
      endpoint: resolved.upstream_endpoint,
      http: &self.http,
      body: body.upstream_body.as_ref(),
      body_bytes: Some(&body.upstream_wire_body),
      content_encoding: body.content_encoding.map(|e| e.as_str()),
      stream: extracted.stream,
      initiator,
      inbound_headers: &inbound_headers,
      behave_as: None,
      profile_headers: Some(headers.headers.clone()),
      outbound: None,
      vars: headers.vars.clone(),
    };

    let provider = resolved.account_handle.provider.clone();
    let resp = match resolved.upstream_endpoint {
      Endpoint::ChatCompletions => provider.chat(req_ctx).await,
      Endpoint::Responses => provider.responses(req_ctx).await,
      Endpoint::Messages => provider.messages(req_ctx).await,
    }
    .map_err(classify_provider_error)?;

    let status = resp.status().as_u16();
    let resp_headers = HeaderMap::from(resp.headers());
    debug!(%status, "upstream responded");

    // Status-based classification: 5xx is recoverable (transient
    // upstream issue), 4xx is permanent (won't change on retry). 2xx/3xx
    // flow through normally.
    if status >= 500 {
      let body_text = match resp.text().await {
        Ok(t) => t,
        Err(e) => {
          return Err(PipelineError::recoverable(
            Stage::Send,
            SmolStr::new(format!("upstream {status}: failed to read body: {e}")),
          ))
        }
      };
      return Err(PipelineError::recoverable(
        Stage::Send,
        SmolStr::new(format!("upstream {status}: {}", truncate(&body_text, 512))),
      ));
    }
    if status >= 400 {
      let body_text = resp.text().await.unwrap_or_default();
      return Err(PipelineError::permanent(
        Stage::Send,
        SmolStr::new(format!("upstream {status}: {}", truncate(&body_text, 512))),
      ));
    }

    Ok(SentResponse {
      status,
      headers: resp_headers,
      stream: extracted.stream,
      upstream_endpoint: resolved.upstream_endpoint,
      response: resp,
    })
  }
}

/// Map an `llm_core::provider::Error` to a [`PipelineError`]. Transport-
/// level failures (connect, timeout, etc.) are recoverable; everything else
/// is permanent for this attempt.
fn classify_provider_error(err: llm_core::provider::Error) -> PipelineError {
  use llm_core::provider::Error as E;
  let recoverable = matches!(&err, E::Http { .. });
  let msg = SmolStr::new(err.to_string());
  if recoverable {
    PipelineError::recoverable(Stage::Send, msg)
  } else {
    PipelineError::permanent(Stage::Send, msg)
  }
}

fn truncate(s: &str, max: usize) -> String {
  if s.len() <= max {
    s.to_string()
  } else {
    format!("{}…", &s[..max])
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::EventBus;
  use crate::pipeline::stages::{BuiltHeaders, ConvertedRequest, Extracted, Resolved};
  use crate::test_support::{mock_handle_with_provider, MockProvider};
  use bytes::Bytes;
  use llm_core::provider::{Endpoint, Result as ProviderResult};
  use serde_json::Value;
  use smol_str::SmolStr;
  use std::sync::Arc;

  fn ctx() -> PipelineCtx {
    PipelineCtx::new("req-send", Endpoint::ChatCompletions, Arc::new(EventBus::new(64)))
  }

  fn extracted() -> Extracted {
    Extracted {
      client_id: None,
      model: SmolStr::new("m"),
      stream: false,
      session_id: None,
      project_id: None,
      initiator: SmolStr::new("user"),
      header_initiator: None,
      route_mode_hint: None,
      headers: llm_headers::HeaderMap::new(),
      raw_body: Bytes::new(),
      decoded_body: Bytes::new(),
      body_json: Arc::new(Value::Null),
      content_encoding: None,
    }
  }

  fn resolved(handle: Arc<llm_accounts::AccountHandle>) -> Resolved {
    Resolved {
      client_id: None,
      model: SmolStr::new("m"),
      upstream_model: SmolStr::new("m"),
      upstream_endpoint: Endpoint::ChatCompletions,
      account_id: SmolStr::new("acct"),
      provider_id: SmolStr::new("mock"),
      account_handle: handle,
    }
  }

  fn body() -> ConvertedRequest {
    let v = serde_json::json!({"model": "m"});
    let bytes = Bytes::from(serde_json::to_vec(&v).unwrap());
    ConvertedRequest {
      upstream_body: Arc::new(v),
      upstream_wire_body: bytes.clone(),
      debug_outbound_body: bytes,
      content_encoding: None,
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

  #[tokio::test]
  async fn dispatches_to_chat_and_returns_sent_response() {
    let provider = MockProvider::new("mock").with_chat_response(ok_response(200, r#"{"ok":true}"#));
    let handle = mock_handle_with_provider("acct", provider);
    let send = DefaultSend::new(reqwest::Client::new());
    let out = send
      .send(
        &ctx(),
        &extracted(),
        &resolved(handle),
        &BuiltHeaders::default(),
        &body(),
      )
      .await
      .expect("send should succeed");
    assert_eq!(out.status, 200);
    assert_eq!(out.upstream_endpoint, Endpoint::ChatCompletions);
    assert!(!out.stream);
  }

  #[tokio::test]
  async fn five_xx_is_recoverable() {
    let provider = MockProvider::new("mock").with_chat_response(ok_response(503, "boom"));
    let handle = mock_handle_with_provider("acct", provider);
    let send = DefaultSend::new(reqwest::Client::new());
    let err = send
      .send(
        &ctx(),
        &extracted(),
        &resolved(handle),
        &BuiltHeaders::default(),
        &body(),
      )
      .await
      .unwrap_err();
    assert_eq!(err.stage, Stage::Send);
    assert!(err.recoverable);
    assert!(err.message.contains("503"));
  }

  #[tokio::test]
  async fn four_xx_is_permanent() {
    let provider = MockProvider::new("mock").with_chat_response(ok_response(401, "no"));
    let handle = mock_handle_with_provider("acct", provider);
    let send = DefaultSend::new(reqwest::Client::new());
    let err = send
      .send(
        &ctx(),
        &extracted(),
        &resolved(handle),
        &BuiltHeaders::default(),
        &body(),
      )
      .await
      .unwrap_err();
    assert_eq!(err.stage, Stage::Send);
    assert!(!err.recoverable);
    assert!(err.message.contains("401"));
  }

  #[tokio::test]
  async fn provider_error_classified_by_kind() {
    let provider =
      MockProvider::new("mock").with_chat_error(|| llm_core::provider::Error::Profiles { message: "boom".into() });
    let handle = mock_handle_with_provider("acct", provider);
    let send = DefaultSend::new(reqwest::Client::new());
    let err = send
      .send(
        &ctx(),
        &extracted(),
        &resolved(handle),
        &BuiltHeaders::default(),
        &body(),
      )
      .await
      .unwrap_err();
    assert_eq!(err.stage, Stage::Send);
    assert!(!err.recoverable);
    assert!(err.message.contains("boom"));
  }

  // Ensure the test harness compiles even if `ProviderResult` is unused
  // in some build variants.
  #[allow(dead_code)]
  fn _types_used() -> Option<ProviderResult<()>> {
    None
  }
}
