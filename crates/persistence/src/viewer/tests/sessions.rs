use rusqlite::Connection;

use super::super::{get_session, list_requests, list_sessions, list_sessions_from_db, RequestListOptions};
use super::support::{tempdir, write_request, write_session};

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
fn legacy_sessions_database_is_not_migrated_for_viewing() {
  let dir = tempdir();
  let sessions_db = dir.join("sessions.db");
  let conn = Connection::open(&sessions_db).unwrap();
  conn
    .execute_batch(include_str!("../../../schemas/snapshot/sessions/v0.0.0.sql"))
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
    .execute_batch(include_str!("../../../schemas/snapshot/sessions/v0.0.0.sql"))
    .unwrap();
  conn
    .execute_batch(include_str!("../../../schemas/migrations/sessions/0002_tree_nodes.sql"))
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
