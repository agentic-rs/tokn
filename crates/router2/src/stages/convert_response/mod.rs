//! No-op ConvertResponse stage. Drops the upstream response and returns an
//! empty buffered placeholder. Pairs with [`NoopSend`](crate::stages::NoopSend);
//! only reachable when the back-half is wired but neither stub has been swapped
//! out yet.

use crate::pipeline::ctx::PipelineCtx;
use crate::pipeline::error::PipelineError;
use crate::pipeline::stages::{ConvertResponseStage, ConvertedResponse, SentResponse};
use async_trait::async_trait;
use bytes::Bytes;
use llm_headers::HeaderMap;
use serde_json::Value;

// pub mod default;
// pub use default::DefaultConvertResponse;

pub struct NoopConvertResponse;

#[async_trait]
impl ConvertResponseStage for NoopConvertResponse {
  async fn convert_response(
    &self,
    _ctx: &PipelineCtx,
    _sent: SentResponse,
  ) -> Result<ConvertedResponse, PipelineError> {
    Ok(ConvertedResponse::Buffered {
      status: 0,
      headers: HeaderMap::new(),
      body_json: Value::Null,
      body_bytes: Bytes::new(),
    })
  }
}
