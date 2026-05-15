use serde::{Deserialize, Serialize};
use serde_json::Value;

use llm_endpoint_core::Extras;

use crate::item::OutputItem;
use crate::usage::ResponsesUsage;

/// Non-streaming response body returned by the Responses API.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponsesResponse {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub object: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub created_at: Option<i64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub output: Vec<OutputItem>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub output_text: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub usage: Option<ResponsesUsage>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub error: Option<Value>,
  #[serde(default, flatten)]
  pub extras: Extras,
}
