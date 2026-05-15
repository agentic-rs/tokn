use serde::{Deserialize, Serialize};

use crate::extras::Extras;

/// Normalized token accounting fields shared across endpoints.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Usage {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub input_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub output_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub total_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cached_input_tokens: Option<u64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub reasoning_output_tokens: Option<u64>,
  #[serde(default, flatten)]
  pub extras: Extras,
}
