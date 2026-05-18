//! Production [`ConvertResponseStage`] implementation.
//!
//! Bridges the upstream HTTP response produced by [`SendStage`] to the
//! shape the client originally asked for. Two branches:
//!
//! 1. **Streaming** (`sent.stream == true`): wrap the response in
//!    [`llm_convert::sse::SsePipeline`]; if the upstream endpoint differs
//!    from `ctx.endpoint`, install an [`EndpointTranslator`] so frames
//!    arrive in the client's expected shape. Returned as
//!    [`ConvertedResponse::Stream`] for forwarding chunk-by-chunk.
//! 2. **Buffered** (default): drain the body, parse JSON, and run
//!    [`llm_convert::convert_response`] when upstream/inbound endpoints
//!    differ. Returned as [`ConvertedResponse::Buffered`] with the
//!    canonical JSON value and its re-serialized bytes.
//!
//! Mirrors the legacy [`router::relay::buffered`] / [`router::relay::stream`]
//! behaviour minus the observer/recording side-channels (those belong in
//! a later PR's observer wiring).

use crate::event::Stage;
use crate::pipeline::ctx::PipelineCtx;
use crate::pipeline::error::PipelineError;
use crate::pipeline::stages::{ConvertResponseStage, ConvertedResponse, SentResponse};
use async_trait::async_trait;
use bytes::Bytes;
use llm_convert::sse::{EndpointTranslator, SsePipeline};
use serde_json::Value;
use smol_str::SmolStr;
use tracing::{debug, instrument};

pub struct DefaultConvertResponse;

impl DefaultConvertResponse {
  pub fn new() -> Self {
    Self
  }
}

