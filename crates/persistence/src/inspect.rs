//! Read-only queries for locally persisted request and session history.
//!
//! The gateway writes request history into one SQLite database per UTC day.
//! This module deliberately opens those files in read-only mode: an inspector
//! must never create a database, apply a migration, or take ownership of a
//! writer connection just to display history.

use crate::migrate;
use base64::Engine;
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params, params_from_iter, Connection, OpenFlags};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use time::macros::format_description;

use crate::Result;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;
const CURRENT_TS_MILLIS_SCHEMA_VERSION: u32 = 8;
const SPLIT_REQUESTS_SCHEMA_VERSION: u32 = 7;
const REQUEST_ID_SCHEMA_VERSION: u32 = 2;
const SESSION_TREE_SCHEMA_VERSION: u32 = 2;
const READ_BUSY_TIMEOUT: Duration = Duration::from_millis(2_500);
const DAY_PROBE_BUSY_TIMEOUT: Duration = Duration::from_millis(100);
const JSON_COLUMNS: &[&str] = &[
  "ctx_json",
  "params_json",
  "usage_json",
  "inbound_req_headers",
  "inbound_req_body",
  "inbound_resp_headers",
  "inbound_resp_body",
  "outbound_req_headers",
  "outbound_req_body",
  "outbound_resp_headers",
  "outbound_resp_body",
];
const REQUEST_PAYLOAD_FIELDS: &[&str] = &[
  "inbound_req_headers",
  "inbound_req_body",
  "inbound_resp_headers",
  "inbound_resp_body",
  "outbound_req_headers",
  "outbound_req_body",
  "outbound_resp_headers",
  "outbound_resp_body",
];

/// Query options accepted by the request list and session timeline.
#[derive(Debug, Clone, Default)]
pub struct RequestListOptions {
  pub day: Option<String>,
  pub limit: Option<usize>,
  pub cursor: Option<RequestCursor>,
  pub session_id: Option<String>,
  pub provider_id: Option<String>,
  pub status: Option<u16>,
  pub errors_only: bool,
  pub query: Option<String>,
}

impl RequestListOptions {
  fn effective_limit(&self) -> usize {
    self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
  }
}

/// The availability of a request history database for one UTC day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestDayState {
  Available,
  Empty,
  Unavailable,
}

/// A request history day and the state of its backing database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequestDay {
  pub day: String,
  pub state: RequestDayState,
}

/// A compact request row suitable for an inspector list or session timeline.
#[derive(Debug, Clone, Serialize)]
pub struct RequestSummary {
  #[serde(serialize_with = "serialize_i64_as_string")]
  pub row_id: i64,
  pub day: String,
  pub request_id: String,
  pub ts: i64,
  pub endpoint: Option<String>,
  pub status: Option<u16>,
  pub request_error: Option<String>,
  pub session_id: Option<String>,
  pub account_id: Option<String>,
  pub provider_id: Option<String>,
  pub model: Option<String>,
  pub inbound_req_method: Option<String>,
  pub inbound_req_url: Option<String>,
  pub outbound_resp_status: Option<u16>,
  pub inbound_resp_status: Option<u16>,
}

/// One page of request summaries in stable, newest-first order.
#[derive(Debug, Clone, Serialize)]
pub struct RequestPage {
  pub requests: Vec<RequestSummary>,
  pub next_cursor: Option<String>,
}

/// An opaque position in the newest-first request history ordering.
///
/// Cursors include the day because SQLite row identities are scoped to a day
/// database. Callers should persist and replay the encoded value rather than
/// constructing one themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCursor {
  day: String,
  ts: i64,
  row_id: i64,
}

impl RequestCursor {
  const VERSION: &'static str = "v2";

  pub fn decode(value: &str) -> std::result::Result<Self, InvalidRequestCursor> {
    let mut parts = value.split('.');
    let version = parts.next().ok_or(InvalidRequestCursor)?;
    let day = parts.next().ok_or(InvalidRequestCursor)?;
    let ts = parts
      .next()
      .ok_or(InvalidRequestCursor)?
      .parse::<i64>()
      .map_err(|_| InvalidRequestCursor)?;
    let row_id = parts
      .next()
      .ok_or(InvalidRequestCursor)?
      .parse::<i64>()
      .map_err(|_| InvalidRequestCursor)?;
    if parts.next().is_some() || version != Self::VERSION || !is_valid_request_day(day) {
      return Err(InvalidRequestCursor);
    }
    Ok(Self {
      day: day.to_string(),
      ts,
      row_id,
    })
  }

  pub fn day(&self) -> &str {
    &self.day
  }

  fn from_request(request: &RequestSummary) -> Self {
    Self {
      day: request.day.clone(),
      ts: request.ts,
      row_id: request.row_id,
    }
  }

  fn encode(&self) -> String {
    format!("{}.{}.{}.{}", Self::VERSION, self.day, self.ts, self.row_id)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRequestCursor;

impl std::fmt::Display for InvalidRequestCursor {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("invalid request cursor")
  }
}

impl std::error::Error for InvalidRequestCursor {}

/// Decoded request metadata for one identity (`day`, `request_id`), excluding
/// large network header/body payloads.
#[derive(Debug, Clone, Serialize)]
pub struct RequestDetail {
  pub day: String,
  #[serde(serialize_with = "serialize_i64_as_string")]
  pub row_id: i64,
  pub request: Map<String, Value>,
}

/// One lazily fetched request payload field.
#[derive(Debug, Clone, Serialize)]
pub struct RequestPayload {
  pub field: String,
  pub value: Value,
}

/// Payload fields that may be fetched separately from the request overview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPayloadField {
  InboundReqHeaders,
  InboundReqBody,
  InboundRespHeaders,
  InboundRespBody,
  OutboundReqHeaders,
  OutboundReqBody,
  OutboundRespHeaders,
  OutboundRespBody,
}

impl RequestPayloadField {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::InboundReqHeaders => "inbound_req_headers",
      Self::InboundReqBody => "inbound_req_body",
      Self::InboundRespHeaders => "inbound_resp_headers",
      Self::InboundRespBody => "inbound_resp_body",
      Self::OutboundReqHeaders => "outbound_req_headers",
      Self::OutboundReqBody => "outbound_req_body",
      Self::OutboundRespHeaders => "outbound_resp_headers",
      Self::OutboundRespBody => "outbound_resp_body",
    }
  }
}

impl std::str::FromStr for RequestPayloadField {
  type Err = InvalidRequestPayloadField;

  fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
    match value {
      "inbound_req_headers" => Ok(Self::InboundReqHeaders),
      "inbound_req_body" => Ok(Self::InboundReqBody),
      "inbound_resp_headers" => Ok(Self::InboundRespHeaders),
      "inbound_resp_body" => Ok(Self::InboundRespBody),
      "outbound_req_headers" => Ok(Self::OutboundReqHeaders),
      "outbound_req_body" => Ok(Self::OutboundReqBody),
      "outbound_resp_headers" => Ok(Self::OutboundRespHeaders),
      "outbound_resp_body" => Ok(Self::OutboundRespBody),
      _ => Err(InvalidRequestPayloadField),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRequestPayloadField;

impl std::fmt::Display for InvalidRequestPayloadField {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("invalid request payload field")
  }
}

impl std::error::Error for InvalidRequestPayloadField {}

fn serialize_i64_as_string<S>(value: &i64, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  serializer.serialize_str(&value.to_string())
}

/// A session inferred from request records sharing a `session_id`.
#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
  pub session_id: String,
  pub first_ts: i64,
  pub last_ts: i64,
  pub request_count: u64,
  pub last_request_day: String,
  pub last_request_id: String,
  pub endpoint: Option<String>,
  pub status: Option<u16>,
  pub account_id: Option<String>,
  pub provider_id: Option<String>,
  pub model: Option<String>,
}

