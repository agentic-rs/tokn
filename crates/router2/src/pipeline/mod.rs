//! Runner that drives a [`Profile`] through the 6-stage pipeline.
//!
//! Responsibilities:
//!
//! * Build a fresh [`PipelineCtx`] from the inbound [`RawInbound`].
//! * Emit [`StageEvent::Started`] before the first stage and
//!   [`StageEvent::Completed`] after the last (always).
//! * Run each stage; on success, emit the matching per-stage event; on
//!   failure, emit [`StageEvent::Error`] (with the stage/recoverable flag
//!   pulled verbatim from [`PipelineError`]) and short-circuit.
//! * The runner can be configured (via [`RunnerOptions::stop_after`]) to
//!   short-circuit with success after a specific stage completes. This is
//!   how dry-run / smoke flows skip the Send half without needing a
//!   special-case profile constructor.
//!
//! Hooks are intentionally absent from PR1.
//!
//! [`Profile`]: crate::profile::Profile
//! [`StageEvent::Started`]: crate::event::StageEvent::Started
//! [`StageEvent::Completed`]: crate::event::StageEvent::Completed
//! [`StageEvent::Error`]: crate::event::StageEvent::Error

pub mod ctx;
pub mod error;
pub mod outcome;
pub mod snapshot;
pub mod stages;

use crate::event::{EventBus, Stage, StageEvent};
use crate::profile::Profile;
use ctx::PipelineCtx;
use error::PipelineError;
use outcome::PipelineOutcome;
use smol_str::SmolStr;
pub use stages::{
  BuildHeadersStage, BuiltHeaders, ConvertRequestStage, ConvertResponseStage, ConvertedRequest, ConvertedResponse,
  ExtractStage, Extracted, RawInbound, ResolveStage, Resolved, SendStage, SentResponse,
};
use std::sync::Arc;

/// Alias for clarity at call sites — the same type as [`PipelineRunner`].
pub type Pipeline = PipelineRunner;

/// Per-run configuration knobs for [`PipelineRunner`].
///
/// `stop_after` short-circuits the run with success once the named stage
/// completes — used by dry-run / smoke flows that want the front-half output
/// (BuildHeaders + ConvertRequest) without invoking Send.
#[derive(Debug, Clone, Default)]
pub struct RunnerOptions {
  pub stop_after: Option<Stage>,
}

impl RunnerOptions {
  pub fn stop_after(stage: Stage) -> Self {
    Self {
      stop_after: Some(stage),
    }
  }
}

pub struct PipelineRunner {
  pub profile: Arc<Profile>,
  pub events: Arc<EventBus>,
  pub options: RunnerOptions,
}

impl PipelineRunner {
  pub fn new(profile: Arc<Profile>, events: Arc<EventBus>) -> Self {
    Self {
      profile,
      events,
      options: RunnerOptions::default(),
    }
  }

  pub fn with_options(profile: Arc<Profile>, events: Arc<EventBus>, options: RunnerOptions) -> Self {
    Self {
      profile,
      events,
      options,
    }
  }

