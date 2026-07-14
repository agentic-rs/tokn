//! Read-only queries for the local request history.
//!
//! The gateway writes request history into one SQLite database per UTC day.
//! This module deliberately opens those files in read-only mode: an inspector
//! must never create a database, apply a migration, or take ownership of a
//! writer connection just to display history.

use crate::migrate;
use rusqlite::types::{Value as SqlValue, ValueRef};
use rusqlite::{params, params_from_iter, Connection, OpenFlags};
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::Result;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 500;
const CURRENT_TS_MILLIS_SCHEMA_VERSION: u32 = 8;
const SPLIT_REQUESTS_SCHEMA_VERSION: u32 = 7;
const REQUEST_ID_SCHEMA_VERSION: u32 = 2;
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

/// Query options accepted by the request list and session timeline.
#[derive(Debug, Clone, Default)]
pub struct RequestListOptions {
  pub limit: Option<usize>,
  pub session_id: Option<String>,
  pub provider_id: Option<String>,
  pub status: Option<u16>,
  pub query: Option<String>,
}

impl RequestListOptions {
  fn effective_limit(&self) -> usize {
    self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
  }
}

/// A compact request row suitable for an inspector list or session timeline.
#[derive(Debug, Clone, Serialize)]
pub struct RequestSummary {
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

/// The complete, decoded row for one request identity (`day`, `request_id`).
#[derive(Debug, Clone, Serialize)]
pub struct RequestDetail {
  pub day: String,
  pub request: Map<String, Value>,
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

/// Return the most recent request rows across every existing request-day DB.
pub fn list_requests(requests_dir: &Path, options: &RequestListOptions) -> Result<Vec<RequestSummary>> {
  let limit = options.effective_limit();
  let mut requests = Vec::new();

  for day_file in request_day_files(requests_dir)? {
    let Some(conn) = open_readonly(&day_file.path)? else {
      continue;
    };
    requests.extend(list_day_requests(&conn, &day_file.day, options, Some(limit))?);
  }

  requests.sort_by(|left, right| {
    right
      .ts
      .cmp(&left.ts)
      .then_with(|| right.day.cmp(&left.day))
      .then_with(|| right.request_id.cmp(&left.request_id))
  });
  requests.truncate(limit);
  Ok(requests)
}

/// Return a complete request row without mutating its source database.
pub fn get_request(requests_dir: &Path, day: &str, request_id: &str) -> Result<Option<RequestDetail>> {
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
  let request_id_condition = if version >= SPLIT_REQUESTS_SCHEMA_VERSION {
    "request_id = ?1".to_string()
  } else {
    format!("{} = ?1", legacy_request_id_sql(version))
  };
  let mut stmt = conn.prepare(&format!("SELECT * FROM requests WHERE {request_id_condition} LIMIT 1"))?;
  let column_count = stmt.column_count();
  let column_names = (0..column_count)
    .map(|index| stmt.column_name(index).unwrap_or_default().to_string())
    .collect::<Vec<_>>();
  let mut rows = stmt.query(params![request_id])?;
  let Some(row) = rows.next()? else {
    return Ok(None);
  };

  let mut request = Map::with_capacity(column_count);
  for (index, name) in column_names.iter().enumerate() {
    request.insert(name.clone(), sqlite_value_to_json(row.get_ref(index)?, name));
  }
  if version < SPLIT_REQUESTS_SCHEMA_VERSION
    && !matches!(request.get("request_id"), Some(Value::String(value)) if !value.is_empty())
  {
    request.insert("request_id".to_string(), Value::String(request_id.to_string()));
  }
  normalize_timestamp(&mut request, version);

  Ok(Some(RequestDetail {
    day: day_file.day,
    request,
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

/// Return a chronological, bounded timeline for one inferred session.
pub fn get_session(requests_dir: &Path, session_id: &str, limit: Option<usize>) -> Result<Option<SessionDetail>> {
  let Some(session) = collect_sessions(requests_dir)?
    .into_iter()
    .find(|session| session.session_id == session_id)
  else {
    return Ok(None);
  };

  let requests = list_requests(
    requests_dir,
    &RequestListOptions {
      limit,
      session_id: Some(session_id.to_string()),
      ..RequestListOptions::default()
    },
  )?;
  let mut requests = requests;
  requests.sort_by(|left, right| {
    left
      .ts
      .cmp(&right.ts)
      .then_with(|| left.day.cmp(&right.day))
      .then_with(|| left.request_id.cmp(&right.request_id))
  });

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
    files.push(DayFile {
      day: day.to_string(),
      path,
    });
  }
  files.sort_by(|left, right| right.day.cmp(&left.day));
  Ok(files)
}

fn open_readonly(path: &Path) -> Result<Option<Connection>> {
  if !path.exists() {
    return Ok(None);
  }
  let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
  conn.busy_timeout(Duration::from_millis(2_500))?;
  conn.execute_batch("PRAGMA query_only = ON;")?;
  Ok(Some(conn))
}

fn schema_version(conn: &Connection) -> Result<u32> {
  migrate::read_current_version(conn)
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
            d.inbound_resp_status
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
    sql.push_str(" AND c.status = ?");
    values.push(SqlValue::Integer(i64::from(status)));
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
  sql.push_str(" ORDER BY c.ts DESC, c.rowid DESC");
  if let Some(limit) = limit {
    sql.push_str(" LIMIT ?");
    values.push(SqlValue::Integer(limit as i64));
  }

  let mut stmt = conn.prepare(&sql)?;
  let rows = stmt
    .query_map(params_from_iter(values.iter()), |row| {
      Ok(RequestSummary {
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
            model, inbound_req_method, inbound_req_url, outbound_resp_status, inbound_resp_status
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
    sql.push_str(" AND status = ?");
    values.push(SqlValue::Integer(i64::from(status)));
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
  sql.push_str(" ORDER BY ts DESC, id DESC");
  if let Some(limit) = limit {
    sql.push_str(" LIMIT ?");
    values.push(SqlValue::Integer(limit as i64));
  }

  let mut stmt = conn.prepare(&sql)?;
  let rows = stmt
    .query_map(params_from_iter(values.iter()), |row| {
      Ok(RequestSummary {
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
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(rows)
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
    let Some(conn) = open_readonly(&day_file.path)? else {
      continue;
    };
    for request in list_day_requests(&conn, &day_file.day, &RequestListOptions::default(), None)? {
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
      Err(_) => Value::Array(value.iter().copied().map(Value::from).collect()),
    },
    ValueRef::Blob(value) => match std::str::from_utf8(value) {
      Ok(value) => serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_string())),
      Err(_) => Value::Array(value.iter().copied().map(Value::from).collect()),
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::requests::open_day_db;

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

    let requests = list_requests(&dir, &RequestListOptions::default()).unwrap();
    assert_eq!(
      requests
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
  }

  #[test]
  fn gets_a_request_by_day_and_decodes_json_fields() {
    let dir = tempdir();
    write_request(
      &dir,
      "2026-07-14",
      "request-detail",
      1_784_444_800_000,
      Some("session-1"),
      Some("openai"),
    );

    let detail = get_request(&dir, "2026-07-14", "request-detail").unwrap().unwrap();
    assert_eq!(detail.request["ctx_json"], serde_json::json!({"route": "default"}));
    assert_eq!(detail.request["params_json"], serde_json::json!({"stream": false}));
    assert_eq!(
      detail.request["inbound_req_body"],
      serde_json::json!({"input": "hello"})
    );
    assert!(get_request(&dir, "2026-07-14", "missing").unwrap().is_none());
    assert!(get_request(&dir, "../../outside", "request-detail").unwrap().is_none());
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

    let requests = list_requests(&dir, &RequestListOptions::default()).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].request_id, "legacy:1");
    assert_eq!(requests[0].ts, 1_784_444_800_000);

    let detail = get_request(&dir, "2026-07-14", "legacy:1").unwrap().unwrap();
    assert_eq!(detail.request["request_id"], "legacy:1");
    assert_eq!(
      detail.request["inbound_req_body"],
      serde_json::json!({"messages": ["hello"]})
    );

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