/// A session timeline derived directly from the request-day databases.
#[derive(Debug, Clone, Serialize)]
pub struct SessionDetail {
  pub session: SessionSummary,
  pub requests: Vec<RequestSummary>,
}

/// The newest available request day and its latest request rows.
#[derive(Debug, Clone, Serialize)]
pub struct LatestRequests {
  pub day: Option<String>,
  pub requests: Vec<RequestSummary>,
  pub next_cursor: Option<String>,
}

/// Return the most recent request rows across every existing request-day DB.
pub fn list_requests(requests_dir: &Path, options: &RequestListOptions) -> Result<RequestPage> {
  let limit = options.effective_limit();
  let mut requests = Vec::new();
  let fetch_limit = limit.saturating_add(1);

  let day_files = request_day_files(requests_dir)?;
  if let Some(day) = options.day.as_deref() {
    if let Some(day_file) = day_files.iter().find(|day_file| day_file.day == day) {
      requests.extend(read_day_requests(day_file, options, Some(fetch_limit))?);
    }
  } else {
    for day_file in &day_files {
      requests.extend(list_day_requests_best_effort(day_file, options, Some(fetch_limit)));
    }
  }

  requests.sort_by(|left, right| {
    right
      .ts
      .cmp(&left.ts)
      .then_with(|| right.day.cmp(&left.day))
      .then_with(|| right.row_id.cmp(&left.row_id))
  });
  Ok(request_page(requests, limit))
}

/// Return whether `day` is a canonical UTC request-history day (`YYYY-MM-DD`).
pub fn is_valid_request_day(day: &str) -> bool {
  let bytes = day.as_bytes();
  if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
    return false;
  }
  if !bytes
    .iter()
    .enumerate()
    .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
  {
    return false;
  }
  time::Date::parse(day, format_description!("[year]-[month]-[day]")).is_ok()
}

/// List all request-day databases from newest to oldest with their availability.
pub fn list_request_days(requests_dir: &Path) -> Result<Vec<RequestDay>> {
  Ok(
    request_day_files(requests_dir)?
      .into_iter()
      .map(|day_file| RequestDay {
        state: probe_request_day_state_best_effort(&day_file),
        day: day_file.day,
      })
      .collect(),
  )
}

/// Return requests from the most recent non-empty, readable request day.
pub fn list_latest_requests(
  requests_dir: &Path,
  limit: Option<usize>,
  cursor: Option<RequestCursor>,
) -> Result<LatestRequests> {
  if let Some(cursor) = cursor {
    let day = cursor.day.clone();
    let page = list_requests(
      requests_dir,
      &RequestListOptions {
        day: Some(day.clone()),
        limit,
        cursor: Some(cursor),
        ..RequestListOptions::default()
      },
    )?;
    return Ok(LatestRequests {
      day: Some(day),
      requests: page.requests,
      next_cursor: page.next_cursor,
    });
  }

  let options = RequestListOptions {
    limit,
    ..RequestListOptions::default()
  };
  let limit = options.effective_limit();
  let fetch_limit = limit.saturating_add(1);

  for day_file in request_day_files(requests_dir)? {
    match read_day_requests(&day_file, &options, Some(fetch_limit)) {
      Ok(requests) if !requests.is_empty() => {
        let page = request_page(requests, limit);
        return Ok(LatestRequests {
          day: Some(day_file.day),
          requests: page.requests,
          next_cursor: page.next_cursor,
        });
      }
      Ok(_) => {}
      Err(error) => log_day_read_failure(&day_file, &error),
    }
  }

  Ok(LatestRequests {
    day: None,
    requests: Vec::new(),
    next_cursor: None,
  })
}

/// Return request metadata without eagerly loading large network payloads.
pub fn get_request(
  requests_dir: &Path,
  day: &str,
  request_id: &str,
  row_id: Option<i64>,
) -> Result<Option<RequestDetail>> {
  let Some(day_file) = request_day_files(requests_dir)?
    .into_iter()
    .find(|file| file.day == day)
  else {
    return Ok(None);
  };
  let Some(conn) = open_readonly(&day_file.path)? else {
    return Ok(None);
  };

  let version = schema_version(&conn)?;
  let request_condition = request_lookup_condition(version, row_id.is_some());
  let row_id_column = request_row_id_column(version);
  let overview_columns = request_overview_columns(&conn)?;
  let mut projection = vec![quote_identifier(row_id_column)];
  projection.extend(overview_columns.iter().map(|column| quote_identifier(column)));
  let projection = projection.join(", ");
  let mut stmt = conn.prepare(&format!(
    "SELECT {projection} FROM requests WHERE {request_condition} LIMIT 1"
  ))?;
  let mut values = vec![SqlValue::Text(request_id.to_string())];
  if let Some(row_id) = row_id {
    values.push(SqlValue::Integer(row_id));
  }
  let mut rows = stmt.query(params_from_iter(values.iter()))?;
  let Some(row) = rows.next()? else {
    return Ok(None);
  };
  let row_id = row.get(0)?;

  let mut request = Map::with_capacity(overview_columns.len());
  for (index, name) in overview_columns.iter().enumerate() {
    request.insert(name.clone(), sqlite_value_to_json(row.get_ref(index + 1)?, name));
  }
  if version < SPLIT_REQUESTS_SCHEMA_VERSION
    && !matches!(request.get("request_id"), Some(Value::String(value)) if !value.is_empty())
  {
    request.insert("request_id".to_string(), Value::String(request_id.to_string()));
  }
  normalize_timestamp(&mut request, version);

  Ok(Some(RequestDetail {
    day: day_file.day,
    row_id,
    request,
  }))
}

/// Return one explicitly selected request payload field without mutating its database.
pub fn get_request_payload(
  requests_dir: &Path,
  day: &str,
  request_id: &str,
  row_id: Option<i64>,
  field: RequestPayloadField,
) -> Result<Option<RequestPayload>> {
  let Some(day_file) = request_day_files(requests_dir)?
    .into_iter()
    .find(|file| file.day == day)
  else {
    return Ok(None);
  };
  let Some(conn) = open_readonly(&day_file.path)? else {
    return Ok(None);
  };

  let version = schema_version(&conn)?;
  let request_condition = request_lookup_condition(version, row_id.is_some());
  let field_name = field.as_str();
  let mut stmt = conn.prepare(&format!(
    "SELECT {} FROM requests WHERE {request_condition} LIMIT 1",
    quote_identifier(field_name)
  ))?;
  let mut values = vec![SqlValue::Text(request_id.to_string())];
  if let Some(row_id) = row_id {
    values.push(SqlValue::Integer(row_id));
  }
  let mut rows = stmt.query(params_from_iter(values.iter()))?;
  let Some(row) = rows.next()? else {
    return Ok(None);
  };

  Ok(Some(RequestPayload {
    field: field_name.to_string(),
    value: sqlite_value_to_json(row.get_ref(0)?, field_name),
  }))
}

/// Return the most recently active inferred sessions.
pub fn list_sessions(requests_dir: &Path, limit: Option<usize>) -> Result<Vec<SessionSummary>> {
  let mut sessions = collect_sessions(requests_dir)?;
  sessions.sort_by(|left, right| {
    right
      .last_ts
      .cmp(&left.last_ts)
      .then_with(|| left.session_id.cmp(&right.session_id))
  });
  sessions.truncate(limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT));
  Ok(sessions)
}

