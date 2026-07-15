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
use tokn_persistence::inspect::{get_request_payload, RequestCursor, RequestPayloadField};
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
  cursor: Option<String>,
  session_id: Option<String>,
  provider_id: Option<String>,
  status: Option<u16>,
  #[serde(default)]
  errors_only: bool,
  query: Option<String>,
}

impl From<RequestsQuery> for RequestListOptions {
  fn from(query: RequestsQuery) -> Self {
    Self {
      day: query.day,
      limit: query.limit,
      cursor: None,
      session_id: query.session_id,
      provider_id: query.provider_id,
      status: query.status,
      errors_only: query.errors_only,
      query: query.query,
    }
  }
}

#[derive(Debug, Deserialize)]
struct LimitQuery {
  limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RequestPageQuery {
  limit: Option<usize>,
  cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RequestDetailQuery {
  day: String,
  request_id: String,
  row_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RequestPayloadQuery {
  day: String,
  request_id: String,
  row_id: Option<i64>,
  field: String,
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
    .route("/api/request-payload", get(request_payload))
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
  let cursor = parse_request_cursor(query.cursor.as_deref())?;
  if let (Some(day), Some(cursor)) = (query.day.as_deref(), cursor.as_ref()) {
    if cursor.day() != day {
      return Err(ApiError::bad_request("cursor does not belong to the selected day"));
    }
  }
  let selected_day = query.day.is_some();
  let requests_dir = state.requests_dir;
  let mut options: RequestListOptions = query.into();
  options.cursor = cursor;
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
  Query(query): Query<RequestPageQuery>,
) -> Result<Response, ApiError> {
  let requests_dir = state.requests_dir;
  let limit = query.limit;
  let cursor = parse_request_cursor(query.cursor.as_deref())?;
  let requests = tokio::task::spawn_blocking(move || list_latest_requests(&requests_dir, limit, cursor))
    .await
    .map_err(ApiError::internal)?
    .map_err(ApiError::internal)?;
  Ok(json_response(StatusCode::OK, requests))
}

fn parse_request_cursor(cursor: Option<&str>) -> Result<Option<RequestCursor>, ApiError> {
  cursor
    .map(RequestCursor::decode)
    .transpose()
    .map_err(|_| ApiError::bad_request("cursor is malformed or unsupported"))
}

async fn request_detail(
  State(state): State<InspectState>,
  Query(query): Query<RequestDetailQuery>,
) -> Result<Response, ApiError> {
  validate_request_day(&query.day)?;
  let requests_dir = state.requests_dir;
  let day = query.day;
  let request_id = query.request_id;
  let row_id = query.row_id;
  let request = tokio::task::spawn_blocking(move || get_request(&requests_dir, &day, &request_id, row_id))
    .await
    .map_err(ApiError::internal)?
    .map_err(|_| ApiError::unavailable("request day"))?;
  let request = request.ok_or_else(|| ApiError::not_found("request"))?;
  Ok(json_response(StatusCode::OK, request))
}

async fn request_payload(
  State(state): State<InspectState>,
  Query(query): Query<RequestPayloadQuery>,
) -> Result<Response, ApiError> {
  validate_request_day(&query.day)?;
  let field = query
    .field
    .parse::<RequestPayloadField>()
    .map_err(|_| ApiError::bad_request("field is not a supported request payload field"))?;
  let requests_dir = state.requests_dir;
  let day = query.day;
  let request_id = query.request_id;
  let row_id = query.row_id;
  let payload =
    tokio::task::spawn_blocking(move || get_request_payload(&requests_dir, &day, &request_id, row_id, field))
      .await
      .map_err(ApiError::internal)?
      .map_err(|_| ApiError::unavailable("request day"))?;
  let payload = payload.ok_or_else(|| ApiError::not_found("request payload"))?;
  Ok(json_response(StatusCode::OK, payload))
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
    conn
      .execute(
        "INSERT INTO request_downstream (request_id, inbound_req_body) VALUES (?1, '{\"input\":\"hello\"}')",
        params![request_id],
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

  fn json_string_field(body: &str, field: &str) -> Option<String> {
    let prefix = format!(r#""{field}":""#);
    let value = body.split_once(&prefix)?.1.split_once('"')?.0;
    Some(value.to_string())
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
    assert!(!response_body(request_response).await.contains("inbound_req_body"));

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
  async fn request_endpoints_page_without_duplicates_and_validate_cursors() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    for request_id in ["request-a", "request-b", "request-c"] {
      write_request(tempdir.path(), "2026-07-14", request_id, "session/one");
    }

    let first_response = get_response(tempdir.path(), &sessions_db, "/api/requests?day=2026-07-14&limit=2").await;
    assert_eq!(first_response.status(), StatusCode::OK);
    let first_body = response_body(first_response).await;
    assert!(first_body.starts_with(r#"{"requests":["#));
    assert!(first_body.contains("request-c"));
    assert!(first_body.contains("request-b"));
    assert!(!first_body.contains("request-a"));
    assert!(first_body.contains(r#""row_id":"3""#));
    let cursor = json_string_field(&first_body, "next_cursor").unwrap();

    let second_response = get_response(
      tempdir.path(),
      &sessions_db,
      &format!("/api/requests?day=2026-07-14&limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(second_response.status(), StatusCode::OK);
    let second_body = response_body(second_response).await;
    assert!(second_body.contains("request-a"));
    assert!(!second_body.contains("request-b"));
    assert!(!second_body.contains("request-c"));
    assert!(second_body.contains(r#""next_cursor":null"#));

    let latest_response = get_response(tempdir.path(), &sessions_db, "/api/requests/latest?limit=2").await;
    assert_eq!(latest_response.status(), StatusCode::OK);
    let latest_body = response_body(latest_response).await;
    assert!(latest_body.contains(r#""day":"2026-07-14""#));
    let latest_cursor = json_string_field(&latest_body, "next_cursor").unwrap();
    let latest_next_response = get_response(
      tempdir.path(),
      &sessions_db,
      &format!("/api/requests/latest?limit=2&cursor={latest_cursor}"),
    )
    .await;
    assert_eq!(latest_next_response.status(), StatusCode::OK);
    let latest_next_body = response_body(latest_next_response).await;
    assert!(latest_next_body.contains("request-a"));
    assert!(!latest_next_body.contains("request-b"));

    let malformed_response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/requests?day=2026-07-14&cursor=not-a-cursor",
    )
    .await;
    assert_eq!(malformed_response.status(), StatusCode::BAD_REQUEST);

    let wrong_day_response = get_response(
      tempdir.path(),
      &sessions_db,
      &format!("/api/requests?day=2026-07-13&cursor={cursor}"),
    )
    .await;
    assert_eq!(wrong_day_response.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn request_payload_endpoint_is_lazy_strict_and_base64_safe() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    write_request(tempdir.path(), "2026-07-14", "request/one", "session/one");
    let conn = open_day_db(&tempdir.path().join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "INSERT INTO request_upstream (request_id, outbound_resp_body) VALUES (?1, ?2)",
        params!["request/one", &[0xff_u8, 0x00]],
      )
      .unwrap();

    let overview = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=request%2Fone&row_id=1",
    )
    .await;
    assert_eq!(overview.status(), StatusCode::OK);
    let overview_body = response_body(overview).await;
    assert!(overview_body.contains("request/one"));
    assert!(overview_body.contains(r#""row_id":"1""#));
    assert!(!overview_body.contains("inbound_req_body"));
    assert!(!overview_body.contains("outbound_resp_body"));

    let json_payload = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request-payload?day=2026-07-14&request_id=request%2Fone&row_id=1&field=inbound_req_body",
    )
    .await;
    assert_eq!(json_payload.status(), StatusCode::OK);
    let json_payload_body = response_body(json_payload).await;
    assert!(json_payload_body.contains(r#""field":"inbound_req_body""#));
    assert!(json_payload_body.contains(r#""input":"hello""#));

    let binary_payload = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request-payload?day=2026-07-14&request_id=request%2Fone&row_id=1&field=outbound_resp_body",
    )
    .await;
    assert_eq!(binary_payload.status(), StatusCode::OK);
    let binary_payload_body = response_body(binary_payload).await;
    assert!(binary_payload_body.contains(r#""encoding":"base64""#));
    assert!(binary_payload_body.contains(r#""data":"/wA=""#));

    let invalid_field = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request-payload?day=2026-07-14&request_id=request%2Fone&field=endpoint",
    )
    .await;
    assert_eq!(invalid_field.status(), StatusCode::BAD_REQUEST);

    let missing_request = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request-payload?day=2026-07-14&request_id=missing&field=inbound_req_body",
    )
    .await;
    assert_eq!(missing_request.status(), StatusCode::NOT_FOUND);

    let mismatched_identity = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=request%2Fone&row_id=2",
    )
    .await;
    assert_eq!(mismatched_identity.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn row_id_disambiguates_duplicate_legacy_request_ids() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    let requests_db = tempdir.path().join("2026-07-14.db");
    let conn = rusqlite::Connection::open(requests_db).unwrap();
    conn
      .execute_batch(
        "CREATE TABLE requests (
           id INTEGER PRIMARY KEY,
           ts INTEGER NOT NULL,
           request_id TEXT,
           model TEXT,
           inbound_req_body BLOB
         );
         CREATE TABLE schema_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_ts INTEGER NOT NULL
         );
         INSERT INTO schema_migrations (version, name, applied_ts) VALUES
           (1, 'initial', 0),
           (2, 'correlation_and_error', 0);
         INSERT INTO requests (id, ts, request_id, model, inbound_req_body) VALUES
           (11, 1784444800, 'duplicate', 'model-11', '{\"row_id\":11}'),
           (12, 1784444800, 'duplicate', 'model-12', '{\"row_id\":12}');",
      )
      .unwrap();
    drop(conn);

    let overview = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=duplicate&row_id=11",
    )
    .await;
    assert_eq!(overview.status(), StatusCode::OK);
    let overview_body = response_body(overview).await;
    assert!(overview_body.contains(r#""row_id":"11""#));
    assert!(overview_body.contains("model-11"));
    assert!(!overview_body.contains("model-12"));

    let payload = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request-payload?day=2026-07-14&request_id=duplicate&row_id=12&field=inbound_req_body",
    )
    .await;
    assert_eq!(payload.status(), StatusCode::OK);
    let payload_body = response_body(payload).await;
    assert!(payload_body.contains(r#""row_id":12"#));

    let fallback = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/request?day=2026-07-14&request_id=duplicate",
    )
    .await;
    assert_eq!(fallback.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn requests_endpoint_filters_errors_only() {
    let tempdir = tempfile::tempdir().unwrap();
    let sessions_db = tempdir.path().join("sessions.db");
    write_request(tempdir.path(), "2026-07-14", "healthy", "session/one");
    write_request(tempdir.path(), "2026-07-14", "failed", "session/one");
    let conn = open_day_db(&tempdir.path().join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "UPDATE request_connection SET request_error = 'failed' WHERE request_id = 'failed'",
        [],
      )
      .unwrap();

    let response = get_response(
      tempdir.path(),
      &sessions_db,
      "/api/requests?day=2026-07-14&errors_only=true",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body(response).await;
    assert!(body.contains("failed"));
    assert!(!body.contains("healthy"));
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
