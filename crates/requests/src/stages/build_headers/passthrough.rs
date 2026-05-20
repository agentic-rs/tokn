//! Pass-through BuildHeaders stage.
//!
//! Mimics the verbatim relay behaviour of `crates/router/src/proxy/passthrough.rs`:
//! the outbound request carries the **inbound** headers as-is, with two
//! categories stripped:
//!
//! 1. **Router-owned** — anything matching `is_router_owned_header` (i.e.
//!    `x-llm-router-*`, `x-route-mode`, `x-behave-as`). These are internal
//!    transport metadata that must never leave the router.
//! 2. **Hop-by-hop** — `host`, `connection`, `proxy-authorization`,
//!    `proxy-connection`, `te`, `trailer`, `transfer-encoding`, `upgrade`,
//!    `keep-alive`. RFC 7230 §6.1; reqwest / hyper will set its own values
//!    for any it needs.
//!
//! Upstream auth is **not** injected here — the provider's `patch_headers`
//! (called from inside `DefaultSend` via `provider.chat/responses/messages`)
//! still adds `Authorization` based on `Resolved.account_handle`.
//!
//! `TemplateVars` is default — passthrough does not interpolate any header.

use crate::pipeline::ctx::PipelineCtx;
use crate::pipeline::error::PipelineError;
use crate::pipeline::stages::{BuildHeadersStage, BuiltHeaders, Extracted, Resolved};
use async_trait::async_trait;
use llm_headers::{HeaderMap, TemplateVars};

/// Hop-by-hop header names (lowercase) that must not be forwarded
/// verbatim to the upstream. The transport layer sets its own value for
/// any it needs.
const HOP_BY_HOP_HEADERS: &[&str] = &[
  "host",
  "connection",
  "proxy-authorization",
  "proxy-connection",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
  "keep-alive",
];

/// Router-owned header names (lowercase) that must never leak upstream.
/// Mirrors `llm_router::api::is_router_owned_header` — duplicated here
/// to keep `llm-requests` free of any dependency on the legacy router
/// crate.
fn is_router_owned(name: &str) -> bool {
  name.starts_with("x-llm-router-") || name == "x-route-mode" || name == "x-behave-as"
}

pub struct PassthroughBuildHeaders;

#[async_trait]
impl BuildHeadersStage for PassthroughBuildHeaders {
  async fn build_headers(
    &self,
    _ctx: &PipelineCtx,
    extracted: &Extracted,
    _resolved: &Resolved,
  ) -> Result<BuiltHeaders, PipelineError> {
    let mut out = HeaderMap::new();
    for (name, value) in extracted.headers.iter() {
      let lower = name.as_str().to_ascii_lowercase();
      if is_router_owned(&lower) {
        continue;
      }
      if HOP_BY_HOP_HEADERS.contains(&lower.as_str()) {
        continue;
      }
      out.insert(name.clone(), value.clone());
    }
    Ok(BuiltHeaders {
      headers: out,
      vars: TemplateVars::default(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::EventBus;
  use crate::pipeline::ctx::PipelineCtx;
  use bytes::Bytes;
  use llm_core::provider::Endpoint;
  use llm_headers::{HeaderName, HeaderValue};
  use serde_json::json;
  use smol_str::SmolStr;
  use std::sync::Arc;

  fn header_map(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut m = HeaderMap::new();
    for (k, v) in pairs {
      m.insert(HeaderName::new(*k), HeaderValue::from_string((*v).to_string()));
    }
    m
  }

  fn extracted(headers: HeaderMap) -> Extracted {
    Extracted {
      client_id: None,
      model: SmolStr::new("m"),
      stream: false,
      session_id: None,
      project_id: None,
      initiator: SmolStr::new("user"),
      header_initiator: None,
      route_mode_hint: None,
      headers,
      raw_body: Bytes::new(),
      decoded_body: Bytes::new(),
      body_json: Arc::new(json!(null)),
      content_encoding: None,
    }
  }

  fn resolved(provider_id: &str) -> Resolved {
    Resolved {
      client_id: None,
      model: SmolStr::new("m"),
      upstream_model: SmolStr::new("m"),
      upstream_endpoint: Endpoint::ChatCompletions,
      account_id: SmolStr::new("acct-1"),
      provider_id: SmolStr::new(provider_id),
      account_handle: crate::test_support::mock_handle("acct-1", provider_id),
    }
  }

  fn ctx() -> PipelineCtx {
    PipelineCtx::new(
      "req-pbh",
      Endpoint::ChatCompletions,
      Arc::new(EventBus::new(64)),
    )
  }

  #[tokio::test]
  async fn forwards_inbound_minus_denylist() {
    let h = header_map(&[
      ("user-agent", "opencode/1.0"),
      ("authorization", "Bearer client-token"),
      ("accept", "application/json"),
      ("host", "api.openai.com"),
      ("connection", "keep-alive"),
      ("x-llm-router-local-addr", "127.0.0.1:8080"),
      ("x-route-mode", "passthrough"),
      ("x-behave-as", "codex"),
      ("x-custom-thing", "hello"),
    ]);
    let out = PassthroughBuildHeaders
      .build_headers(&ctx(), &extracted(h), &resolved("openai"))
      .await
      .unwrap();
    assert!(out.headers.contains_key("user-agent"));
    assert!(out.headers.contains_key("authorization"));
    assert!(out.headers.contains_key("accept"));
    assert!(out.headers.contains_key("x-custom-thing"));
    assert!(!out.headers.contains_key("host"), "host stripped");
    assert!(!out.headers.contains_key("connection"), "connection stripped");
    assert!(
      !out.headers.contains_key("x-llm-router-local-addr"),
      "router-owned stripped"
    );
    assert!(!out.headers.contains_key("x-route-mode"));
    assert!(!out.headers.contains_key("x-behave-as"));
  }

  #[tokio::test]
  async fn empty_template_vars() {
    let out = PassthroughBuildHeaders
      .build_headers(&ctx(), &extracted(HeaderMap::new()), &resolved("openai"))
      .await
      .unwrap();
    assert!(out.vars.session_id.is_none());
    assert!(out.vars.request_id.is_none());
  }
}