/// Return the most recently active sessions stored in one `sessions.db` file.
///
/// Unlike [`list_sessions`], this does not scan request-day databases. A
/// missing database is normal when session recording has not been enabled, so
/// it returns an empty list without creating the file. The database must use
/// the tree-shaped session schema introduced in version 2.
pub fn list_sessions_from_db(sessions_db: &Path, limit: Option<usize>) -> Result<Vec<SessionSummary>> {
  let Some(conn) = open_readonly(sessions_db)? else {
    return Ok(Vec::new());
  };

  let version = schema_version(&conn)?;
  if version < SESSION_TREE_SCHEMA_VERSION {
    return Err(crate::Error::UnsupportedSessionSchema { version });
  }

  let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
  let mut stmt = conn.prepare(
    "SELECT
       s.id,
       s.first_seen_ts,
       COALESCE(head.ts, s.last_seen_ts),
       (SELECT COUNT(*) FROM session_nodes AS node_count WHERE node_count.session_id = s.id),
       head.request_id,
       head.endpoint,
       head.status,
       COALESCE(head.account_id, s.account_id),
       COALESCE(head.provider_id, s.provider_id),
       COALESCE(head.model, s.model)
     FROM sessions AS s
     LEFT JOIN session_heads AS head_ref ON head_ref.session_id = s.id
     LEFT JOIN session_nodes AS head ON head.id = head_ref.node_id
     ORDER BY COALESCE(head.ts, s.last_seen_ts) DESC, s.id ASC
     LIMIT ?1",
  )?;
  let sessions = stmt
    .query_map(params![limit as i64], session_summary_from_db_row)?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(sessions)
}

/// Return a chronological, bounded timeline for one inferred session.
pub fn get_session(requests_dir: &Path, session_id: &str, limit: Option<usize>) -> Result<Option<SessionDetail>> {
  let options = RequestListOptions {
    session_id: Some(session_id.to_string()),
    ..RequestListOptions::default()
  };
  let mut session = None;
  let mut requests = Vec::new();

  for day_file in request_day_files(requests_dir)? {
    for request in list_day_requests_best_effort(&day_file, &options, None) {
      if request.session_id.as_deref() != Some(session_id) {
        continue;
      }
      if let Some(summary) = session.as_mut() {
        update_session_summary(summary, &request);
      } else {
        session = Some(new_session_summary(session_id, &request));
      }
      requests.push(request);
    }
  }

  let Some(session) = session else {
    return Ok(None);
  };

  requests.sort_by(|left, right| {
    left
      .ts
      .cmp(&right.ts)
      .then_with(|| left.day.cmp(&right.day))
      .then_with(|| left.request_id.cmp(&right.request_id))
  });
  let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
  if requests.len() > limit {
    requests.drain(..requests.len() - limit);
  }

  Ok(Some(SessionDetail { session, requests }))
}

#[derive(Debug, Clone)]
struct DayFile {
  day: String,
  path: PathBuf,
}

fn request_day_files(requests_dir: &Path) -> Result<Vec<DayFile>> {
  if !requests_dir.exists() {
    return Ok(Vec::new());
  }

  let mut files = Vec::new();
  for entry in std::fs::read_dir(requests_dir)? {
    let entry = entry?;
    if !entry.file_type()?.is_file() {
      continue;
    }
    let path = entry.path();
    if path.extension().and_then(|value| value.to_str()) != Some("db") {
      continue;
    }
    let Some(day) = path.file_stem().and_then(|value| value.to_str()) else {
      continue;
    };
    if !is_valid_request_day(day) {
      continue;
    }
    files.push(DayFile {
      day: day.to_string(),
      path,
    });
  }
  files.sort_by(|left, right| right.day.cmp(&left.day));
  Ok(files)
}

fn open_readonly(path: &Path) -> Result<Option<Connection>> {
  open_readonly_with_timeout(path, READ_BUSY_TIMEOUT)
}

fn open_readonly_with_timeout(path: &Path, busy_timeout: Duration) -> Result<Option<Connection>> {
  if !path.exists() {
    return Ok(None);
  }
  let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
  conn.busy_timeout(busy_timeout)?;
  conn.execute_batch("PRAGMA query_only = ON;")?;
  Ok(Some(conn))
}

fn probe_request_day_state_best_effort(day_file: &DayFile) -> RequestDayState {
  let result = (|| -> Result<RequestDayState> {
    let Some(conn) = open_readonly_with_timeout(&day_file.path, DAY_PROBE_BUSY_TIMEOUT)? else {
      tracing::warn!(
        path = %day_file.path.display(),
        "request history database disappeared while checking its availability"
      );
      return Ok(RequestDayState::Unavailable);
    };
    let state = if day_has_requests(&conn)? {
      RequestDayState::Available
    } else {
      RequestDayState::Empty
    };
    Ok(state)
  })();

  match result {
    Ok(state) => state,
    Err(error) => {
      tracing::warn!(
        path = %day_file.path.display(),
        error = %error,
        "marking request history database unavailable after read failure"
      );
      RequestDayState::Unavailable
    }
  }
}

fn day_has_requests(conn: &Connection) -> Result<bool> {
  let sql = if schema_version(conn)? >= SPLIT_REQUESTS_SCHEMA_VERSION {
    "SELECT EXISTS(SELECT 1 FROM request_connection)"
  } else {
    "SELECT EXISTS(SELECT 1 FROM requests)"
  };
  let has_requests = conn.query_row(sql, [], |row| row.get::<_, i64>(0))?;
  Ok(has_requests != 0)
}

fn list_day_requests_best_effort(
  day_file: &DayFile,
  options: &RequestListOptions,
  limit: Option<usize>,
) -> Vec<RequestSummary> {
  match read_day_requests(day_file, options, limit) {
    Ok(requests) => requests,
    Err(error) => {
      log_day_read_failure(day_file, &error);
      Vec::new()
    }
  }
}

fn read_day_requests(
  day_file: &DayFile,
  options: &RequestListOptions,
  limit: Option<usize>,
) -> Result<Vec<RequestSummary>> {
  let Some(conn) = open_readonly(&day_file.path)? else {
    return Ok(Vec::new());
  };
  list_day_requests(&conn, &day_file.day, options, limit)
}

fn log_day_read_failure(day_file: &DayFile, error: &crate::Error) {
  tracing::warn!(
    path = %day_file.path.display(),
    error = %error,
    "skipping request history database after read failure"
  );
}

fn schema_version(conn: &Connection) -> Result<u32> {
  migrate::read_current_version(conn)
}

fn request_overview_columns(conn: &Connection) -> Result<Vec<String>> {
  let stmt = conn.prepare("SELECT * FROM requests LIMIT 0")?;
  Ok(
    stmt
      .column_names()
      .into_iter()
      .filter(|column| !REQUEST_PAYLOAD_FIELDS.contains(column))
      .map(str::to_string)
      .collect(),
  )
}

fn request_row_id_column(version: u32) -> &'static str {
  if version >= SPLIT_REQUESTS_SCHEMA_VERSION {
    "idx"
  } else {
    "id"
  }
}

fn request_lookup_condition(version: u32, include_row_id: bool) -> String {
  let request_id = if version >= SPLIT_REQUESTS_SCHEMA_VERSION {
    "request_id".to_string()
  } else {
    legacy_request_id_sql(version).to_string()
  };
  if include_row_id {
    format!("{request_id} = ?1 AND {} = ?2", request_row_id_column(version))
  } else {
    format!("{request_id} = ?1")
  }
}

