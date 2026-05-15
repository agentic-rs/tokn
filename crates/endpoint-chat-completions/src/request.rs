use serde::{Deserialize, Serialize};
use serde_json::Value;

use llm_endpoint_core::Extras;

use crate::message::ChatMessage;

/// Request body for `POST /v1/chat/completions`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
  pub model: String,
  pub messages: Vec<ChatMessage>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub tools: Vec<ChatToolDef>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub tool_choice: Option<ChatToolChoice>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub max_completion_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stop: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub n: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub seed: Option<i64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub stream: Option<bool>,
  /// Provider extension. Some gateways accept `reasoning`/`thinking`
  /// hints alongside chat completions.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub reasoning: Option<Value>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub thinking: Option<Value>,
  #[serde(default, flatten)]
  pub extras: Extras,
}

/// A `tools[]` entry. Chat Completions only defines function tools.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatToolDef {
  #[serde(rename = "type", default = "default_function_type")]
  pub kind: String,
  pub function: Value,
  #[serde(default, flatten)]
  pub extras: Extras,
}

fn default_function_type() -> String {
  "function".into()
}

/// Allowed `tool_choice` shapes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatToolChoice {
  /// One of `none`, `auto`, `required`.
  Mode(String),
  /// Named tool selection.
  Named(Value),
}
