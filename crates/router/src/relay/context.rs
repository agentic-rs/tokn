use crate::pipeline::{BodyExtract, HeaderExtract};
use crate::provider::Endpoint;
use crate::relay::passthrough::passthrough_endpoint;
use axum::http::Method;
use std::time::Instant;

/// Bundled request metadata needed by forward/response functions.
/// Does not include the request body — that is passed separately.
pub(crate) struct ForwardContext {
  /// Base request ID (no retry suffix).
  pub request_id: String,
  /// Retry attempt number (0 = first attempt).
  pub attempt: u32,
  pub session_id: Option<String>,
  pub endpoint: Option<Endpoint>,
  pub upstream_endpoint: Endpoint,
  pub downstream_headers: reqwest::header::HeaderMap,
  pub model: String,
  pub started: Instant,
}

impl ForwardContext {
  /// Build a ForwardContext from pipeline metadata (routed requests).
  pub fn from_pipeline(
    endpoint: Endpoint,
    upstream_endpoint: Endpoint,
    model: String,
    session_id: Option<String>,
    request_id: String,
    attempt: u32,
    started: Instant,
  ) -> Self {
    Self {
      request_id,
      attempt,
      session_id,
      endpoint: Some(endpoint),
      upstream_endpoint,
      downstream_headers: reqwest::header::HeaderMap::new(),
      model,
      started,
    }
  }

  /// Build a ForwardContext from passthrough request data.
  pub fn from_passthrough(
    method: &Method,
    path: &str,
    headers: &HeaderExtract,
    body: &BodyExtract,
    downstream_headers: reqwest::header::HeaderMap,
    started: Instant,
  ) -> Self {
    let endpoint = passthrough_endpoint(method, path);
    // For passthrough, upstream_endpoint == endpoint (no translation)
    let upstream_endpoint = endpoint.unwrap_or(Endpoint::ChatCompletions);

    Self {
      request_id: headers.request_id.clone(),
      attempt: 0,
      session_id: headers.session_id.clone(),
      endpoint,
      upstream_endpoint,
      downstream_headers,
      model: body.model.clone(),
      started,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn passthrough_context_preserves_ingress_ids() {
    let headers = HeaderExtract {
      request_id: "request-123".to_string(),
      session_id: Some("session-123".to_string()),
      project_id: None,
      header_initiator: None,
      route_mode_hint: None,
    };
    let body = BodyExtract {
      model: "gpt-5".to_string(),
      stream: true,
      initiator: "user".to_string(),
      header_initiator: None,
    };
    let ctx = ForwardContext::from_passthrough(
      &Method::POST,
      "/v1/chat/completions",
      &headers,
      &body,
      reqwest::header::HeaderMap::new(),
      Instant::now(),
    );

    assert_eq!(ctx.request_id, "request-123");
    assert_eq!(ctx.session_id.as_deref(), Some("session-123"));
    assert_eq!(ctx.endpoint, Some(Endpoint::ChatCompletions));
    assert_eq!(ctx.upstream_endpoint, Endpoint::ChatCompletions);
    assert_eq!(ctx.model, "gpt-5");
  }
}