fn quote_identifier(identifier: &str) -> String {
  format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn list_day_requests(
  conn: &Connection,
  day: &str,
  options: &RequestListOptions,
  limit: Option<usize>,
) -> Result<Vec<RequestSummary>> {
  let version = schema_version(conn)?;
  if version >= SPLIT_REQUESTS_SCHEMA_VERSION {
    list_split_day_requests(conn, day, options, limit, version)
  } else {
    list_legacy_day_requests(conn, day, options, limit, version)
  }
}

fn list_split_day_requests(
  conn: &Connection,
  day: &str,
  options: &RequestListOptions,
  limit: Option<usize>,
  version: u32,
) -> Result<Vec<RequestSummary>> {
  let mut sql = String::from(
    "SELECT c.request_id, c.ts, c.endpoint, c.status, c.request_error, m.session_id, m.account_id,
            m.provider_id, m.model, d.inbound_req_method, d.inbound_req_url, u.outbound_resp_status,
            d.inbound_resp_status, c.rowid
     FROM request_connection c
     LEFT JOIN request_metadata m ON m.request_id = c.request_id
     LEFT JOIN request_downstream d ON d.request_id = c.request_id
     LEFT JOIN request_upstream u ON u.request_id = c.request_id
     WHERE 1 = 1",
  );
  let mut values = Vec::new();

  if let Some(session_id) = options.session_id.as_deref() {
    sql.push_str(" AND m.session_id = ?");
    values.push(SqlValue::Text(session_id.to_string()));
  }
  if let Some(provider_id) = options.provider_id.as_deref() {
    sql.push_str(" AND m.provider_id = ?");
    values.push(SqlValue::Text(provider_id.to_string()));
  }
  if let Some(status) = options.status {
    sql.push_str(" AND COALESCE(d.inbound_resp_status, u.outbound_resp_status, c.status) = ?");
    values.push(SqlValue::Integer(i64::from(status)));
  }
  if options.errors_only {
    sql.push_str(
      " AND (c.request_error IS NOT NULL OR c.status >= 400 OR u.outbound_resp_status >= 400
             OR d.inbound_resp_status >= 400)",
    );
  }
  if let Some(query) = options.query.as_deref().filter(|value| !value.is_empty()) {
    sql.push_str(" AND (c.request_id LIKE ? OR m.session_id LIKE ? OR m.model LIKE ?)");
    let pattern = format!("%{query}%");
    values.extend([
      SqlValue::Text(pattern.clone()),
      SqlValue::Text(pattern.clone()),
      SqlValue::Text(pattern),
    ]);
  }
  append_cursor_condition(
    &mut sql,
    &mut values,
    "c.ts",
    "c.rowid",
    day,
    version,
    options.cursor.as_ref(),
  );
  sql.push_str(" ORDER BY c.ts DESC, c.rowid DESC");
  if let Some(limit) = limit {
    sql.push_str(" LIMIT ?");
    values.push(SqlValue::Integer(limit as i64));
  }

  let mut stmt = conn.prepare(&sql)?;
  let rows = stmt
    .query_map(params_from_iter(values.iter()), |row| {
      request_summary_from_row(row, day, version)
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(rows)
}

fn list_legacy_day_requests(
  conn: &Connection,
  day: &str,
  options: &RequestListOptions,
  limit: Option<usize>,
  version: u32,
) -> Result<Vec<RequestSummary>> {
  let request_id = legacy_request_id_sql(version);
  let request_error = if version >= REQUEST_ID_SCHEMA_VERSION {
    "request_error"
  } else {
    "NULL"
  };
  let mut sql = format!(
    "SELECT {request_id}, ts, endpoint, status, {request_error}, session_id, account_id, provider_id,
            model, inbound_req_method, inbound_req_url, outbound_resp_status, inbound_resp_status, id
     FROM requests
     WHERE 1 = 1"
  );
  let mut values = Vec::new();

  if let Some(session_id) = options.session_id.as_deref() {
    sql.push_str(" AND session_id = ?");
    values.push(SqlValue::Text(session_id.to_string()));
  }
  if let Some(provider_id) = options.provider_id.as_deref() {
    sql.push_str(" AND provider_id = ?");
    values.push(SqlValue::Text(provider_id.to_string()));
  }
  if let Some(status) = options.status {
    sql.push_str(" AND COALESCE(inbound_resp_status, outbound_resp_status, status) = ?");
    values.push(SqlValue::Integer(i64::from(status)));
  }
  if options.errors_only {
    sql.push_str(&format!(
      " AND ({request_error} IS NOT NULL OR status >= 400 OR outbound_resp_status >= 400
             OR inbound_resp_status >= 400)"
    ));
  }
  if let Some(query) = options.query.as_deref().filter(|value| !value.is_empty()) {
    sql.push_str(&format!(
      " AND ({request_id} LIKE ? OR session_id LIKE ? OR model LIKE ?)"
    ));
    let pattern = format!("%{query}%");
    values.extend([
      SqlValue::Text(pattern.clone()),
      SqlValue::Text(pattern.clone()),
      SqlValue::Text(pattern),
    ]);
  }
  append_cursor_condition(&mut sql, &mut values, "ts", "id", day, version, options.cursor.as_ref());
  sql.push_str(" ORDER BY ts DESC, id DESC");
  if let Some(limit) = limit {
    sql.push_str(" LIMIT ?");
    values.push(SqlValue::Integer(limit as i64));
  }

  let mut stmt = conn.prepare(&sql)?;
  let rows = stmt
    .query_map(params_from_iter(values.iter()), |row| {
      request_summary_from_row(row, day, version)
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(rows)
}

fn append_cursor_condition(
  sql: &mut String,
  values: &mut Vec<SqlValue>,
  ts_sql: &str,
  row_id_sql: &str,
  day: &str,
  version: u32,
  cursor: Option<&RequestCursor>,
) {
  let Some(cursor) = cursor else {
    return;
  };
  // Legacy schemas persisted whole seconds. Compare in the normalized
  // millisecond domain so a cursor can also safely traverse mixed-version
  // day files in the aggregate listing.
  let ts_sql = if version < CURRENT_TS_MILLIS_SCHEMA_VERSION {
    format!("({ts_sql} * 1000)")
  } else {
    ts_sql.to_string()
  };
  let cursor_ts = cursor.ts;

  match day.cmp(cursor.day.as_str()) {
    // At the cursor timestamp, a newer day sorts before the cursor and must
    // not be returned. Only strictly older timestamps remain eligible.
    std::cmp::Ordering::Greater => {
      sql.push_str(&format!(" AND {ts_sql} < ?"));
      values.push(SqlValue::Integer(cursor_ts));
    }
    // On the cursor day, the SQLite row identity is the stable tie-breaker.
    std::cmp::Ordering::Equal => {
      sql.push_str(&format!(" AND ({ts_sql} < ? OR ({ts_sql} = ? AND {row_id_sql} < ?))"));
      values.extend([
        SqlValue::Integer(cursor_ts),
        SqlValue::Integer(cursor_ts),
        SqlValue::Integer(cursor.row_id),
      ]);
    }
    // An older day sorts after the cursor when timestamps are equal.
    std::cmp::Ordering::Less => {
      sql.push_str(&format!(" AND {ts_sql} <= ?"));
      values.push(SqlValue::Integer(cursor_ts));
    }
  }
}

fn request_page(mut requests: Vec<RequestSummary>, limit: usize) -> RequestPage {
  let has_more = requests.len() > limit;
  requests.truncate(limit);
  let next_cursor = has_more
    .then(|| requests.last().map(RequestCursor::from_request))
    .flatten()
    .map(|cursor| cursor.encode());
  RequestPage { requests, next_cursor }
}

fn request_summary_from_row(row: &rusqlite::Row<'_>, day: &str, version: u32) -> rusqlite::Result<RequestSummary> {
  Ok(RequestSummary {
    row_id: row.get(13)?,
    day: day.to_string(),
    request_id: row.get(0)?,
    ts: normalized_timestamp(row.get(1)?, version),
    endpoint: row.get(2)?,
    status: sqlite_status(row.get(3)?),
    request_error: row.get(4)?,
    session_id: row.get(5)?,
    account_id: row.get(6)?,
    provider_id: row.get(7)?,
    model: row.get(8)?,
    inbound_req_method: row.get(9)?,
    inbound_req_url: row.get(10)?,
    outbound_resp_status: sqlite_status(row.get(11)?),
    inbound_resp_status: sqlite_status(row.get(12)?),
  })
}

fn session_summary_from_db_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionSummary> {
  let last_ts: i64 = row.get(2)?;
  let request_count = row.get::<_, i64>(3)?.max(0) as u64;
  Ok(SessionSummary {
    session_id: row.get(0)?,
    first_ts: row.get(1)?,
    last_ts,
    request_count,
    last_request_day: crate::requests::day_key(last_ts),
    last_request_id: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
    endpoint: row.get(5)?,
    status: sqlite_status(row.get(6)?),
    account_id: row.get(7)?,
    provider_id: row.get(8)?,
    model: row.get(9)?,
  })
}

fn legacy_request_id_sql(version: u32) -> &'static str {
  if version >= REQUEST_ID_SCHEMA_VERSION {
    "CASE WHEN request_id IS NULL OR request_id = '' THEN 'legacy:' || id ELSE request_id END"
  } else {
    "'legacy:' || id"
  }
}

fn collect_sessions(requests_dir: &Path) -> Result<Vec<SessionSummary>> {
  let mut sessions = HashMap::<String, SessionSummary>::new();

  for day_file in request_day_files(requests_dir)? {
    for request in list_day_requests_best_effort(&day_file, &RequestListOptions::default(), None) {
      let Some(session_id) = request.session_id else {
        continue;
      };
      match sessions.get_mut(&session_id) {
        Some(summary) => {
          summary.first_ts = summary.first_ts.min(request.ts);
          summary.request_count += 1;
          if (request.ts, request.day.as_str(), request.request_id.as_str())
            > (
              summary.last_ts,
              summary.last_request_day.as_str(),
              summary.last_request_id.as_str(),
            )
          {
            summary.last_ts = request.ts;
            summary.last_request_day = request.day;
            summary.last_request_id = request.request_id;
            summary.endpoint = request.endpoint;
            summary.status = request.status;
            summary.account_id = request.account_id;
            summary.provider_id = request.provider_id;
            summary.model = request.model;
          }
        }
        None => {
          sessions.insert(
            session_id.clone(),
            SessionSummary {
              session_id,
              first_ts: request.ts,
              last_ts: request.ts,
              request_count: 1,
              last_request_day: request.day,
              last_request_id: request.request_id,
              endpoint: request.endpoint,
              status: request.status,
              account_id: request.account_id,
              provider_id: request.provider_id,
              model: request.model,
            },
          );
        }
      }
    }
  }

  Ok(sessions.into_values().collect())
}

fn new_session_summary(session_id: &str, request: &RequestSummary) -> SessionSummary {
  SessionSummary {
    session_id: session_id.to_string(),
    first_ts: request.ts,
    last_ts: request.ts,
    request_count: 1,
    last_request_day: request.day.clone(),
    last_request_id: request.request_id.clone(),
    endpoint: request.endpoint.clone(),
    status: request.status,
    account_id: request.account_id.clone(),
    provider_id: request.provider_id.clone(),
    model: request.model.clone(),
  }
}

fn update_session_summary(summary: &mut SessionSummary, request: &RequestSummary) {
  summary.first_ts = summary.first_ts.min(request.ts);
  summary.request_count += 1;
  if (request.ts, request.day.as_str(), request.request_id.as_str())
    > (
      summary.last_ts,
      summary.last_request_day.as_str(),
      summary.last_request_id.as_str(),
    )
  {
    summary.last_ts = request.ts;
    summary.last_request_day = request.day.clone();
    summary.last_request_id = request.request_id.clone();
    summary.endpoint = request.endpoint.clone();
    summary.status = request.status;
    summary.account_id = request.account_id.clone();
    summary.provider_id = request.provider_id.clone();
    summary.model = request.model.clone();
  }
}

fn normalized_timestamp(ts: i64, schema_version: u32) -> i64 {
  if schema_version < CURRENT_TS_MILLIS_SCHEMA_VERSION {
    ts.saturating_mul(1_000)
  } else {
    ts
  }
}

fn normalize_timestamp(request: &mut Map<String, Value>, schema_version: u32) {
  let Some(ts) = request.get("ts").and_then(Value::as_i64) else {
    return;
  };
  request.insert("ts".to_string(), Value::from(normalized_timestamp(ts, schema_version)));
}

fn sqlite_status(value: Option<i64>) -> Option<u16> {
  value.and_then(|value| u16::try_from(value).ok())
}

fn sqlite_value_to_json(value: ValueRef<'_>, name: &str) -> Value {
  match value {
    ValueRef::Null => Value::Null,
    ValueRef::Integer(value) => Value::from(value),
    ValueRef::Real(value) => serde_json::Number::from_f64(value)
      .map(Value::Number)
      .unwrap_or(Value::Null),
    ValueRef::Text(value) => match std::str::from_utf8(value) {
      Ok(value) if JSON_COLUMNS.contains(&name) => {
        serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
      }
      Ok(value) => Value::String(value.to_string()),
      Err(_) => base64_json(value),
    },
    ValueRef::Blob(value) => match std::str::from_utf8(value) {
      Ok(value) if JSON_COLUMNS.contains(&name) => {
        serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string()))
      }
      Ok(value) => Value::String(value.to_string()),
      Err(_) => base64_json(value),
    },
  }
}

