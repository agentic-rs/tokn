//! Shared primitives and traits for typed LLM endpoint wire schemas.
//!
//! This crate intentionally has no knowledge of any specific endpoint
//! beyond identifying which endpoints exist. Endpoint-specific request,
//! response, item, and event types live in the per-endpoint crates and
//! implement the traits exposed here.

pub mod endpoint;
pub mod extras;
pub mod finish;
pub mod role;
pub mod tool;
pub mod traits;
pub mod usage;

pub use endpoint::Endpoint;
pub use extras::Extras;
pub use finish::FinishReason;
pub use role::Role;
pub use tool::{ToolCall, ToolDef};
pub use traits::{EndpointEvent, EndpointItem, EndpointRequest, EndpointResponse};
pub use usage::Usage;