impl Default for DefaultConvertResponse {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl ConvertResponseStage for DefaultConvertResponse {
  #[instrument(name = "default_convert_response", skip_all, fields(
    status = sent.status,
    stream = sent.stream,
    upstream_endpoint = ?sent.upstream_endpoint,
    inbound_endpoint = ?ctx.endpoint,
  ))]
  async fn convert_response(&self, ctx: &PipelineCtx, sent: SentResponse) -> Result<ConvertedResponse, PipelineError> {
    let SentResponse {
      status,
      headers,
      stream,
      upstream_endpoint,
      response,
    } = sent;
    let inbound_endpoint = ctx.endpoint;

    if stream {
      debug!("wrapping upstream response as SSE stream");
      let mut pipeline = SsePipeline::from_response(response);
      if upstream_endpoint != inbound_endpoint {
        pipeline = pipeline.with_transformer(EndpointTranslator::new(upstream_endpoint, inbound_endpoint));
      }
      return Ok(ConvertedResponse::Stream {
        status,
        headers,
        body: pipeline.run(),
      });
    }

    // Buffered branch: drain the body and (optionally) translate shape.
    let raw = response.bytes().await.map_err(|e| {
      // Body-read failures are transport-ish; recoverable.
      PipelineError::recoverable(
        Stage::ConvertResponse,
        SmolStr::new(format!("reading upstream body: {e}")),
      )
    })?;

    // If the body is empty (e.g. some error responses), short-circuit
    // with a Null JSON value rather than failing — matches the legacy
    // buffered path's tolerance for blank upstream bodies.
    if raw.is_empty() {
      return Ok(ConvertedResponse::Buffered {
        status,
        headers,
        body_json: Value::Null,
        body_bytes: Bytes::new(),
      });
    }

    let upstream_json: Value = serde_json::from_slice(&raw).map_err(|e| {
      PipelineError::permanent(
        Stage::ConvertResponse,
        SmolStr::new(format!("upstream body not valid JSON: {e}")),
      )
    })?;

    let (body_json, body_bytes) = if upstream_endpoint == inbound_endpoint {
      // No translation needed — keep the bytes exactly as received so
      // downstream consumers can match upstream's serialization quirks.
      (upstream_json, raw)
    } else {
      let translated =
        llm_convert::convert_response(upstream_endpoint, inbound_endpoint, &upstream_json).map_err(|e| {
          PipelineError::permanent(
            Stage::ConvertResponse,
            SmolStr::new(format!("response conversion failed: {e}")),
          )
        })?;
      let bytes = serde_json::to_vec(&translated).map(Bytes::from).map_err(|e| {
        PipelineError::permanent(
          Stage::ConvertResponse,
          SmolStr::new(format!("serializing translated response failed: {e}")),
        )
      })?;
      (translated, bytes)
    };

    Ok(ConvertedResponse::Buffered {
      status,
      headers,
      body_json,
      body_bytes,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::EventBus;
  use crate::pipeline::stages::SentResponse;
  use futures_util::StreamExt;
  use llm_core::provider::Endpoint;
  use llm_headers::HeaderMap;
  use std::sync::Arc;

  fn ctx(endpoint: Endpoint) -> PipelineCtx {
    PipelineCtx::new("req-cr", endpoint, Arc::new(EventBus::new()))
  }

  fn response(status: u16, body: &'static str, content_type: &'static str) -> reqwest::Response {
    let resp = http::Response::builder()
      .status(status)
      .header("content-type", content_type)
      .body(body)
      .unwrap();
    reqwest::Response::from(resp)
  }

  fn sent(endpoint: Endpoint, stream: bool, response: reqwest::Response) -> SentResponse {
    SentResponse {
      status: response.status().as_u16(),
      headers: HeaderMap::new(),
      stream,
      upstream_endpoint: endpoint,
      response,
    }
  }

  #[tokio::test]
  async fn buffered_passthrough_same_endpoint() {
    let stage = DefaultConvertResponse::new();
    let s = sent(
      Endpoint::ChatCompletions,
      false,
      response(200, r#"{"id":"x","choices":[]}"#, "application/json"),
    );
    let out = stage
      .convert_response(&ctx(Endpoint::ChatCompletions), s)
      .await
      .unwrap();
    match out {
      ConvertedResponse::Buffered {
        status,
        body_json,
        body_bytes,
        ..
      } => {
        assert_eq!(status, 200);
        assert_eq!(body_json["id"], "x");
        // Bytes are preserved verbatim when endpoints match.
        assert_eq!(body_bytes.as_ref(), br#"{"id":"x","choices":[]}"#);
      }
      _ => panic!("expected buffered"),
    }
  }

  #[tokio::test]
  async fn buffered_empty_body_yields_null() {
    let stage = DefaultConvertResponse::new();
    let s = sent(Endpoint::ChatCompletions, false, response(502, "", "text/plain"));
    let out = stage
      .convert_response(&ctx(Endpoint::ChatCompletions), s)
      .await
      .unwrap();
    match out {
      ConvertedResponse::Buffered {
        status,
        body_json,
        body_bytes,
        ..
      } => {
        assert_eq!(status, 502);
        assert!(body_json.is_null());
        assert!(body_bytes.is_empty());
      }
      _ => panic!("expected buffered"),
    }
  }

  #[tokio::test]
  async fn buffered_invalid_json_is_permanent() {
    let stage = DefaultConvertResponse::new();
    let s = sent(
      Endpoint::ChatCompletions,
      false,
      response(200, "not json", "text/plain"),
    );
    let err = stage
      .convert_response(&ctx(Endpoint::ChatCompletions), s)
      .await
      .unwrap_err();
    assert_eq!(err.stage, Stage::ConvertResponse);
    assert!(!err.recoverable);
    assert!(err.message.contains("not valid JSON"));
  }

  #[tokio::test]
  async fn stream_branch_returns_stream_variant() {
    let stage = DefaultConvertResponse::new();
    let body = "data: {\"hello\":1}\n\ndata: [DONE]\n\n";
    let s = sent(
      Endpoint::ChatCompletions,
      true,
      response(200, body, "text/event-stream"),
    );
    let out = stage
      .convert_response(&ctx(Endpoint::ChatCompletions), s)
      .await
      .unwrap();
    match out {
      ConvertedResponse::Stream { status, mut body, .. } => {
        assert_eq!(status, 200);
        // Drain a chunk to confirm the stream is live.
        let chunk = body.next().await.expect("at least one chunk").expect("ok chunk");
        assert!(!chunk.is_empty());
      }
      _ => panic!("expected stream"),
    }
  }
}
