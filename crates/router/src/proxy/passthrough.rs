use crate::api::{codec::decode_json_request, AppState};
use crate::pipeline::infer_stream_request;
use crate::relay::{is_sse_response, passthrough_buffered_response, passthrough_streaming_response, ForwardContext};
use anyhow::{Context, Result};
use axum::body::Body;
use axum::http::{Request, Response};
use axum::response::IntoResponse;
use bytes::Bytes;
use http::header::{HeaderValue, HOST};

pub(super) async fn proxy_passthrough(
  state: &AppState,
  http: &reqwest::Client,
  host: &str,
  req: Request<hyper::body::Incoming>,
) -> Result<Response<Body>> {
  let started = std::time::Instant::now();
  let path_and_query = req
    .uri()
    .path_and_query()
    .map(|v| v.as_str().to_string())
    .unwrap_or_else(|| "/".to_string());
  let url = format!("https://{host}{path_and_query}");
  let (parts, body) = req.into_parts();
  let request_body = axum::body::to_bytes(Body::new(body), usize::MAX)
    .await
    .context("read passthrough request body")?;
  let decoded_req = match decode_json_request(&parts.headers, request_body.clone()) {
    Ok(decoded) => decoded,
    Err(err) => return Ok(err.into_response()),
  };

  let mut upstream = http.request(parts.method.clone(), &url).body(request_body.clone());
  let mut outbound_req_headers = parts.headers.clone();
  for (name, value) in &parts.headers {
    if name != HOST {
      upstream = upstream.header(name, value);
    }
  }
  upstream = upstream.header(HOST, host);
  outbound_req_headers.insert(
    HOST,
    HeaderValue::from_str(host).unwrap_or_else(|_| HeaderValue::from_static("localhost")),
  );

  let path = path_and_query.split('?').next().unwrap_or(&path_and_query);
  let req_body_json = decoded_req.value.clone();
  let ctx = ForwardContext::from_passthrough(&parts.method, path, &parts.headers, &req_body_json, started);

  let project_id = crate::api::first_header(&parts.headers, crate::api::PROJECT_ID_HEADERS).map(|s| s.to_string());
  let header_initiator = parts
    .headers
    .get("x-initiator")
    .and_then(|v| v.to_str().ok())
    .map(|v| v.trim().to_ascii_lowercase())
    .filter(|v| v == "user" || v == "agent");
  state.events.emit(llm_core::event::Event::RequestStarted {
    request_id: ctx.request_id.clone(),
    ts: std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs() as i64,
    endpoint: ctx.endpoint.map(|e| e.as_str()).unwrap_or(path).to_string(),
    initiator: header_initiator.clone(),
    session_id: ctx.session_id.clone(),
    project_id: project_id.clone(),
    inbound_req: crate::db::HttpSnapshot {
      method: Some(parts.method.to_string()),
      url: Some(url.clone()),
      status: None,
      headers: parts.headers.clone(),
      body: request_body.clone(),
    },
  });
  let mut completion = crate::pipeline::completion::CompletionGuard::new(state.events.clone(), ctx.request_id.clone(), started);
  let initiator = header_initiator
    .unwrap_or_else(|| crate::util::initiator::classify_initiator(&req_body_json).to_string());
  let stream = infer_stream_request(&parts.headers, &req_body_json);
  state.events.emit(llm_core::event::Event::RequestParsed {
    request_id: ctx.request_id.clone(),
    attempt: ctx.attempt,
    account_id: "passthrough".to_string(),
    provider_id: host.to_string(),
    model: ctx.model.clone(),
    stream,
    initiator,
    outbound_req: Some(crate::db::HttpSnapshot {
      method: Some(parts.method.to_string()),
      url: Some(url.clone()),
      status: None,
      headers: outbound_req_headers.clone(),
      body: Bytes::from(serde_json::to_vec(&req_body_json).unwrap_or_default()),
    }),
  });

  let response = match upstream.send().await {
    Ok(response) => response,
    Err(err) => {
      completion.failure(None, err.to_string());
      return Err(err).context("send passthrough upstream request");
    }
  };
  let status = response.status();
  state.events.emit(llm_core::event::Event::RequestResponded {
    request_id: ctx.request_id.clone(),
    attempt: ctx.attempt,
    status: status.as_u16(),
    latency_ms: started.elapsed().as_millis() as u64,
    resp_headers: response.headers().clone(),
  });

  if is_sse_response(response.headers(), stream) {
    completion.disarm();
    let resp = passthrough_streaming_response(state.clone(), ctx, &req_body_json, response);
    return Ok(resp);
  }

  let resp = passthrough_buffered_response(state, &ctx, &req_body_json, response).await;
  completion.disarm();
  Ok(resp)
}