  pub async fn run(&self, raw: RawInbound) -> PipelineOutcome {
    let request_id = raw.request_id.clone().unwrap_or_else(|| SmolStr::new(uuid_like()));
    let ctx = PipelineCtx::new(request_id, raw.endpoint, self.events.clone());
    ctx.emit_known(StageEvent::Started { endpoint: raw.endpoint });

    // Build the outcome incrementally. Each stage's success mutates the
    // accumulator before we snapshot+emit, so observers see a growing
    // snapshot per event. On failure the same accumulator (with `error`
    // populated) is returned to the caller — no partial state is lost.
    let mut outcome = PipelineOutcome::success(0);

    // ---- Extract ----
    let extracted = match self.profile.extract.extract(&ctx, raw).await {
      Ok(e) => {
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::Extract {
          client_id: e.client_id.clone(),
          model: e.model.clone(),
          stream: e.stream,
          snapshot,
        });
        e
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };
    if self.options.stop_after == Some(Stage::Extract) {
      return self.complete(&ctx, outcome);
    }

    // ---- Resolve ----
    let resolved = match self.profile.resolve.resolve(&ctx, &extracted).await {
      Ok(r) => {
        outcome.resolved = Some(r.clone());
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::Resolve {
          client_id: r.client_id.clone(),
          model: r.model.clone(),
          upstream_model: r.upstream_model.clone(),
          account_id: r.account_id.clone(),
          provider_id: r.provider_id.clone(),
          upstream_endpoint: r.upstream_endpoint,
          snapshot,
        });
        r
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };
    if self.options.stop_after == Some(Stage::Resolve) {
      return self.complete(&ctx, outcome);
    }

    // ---- BuildHeaders ----
    let headers = match self
      .profile
      .build_headers
      .build_headers(&ctx, &extracted, &resolved)
      .await
    {
      Ok(h) => {
        outcome.built_headers = Some(h.clone());
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::BuildHeaders { snapshot });
        h
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };
    if self.options.stop_after == Some(Stage::BuildHeaders) {
      return self.complete(&ctx, outcome);
    }

    // ---- ConvertRequest ----
    let converted = match self
      .profile
      .convert_request
      .convert_request(&ctx, &extracted, &resolved)
      .await
    {
      Ok(c) => {
        outcome.converted_request = Some(c.clone());
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::ConvertRequest { snapshot });
        c
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };
    if self.options.stop_after == Some(Stage::ConvertRequest) {
      return self.complete(&ctx, outcome);
    }

    // ---- Send ----
    let sent = match self
      .profile
      .send
      .send(&ctx, &extracted, &resolved, &headers, &converted)
      .await
    {
      Ok(s) => {
        // SentResponse is not Clone (single-shot reqwest::Response), so
        // the accumulator's `sent_response` is populated only if we
        // short-circuit here; the snapshot meanwhile reflects whatever
        // is already in `outcome` (resolved + headers + converted req).
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::Send { snapshot });
        s
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };
    if self.options.stop_after == Some(Stage::Send) {
      outcome.sent_response = Some(sent);
      return self.complete(&ctx, outcome);
    }

    // ---- ConvertResponse ----
    let converted_response = match self.profile.convert_response.convert_response(&ctx, sent).await {
      Ok(c) => {
        // Build the snapshot from a temporary view that includes the
        // converted response (without moving it into `outcome` yet, so
        // the snapshot's body_json Arc can be shared with the returned
        // outcome). We populate `outcome.converted_response` first, then
        // snapshot from `&outcome`.
        outcome.converted_response = Some(c);
        let snapshot = Arc::new((&outcome).into());
        ctx.emit_known(StageEvent::ConvertResponse { snapshot });
        // unwrap is safe: we just set it.
        outcome.converted_response.take().expect("just populated")
      }
      Err(err) => return self.fail(&ctx, &mut outcome, err),
    };

    outcome.success = true;
    outcome.attempts = ctx.attempt + 1;
    outcome.converted_response = Some(converted_response);
    self.complete(&ctx, outcome)
  }

  /// Emit [`StageEvent::Completed`] for a successful (or short-circuited)
  /// run and return the assembled outcome.
  fn complete(&self, ctx: &PipelineCtx, mut outcome: PipelineOutcome) -> PipelineOutcome {
    outcome.success = true;
    outcome.attempts = ctx.attempt + 1;
    let snapshot = Arc::new((&outcome).into());
    ctx.emit_known(StageEvent::Completed {
      success: true,
      attempts: outcome.attempts,
      snapshot,
    });
    outcome
  }

  /// Record the failure on `outcome`, emit [`StageEvent::Error`] +
  /// [`StageEvent::Completed`] (sharing one snapshot `Arc`), and return the
  /// partially-populated outcome to the caller.
  fn fail(&self, ctx: &PipelineCtx, outcome: &mut PipelineOutcome, err: PipelineError) -> PipelineOutcome {
    outcome.success = false;
    outcome.attempts = ctx.attempt + 1;
    outcome.error = Some(err.clone());
    let snapshot: Arc<snapshot::OutcomeSnapshot> = Arc::new((&*outcome).into());
    ctx.emit_known(StageEvent::Error {
      stage: err.stage,
      message: err.message.clone(),
      recoverable: err.recoverable,
      snapshot: snapshot.clone(),
    });
    ctx.emit_known(StageEvent::Completed {
      success: false,
      attempts: outcome.attempts,
      snapshot,
    });
    // Move the populated outcome out by replacing it with a tombstone;
    // the caller's `&mut` reference is local to this method's parent
    // frame and dies immediately on return, so this is sound.
    std::mem::replace(outcome, PipelineOutcome::failure(0, err))
  }
}

/// Cheap unique-ish id without pulling in the `uuid` crate. The runner only
/// uses this when the caller did not supply a request id (tests, smoke
/// fixtures); production transports always populate `RawInbound.request_id`.
fn uuid_like() -> String {
  use std::sync::atomic::{AtomicU64, Ordering};
  static COUNTER: AtomicU64 = AtomicU64::new(0);
  let n = COUNTER.fetch_add(1, Ordering::Relaxed);
  let ts = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  format!("req-{ts:032x}-{n:08x}")
}

// `Stage` is re-exported at the crate root via `lib.rs`.
