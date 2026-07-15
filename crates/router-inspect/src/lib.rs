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
use tokn_persistence::{
  get_request, get_session, is_valid_request_day, list_latest_requests, list_request_days, list_requests,
  list_sessions_from_db, RequestListOptions,
};

const INDEX_HTML: &str = include_str!("../web/dist/index.html");
const VIEWER_JS: &str = include_str!("../web/dist/viewer.js");
const VIEWER_CSS: &str = include_str!("../web/dist/viewer.css");
const CONTENT_SECURITY_POLICY_VALUE: &str =
  "default-src 'self'; connect-src 'self'; style-src 'self'; script-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

#[derive(Clone)]
struct InspectState {
  requests_dir: PathBuf,
  sessions_db: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RequestsQuery {
  day: Option<String>,
  limit: Option<usize>,
  session_id: Option<String>,
  provider_id: Option<String>,
  status: Option<u16>,
  query: Option<String>,
}

impl From<RequestsQuery> for RequestListOptions {
  fn from(query: RequestsQuery) -> Self {
    Self {
      day: query.day,
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
  sessions_db: String,
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

  fn bad_request(message: &str) -> Self {
    Self {
      status: StatusCode::BAD_REQUEST,
      message: message.to_string(),
    }
  }

  fn unavailable(kind: &str) -> Self {
    Self {
      status: StatusCode::SERVICE_UNAVAILABLE,
      message: format!("{kind} unavailable"),
    }
  }

  fn unavailable_message(message: impl Into<String>) -> Self {
    Self {
      status: StatusCode::SERVICE_UNAVAILABLE,
      message: message.into(),
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    json_response(self.status, ErrorResponse { error: self.message })
  }
}

/// Serve the viewer on a separate loopback listener until Ctrl-C.
pub async fn serve(requests_dir: PathBuf, sessions_db: PathBuf, port: u16) -> anyhow::Result<()> {
  let listener = tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))).await?;
  let address = listener.local_addr()?;
  println!("Inspect viewer: http://{address}");
  println!("Request history may contain sensitive prompts and responses. Press Ctrl-C to stop.");

  axum::serve(
    listener,
    router(InspectState {
      requests_dir,
      sessions_db,
    }),
  )
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
    .route("/api/request-days", get(request_days))
    .route("/api/requests", get(requests))
    .route("/api/requests/latest", get(latest_requests))
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
      sessions_db: state.sessions_db.display().to_string(),
    },
  )
}

async fn request_days(State(state): State<InspectState>) -> Result<Response, ApiError> {
  let requests_dir = state.requests_dir;
  let days = tokio::task::spawn_blocking(move || list_request_days(&requests_dir))
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
  Ok(json_response(StatusCode::OK, days))
}

async fn requests(State(state): State<InspectState>, Query(query): Query<RequestsQuery>) -> Result<Response, ApiError> {
  if let Some(day) = query.day.as_deref() {
    validate_request_day(day)?;
  }
  let selected_day = query.day.is_some();
  let requests_dir = state.requests_dir;
  let options = query.into();
  let requests = tokio::task::spawn_blocking(move || list_requests(&requests_dir, &options))
    .await
    .map_err(ApiError::internal)?;
  let requests = requests.map_err(|error| {
    if selected_day {
      ApiError::unavailable("request day")
    } else {
      ApiError::internal(error)
    }
  })?;
  Ok(json_response(StatusCode::OK, requests))
}

async fn latest_requests(
  State(state): State<InspectState>,
  Query(query): Query<LimitQuery>,
) -> Result<Response, ApiError> {
  let requests_dir = state.requests_dir;
  let limit = query.limit;
  let requests = tokio::task::spawn_blocking(move || list_latest_requests(&requests_dir, limit))
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
  Ok(json_response(StatusCode::OK, requests))
}

async fn request_detail(
  State(state): State<InspectState>,
  Query(query): Query<RequestDetailQuery>,
) -> Result<Response, ApiError> {
  validate_request_day(&query.day)?;
  let requests_dir = state.requests_dir;
  let day = query.day;
  let request_id = query.request_id;
  let request = tokio::task::spawn_blocking(move || get_request(&requests_dir, &day, &request_id))
    .await
    .map_err(ApiError::internal)?
    .map_err(|_| ApiError::unavailable("request day"))?;
  let request = request.ok_or_else(|| ApiError::not_found("request"))?;
  Ok(json_response(StatusCode::OK, request))
}

