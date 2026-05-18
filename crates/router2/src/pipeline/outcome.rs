//! Terminal outcome of a single [`PipelineRunner::run`] invocation.
//!
//! Mirrors the final [`StageEvent::Completed`] event but carries the typed
//! error (if any) and the per-stage outputs so callers don't need to
//! subscribe to the bus just to learn the result.
//!
//! Front-half outputs (`resolved`, `built_headers`, `converted_request`) are
//! populated when the runner completes through ConvertRequest successfully.
//! Back-half outputs are populated when those stages run:
//!
//! * `sent_response`: present when `stop_after == Some(Stage::Send)` short-
//!   circuits the runner after Send completes. When the full pipeline runs,
//!   the response is consumed by ConvertResponse and this field is `None`.
//! * `converted_response`: present when the full pipeline runs to
//!   completion. Carries either a buffered JSON response or a live SSE
//!   stream depending on the upstream's `Content-Type`.
//!
//! `PipelineOutcome` is intentionally **not** `Clone`: `SentResponse` wraps
//! a single-shot `reqwest::Response`, and `ConvertedResponse::Stream` owns
//! a one-time `BoxStream`. Callers move the outcome into the consumer that
//! drains the body.
//!
//! [`PipelineRunner::run`]: crate::pipeline::PipelineRunner::run
//! [`StageEvent::Completed`]: crate::event::StageEvent::Completed

use crate::pipeline::error::PipelineError;
use crate::pipeline::stages::{BuiltHeaders, ConvertedRequest, ConvertedResponse, Resolved, SentResponse};

#[derive(Debug)]
pub struct PipelineOutcome {
  pub success: bool,
  pub attempts: u32,
  pub error: Option<PipelineError>,
  /// Resolve-stage output. `Some` once Resolve has run successfully.
  pub resolved: Option<Resolved>,
  /// BuildHeaders-stage output. `Some` once BuildHeaders has run successfully.
  pub built_headers: Option<BuiltHeaders>,
  /// ConvertRequest-stage output. `Some` once ConvertRequest has run
  /// successfully — for stop_before_send / dry-run callers this is the
  /// final outbound payload.
  pub converted_request: Option<ConvertedRequest>,
  /// Send-stage output. Only populated when the runner short-circuits with
  /// `stop_after == Some(Stage::Send)`; otherwise the response is consumed
  /// by ConvertResponse.
  pub sent_response: Option<SentResponse>,
  /// ConvertResponse-stage output. `Some` once the full pipeline has run
  /// to completion.
  pub converted_response: Option<ConvertedResponse>,
}

impl PipelineOutcome {
  pub fn success(attempts: u32) -> Self {
    Self {
      success: true,
      attempts,
      error: None,
      resolved: None,
      built_headers: None,
      converted_request: None,
      sent_response: None,
      converted_response: None,
    }
  }

  pub fn failure(attempts: u32, error: PipelineError) -> Self {
    Self {
      success: false,
      attempts,
      error: Some(error),
      resolved: None,
      built_headers: None,
      converted_request: None,
      sent_response: None,
      converted_response: None,
    }
  }

  pub fn with_resolved(mut self, resolved: Resolved) -> Self {
    self.resolved = Some(resolved);
    self
  }

  pub fn with_built_headers(mut self, headers: BuiltHeaders) -> Self {
    self.built_headers = Some(headers);
    self
  }

  pub fn with_converted_request(mut self, converted: ConvertedRequest) -> Self {
    self.converted_request = Some(converted);
    self
  }

  pub fn with_sent_response(mut self, sent: SentResponse) -> Self {
    self.sent_response = Some(sent);
    self
  }

  pub fn with_converted_response(mut self, converted: ConvertedResponse) -> Self {
    self.converted_response = Some(converted);
    self
  }
}
