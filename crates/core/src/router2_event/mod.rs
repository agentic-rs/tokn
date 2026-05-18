//! Event payload types for the `llm-router2` pipeline.
//!
//! These types live in `llm-core` so that the workspace's
//! [`llm_core::event::Event`] enum can embed a [`Router2(Router2Event)`]
//! variant without inverting the dep graph (router2 already depends on
//! llm-core).
//!
//! Two payload shapes are supported:
//!
//! * [`StageEvent`] — a closed enum of stage-observation variants the runner
//!   emits at well-defined points. New variants are added as the pipeline
//!   grows; subscribers `match` on them.
//! * [`CustomEvent`] — an `Any`-typed escape hatch for middleware / decorator
//!   stages (e.g. retry, cache) to publish their own structured records
//!   without modifying [`StageEvent`]. The payload is shared via `Arc` so
//!   subscribers can cheaply clone and downcast.
//!
//! The event bus itself is `llm_core::event::EventBus` (a tokio broadcast
//! channel); router2 publishes `llm_core::event::Event::Router2(Router2Event
//! { ... })` directly onto it.
//!
//! [`Router2(Router2Event)`]: crate::event::Event::Router2

pub mod stage;

pub use stage::{
  BuiltHeadersSummary, ConvertedRequestSummary, ConvertedResponseSummary, ExtractedSummary, ResolvedSummary,
  SentSummary, Stage, StageEvent,
};

use smol_str::SmolStr;
use std::any::Any;
use std::sync::Arc;

/// A single router2 pipeline event. Carries the per-request bookkeeping
/// (request_id, attempt) plus a typed or `Any`-typed payload.
#[derive(Clone, Debug)]
pub struct Router2Event {
  pub request_id: SmolStr,
  pub attempt: u32,
  pub payload: Router2EventPayload,
}

/// Either a typed pipeline event or an arbitrary user-defined record.
#[derive(Clone, Debug)]
pub enum Router2EventPayload {
  Known(StageEvent),
  Custom(CustomEvent),
}

/// `Any`-typed payload published by stages or decorators that need to share
/// structured data outside the closed [`StageEvent`] set.
#[derive(Clone)]
pub struct CustomEvent {
  /// Stable namespaced identifier (e.g. `"retry.attempt"`). Subscribers match
  /// on this before downcasting.
  pub kind: &'static str,
  /// Reference-counted so subscribers can cheaply clone the event and
  /// downcast independently.
  pub payload: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for CustomEvent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CustomEvent")
      .field("kind", &self.kind)
      .field("payload", &"<Any>")
      .finish()
  }
}

impl CustomEvent {
  pub fn new<T: Any + Send + Sync>(kind: &'static str, value: T) -> Self {
    Self {
      kind,
      payload: Arc::new(value),
    }
  }

  pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
    self.payload.downcast_ref::<T>()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_event_roundtrips_via_downcast() {
    #[derive(Debug, PartialEq)]
    struct RetryAttempt {
      n: u32,
      reason: SmolStr,
    }

    let ev = CustomEvent::new(
      "retry.attempt",
      RetryAttempt {
        n: 2,
        reason: SmolStr::new("timeout"),
      },
    );
    assert_eq!(ev.kind, "retry.attempt");
    let inner = ev
      .downcast_ref::<RetryAttempt>()
      .expect("payload should downcast back to its declared type");
    assert_eq!(
      inner,
      &RetryAttempt {
        n: 2,
        reason: SmolStr::new("timeout"),
      }
    );
    assert!(
      ev.downcast_ref::<u32>().is_none(),
      "downcast to the wrong type must return None"
    );
  }
}