fn validate_request_day(day: &str) -> Result<(), ApiError> {
  if is_valid_request_day(day) {
    Ok(())
  } else {
    Err(ApiError::bad_request("day must be a UTC date in YYYY-MM-DD format"))
  }
}

async fn sessions(State(state): State<InspectState>, Query(query): Query<LimitQuery>) -> Result<Response, ApiError> {
  let sessions_db = state.sessions_db;
  let limit = query.limit;
  let sessions = tokio::task::spawn_blocking(move || list_sessions_from_db(&sessions_db, limit))
    .await
    .map_err(ApiError::internal)?
    .map_err(|error| match error {
      tokn_persistence::Error::UnsupportedSessionSchema { .. } => ApiError::unavailable_message(
        "sessions database requires migration; migrate the selected database before opening the sessions view",
      ),
      _ => ApiError::unavailable("session database"),
    })?;
  Ok(json_response(StatusCode::OK, sessions))
}

async fn session_detail(
  State(state): State<InspectState>,
  Query(query): Query<SessionDetailQuery>,
) -> Result<Response, ApiError> {
  let requests_dir = state.requests_dir;
  let session_id = query.session_id;
  let limit = query.limit;
  let session = tokio::task::spawn_blocking(move || get_session(&requests_dir, &session_id, limit))
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
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
  use tokn_persistence::sessions::{SessionsDb, TreeRequestRecord};
  use tokn_persistence::MessageRecord;
  use tower::ServiceExt;

  fn write_request(dir: &std::path::Path, day: &str, request_id: &str, session_id: &str) {
    let conn = open_day_db(&dir.join(format!("{day}.db"))).unwrap();
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

  fn write_session(sessions_db: &std::path::Path, session_id: &str, request_id: &str) {
    let mut sessions = SessionsDb::open(sessions_db).unwrap();
    sessions
      .record_tree(&TreeRequestRecord {
        ts: 1_783_987_200_000,
        session_id: session_id.to_string(),
        parent_session_id: None,
        request_id: request_id.to_string(),
        endpoint: "responses".to_string(),
        status: Some(200),
        account_id: Some("account-1".to_string()),
        provider_id: Some("openai".to_string()),
        model: Some("gpt-test".to_string()),
        request_messages: vec![MessageRecord {
          role: "user".to_string(),
          status: None,
          parts: Vec::new(),
        }],
        response_messages: Vec::new(),
      })
      .unwrap();
  }

  async fn get_response(requests_dir: &std::path::Path, sessions_db: &std::path::Path, uri: &str) -> Response {
    router(InspectState {
      requests_dir: requests_dir.to_path_buf(),
      sessions_db: sessions_db.to_path_buf(),
    })
    .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
    .await
    .unwrap()
  }

  async fn response_body(response: Response) -> String {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
  }

  #[tokio::test]
  async fn history_endpoints_preserve_success_and_not_found_statuses() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    write_request(tempdir.path(), "2026-07-14", "request/one", "session/one");
    write_request(tempdir.path(), "2026-07-13", "request/two", "session/two");
    drop(open_day_db(&tempdir.path().join("2026-07-15.db")).unwrap());
    std::fs::write(tempdir.path().join("2026-07-16.db"), b"not a sqlite database").unwrap();

    let requests_response = get_response(tempdir.path(), &sessions_db, "/api/requests").await;
    assert_eq!(requests_response.status(), StatusCode::OK);
    assert_eq!(requests_response.headers()[CACHE_CONTROL], "no-store");

    let day_requests_response = get_response(tempdir.path(), &sessions_db, "/api/requests?day=2026-07-14").await;
    assert_eq!(day_requests_response.status(), StatusCode::OK);
    let day_requests_body = response_body(day_requests_response).await;
    assert!(day_requests_body.contains("request/one"));
    assert!(!day_requests_body.contains("request/two"));

    let request_days_response = get_response(tempdir.path(), &sessions_db, "/api/request-days").await;
    assert_eq!(request_days_response.status(), StatusCode::OK);
    let request_days_body = response_body(request_days_response).await;
    assert!(request_days_body.contains(r#"{"day":"2026-07-14","state":"available"}"#));
    assert!(request_days_body.contains(r#"{"day":"2026-07-15","state":"empty"}"#));
    assert!(request_days_body.contains(r#"{"day":"2026-07-16","state":"unavailable"}"#));

    let latest_requests_response = get_response(tempdir.path(), &sessions_db, "/api/requests/latest?limit=1").await;
    assert_eq!(latest_requests_response.status(), StatusCode::OK);
    let latest_requests_body = response_body(latest_requests_response).await;
    assert!(latest_requests_body.contains("2026-07-14"));
    assert!(latest_requests_body.contains("request/one"));

    let request_response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=request%2Fone",
    )
    .await;
    assert_eq!(request_response.status(), StatusCode::OK);

    let sessions_response = get_response(tempdir.path(), &sessions_db, "/api/sessions").await;
    assert_eq!(sessions_response.status(), StatusCode::OK);
    assert_eq!(response_body(sessions_response).await, "[]");

    let session_response = get_response(tempdir.path(), &sessions_db, "/api/session?session_id=session%2Fone").await;
    assert_eq!(session_response.status(), StatusCode::OK);

    let missing_request_response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=request%2Fmissing",
    )
    .await;
    assert_eq!(missing_request_response.status(), StatusCode::NOT_FOUND);

    let invalid_day_response = get_response(tempdir.path(), &sessions_db, "/api/requests?day=not-a-day").await;
    assert_eq!(invalid_day_response.status(), StatusCode::BAD_REQUEST);

    let unavailable_day_response = get_response(tempdir.path(), &sessions_db, "/api/requests?day=2026-07-16").await;
    assert_eq!(unavailable_day_response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let invalid_detail_day_response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-02-30&request_id=request%2Fone",
    )
    .await;
    assert_eq!(invalid_detail_day_response.status(), StatusCode::BAD_REQUEST);

    let missing_session_response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/session?session_id=session%2Fmissing",
    )
    .await;
    assert_eq!(missing_session_response.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn sessions_endpoint_reads_only_the_sessions_database() {
    let tempdir = tempfile::tempdir().unwrap();
    let requests_dir = tempdir.path().join("requests");
    std::fs::create_dir(&requests_dir).unwrap();
    write_request(&requests_dir, "2026-07-14", "request-only", "request-only-session");
    let sessions_db = tempdir.path().join("sessions.db");
    write_session(&sessions_db, "stored-session", "stored-request");

    let response = get_response(&requests_dir, &sessions_db, "/api/sessions").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body(response).await;
    assert!(body.contains("stored-session"));
    assert!(!body.contains("request-only-session"));
  }

  #[tokio::test]
  async fn sessions_endpoint_succeeds_without_request_history() {
    let tempdir = tempfile::tempdir().unwrap();
    let requests_dir = tempdir.path().join("requests");
    let sessions_db = tempdir.path().join("sessions.db");
    write_session(&sessions_db, "stored-session", "stored-request");

    let response = get_response(&requests_dir, &sessions_db, "/api/sessions").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response_body(response).await.contains("stored-session"));
    assert!(!requests_dir.exists());
  }

  #[tokio::test]
  async fn sessions_endpoint_reports_an_unavailable_sessions_database() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    std::fs::write(&sessions_db, b"not a sqlite database").unwrap();

    let response = get_response(tempdir.path(), &sessions_db, "/api/sessions").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
  }

  #[tokio::test]
  async fn sessions_endpoint_explains_when_the_database_needs_migration() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    let conn = rusqlite::Connection::open(&sessions_db).unwrap();
    conn
      .execute_batch(
        "CREATE TABLE schema_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_ts INTEGER NOT NULL
         );
         INSERT INTO schema_migrations (version, name, applied_ts) VALUES (1, 'initial', 0);",
      )
      .unwrap();
    drop(conn);

    let response = get_response(tempdir.path(), &sessions_db, "/api/sessions").await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(response_body(response).await.contains("migration"));
  }
}