fn base64_json(value: &[u8]) -> Value {
  serde_json::json!({
    "encoding": "base64",
    "data": base64::engine::general_purpose::STANDARD.encode(value),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::requests::open_day_db;
  use crate::sessions::{SessionsDb, TreeRequestRecord};
  use crate::{MessageRecord, PartRecord};
  use bytes::Bytes;

  fn tempdir() -> PathBuf {
    let path = std::env::temp_dir().join(format!("tokn-router-inspect-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&path).unwrap();
    path
  }

  fn write_request(
    dir: &Path,
    day: &str,
    request_id: &str,
    ts: i64,
    session_id: Option<&str>,
    provider_id: Option<&str>,
  ) {
    let conn = open_day_db(&dir.join(format!("{day}.db"))).unwrap();
    conn
      .execute(
        "INSERT INTO request_connection (request_id, ts, endpoint, status, request_error, ctx_json)
         VALUES (?1, ?2, 'responses', 200, NULL, '{\"route\":\"default\"}')",
        params![request_id, ts],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_metadata (request_id, session_id, account_id, provider_id, model, params_json, usage_json)
         VALUES (?1, ?2, 'account-1', ?3, 'gpt-test', '{\"stream\":false}', '{\"input\":1}')",
        params![request_id, session_id, provider_id],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_downstream (request_id, inbound_req_method, inbound_req_url, inbound_req_body)
         VALUES (?1, 'POST', '/v1/responses', '{\"input\":\"hello\"}')",
        params![request_id],
      )
      .unwrap();
  }

  fn request_ids(requests: &[RequestSummary]) -> Vec<&str> {
    requests.iter().map(|request| request.request_id.as_str()).collect()
  }

  fn write_session(sessions_db: &Path, session_id: &str, request_id: &str, ts: i64, provider_id: &str, model: &str) {
    let mut sessions = SessionsDb::open(sessions_db).unwrap();
    sessions
      .record_tree(&TreeRequestRecord {
        ts,
        session_id: session_id.to_string(),
        parent_session_id: None,
        request_id: request_id.to_string(),
        endpoint: "responses".to_string(),
        status: Some(200),
        account_id: Some("account-1".to_string()),
        provider_id: Some(provider_id.to_string()),
        model: Some(model.to_string()),
        request_messages: vec![MessageRecord {
          role: "user".to_string(),
          status: None,
          parts: vec![PartRecord {
            part_type: "text".to_string(),
            content: Bytes::from_static(b"hello"),
          }],
        }],
        response_messages: Vec::new(),
      })
      .unwrap();
  }

  #[test]
  fn lists_sessions_from_the_sessions_database() {
    let dir = tempdir();
    let sessions_db = dir.join("sessions.db");
    write_session(
      &sessions_db,
      "session-1",
      "request-new",
      1_783_987_200_000,
      "openai",
      "gpt-test",
    );
    write_session(
      &sessions_db,
      "session-1",
      "request-old",
      1_783_900_800_000,
      "openai",
      "gpt-test",
    );
    write_session(
      &sessions_db,
      "session-2",
      "request-latest",
      1_784_073_600_000,
      "zai",
      "glm-test",
    );

    let sessions = list_sessions_from_db(&sessions_db, None).unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].session_id, "session-2");
    let session = sessions
      .iter()
      .find(|session| session.session_id == "session-1")
      .unwrap();
    assert_eq!(session.first_ts, 1_783_987_200_000);
    assert_eq!(session.last_ts, 1_783_987_200_000);
    assert_eq!(session.request_count, 2);
    assert_eq!(session.last_request_day, "2026-07-14");
    assert_eq!(session.last_request_id, "request-new");
    assert_eq!(session.endpoint.as_deref(), Some("responses"));
    assert_eq!(session.status, Some(200));
    assert_eq!(session.provider_id.as_deref(), Some("openai"));

    let limited = list_sessions_from_db(&sessions_db, Some(1)).unwrap();
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].session_id, "session-2");
  }

  #[test]
  fn missing_sessions_database_is_empty_without_creating_a_file() {
    let dir = tempdir();
    let sessions_db = dir.join("sessions.db");

    assert!(list_sessions_from_db(&sessions_db, None).unwrap().is_empty());
    assert!(!sessions_db.exists());
  }

  #[test]
  fn legacy_sessions_database_is_not_migrated_for_inspection() {
    let dir = tempdir();
    let sessions_db = dir.join("sessions.db");
    let conn = Connection::open(&sessions_db).unwrap();
    conn
      .execute_batch(include_str!("../schemas/snapshot/sessions/v0.0.0.sql"))
      .unwrap();
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

    assert!(matches!(
      list_sessions_from_db(&sessions_db, None),
      Err(crate::Error::UnsupportedSessionSchema { version: 1 })
    ));

    let conn = Connection::open(sessions_db).unwrap();
    let version: i64 = conn
      .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))
      .unwrap();
    assert_eq!(version, 1);
    let tree_table_count: i64 = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'session_nodes'",
        [],
        |row| row.get(0),
      )
      .unwrap();
    assert_eq!(tree_table_count, 0);
  }

  #[test]
  fn reads_tree_sessions_without_requiring_session_views() {
    let dir = tempdir();
    let sessions_db = dir.join("sessions.db");
    let conn = Connection::open(&sessions_db).unwrap();
    conn
      .execute_batch(include_str!("../schemas/snapshot/sessions/v0.0.0.sql"))
      .unwrap();
    conn
      .execute_batch(include_str!("../schemas/migrations/sessions/0002_tree_nodes.sql"))
      .unwrap();
    conn
      .execute_batch(
        "CREATE TABLE schema_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_ts INTEGER NOT NULL
         );
         INSERT INTO schema_migrations (version, name, applied_ts) VALUES
           (1, 'initial', 0),
           (2, 'tree_nodes', 0);",
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO sessions (id, first_seen_ts, last_seen_ts, source, account_id, provider_id, model)
         VALUES ('session-v2', 1783987200000, 1784073600000, 'header', 'account-v2', 'openai', 'gpt-v2')",
        [],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO session_nodes (
           id, session_id, parent_id, request_id, ts, endpoint, status, account_id, provider_id, model,
           reduction_kind, parent_source, common_prefix_messages, request_message_count, response_message_count
         ) VALUES (
           'node-v2', 'session-v2', NULL, 'request-v2', 1784073600000, 'responses', 201,
           'account-v2', 'openai', 'gpt-v2', 'root_snapshot', 'none', 0, 1, 0
         )",
        [],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO session_heads (session_id, node_id, updated_ts)
         VALUES ('session-v2', 'node-v2', 1784073600000)",
        [],
      )
      .unwrap();
    drop(conn);

    let sessions = list_sessions_from_db(&sessions_db, None).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "session-v2");
    assert_eq!(sessions[0].request_count, 1);
    assert_eq!(sessions[0].last_request_id, "request-v2");
    assert_eq!(sessions[0].status, Some(201));
  }

  #[test]
  fn reads_requests_and_infers_sessions_across_days() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-13",
      "request-old",
      1_784_358_400_000,
      Some("session-1"),
      Some("openai"),
    );
    write_request(
      &dir,
      "2026-07-14",
      "request-new",
      1_784_444_800_000,
      Some("session-1"),
      Some("openai"),
    );
    write_request(
      &dir,
      "2026-07-14",
      "request-other",
      1_784_444_801_000,
      Some("session-2"),
      Some("zai"),
    );

    let page = list_requests(&dir, &RequestListOptions::default()).unwrap();
    assert_eq!(
      page
        .requests
        .iter()
        .map(|request| request.request_id.as_str())
        .collect::<Vec<_>>(),
      ["request-other", "request-new", "request-old"]
    );

    let sessions = list_sessions(&dir, None).unwrap();
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].session_id, "session-2");
    let session = sessions
      .iter()
      .find(|session| session.session_id == "session-1")
      .unwrap();
    assert_eq!(session.request_count, 2);
    assert_eq!(session.last_request_id, "request-new");

    let detail = get_session(&dir, "session-1", None).unwrap().unwrap();
    assert_eq!(detail.requests.len(), 2);
    assert_eq!(detail.requests[0].request_id, "request-old");
    assert_eq!(detail.requests[1].request_id, "request-new");

    let limited_detail = get_session(&dir, "session-1", Some(1)).unwrap().unwrap();
    assert_eq!(limited_detail.session.request_count, 2);
    assert_eq!(limited_detail.requests.len(), 1);
    assert_eq!(limited_detail.requests[0].request_id, "request-new");
  }

  #[test]
  fn paginates_a_request_day_without_duplicates_at_equal_timestamps() {
    let dir = tempdir();
    for request_id in ["request-a", "request-b", "request-c", "request-d", "request-e"] {
      write_request(
        &dir,
        "2026-07-14",
        request_id,
        1_784_444_800_000,
        Some("session-1"),
        Some("openai"),
      );
    }

    let mut options = RequestListOptions {
      day: Some("2026-07-14".to_string()),
      limit: Some(2),
      ..RequestListOptions::default()
    };
    let first = list_requests(&dir, &options).unwrap();
    assert_eq!(request_ids(&first.requests), ["request-e", "request-d"]);
    options.cursor = Some(RequestCursor::decode(first.next_cursor.as_deref().unwrap()).unwrap());

    let second = list_requests(&dir, &options).unwrap();
    assert_eq!(request_ids(&second.requests), ["request-c", "request-b"]);
    options.cursor = Some(RequestCursor::decode(second.next_cursor.as_deref().unwrap()).unwrap());

    let third = list_requests(&dir, &options).unwrap();
    assert_eq!(request_ids(&third.requests), ["request-a"]);
    assert!(third.next_cursor.is_none());

    let all_ids = first
      .requests
      .iter()
      .chain(&second.requests)
      .chain(&third.requests)
      .map(|request| request.request_id.as_str())
      .collect::<std::collections::HashSet<_>>();
    assert_eq!(all_ids.len(), 5);
  }

  #[test]
  fn rejects_malformed_request_cursors() {
    for cursor in [
      "",
      "v1.2026-07-14.1784444800000.1",
      "v2.not-a-day.1784444800000.1",
      "v2.2026-07-14.not-a-timestamp.1",
      "v2.2026-07-14.1784444800000.not-a-rowid",
      "v2.2026-07-14.1784444800000",
      "v2.2026-07-14.1784444800000.1.extra",
    ] {
      assert_eq!(RequestCursor::decode(cursor), Err(InvalidRequestCursor));
    }
  }

  #[test]
  fn request_overview_omits_payloads_and_payloads_are_loaded_separately() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-detail",
      1_784_444_800_000,
      Some("session-1"),
      Some("openai"),
    );
    let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "INSERT INTO request_upstream (request_id, outbound_resp_body) VALUES (?1, ?2)",
        params!["request-detail", &[0xff_u8, 0x00]],
      )
      .unwrap();

    let detail = get_request(&dir, "2026-07-14", "request-detail", None)
      .unwrap()
      .unwrap();
    assert_eq!(detail.request["ctx_json"], serde_json::json!({"route": "default"}));
    assert_eq!(detail.request["params_json"], serde_json::json!({"stream": false}));
    for field in REQUEST_PAYLOAD_FIELDS {
      assert!(!detail.request.contains_key(*field));
    }

    let inbound_body = get_request_payload(
      &dir,
      "2026-07-14",
      "request-detail",
      None,
      RequestPayloadField::InboundReqBody,
    )
    .unwrap()
    .unwrap();
    assert_eq!(inbound_body.field, "inbound_req_body");
    assert_eq!(inbound_body.value, serde_json::json!({"input": "hello"}));

    let binary_body = get_request_payload(
      &dir,
      "2026-07-14",
      "request-detail",
      None,
      RequestPayloadField::OutboundRespBody,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
      binary_body.value,
      serde_json::json!({"encoding": "base64", "data": "/wA="})
    );
    assert!("endpoint".parse::<RequestPayloadField>().is_err());
    assert!(get_request(&dir, "2026-07-14", "missing", None).unwrap().is_none());
    assert!(get_request(&dir, "../../outside", "request-detail", None)
      .unwrap()
      .is_none());
  }

  #[test]
  fn errors_only_includes_all_split_request_failure_signals() {
    let dir = tempdir();
    for request_id in [
      "healthy",
      "request-error",
      "lifecycle-error",
      "upstream-error",
      "downstream-error",
    ] {
      write_request(
        &dir,
        "2026-07-14",
        request_id,
        1_784_444_800_000,
        Some("session-1"),
        Some("openai"),
      );
    }
    let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "UPDATE request_connection SET request_error = 'failed' WHERE request_id = 'request-error'",
        [],
      )
      .unwrap();
    conn
      .execute(
        "UPDATE request_connection SET status = 500 WHERE request_id = 'lifecycle-error'",
        [],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_upstream (request_id, outbound_resp_status) VALUES ('upstream-error', 502)",
        [],
      )
      .unwrap();
    conn
      .execute(
        "UPDATE request_downstream SET inbound_resp_status = 404 WHERE request_id = 'downstream-error'",
        [],
      )
      .unwrap();

    let page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        errors_only: true,
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    let ids = request_ids(&page.requests)
      .into_iter()
      .collect::<std::collections::HashSet<_>>();
    assert_eq!(ids.len(), 4);
    assert!(ids.contains("request-error"));
    assert!(ids.contains("lifecycle-error"));
    assert!(ids.contains("upstream-error"));
    assert!(ids.contains("downstream-error"));
    assert!(!ids.contains("healthy"));
  }

  #[test]
  fn exact_status_filter_uses_downstream_then_upstream_then_lifecycle_precedence() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-status",
      1_784_444_800_000,
      Some("session-1"),
      Some("openai"),
    );
    let conn = open_day_db(&dir.join("2026-07-14.db")).unwrap();
    conn
      .execute(
        "UPDATE request_connection SET status = 500 WHERE request_id = 'request-status'",
        [],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_upstream (request_id, outbound_resp_status) VALUES ('request-status', 502)",
        [],
      )
      .unwrap();
    conn
      .execute(
        "UPDATE request_downstream SET inbound_resp_status = 201 WHERE request_id = 'request-status'",
        [],
      )
      .unwrap();

    let matching = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        status: Some(201),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert_eq!(request_ids(&matching.requests), ["request-status"]);
    for shadowed_status in [500, 502] {
      let page = list_requests(
        &dir,
        &RequestListOptions {
          day: Some("2026-07-14".to_string()),
          status: Some(shadowed_status),
          ..RequestListOptions::default()
        },
      )
      .unwrap();
      assert!(page.requests.is_empty());
    }
  }

  #[test]
  fn aggregate_queries_skip_corrupt_request_day_databases() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-valid",
      1_784_444_800_000,
      Some("session-valid"),
      Some("openai"),
    );
    std::fs::write(dir.join("2026-07-15.db"), b"not a sqlite database").unwrap();

    let page = list_requests(&dir, &RequestListOptions::default()).unwrap();
    assert_eq!(page.requests.len(), 1);
    assert_eq!(page.requests[0].request_id, "request-valid");

    let sessions = list_sessions(&dir, None).unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "session-valid");

    assert!(list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-15".to_string()),
        ..RequestListOptions::default()
      }
    )
    .is_err());
    assert!(get_request(&dir, "2026-07-15", "missing", None).is_err());
  }

  #[test]
  fn lists_request_day_states_newest_first() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-available",
      1_784_444_800_000,
      Some("session-available"),
      Some("openai"),
    );
    drop(open_day_db(&dir.join("2026-07-15.db")).unwrap());
    std::fs::write(dir.join("2026-07-16.db"), b"not a sqlite database").unwrap();
    drop(open_day_db(&dir.join("not-a-day.db")).unwrap());
    drop(open_day_db(&dir.join("2026-02-30.db")).unwrap());

    let days = list_request_days(&dir).unwrap();
    assert_eq!(
      days,
      vec![
        RequestDay {
          day: "2026-07-16".to_string(),
          state: RequestDayState::Unavailable,
        },
        RequestDay {
          day: "2026-07-15".to_string(),
          state: RequestDayState::Empty,
        },
        RequestDay {
          day: "2026-07-14".to_string(),
          state: RequestDayState::Available,
        },
      ]
    );
    assert_eq!(
      serde_json::to_value(RequestDayState::Unavailable).unwrap(),
      serde_json::json!("unavailable")
    );
    assert!(is_valid_request_day("2026-07-14"));
    assert!(!is_valid_request_day("2026-7-14"));
    assert!(!is_valid_request_day("2026-02-30"));
  }

  #[test]
  fn latest_requests_skip_empty_and_unavailable_days() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-old",
      1_784_444_800_000,
      Some("session-old"),
      Some("openai"),
    );
    write_request(
      &dir,
      "2026-07-14",
      "request-latest",
      1_784_444_801_000,
      Some("session-old"),
      Some("openai"),
    );
    drop(open_day_db(&dir.join("2026-07-15.db")).unwrap());
    std::fs::write(dir.join("2026-07-16.db"), b"not a sqlite database").unwrap();

    let latest = list_latest_requests(&dir, Some(1), None).unwrap();
    assert_eq!(latest.day.as_deref(), Some("2026-07-14"));
    assert_eq!(latest.requests.len(), 1);
    assert_eq!(latest.requests[0].request_id, "request-latest");
    let cursor = RequestCursor::decode(latest.next_cursor.as_deref().unwrap()).unwrap();
    let next = list_latest_requests(&dir, Some(1), Some(cursor)).unwrap();
    assert_eq!(next.day.as_deref(), Some("2026-07-14"));
    assert_eq!(request_ids(&next.requests), ["request-old"]);
    assert!(next.next_cursor.is_none());
  }

  #[test]
  fn lists_requests_from_only_the_selected_day() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-old",
      1_784_444_800_000,
      Some("session-old"),
      Some("openai"),
    );
    write_request(
      &dir,
      "2026-07-15",
      "request-new",
      1_784_531_200_000,
      Some("session-new"),
      Some("zai"),
    );

    let page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert_eq!(page.requests.len(), 1);
    assert_eq!(page.requests[0].request_id, "request-old");

    let missing_day_page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-13".to_string()),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert!(missing_day_page.requests.is_empty());
  }

  #[test]
  fn decodes_json_blobs_only_for_json_columns() {
    let json = br#"{"route":"default"}"#;

    assert_eq!(
      sqlite_value_to_json(ValueRef::Blob(json), "ctx_json"),
      serde_json::json!({"route": "default"})
    );
    assert_eq!(
      sqlite_value_to_json(ValueRef::Blob(json), "non_json_column"),
      Value::String("{\"route\":\"default\"}".to_string())
    );
    assert_eq!(
      sqlite_value_to_json(ValueRef::Blob(b"plain value"), "ctx_json"),
      Value::String("plain value".to_string())
    );
  }

  #[test]
  fn legacy_pagination_uses_numeric_row_id_with_duplicate_and_null_request_ids() {
    let dir = tempdir();
    let path = dir.join("2026-07-14.db");
    let conn = Connection::open(&path).unwrap();
    conn
      .execute_batch(include_str!("../schemas/snapshot/requests/v0.0.0.sql"))
      .unwrap();
    conn
      .execute_batch(include_str!(
        "../schemas/migrations/requests/0002_add_correlation_and_error.sql"
      ))
      .unwrap();
    for (id, request_id, model) in [
      (9_i64, None, "model-9"),
      (10_i64, None, "model-10"),
      (11_i64, Some("duplicate"), "model-11"),
      (12_i64, Some("duplicate"), "model-12"),
    ] {
      let body = format!("{{\"row_id\":{id}}}");
      conn
        .execute(
          "INSERT INTO requests (
             id, ts, session_id, endpoint, account_id, provider_id, model, initiator, status, stream,
             latency_ms, inbound_req_headers, inbound_req_body, request_id
           ) VALUES (?1, 1784444800, 'session', 'responses', 'account', 'openai', ?2, 'test', 200, 0,
                     1, '{}', ?3, ?4)",
          params![id, model, body, request_id],
        )
        .unwrap();
    }
    conn
      .execute_batch(
        "CREATE TABLE schema_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_ts INTEGER NOT NULL
         );
         INSERT INTO schema_migrations (version, name, applied_ts) VALUES
           (1, 'initial', 0),
           (2, 'correlation_and_error', 0);",
      )
      .unwrap();
    drop(conn);

    let mut options = RequestListOptions {
      day: Some("2026-07-14".to_string()),
      limit: Some(1),
      ..RequestListOptions::default()
    };
    let mut request_ids = Vec::new();
    let mut models = Vec::new();
    loop {
      let page = list_requests(&dir, &options).unwrap();
      request_ids.extend(page.requests.iter().map(|request| request.request_id.clone()));
      models.extend(page.requests.iter().map(|request| request.model.clone().unwrap()));
      let Some(cursor) = page.next_cursor else {
        break;
      };
      options.cursor = Some(RequestCursor::decode(&cursor).unwrap());
    }

    assert_eq!(models, ["model-12", "model-11", "model-10", "model-9"]);
    assert_eq!(request_ids, ["duplicate", "duplicate", "legacy:10", "legacy:9"]);

    let detail = get_request(&dir, "2026-07-14", "duplicate", Some(11)).unwrap().unwrap();
    assert_eq!(detail.row_id, 11);
    assert_eq!(detail.request["model"], "model-11");
    assert_eq!(serde_json::to_value(&detail).unwrap()["row_id"], "11");

    let payload = get_request_payload(
      &dir,
      "2026-07-14",
      "duplicate",
      Some(12),
      RequestPayloadField::InboundReqBody,
    )
    .unwrap()
    .unwrap();
    assert_eq!(payload.value, serde_json::json!({"row_id": 12}));
    assert!(get_request(&dir, "2026-07-14", "duplicate", Some(10))
      .unwrap()
      .is_none());

    let page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        limit: Some(1),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert_eq!(serde_json::to_value(&page.requests[0]).unwrap()["row_id"], "12");
  }

  #[test]
  fn reads_legacy_request_days_without_migrating_them() {
    let dir = tempdir();
    let path = dir.join("2026-07-14.db");
    let conn = Connection::open(&path).unwrap();
    conn
      .execute_batch(include_str!("../schemas/snapshot/requests/v0.0.0.sql"))
      .unwrap();
    conn
      .execute(
        "INSERT INTO requests (
           ts, session_id, endpoint, account_id, provider_id, model, initiator,
           status, stream, latency_ms, inbound_req_headers, inbound_req_body
         ) VALUES (
           ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
         )",
        params![
          1_784_444_800_i64,
          "legacy-session",
          "chat.completions",
          "account-1",
          "openai",
          "gpt-legacy",
          "test",
          200_i64,
          0_i64,
          12_i64,
          br#"{"content-type":"application/json"}"#,
          br#"{"messages":["hello"]}"#,
        ],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO requests (
           ts, session_id, endpoint, account_id, provider_id, model, initiator,
           status, stream, latency_ms, inbound_req_headers, inbound_req_body
         ) VALUES (
           ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
         )",
        params![
          1_784_444_800_i64,
          "legacy-session",
          "chat.completions",
          "account-1",
          "openai",
          "gpt-legacy",
          "test",
          200_i64,
          0_i64,
          12_i64,
          br#"{"content-type":"application/json"}"#,
          br#"{"messages":["second"]}"#,
        ],
      )
      .unwrap();
    conn
      .execute("UPDATE requests SET outbound_resp_status = 502 WHERE id = 2", [])
      .unwrap();
    conn
      .execute(
        "UPDATE requests SET outbound_resp_status = 202, inbound_resp_status = 201 WHERE id = 1",
        [],
      )
      .unwrap();
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

    let mut options = RequestListOptions {
      day: Some("2026-07-14".to_string()),
      limit: Some(1),
      ..RequestListOptions::default()
    };
    let page = list_requests(&dir, &options).unwrap();
    assert_eq!(request_ids(&page.requests), ["legacy:2"]);
    assert_eq!(page.requests[0].ts, 1_784_444_800_000);
    options.cursor = Some(RequestCursor::decode(page.next_cursor.as_deref().unwrap()).unwrap());
    let next_page = list_requests(&dir, &options).unwrap();
    assert_eq!(request_ids(&next_page.requests), ["legacy:1"]);
    assert!(next_page.next_cursor.is_none());
    let error_page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        errors_only: true,
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert_eq!(request_ids(&error_page.requests), ["legacy:2"]);
    let downstream_status_page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        status: Some(201),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert_eq!(request_ids(&downstream_status_page.requests), ["legacy:1"]);
    let shadowed_upstream_status_page = list_requests(
      &dir,
      &RequestListOptions {
        day: Some("2026-07-14".to_string()),
        status: Some(202),
        ..RequestListOptions::default()
      },
    )
    .unwrap();
    assert!(shadowed_upstream_status_page.requests.is_empty());
    assert_eq!(
      list_request_days(&dir).unwrap(),
      vec![RequestDay {
        day: "2026-07-14".to_string(),
        state: RequestDayState::Available,
      }]
    );

    let detail = get_request(&dir, "2026-07-14", "legacy:1", None).unwrap().unwrap();
    assert_eq!(detail.request["request_id"], "legacy:1");
    for field in REQUEST_PAYLOAD_FIELDS {
      assert!(!detail.request.contains_key(*field));
    }
    let payload = get_request_payload(
      &dir,
      "2026-07-14",
      "legacy:1",
      None,
      RequestPayloadField::InboundReqBody,
    )
    .unwrap()
    .unwrap();
    assert_eq!(payload.value, serde_json::json!({"messages": ["hello"]}));

    let sessions = list_sessions(&dir, None).unwrap();
    assert_eq!(sessions[0].session_id, "legacy-session");

    let conn = Connection::open(path).unwrap();
    let version: i64 = conn
      .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))
      .unwrap();
    assert_eq!(version, 1);
    let split_table_count: i64 = conn
      .query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'request_connection'",
        [],
        |row| row.get(0),
      )
      .unwrap();
    assert_eq!(split_table_count, 0);
  }
}
