use serde::{Deserialize, Serialize};

use llm_endpoint_core::{Extras, FinishReason, Usage};

use crate::message::ChatMessage;

/// Non-streaming response body returned by `chat.completions`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub object: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub created: Option<i64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub model: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub choices: Vec<ChatChoice>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub usage: Option<Usage>,
  #[serde(default, flatten)]
  pub extras: Extras,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatChoice {
  #[serde(default)]
  pub index: u32,
  pub message: ChatMessage,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub finish_reason: Option<FinishReason>,
  #[serde(default, flatten)]
  pub extras: Extras,
}
