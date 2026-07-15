use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use super::database::open_readonly;
use super::days::request_day_files;
use super::effective_limit;
use super::requests::{list_day_requests_best_effort, RequestListOptions, RequestSummary};
use super::schema::{read_schema_version, SESSION_TREE_SCHEMA_VERSION};
use super::value::sqlite_status;
use crate::Result;

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

/// Return the most recently active inferred sessions.
pub fn list_sessions(requests_dir: &Path, limit: Option<usize>) -> Result<Vec<SessionSummary>> {
  let mut sessions = collect_sessions(requests_dir)?;
  sessions.sort_by(|left, right| {
    right
      .last_ts
      .cmp(&left.last_ts)
      .then_with(|| left.session_id.cmp(&right.session_id))
  });
  sessions.truncate(effective_limit(limit));
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

  let version = read_schema_version(&conn)?;
  if version < SESSION_TREE_SCHEMA_VERSION {
    return Err(crate::Error::UnsupportedSessionSchema { version });
  }

  let limit = effective_limit(limit);
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
  let limit = effective_limit(limit);
  if requests.len() > limit {
    requests.drain(..requests.len() - limit);
  }

  Ok(Some(SessionDetail { session, requests }))
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
