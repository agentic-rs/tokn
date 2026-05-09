use crate::api::{first_header, PROJECT_ID_HEADERS, REQUEST_ID_HEADERS, SESSION_ID_HEADERS};
use crate::provider::Endpoint;
use axum::http::header::ACCEPT;
use axum::http::HeaderMap;
use llm_core::pipeline::{ParsedRequest, RequestMeta};
use serde_json::Value;

pub(crate) fn infer_stream_request(headers: &HeaderMap, body: &Value) -> bool {
  if let Some(stream) = body.get("stream").and_then(|v| v.as_bool()) {
    return stream;
  }
  headers
    .get(ACCEPT)
    .and_then(|v| v.to_str().ok())
    .map(|v| {
      v.split(',')
        .any(|part| part.split(';').next().map(str::trim) == Some("text/event-stream"))
    })
    .unwrap_or(false)
}

pub(crate) trait RequestParser: Send + Sync {
  fn endpoint(&self) -> Endpoint;

  fn auto_classify_initiator(&self, body: &Value) -> &'static str;

  fn parse(&self, headers: HeaderMap, body: Value) -> ParsedRequest {
    let model = body
      .get("model")
      .and_then(|v| v.as_str())
      .unwrap_or("unknown")
      .to_string();
    let stream = infer_stream_request(&headers, &body);
    let session_id = first_header(&headers, SESSION_ID_HEADERS).map(str::to_string);
    let request_id = first_header(&headers, REQUEST_ID_HEADERS).map(str::to_string);
    let project_id = first_header(&headers, PROJECT_ID_HEADERS).map(str::to_string);
    let header_initiator = headers
      .get("x-initiator")
      .and_then(|v| v.to_str().ok())
      .map(|v| v.trim().to_ascii_lowercase())
      .filter(|v| v == "user" || v == "agent");
    let initiator = header_initiator
      .clone()
      .unwrap_or_else(|| self.auto_classify_initiator(&body).to_string());
    let behave_as = headers
      .get("x-behave-as")
      .and_then(|v| v.to_str().ok())
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty());

    ParsedRequest {
      meta: RequestMeta {
        endpoint: self.endpoint(),
        upstream_endpoint: self.endpoint(),
        model: model.clone(),
        upstream_model: model,
        stream,
        session_id,
        request_id,
        attempt: 0,
        project_id,
        initiator,
        header_initiator,
        behave_as,
        inbound_headers: headers,
      },
      body,
    }
  }
}

pub(crate) struct ChatParser;
pub(crate) struct ResponsesParser;
pub(crate) struct MessagesParser;

impl RequestParser for ChatParser {
  fn endpoint(&self) -> Endpoint {
    Endpoint::ChatCompletions
  }

  fn auto_classify_initiator(&self, body: &Value) -> &'static str {
    crate::util::initiator::classify_initiator(body)
  }
}

impl RequestParser for ResponsesParser {
  fn endpoint(&self) -> Endpoint {
    Endpoint::Responses
  }

  fn auto_classify_initiator(&self, body: &Value) -> &'static str {
    crate::util::initiator::classify_initiator_responses(body)
  }
}

impl RequestParser for MessagesParser {
  fn endpoint(&self) -> Endpoint {
    Endpoint::Messages
  }

  fn auto_classify_initiator(&self, body: &Value) -> &'static str {
    crate::util::initiator::classify_initiator(body)
  }
}
