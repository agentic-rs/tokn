use serde_json::{Map, Value};

/// Catch-all map for unknown JSON fields preserved during deserialization.
///
/// Endpoint structs typically embed this with
/// `#[serde(default, flatten)] pub extras: Extras` to remain forward
/// compatible with provider-specific or unreleased fields.
pub type Extras = Map<String, Value>;
