//! Closed set of pipeline-observation events emitted by [`PipelineRunner`].
//!
//! Successful stage completions get a per-stage variant (`Extract`, `Resolve`,
//! ...). Failures are funneled through a single [`StageEvent::Error`] variant
//! tagged with the originating [`Stage`] so subscribers can filter without
//! pattern-matching on N error variants.
//!
//! Every variant except [`StageEvent::Started`] carries an
//! `Arc<OutcomeSnapshot>` capturing whatever the pipeline had accumulated up
//! to (and including) that event. Subscribers — the CLI's failure printer,
//! retry decorators, observability sinks — read directly from the snapshot
//! instead of replaying earlier events. The runner reuses the same `Arc` for
//! the `Error` + `Completed` pair after a failure.
//!
//! [`PipelineRunner`]: crate::pipeline::PipelineRunner
//! [`OutcomeSnapshot`]: crate::pipeline::snapshot::OutcomeSnapshot

use crate::pipeline::snapshot::OutcomeSnapshot;
use llm_core::provider::Endpoint;
use llm_core::ClientId;
use smol_str::SmolStr;
use std::sync::Arc;

/// Identifies which pipeline stage produced an event. Used both as a tag on
/// success variants and as a field on [`StageEvent::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
  Extract,
  Resolve,
  BuildHeaders,
  ConvertRequest,
  Send,
  ConvertResponse,
}

impl Stage {
  pub fn as_str(self) -> &'static str {
    match self {
      Stage::Extract => "extract",
      Stage::Resolve => "resolve",
      Stage::BuildHeaders => "build_headers",
      Stage::ConvertRequest => "convert_request",
      Stage::Send => "send",
      Stage::ConvertResponse => "convert_response",
    }
  }
}

impl std::fmt::Display for Stage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[derive(Debug, Clone)]
pub enum StageEvent {
  /// Emitted once at the very start of [`PipelineRunner::run`]; no
  /// snapshot because no stage has produced state yet.
  ///
  /// [`PipelineRunner::run`]: crate::pipeline::PipelineRunner::run
  Started {
    endpoint: Endpoint,
  },
  /// Extract stage completed successfully.
  Extract {
    client_id: Option<ClientId>,
    model: SmolStr,
    stream: bool,
    snapshot: Arc<OutcomeSnapshot>,
  },
  /// Resolve stage completed successfully.
  Resolve {
    client_id: Option<ClientId>,
    model: SmolStr,
    upstream_model: SmolStr,
    account_id: SmolStr,
    provider_id: SmolStr,
    upstream_endpoint: Endpoint,
    snapshot: Arc<OutcomeSnapshot>,
  },
  BuildHeaders {
    snapshot: Arc<OutcomeSnapshot>,
  },
  ConvertRequest {
    snapshot: Arc<OutcomeSnapshot>,
  },
  Send {
    snapshot: Arc<OutcomeSnapshot>,
  },
  ConvertResponse {
    snapshot: Arc<OutcomeSnapshot>,
  },
  /// Any stage failure. `recoverable` is propagated verbatim from the
  /// [`PipelineError`] returned by the stage; the runner does not infer it.
  /// `snapshot` carries every field the pipeline managed to populate
  /// before the failure (resolved account, built headers, converted
  /// request body, etc.) so subscribers can diagnose without replay.
  ///
  /// [`PipelineError`]: crate::pipeline::error::PipelineError
  Error {
    stage: Stage,
    message: SmolStr,
    recoverable: bool,
    snapshot: Arc<OutcomeSnapshot>,
  },
  /// Emitted once at the end of [`PipelineRunner::run`], success or
  /// failure. After a failure the runner reuses the same `snapshot` `Arc`
  /// as the preceding [`StageEvent::Error`].
  ///
  /// [`PipelineRunner::run`]: crate::pipeline::PipelineRunner::run
  Completed {
    success: bool,
    attempts: u32,
    snapshot: Arc<OutcomeSnapshot>,
  },
}
