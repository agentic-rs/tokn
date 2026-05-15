use serde::{Deserialize, Serialize};
use serde_json::Value;

use llm_endpoint_core::Extras;

use crate::item::InputItem;

/// `input` field accepts either a plain string or a list of items.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsesInput {
  Text(String),
  Items(Vec<InputItem>),
}

impl Default for ResponsesInput {
  fn default() -> Self {
    Self::Items(Vec::new())
  }
}

/// Request body for `POST /v1/responses`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponsesRequest {
  pub model: String,
  pub input: ResponsesInput,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub instructions: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub tools: Vec<ResponsesToolDef>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tool_choice: Option<ResponsesToolChoice>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_output_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stop: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub reasoning: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stream: Option<bool>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub store: Option<bool>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub metadata: Option<Value>,
  #[serde(default, flatten)]
  pub extras: Extras,
}

/// `tools[]` entry. The Responses API permits richer shapes (function,
/// web_search, file_search, custom, etc.); this struct keeps the
/// discriminator typed and stows the rest in `extras`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponsesToolDef {
  #[serde(rename = "type")]
  pub kind: String,
  #[serde(default, flatten)]
  pub extras: Extras,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsesToolChoice {
  Mode(String),
  Named(Value),
}
