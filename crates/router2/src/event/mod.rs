//! Re-export shim. The event payload types live in `llm_core::router2_event`
//! and the bus itself is `llm_core::event::EventBus` (a tokio broadcast
//! channel). This module keeps the historical `crate::event::*` import paths
//! working by re-exporting the relocated types under their old names.
//!
//! Conversions between router2's full stage-output structs and the lossy
//! `*Summary` types in llm-core live in [`stage`].

pub mod stage;

pub use llm_core::event::EventBus;
pub use llm_core::router2_event::{
  BuiltHeadersSummary, ConvertedRequestSummary, ConvertedResponseSummary, CustomEvent, ExtractedSummary, RecordEvent,
  ResolvedSummary, Router2Event as Event, Router2EventPayload as EventPayload, SentSummary, Stage, StageEvent,
};
