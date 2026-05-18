//! Cheap-clone, observer-friendly snapshot of [`PipelineOutcome`].
//!
//! The runner attaches an `Arc<OutcomeSnapshot>` to every per-stage
//! [`StageEvent`] variant (except [`StageEvent::Started`], which fires before
//! any stage has produced state). Subscribers — observability sinks, the
//! CLI's failure printer, future retry decorators — can clone the `Arc` for
//! free and inspect whatever the pipeline had accumulated up to that point.
//!
//! `OutcomeSnapshot` mirrors [`PipelineOutcome`] field-for-field with two
//! deliberate omissions:
//!
//! * `sent_response`: [`SentResponse`] wraps a single-shot
//!   [`reqwest::Response`] that cannot be cloned. The status + headers we
//!   *can* clone are surfaced via [`ConvertedResponseSnapshot`] once the
//!   response reaches the ConvertResponse stage; before then, observers see
//!   `converted_response: None`.
//! * `ConvertedResponse::Stream.body`: live SSE byte stream is also
//!   single-shot. The snapshot's [`ConvertedResponseSnapshot::Stream`]
//!   carries status + headers only.
//!
//! The buffered response body is already `Arc<serde_json::Value>` in the
//! source [`PipelineOutcome`]; the snapshot reuses the same `Arc` so cloning
//! is `O(1)` regardless of payload size.
//!
//! [`StageEvent`]: crate::event::StageEvent
//! [`StageEvent::Started`]: crate::event::StageEvent::Started
//! [`SentResponse`]: crate::pipeline::stages::SentResponse

use crate::pipeline::error::PipelineError;
use crate::pipeline::outcome::PipelineOutcome;
use crate::pipeline::stages::{BuiltHeaders, ConvertedRequest, ConvertedResponse, Resolved};
use llm_headers::HeaderMap;
use serde_json::Value;
use std::sync::Arc;

/// Snapshot of a [`PipelineOutcome`] suitable for cheap cloning and embedding
/// in [`StageEvent`]s.
///
/// [`StageEvent`]: crate::event::StageEvent
#[derive(Clone, Debug)]
pub struct OutcomeSnapshot {
  pub success: bool,
  pub attempts: u32,
  pub error: Option<PipelineError>,
  pub resolved: Option<Resolved>,
  pub built_headers: Option<BuiltHeaders>,
  pub converted_request: Option<ConvertedRequest>,
  /// Snapshot of the converted response when the pipeline has reached
  /// ConvertResponse. For streaming responses the body is omitted (the live
  /// stream is single-shot); only status + headers are preserved.
  pub converted_response: Option<ConvertedResponseSnapshot>,
}

/// Cloneable mirror of [`ConvertedResponse`]; see module docs for why the
/// stream body is dropped.
#[derive(Clone, Debug)]
pub enum ConvertedResponseSnapshot {
  Buffered {
    status: u16,
    headers: HeaderMap,
    body_json: Arc<Value>,
  },
  Stream {
    status: u16,
    headers: HeaderMap,
  },
}

impl ConvertedResponseSnapshot {
  pub fn status(&self) -> u16 {
    match self {
      Self::Buffered { status, .. } | Self::Stream { status, .. } => *status,
    }
  }

  pub fn headers(&self) -> &HeaderMap {
    match self {
      Self::Buffered { headers, .. } | Self::Stream { headers, .. } => headers,
    }
  }
}

impl From<&ConvertedResponse> for ConvertedResponseSnapshot {
  fn from(cr: &ConvertedResponse) -> Self {
    match cr {
      ConvertedResponse::Buffered {
        status,
        headers,
        body_json,
        ..
      } => Self::Buffered {
        status: *status,
        headers: headers.clone(),
        body_json: body_json.clone(),
      },
      ConvertedResponse::Stream { status, headers, .. } => Self::Stream {
        status: *status,
        headers: headers.clone(),
      },
    }
  }
}

impl From<&PipelineOutcome> for OutcomeSnapshot {
  fn from(o: &PipelineOutcome) -> Self {
    Self {
      success: o.success,
      attempts: o.attempts,
      error: o.error.clone(),
      resolved: o.resolved.clone(),
      built_headers: o.built_headers.clone(),
      converted_request: o.converted_request.clone(),
      converted_response: o.converted_response.as_ref().map(Into::into),
    }
  }
}

impl OutcomeSnapshot {
  /// Build a snapshot for the very first event (no stage has run yet).
  /// Equivalent to `(&PipelineOutcome::success(0)).into()` but doesn't need
  /// to construct an intermediate outcome.
  pub fn initial() -> Self {
    Self {
      success: true,
      attempts: 0,
      error: None,
      resolved: None,
      built_headers: None,
      converted_request: None,
      converted_response: None,
    }
  }
}
