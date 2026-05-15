use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::extras::Extras;

/// A tool invocation requested by the model.
///
/// `arguments` is intentionally a free-form JSON value; some endpoints
/// emit it as a JSON-encoded string while others emit a structured object.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolCall {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  #[serde(default)]
  pub arguments: Value,
  #[serde(default, flatten)]
  pub extras: Extras,
}

/// Generic tool definition envelope. Endpoint-specific schemas may
/// further refine `parameters` or wrap this struct.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolDef {
  pub name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub parameters: Option<Value>,
  #[serde(default, flatten)]
  pub extras: Extras,
}
