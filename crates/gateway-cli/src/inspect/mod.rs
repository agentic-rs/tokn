//! Standalone, loopback-only inspector for persisted request history.
//!
//! The inspector intentionally stays out of the main API router. Its request
//! data can include prompts and responses, so it gets a separate local listener
//! and reads existing day databases without changing them.

use axum::extract::{Query, State};
use axum::http::header::{
  CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use tokn_persistence::{get_request, get_session, list_requests, list_sessions, RequestListOptions};

const INDEX_HTML: &str = include_str!("../../web/dist/index.html");
const VIEWER_JS: &str = include_str!("../../web/dist/viewer.js");
const VIEWER_CSS: &str = include_str!("../../web/dist/viewer.css");
const CONTENT_SECURITY_POLICY_VALUE: &str =
  "default-src 'self'; connect-src 'self'; style-src 'self'; script-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

#[derive(Clone)]
struct InspectState {
  requests_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RequestsQuery {
  limit: Option<usize>,
  session_id: Option<String>,
  provider_id: Option<String>,
  status: Option<u16>,
  query: Option<String>,
}

impl From<RequestsQuery> for RequestListOptions {
  fn from(query: RequestsQuery) -> Self {
    Self {
      limit: query.limit,
      session_id: query.session_id,
      provider_id: query.provider_id,
      status: query.status,
      query: query.query,
    }
  }
}

#[derive(Debug, Deserialize)]
struct LimitQuery {
  limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RequestDetailQuery {
  day: String,
  request_id: String,
}

#[derive(Debug, Deserialize)]
struct SessionDetailQuery {
  session_id: String,
  limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ViewerInfo {
  requests_dir: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
  error: String,
}

struct ApiError {
  status: StatusCode,
  message: String,
}

impl ApiError {
  fn internal(error: impl std::fmt::Display) -> Self {
    Self {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      message: error.to_string(),
    }
  }

  fn not_found(kind: &str) -> Self {
    Self {
      status: StatusCode::NOT_FOUND,
      message: format!("{kind} not found"),
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    json_response(self.status, ErrorResponse { error: self.message })
  }
}

/// Serve the viewer on a separate loopback listener until Ctrl-C.
pub async fn serve(requests_dir: PathBuf, port: u16) -> anyhow::Result<()> {
  let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).await?;
  let address = listener.local_addr()?;
  println!("Inspect viewer: http://{address}");
  println!("Request history may contain sensitive prompts and responses. Press Ctrl-C to stop.");

  axum::serve(listener, router(InspectState { requests_dir }))
    .with_graceful_shutdown(wait_for_shutdown())
    .await?;
  Ok(())
}

fn router(state: InspectState) -> Router {
  Router::new()
    .route("/", get(index))
    .route("/assets/viewer.js", get(viewer_js))
    .route("/assets/viewer.css", get(viewer_css))
    .route("/api/info", get(info))
    .route("/api/requests", get(requests))
    .route("/api/request", get(request_detail))
    .route("/api/sessions", get(sessions))
    .route("/api/session", get(session_detail))
    .with_state(state)
}

async fn index() -> Response {
  text_response("text/html; charset=utf-8", INDEX_HTML)
}

async fn viewer_js() -> Response {
  text_response("application/javascript; charset=utf-8", VIEWER_JS)
}

async fn viewer_css() -> Response {
  text_response("text/css; charset=utf-8", VIEWER_CSS)
}

async fn info(State(state): State<InspectState>) -> Response {
  json_response(
    StatusCode::OK,
    ViewerInfo {
      requests_dir: state.requests_dir.display().to_string(),
    },
  )
}

async fn requests(State(state): State<InspectState>, Query(query): Query<RequestsQuery>) -> Result<Response, ApiError> {
  let requests = list_requests(&state.requests_dir, &query.into()).map_err(ApiError::internal)?;
  Ok(json_response(StatusCode::OK, requests))
}

async fn request_detail(
  State(state): State<InspectState>,
  Query(query): Query<RequestDetailQuery>,
) -> Result<Response, ApiError> {
  let request = get_request(&state.requests_dir, &query.day, &query.request_id).map_err(ApiError::internal)?;
  let request = request.ok_or_else(|| ApiError::not_found("request"))?;
  Ok(json_response(StatusCode::OK, request))
}

async fn sessions(State(state): State<InspectState>, Query(query): Query<LimitQuery>) -> Result<Response, ApiError> {
  let sessions = list_sessions(&state.requests_dir, query.limit).map_err(ApiError::internal)?;
  Ok(json_response(StatusCode::OK, sessions))
}

async fn session_detail(
  State(state): State<InspectState>,
  Query(query): Query<SessionDetailQuery>,
) -> Result<Response, ApiError> {
  let session = get_session(&state.requests_dir, &query.session_id, query.limit).map_err(ApiError::internal)?;
  let session = session.ok_or_else(|| ApiError::not_found("session"))?;
  Ok(json_response(StatusCode::OK, session))
}

fn text_response(content_type: &'static str, body: &'static str) -> Response {
  let mut response = body.into_response();
  response
    .headers_mut()
    .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
  add_security_headers(&mut response);
  response
}

fn json_response<T: Serialize>(status: StatusCode, body: T) -> Response {
  let mut response = (status, Json(body)).into_response();
  add_security_headers(&mut response);
  response
}

fn add_security_headers(response: &mut Response) {
  let headers = response.headers_mut();
  headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
  headers.insert(
    CONTENT_SECURITY_POLICY,
    HeaderValue::from_static(CONTENT_SECURITY_POLICY_VALUE),
  );
  headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
  headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
}

async fn wait_for_shutdown() {
  let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::Body;
  use axum::http::Request;
  use rusqlite::params;
  use tokn_persistence::requests::open_day_db;
  use tower::ServiceExt;

  fn write_request(dir: &std::path::Path, request_id: &str, session_id: &str) {
    let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "INSERT INTO request_connection (request_id, ts, endpoint, status)
         VALUES (?1, 1784444800000, 'responses', 200)",
        params![request_id],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_metadata (request_id, session_id, account_id, provider_id, model)
         VALUES (?1, ?2, 'account-1', 'openai', 'gpt-test')",
        params![request_id, session_id],
      )
      .unwrap();
  }

  #[tokio::test]
  async fn detail_endpoints_accept_encoded_request_and_session_ids() {
    let tempdir = tempfile::tempdir().unwrap();
    write_request(tempdir.path(), "request/one", "session/one");

    let request_response = router(InspectState {
      requests_dir: tempdir.path().to_path_buf(),
    })
    .oneshot(
      Request::builder()
        .uri("/api/request?day=2026-07-14&request_id=request%2Fone")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(request_response.status(), StatusCode::OK);
    assert_eq!(request_response.headers()[CACHE_CONTROL], "no-store");

    let session_response = router(InspectState {
      requests_dir: tempdir.path().to_path_buf(),
    })
    .oneshot(
      Request::builder()
        .uri("/api/session?session_id=session%2Fone")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(session_response.status(), StatusCode::OK);
  }
}
