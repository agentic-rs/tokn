//! Account management. The pool owns provider-bucketed acquisition while
//! affinity keeps session ids pinned to accounts across related requests.

pub mod affinity;
pub mod registry;

mod handle;
mod pool;

pub use handle::AccountHandle;
pub use pool::{AccountPool, EndpointAcquire, Error, Result, SessionAcquire};
