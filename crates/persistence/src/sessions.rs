use super::{migrate, MessageRecord, PartRecord, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokn_core::db::SessionSource;
use tracing::debug;

const BOOTSTRAP: &str = include_str!("../schemas/snapshot/sessions/v0.2.0.sql");
const REQUESTS_TS_MILLIS_SCHEMA_VERSION: u32 = 8;
const MIGRATIONS: &[migrate::Migration] = &[
  migrate::Migration {
    version: 1,
    name: "initial",
    sql: include_str!("../schemas/snapshot/sessions/v0.0.0.sql"),
  },
  migrate::Migration {
    version: 2,
    name: "tree_nodes",
    sql: include_str!("../schemas/migrations/sessions/0002_tree_nodes.sql"),
  },
  migrate::Migration {
    version: 3,
    name: "session_views",
    sql: include_str!("../schemas/migrations/sessions/0003_session_views.sql"),
  },
];

pub fn latest_version() -> u32 {
  migrate::latest_version(MIGRATIONS)
}

pub struct SessionsDb {
  conn: Connection,
}

#[derive(Debug, Clone)]
pub struct TreeRequestRecord {
  pub ts: i64,
  pub session_id: String,
  pub parent_session_id: Option<String>,
  pub request_id: String,
  pub endpoint: String,
  pub status: Option<u16>,
  pub account_id: Option<String>,
  pub provider_id: Option<String>,
  pub model: Option<String>,
  pub request_messages: Vec<MessageRecord>,
  pub response_messages: Vec<MessageRecord>,
}

#[derive(Debug, Default)]
pub struct PlaybackReport {
  pub rows_seen: u64,
  pub rows_with_session: u64,
  pub rows_recorded: u64,
  pub rows_existing: u64,
  pub rows_skipped: u64,
  pub decode_errors: u64,
  pub reduction_mismatches: u64,
  pub latest_mismatches: Vec<LatestMismatch>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlaybackStats {
  pub rows_seen: u64,
  pub rows_with_session: u64,
  pub rows_recorded: u64,
  pub rows_existing: u64,
  pub rows_skipped: u64,
  pub decode_errors: u64,
  pub reduction_mismatches: u64,
}

impl PlaybackStats {
  fn from_report(report: &PlaybackReport) -> Self {
    Self {
      rows_seen: report.rows_seen,
      rows_with_session: report.rows_with_session,
      rows_recorded: report.rows_recorded,
      rows_existing: report.rows_existing,
      rows_skipped: report.rows_skipped,
      decode_errors: report.decode_errors,
      reduction_mismatches: report.reduction_mismatches,
    }
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlaybackOptions {
  pub force: bool,
}

#[derive(Debug, Clone)]
pub enum PlaybackProgressEvent {
  Started {
    files_total: usize,
    rows_total: u64,
  },
  FileStarted {
    path: PathBuf,
    file_index: usize,
    files_total: usize,
    rows_total: u64,
  },
  RowProcessed {
    path: PathBuf,
    file_index: usize,
    files_total: usize,
    rows_seen: u64,
    rows_total: u64,
    file_stats: PlaybackStats,
    global_stats: PlaybackStats,
  },
  FileFinished {
    path: PathBuf,
    file_index: usize,
    files_total: usize,
    file_stats: PlaybackStats,
    global_stats: PlaybackStats,
  },
  Finished {
    global_stats: PlaybackStats,
  },
}

#[derive(Debug, Clone)]
pub enum PlaybackSource {
  File(PathBuf),
  Dir(PathBuf),
}

#[derive(Debug)]
pub struct LatestMismatch {
  pub session_id: String,
  pub expected_request_id: String,
  pub actual_request_id: Option<String>,
}

impl SessionsDb {
  pub fn open(path: &Path) -> Result<Self> {
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let mut conn = Connection::open(path)?;
    migrate::apply(
      &mut conn,
      path,
      "sessions",
      migrate::Bootstrap { sql: BOOTSTRAP },
      MIGRATIONS,
    )?;
    Ok(Self { conn })
  }

  pub fn record_tree(&mut self, r: &TreeRequestRecord) -> Result<()> {
    if r.request_messages.is_empty() && r.response_messages.is_empty() {
      debug!(session_id = %r.session_id, request_id = %r.request_id, "sessions.record_tree: no messages, skipping");
      return Ok(());
    }

    let parent_id = self.head_for_session(&r.session_id)?;
    let parent_view = match parent_id.as_deref() {
      Some(parent_id) => self.materialize_request_messages(parent_id)?,
      None => Vec::new(),
    };
    let common_prefix = common_message_prefix(&parent_view, &r.request_messages);
    let parent_matches = common_prefix == parent_view.len();
    let request_delta = if parent_matches {
      r.request_messages[common_prefix..].to_vec()
    } else {
      r.request_messages.clone()
    };
    let reduction_kind = if parent_id.is_none() {
      "root_snapshot"
    } else if parent_matches {
      "suffix_append"
    } else {
      "conflict_snapshot"
    };
    let parent_source = if parent_id.is_some() { "inferred_head" } else { "none" };

    let tx = self.conn.transaction()?;
    tx.execute(
      "INSERT INTO sessions (id, first_seen_ts, last_seen_ts, source, account_id, provider_id, model)
       VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)
       ON CONFLICT(id) DO UPDATE SET
         last_seen_ts = excluded.last_seen_ts,
         account_id = COALESCE(excluded.account_id, sessions.account_id),
         provider_id = COALESCE(excluded.provider_id, sessions.provider_id),
         model = COALESCE(excluded.model, sessions.model)",
      params![
        r.session_id,
        r.ts,
        SessionSource::Header.as_str(),
        r.account_id.as_deref(),
        r.provider_id.as_deref(),
        r.model.as_deref(),
      ],
    )?;
    tx.execute(
      "INSERT INTO session_nodes
         (id, session_id, parent_id, request_id, ts, endpoint, status, account_id, provider_id, model,
          reduction_kind, parent_source, common_prefix_messages, request_message_count, response_message_count)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
       ON CONFLICT(session_id, request_id) DO UPDATE SET
         status = excluded.status,
         account_id = COALESCE(excluded.account_id, session_nodes.account_id),
         provider_id = COALESCE(excluded.provider_id, session_nodes.provider_id),
         model = COALESCE(excluded.model, session_nodes.model),
         response_message_count = excluded.response_message_count",
      params![
        r.request_id,
        r.session_id,
        parent_id.as_deref(),
        r.request_id,
        r.ts,
        r.endpoint,
        r.status.map(i64::from),
        r.account_id.as_deref(),
        r.provider_id.as_deref(),
        r.model.as_deref(),
        reduction_kind,
        parent_source,
        common_prefix as i64,
        request_delta.len() as i64,
        r.response_messages.len() as i64,
      ],
    )?;
    insert_node_messages(&tx, &r.request_id, "request", &request_delta)?;
    insert_node_messages(&tx, &r.request_id, "response", &r.response_messages)?;
    tx.execute(
      "INSERT INTO session_heads (session_id, node_id, updated_ts)
       VALUES (?1, ?2, ?3)
       ON CONFLICT(session_id) DO UPDATE SET
         node_id = excluded.node_id,
         updated_ts = excluded.updated_ts
       WHERE excluded.updated_ts >= session_heads.updated_ts",
      params![r.session_id, r.request_id, r.ts],
    )?;
    if let Some(parent_session_id) = r.parent_session_id.as_deref() {
      tx.execute(
        "INSERT INTO session_relations
           (parent_session_id, child_session_id, relation_kind, first_seen_ts, last_seen_ts, source)
         VALUES (?1, ?2, 'subagent', ?3, ?3, 'x-parent-session-id')
         ON CONFLICT(parent_session_id, child_session_id, relation_kind) DO UPDATE SET
           last_seen_ts = excluded.last_seen_ts",
        params![parent_session_id, r.session_id, r.ts],
      )?;
    }
    tx.commit()?;
    Ok(())
  }

  fn head_for_session(&self, session_id: &str) -> Result<Option<String>> {
    Ok(
      self
        .conn
        .query_row(
          "SELECT node_id FROM session_heads WHERE session_id = ?1",
          params![session_id],
          |r| r.get(0),
        )
        .optional()?,
    )
  }

  fn node_exists(&self, session_id: &str, request_id: &str) -> Result<bool> {
    let exists = self.conn.query_row(
      "SELECT EXISTS(SELECT 1 FROM session_nodes WHERE session_id = ?1 AND request_id = ?2)",
      params![session_id, request_id],
      |r| r.get::<_, bool>(0),
    )?;
    Ok(exists)
  }

  fn materialize_request_messages(&self, node_id: &str) -> Result<Vec<MessageRecord>> {
    let mut lineage = self.lineage(node_id)?;
    lineage.reverse();
    let mut out = Vec::new();
    for node in lineage {
      let (kind, messages) = self.node_request_messages(&node)?;
      if kind == "root_snapshot" || kind == "conflict_snapshot" {
        out = messages;
      } else {
        out.extend(messages);
      }
    }
    Ok(out)
  }

  fn lineage(&self, node_id: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut current = Some(node_id.to_string());
    while let Some(id) = current {
      current = self
        .conn
        .query_row("SELECT parent_id FROM session_nodes WHERE id = ?1", params![id], |r| {
          r.get::<_, Option<String>>(0)
        })
        .optional()?
        .flatten();
      out.push(id);
    }
    Ok(out)
  }

  fn node_request_messages(&self, node_id: &str) -> Result<(String, Vec<MessageRecord>)> {
    let kind: String = self.conn.query_row(
      "SELECT reduction_kind FROM session_nodes WHERE id = ?1",
      params![node_id],
      |r| r.get(0),
    )?;
    Ok((kind, select_node_messages(&self.conn, node_id, "request")?))
  }
}

fn upsert_part_blob(tx: &rusqlite::Transaction<'_>, part: &PartRecord) -> Result<()> {
  let hash = hash_part(&part.part_type, part.content.as_ref());
  tx.execute(
    "INSERT OR IGNORE INTO part_blobs (hash, part_type, content) VALUES (?1, ?2, ?3)",
    params![hash, part.part_type, part.content.as_ref()],
  )?;
  Ok(())
}

fn insert_node_messages(
  tx: &rusqlite::Transaction<'_>,
  node_id: &str,
  side: &str,
  messages: &[MessageRecord],
) -> Result<()> {
  tx.execute(
    "DELETE FROM node_parts
     WHERE message_id IN (SELECT id FROM node_messages WHERE node_id = ?1 AND side = ?2)",
    params![node_id, side],
  )?;
  tx.execute(
    "DELETE FROM node_messages WHERE node_id = ?1 AND side = ?2",
    params![node_id, side],
  )?;
  for (message_idx, message) in messages.iter().enumerate() {
    let message_id = format!("{node_id}:{side}:{message_idx}");
    tx.execute(
      "INSERT INTO node_messages (id, node_id, side, message_seq, role, status)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
      params![
        message_id,
        node_id,
        side,
        message_idx as i64,
        message.role,
        message.status.map(i64::from),
      ],
    )?;
    for (part_idx, part) in message.parts.iter().enumerate() {
      upsert_part_blob(tx, part)?;
      tx.execute(
        "INSERT INTO node_parts (message_id, part_index, part_hash)
         VALUES (?1, ?2, ?3)",
        params![
          message_id,
          part_idx as i64,
          hash_part(&part.part_type, part.content.as_ref())
        ],
      )?;
    }
  }
  Ok(())
}

fn select_node_messages(conn: &Connection, node_id: &str, side: &str) -> Result<Vec<MessageRecord>> {
  let mut stmt = conn.prepare(
    "SELECT id, role, status
     FROM node_messages
     WHERE node_id = ?1 AND side = ?2
     ORDER BY message_seq",
  )?;
  let rows = stmt
    .query_map(params![node_id, side], |r| {
      Ok((
        r.get::<_, String>(0)?,
        r.get::<_, String>(1)?,
        r.get::<_, Option<i64>>(2)?,
      ))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  rows
    .into_iter()
    .map(|(message_id, role, status)| {
      Ok(MessageRecord {
        role,
        status: status.map(|v| v as u16),
        parts: select_message_parts(conn, &message_id)?,
      })
    })
    .collect()
}

fn select_message_parts(conn: &Connection, message_id: &str) -> Result<Vec<PartRecord>> {
  let mut stmt = conn.prepare(
    "SELECT b.part_type, b.content
     FROM node_parts p
     JOIN part_blobs b ON b.hash = p.part_hash
     WHERE p.message_id = ?1
     ORDER BY p.part_index",
  )?;
  let parts = stmt
    .query_map(params![message_id], |r| {
      Ok(PartRecord {
        part_type: r.get(0)?,
        content: Bytes::from(r.get::<_, Vec<u8>>(1)?),
      })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(parts)
}

fn common_message_prefix(left: &[MessageRecord], right: &[MessageRecord]) -> usize {
  left.iter().zip(right).take_while(|(a, b)| messages_equal(a, b)).count()
}

fn messages_equal(a: &MessageRecord, b: &MessageRecord) -> bool {
  a.role == b.role
    && a.parts.len() == b.parts.len()
    && a
      .parts
      .iter()
      .zip(&b.parts)
      .all(|(a, b)| a.part_type == b.part_type && a.content == b.content)
}

pub fn playback_requests_into_sessions(requests_db: &Path, sessions_db: &Path) -> Result<PlaybackReport> {
  playback_requests_into_sessions_with_options(requests_db, sessions_db, PlaybackOptions::default())
}

pub fn playback_requests_into_sessions_with_options(
  requests_db: &Path,
  sessions_db: &Path,
  options: PlaybackOptions,
) -> Result<PlaybackReport> {
  playback_requests_source_into_sessions(PlaybackSource::File(requests_db.to_path_buf()), sessions_db, options)
}

pub fn playback_requests_source_into_sessions(
  source: PlaybackSource,
  sessions_db: &Path,
  options: PlaybackOptions,
) -> Result<PlaybackReport> {
  playback_requests_source_into_sessions_with_progress(source, sessions_db, options, |_| {})
}

pub fn playback_requests_source_into_sessions_with_progress(
  source: PlaybackSource,
  sessions_db: &Path,
  options: PlaybackOptions,
  mut progress: impl FnMut(PlaybackProgressEvent),
) -> Result<PlaybackReport> {
  let request_dbs = match source {
    PlaybackSource::File(path) => vec![path],
    PlaybackSource::Dir(dir) => request_db_files(&dir)?,
  };
  let files_total = request_dbs.len();
  let rows_total = count_request_rows(&request_dbs)?;
  let mut sessions = SessionsDb::open(sessions_db)?;
  let mut report = PlaybackReport::default();
  let mut expected_latest = HashMap::new();
  progress(PlaybackProgressEvent::Started {
    files_total,
    rows_total,
  });
  for (file_index, requests_db) in request_dbs.into_iter().enumerate() {
    let requests = Connection::open_with_flags(&requests_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let file_rows_total = count_request_rows_in_connection(&requests)?;
    progress(PlaybackProgressEvent::FileStarted {
      path: requests_db.clone(),
      file_index,
      files_total,
      rows_total: file_rows_total,
    });
    let file_start = PlaybackStats::from_report(&report);
    playback_request_connection(
      &requests,
      &mut sessions,
      options,
      &mut report,
      &mut expected_latest,
      PlaybackFileContext {
        requests_db: &requests_db,
        file_start,
        rows_total: file_rows_total,
        file_index,
        files_total,
      },
      &mut progress,
    )?;
    let file_stats = subtract_stats(PlaybackStats::from_report(&report), file_start);
    progress(PlaybackProgressEvent::FileFinished {
      path: requests_db,
      file_index,
      files_total,
      file_stats,
      global_stats: PlaybackStats::from_report(&report),
    });
  }
  report.latest_mismatches = check_latest_heads(&expected_latest, &sessions.conn)?;
  progress(PlaybackProgressEvent::Finished {
    global_stats: PlaybackStats::from_report(&report),
  });
  Ok(report)
}

#[derive(Debug, Clone, Copy)]
struct PlaybackFileContext<'a> {
  requests_db: &'a Path,
  file_start: PlaybackStats,
  rows_total: u64,
  file_index: usize,
  files_total: usize,
}

fn playback_request_connection(
  requests: &Connection,
  sessions: &mut SessionsDb,
  options: PlaybackOptions,
  report: &mut PlaybackReport,
  expected_latest: &mut HashMap<String, (i64, i64, String)>,
  ctx: PlaybackFileContext<'_>,
  progress: &mut impl FnMut(PlaybackProgressEvent),
) -> Result<()> {
  let order_index = request_order_index(requests)?;
  let playback_ts = if migrate::read_current_version(requests)? < REQUESTS_TS_MILLIS_SCHEMA_VERSION {
    "ts * 1000"
  } else {
    "ts"
  };
  let mut stmt = requests.prepare(&format!(
    "SELECT {playback_ts}, session_id, request_id, endpoint, account_id, provider_id, model, status,
            {order_index}
     FROM requests
     WHERE session_id IS NOT NULL
     ORDER BY {playback_ts}, {order_index}",
  ))?;
  let mut rows = stmt.query([])?;
  while let Some(row) = rows.next()? {
    report.rows_seen += 1;
    report.rows_with_session += 1;
    let ts: i64 = row.get(0)?;
    let session_id: String = row.get(1)?;
    let request_id: String = row.get(2)?;
    let endpoint: String = row.get(3)?;
    let request_order: i64 = row.get(8)?;
    if !options.force && sessions.node_exists(&session_id, &request_id)? {
      report.rows_existing += 1;
      update_expected_latest(expected_latest, &session_id, &request_id, ts, request_order);
      emit_playback_row_progress(progress, ctx, report);
      continue;
    }
    let (headers, body, response_body) = select_playback_payload(requests, &request_id)?;
    let header_json = parse_json_bytes(&headers).unwrap_or(Value::Null);
    let body_json = if body.is_empty() {
      if headers_expect_body(&header_json) {
        tracing::warn!(
          requests_db = %ctx.requests_db.display(),
          request_id = %request_id,
          "request playback body missing"
        );
        report.decode_errors += 1;
        report.rows_skipped += 1;
        emit_playback_row_progress(progress, ctx, report);
        continue;
      }
      Value::Null
    } else {
      let decoded = match decode_request_body(&header_json, &body) {
        Ok(decoded) => decoded,
        Err(e) => {
          tracing::warn!(
            requests_db = %ctx.requests_db.display(),
            request_id = %request_id,
            error = %e,
            "request playback decode failed"
          );
          report.decode_errors += 1;
          report.rows_skipped += 1;
          emit_playback_row_progress(progress, ctx, report);
          continue;
        }
      };
      match serde_json::from_slice::<Value>(&decoded) {
        Ok(value) => value,
        Err(e) => {
          tracing::warn!(
            requests_db = %ctx.requests_db.display(),
            request_id = %request_id,
            error = %e,
            "request playback json parse failed"
          );
          report.decode_errors += 1;
          report.rows_skipped += 1;
          emit_playback_row_progress(progress, ctx, report);
          continue;
        }
      }
    };
    let request_messages = request_messages_from_json(&endpoint, &body_json);
    let response_messages = response_messages_from_body(&response_body);
    if request_messages.is_empty() && response_messages.is_empty() {
      report.rows_skipped += 1;
      emit_playback_row_progress(progress, ctx, report);
      continue;
    }
    let record = TreeRequestRecord {
      ts,
      session_id,
      parent_session_id: header_str(&header_json, "x-parent-session-id").map(str::to_string),
      request_id,
      endpoint,
      status: row.get::<_, Option<i64>>(7)?.map(|v| v as u16),
      account_id: row.get(4)?,
      provider_id: row.get(5)?,
      model: row.get(6)?,
      request_messages,
      response_messages,
    };
    let parent = sessions.head_for_session(&record.session_id)?;
    sessions.record_tree(&record)?;
    if parent.is_some() {
      let stored = sessions
        .conn
        .query_row(
          "SELECT reduction_kind FROM session_nodes WHERE id = ?1",
          params![record.request_id],
          |r| r.get::<_, String>(0),
        )
        .unwrap_or_default();
      if stored == "conflict_snapshot" {
        report.reduction_mismatches += 1;
      }
    }
    report.rows_recorded += 1;
    update_expected_latest(
      expected_latest,
      &record.session_id,
      &record.request_id,
      record.ts,
      request_order,
    );
    emit_playback_row_progress(progress, ctx, report);
  }
  Ok(())
}

fn request_db_files(dir: &Path) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();
  for entry in std::fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension().and_then(|value| value.to_str()) == Some("db") {
      files.push(path);
    }
  }
  files.sort();
  Ok(files)
}

fn select_playback_payload(requests: &Connection, request_id: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
  requests
    .query_row(
      "SELECT inbound_req_headers, inbound_req_body, inbound_resp_body
       FROM requests
       WHERE request_id = ?1",
      params![request_id],
      |r| {
        Ok((
          r.get::<_, Option<Vec<u8>>>(0)?.unwrap_or_default(),
          r.get::<_, Option<Vec<u8>>>(1)?.unwrap_or_default(),
          r.get::<_, Option<Vec<u8>>>(2)?.unwrap_or_default(),
        ))
      },
    )
    .map_err(Into::into)
}

fn count_request_rows(paths: &[PathBuf]) -> Result<u64> {
  let mut total = 0;
  for path in paths {
    let requests = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    total += count_request_rows_in_connection(&requests)?;
  }
  Ok(total)
}

fn count_request_rows_in_connection(requests: &Connection) -> Result<u64> {
  let count = requests.query_row("SELECT COUNT(*) FROM requests WHERE session_id IS NOT NULL", [], |r| {
    r.get::<_, i64>(0)
  })?;
  Ok(count.max(0) as u64)
}

fn emit_playback_row_progress(
  progress: &mut impl FnMut(PlaybackProgressEvent),
  ctx: PlaybackFileContext<'_>,
  report: &PlaybackReport,
) {
  let global_stats = PlaybackStats::from_report(report);
  let file_stats = subtract_stats(global_stats, ctx.file_start);
  progress(PlaybackProgressEvent::RowProcessed {
    path: ctx.requests_db.to_path_buf(),
    file_index: ctx.file_index,
    files_total: ctx.files_total,
    rows_seen: file_stats.rows_seen,
    rows_total: ctx.rows_total,
    file_stats,
    global_stats,
  });
}

fn subtract_stats(after: PlaybackStats, before: PlaybackStats) -> PlaybackStats {
  PlaybackStats {
    rows_seen: after.rows_seen.saturating_sub(before.rows_seen),
    rows_with_session: after.rows_with_session.saturating_sub(before.rows_with_session),
    rows_recorded: after.rows_recorded.saturating_sub(before.rows_recorded),
    rows_existing: after.rows_existing.saturating_sub(before.rows_existing),
    rows_skipped: after.rows_skipped.saturating_sub(before.rows_skipped),
    decode_errors: after.decode_errors.saturating_sub(before.decode_errors),
    reduction_mismatches: after.reduction_mismatches.saturating_sub(before.reduction_mismatches),
  }
}

fn update_expected_latest(
  out: &mut HashMap<String, (i64, i64, String)>,
  session_id: &str,
  request_id: &str,
  ts: i64,
  idx: i64,
) {
  let update = out
    .get(session_id)
    .map(|(existing_ts, existing_idx, _)| (ts, idx) >= (*existing_ts, *existing_idx))
    .unwrap_or(true);
  if update {
    out.insert(session_id.to_string(), (ts, idx, request_id.to_string()));
  }
}

fn request_order_index(requests: &Connection) -> Result<&'static str> {
  if requests_column_exists(requests, "idx")? {
    Ok("idx")
  } else {
    Ok("rowid")
  }
}

fn requests_column_exists(requests: &Connection, name: &str) -> Result<bool> {
  Ok(
    requests
      .prepare("SELECT 1 FROM pragma_table_info('requests') WHERE name = ?1")?
      .exists(params![name])?,
  )
}

fn check_latest_heads(
  expected: &HashMap<String, (i64, i64, String)>,
  sessions: &Connection,
) -> Result<Vec<LatestMismatch>> {
  let mut out = Vec::new();
  let mut expected: Vec<_> = expected.iter().collect();
  expected.sort_by(|a, b| a.0.cmp(b.0));
  for (session_id, (_, _, expected_request_id)) in expected {
    let actual_request_id = sessions
      .query_row(
        "SELECT n.request_id
         FROM session_heads h
         JOIN session_nodes n ON n.id = h.node_id
         WHERE h.session_id = ?1",
        params![session_id],
        |r| r.get::<_, String>(0),
      )
      .optional()?;
    if actual_request_id.as_deref() != Some(expected_request_id.as_str()) {
      out.push(LatestMismatch {
        session_id: session_id.clone(),
        expected_request_id: expected_request_id.clone(),
        actual_request_id,
      });
    }
  }
  Ok(out)
}

fn decode_request_body(headers: &Value, body: &[u8]) -> std::result::Result<Vec<u8>, String> {
  match header_str(headers, "content-encoding").unwrap_or("identity") {
    "identity" | "" => Ok(body.to_vec()),
    "gzip" => {
      let mut decoder = GzDecoder::new(body);
      let mut out = Vec::new();
      if let Err(e) = decoder.read_to_end(&mut out) {
        return decode_raw_json_body(body).ok_or_else(|| format!("gzip decode failed: {e}"));
      }
      Ok(out)
    }
    "zstd" => zstd::stream::decode_all(body)
      .or_else(|e| decode_raw_json_body(body).ok_or_else(|| format!("zstd decode failed: {e}"))),
    other => Err(format!("unsupported content-encoding: {other}")),
  }
}

fn headers_expect_body(headers: &Value) -> bool {
  let has_encoding = header_str(headers, "content-encoding")
    .map(|value| {
      let value = value.trim();
      !value.is_empty() && !value.eq_ignore_ascii_case("identity")
    })
    .unwrap_or(false);
  let has_content_length = header_str(headers, "content-length")
    .and_then(|value| value.parse::<u64>().ok())
    .map(|value| value > 0)
    .unwrap_or(false);
  has_encoding || has_content_length
}

fn decode_raw_json_body(body: &[u8]) -> Option<Vec<u8>> {
  parse_json_bytes(body).map(|_| body.to_vec())
}

fn header_str<'a>(headers: &'a Value, name: &str) -> Option<&'a str> {
  headers
    .as_object()?
    .iter()
    .find(|(key, _)| key.eq_ignore_ascii_case(name))
    .and_then(|(_, value)| value.as_str())
}

fn parse_json_bytes(bytes: &[u8]) -> Option<Value> {
  serde_json::from_slice(bytes).ok()
}

fn request_messages_from_json(endpoint: &str, value: &Value) -> Vec<MessageRecord> {
  let mut out = Vec::new();
  if let Some(instructions) = value.get("instructions").and_then(Value::as_str) {
    if !instructions.is_empty() {
      out.push(text_message("system", instructions));
    }
  }
  match endpoint {
    "chat_completions" | "chat/completions" => {
      if let Some(messages) = value.get("messages").and_then(Value::as_array) {
        out.extend(messages.iter().filter_map(message_from_value));
      }
    }
    "responses" => {
      if let Some(input) = value.get("input") {
        out.extend(input_messages(input));
      }
    }
    _ => {}
  }
  out
}

fn input_messages(input: &Value) -> Vec<MessageRecord> {
  match input {
    Value::String(text) => vec![text_message("user", text)],
    Value::Array(items) => items.iter().filter_map(message_from_value).collect(),
    Value::Object(_) => message_from_value(input).into_iter().collect(),
    _ => Vec::new(),
  }
}

fn message_from_value(value: &Value) -> Option<MessageRecord> {
  let obj = value.as_object()?;
  let role = obj
    .get("role")
    .and_then(Value::as_str)
    .or_else(|| obj.get("type").and_then(Value::as_str))
    .unwrap_or("user");
  let parts = obj
    .get("content")
    .map(parts_from_value)
    .filter(|parts| !parts.is_empty())
    .unwrap_or_else(|| vec![json_part(value)]);
  Some(MessageRecord {
    role: role.to_string(),
    status: None,
    parts,
  })
}

fn parts_from_value(value: &Value) -> Vec<PartRecord> {
  match value {
    Value::String(text) => vec![PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    }],
    Value::Array(parts) => parts.iter().map(part_from_value).collect(),
    Value::Object(_) => vec![part_from_value(value)],
    _ => Vec::new(),
  }
}

fn part_from_value(value: &Value) -> PartRecord {
  if let Some(text) = value
    .get("text")
    .and_then(Value::as_str)
    .or_else(|| value.get("input_text").and_then(Value::as_str))
    .or_else(|| value.get("output_text").and_then(Value::as_str))
  {
    return PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    };
  }
  json_part(value)
}

fn json_part(value: &Value) -> PartRecord {
  let part_type = value.get("type").and_then(Value::as_str).unwrap_or("json").to_string();
  PartRecord {
    part_type,
    content: Bytes::from(serde_json::to_vec(value).unwrap_or_default()),
  }
}

fn text_message(role: &str, text: &str) -> MessageRecord {
  MessageRecord {
    role: role.to_string(),
    status: None,
    parts: vec![PartRecord {
      part_type: "text".to_string(),
      content: Bytes::from(text.to_string()),
    }],
  }
}

fn response_messages_from_body(body: &[u8]) -> Vec<MessageRecord> {
  if body.is_empty() {
    return Vec::new();
  }
  if let Ok(value) = serde_json::from_slice::<Value>(body) {
    return response_messages_from_json(&value);
  }
  let Ok(text) = std::str::from_utf8(body) else {
    return Vec::new();
  };
  response_messages_from_sse(text)
}

fn response_messages_from_sse(text: &str) -> Vec<MessageRecord> {
  let mut completed = None;
  let mut deltas = String::new();
  for event in text.split("\n\n") {
    let mut event_name = "";
    let mut data = String::new();
    for line in event.lines() {
      if let Some(value) = line.strip_prefix("event:") {
        event_name = value.trim();
      } else if let Some(value) = line.strip_prefix("data:") {
        if !data.is_empty() {
          data.push('\n');
        }
        data.push_str(value.trim());
      }
    }
    if data.is_empty() || data == "[DONE]" {
      continue;
    }
    let Ok(value) = serde_json::from_str::<Value>(&data) else {
      continue;
    };
    if event_name == "response.completed" {
      completed = value.get("response").cloned().or(Some(value));
    } else if event_name.ends_with(".delta") {
      if let Some(delta) = value.get("delta").and_then(Value::as_str) {
        deltas.push_str(delta);
      }
    }
  }
  if let Some(value) = completed {
    let messages = response_messages_from_json(&value);
    if !messages.is_empty() {
      return messages;
    }
  }
  if deltas.is_empty() {
    Vec::new()
  } else {
    vec![text_message("assistant", &deltas)]
  }
}

fn response_messages_from_json(value: &Value) -> Vec<MessageRecord> {
  if let Some(output) = value.get("output").and_then(Value::as_array) {
    let messages: Vec<_> = output.iter().filter_map(message_from_value).collect();
    if !messages.is_empty() {
      return messages;
    }
  }
  if let Some(text) = value.get("output_text").and_then(Value::as_str) {
    return vec![text_message("assistant", text)];
  }
  Vec::new()
}

fn hash_part(part_type: &str, content: &[u8]) -> String {
  let mut h = Sha256::new();
  h.update(part_type.as_bytes());
  h.update([0u8]);
  h.update(content);
  h.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use bytes::Bytes;
  use rusqlite::params;
  use serde_json::json;

  #[test]
  fn record_tree_reduces_against_head_without_splitting_boundaries() {
    let dir = tempdir();
    let path = dir.join("sessions.db");
    let mut db = SessionsDb::open(&path).unwrap();
    let first = vec![msg("system", "instructions"), msg("user", "hello")];
    let second = vec![
      msg("system", "instructions"),
      msg("user", "hello"),
      msg("user", "again"),
    ];
    let conflict = vec![msg("system", "changed"), msg("user", "branch")];

    db.record_tree(&TreeRequestRecord {
      ts: 100,
      session_id: "sess-1".into(),
      parent_session_id: Some("parent-sess".into()),
      request_id: "req-1".into(),
      endpoint: "responses".into(),
      status: Some(200),
      account_id: Some("acct".into()),
      provider_id: Some("prov".into()),
      model: Some("model".into()),
      request_messages: first,
      response_messages: vec![msg("assistant", "hi")],
    })
    .unwrap();
    db.record_tree(&TreeRequestRecord {
      ts: 110,
      session_id: "sess-1".into(),
      parent_session_id: None,
      request_id: "req-2".into(),
      endpoint: "responses".into(),
      status: Some(200),
      account_id: Some("acct".into()),
      provider_id: Some("prov".into()),
      model: Some("model".into()),
      request_messages: second,
      response_messages: vec![msg("assistant", "again")],
    })
    .unwrap();
    db.record_tree(&TreeRequestRecord {
      ts: 120,
      session_id: "sess-1".into(),
      parent_session_id: None,
      request_id: "req-3".into(),
      endpoint: "responses".into(),
      status: Some(200),
      account_id: Some("acct".into()),
      provider_id: Some("prov".into()),
      model: Some("model".into()),
      request_messages: conflict,
      response_messages: Vec::new(),
    })
    .unwrap();

    let rows = db
      .conn
      .prepare(
        "SELECT request_id, parent_id, reduction_kind, common_prefix_messages, request_message_count
         FROM session_nodes
         ORDER BY ts",
      )
      .unwrap()
      .query_map([], |r| {
        Ok((
          r.get::<_, String>(0)?,
          r.get::<_, Option<String>>(1)?,
          r.get::<_, String>(2)?,
          r.get::<_, i64>(3)?,
          r.get::<_, i64>(4)?,
        ))
      })
      .unwrap()
      .collect::<rusqlite::Result<Vec<_>>>()
      .unwrap();
    assert_eq!(rows[0], ("req-1".into(), None, "root_snapshot".into(), 0, 2));
    assert_eq!(
      rows[1],
      ("req-2".into(), Some("req-1".into()), "suffix_append".into(), 2, 1)
    );
    assert_eq!(
      rows[2],
      ("req-3".into(), Some("req-2".into()), "conflict_snapshot".into(), 0, 2)
    );
    let head: String = db
      .conn
      .query_row(
        "SELECT node_id FROM session_heads WHERE session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head, "req-3");

    db.record_tree(&TreeRequestRecord {
      ts: 90,
      session_id: "sess-1".into(),
      parent_session_id: None,
      request_id: "req-old".into(),
      endpoint: "responses".into(),
      status: Some(200),
      account_id: Some("acct".into()),
      provider_id: Some("prov".into()),
      model: Some("model".into()),
      request_messages: vec![msg("user", "old")],
      response_messages: Vec::new(),
    })
    .unwrap();
    let head_after_old_insert: String = db
      .conn
      .query_row(
        "SELECT node_id FROM session_heads WHERE session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head_after_old_insert, "req-3");

    let relation_count: i64 = db
      .conn
      .query_row("SELECT COUNT(*) FROM session_relations", [], |r| r.get(0))
      .unwrap();
    assert_eq!(relation_count, 1);
  }

  #[test]
  fn playback_requests_decodes_zstd_reduces_and_verifies_latest_head() {
    let dir = tempdir();
    let requests_path = dir.join("2026-05-22.db");
    let sessions_path = dir.join("sessions.db");
    crate::requests::open_day_db(&requests_path).unwrap();
    let conn = Connection::open(&requests_path).unwrap();
    insert_request_row(
      &conn,
      100,
      "req-1",
      "sess-1",
      &json!({
        "instructions": "be useful",
        "input": [{"role": "user", "content": [{"type": "input_text", "text": "hello"}]}]
      }),
      sse_completed("hi"),
    );
    insert_request_row(
      &conn,
      110,
      "req-2",
      "sess-1",
      &json!({
        "instructions": "be useful",
        "input": [
          {"role": "user", "content": [{"type": "input_text", "text": "hello"}]},
          {"role": "user", "content": [{"type": "input_text", "text": "again"}]}
        ]
      }),
      "event: response.output_text.delta\ndata: {\"delta\":\"partial\"}\n\n",
    );

    let report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(report.rows_seen, 2);
    assert_eq!(report.rows_recorded, 2);
    assert_eq!(report.rows_existing, 0);
    assert_eq!(report.decode_errors, 0);
    assert!(report.latest_mismatches.is_empty());

    let second_report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(second_report.rows_seen, 2);
    assert_eq!(second_report.rows_recorded, 0);
    assert_eq!(second_report.rows_existing, 2);
    assert!(second_report.latest_mismatches.is_empty());

    let forced_report =
      playback_requests_into_sessions_with_options(&requests_path, &sessions_path, PlaybackOptions { force: true })
        .unwrap();
    assert_eq!(forced_report.rows_seen, 2);
    assert_eq!(forced_report.rows_recorded, 2);
    assert_eq!(forced_report.rows_existing, 0);
    assert!(forced_report.latest_mismatches.is_empty());

    let sessions = Connection::open(&sessions_path).unwrap();
    let reduction: String = sessions
      .query_row(
        "SELECT reduction_kind FROM session_nodes WHERE request_id = 'req-2'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(reduction, "suffix_append");
    let head: String = sessions
      .query_row(
        "SELECT n.request_id FROM session_heads h JOIN session_nodes n ON n.id = h.node_id WHERE h.session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head, "req-2");
    let timestamps = sessions
      .query_row(
        "SELECT s.first_seen_ts, s.last_seen_ts, h.updated_ts, n.ts
         FROM sessions s
         JOIN session_heads h ON h.session_id = s.id
         JOIN session_nodes n ON n.id = h.node_id
         WHERE s.id = 'sess-1'",
        [],
        |r| {
          Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
          ))
        },
      )
      .unwrap();
    assert_eq!(timestamps, (100, 110, 110, 110));
    let response_messages: i64 = sessions
      .query_row(
        "SELECT COUNT(*) FROM node_messages WHERE side = 'response' AND role = 'assistant'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(response_messages, 2);
  }

  #[test]
  fn session_views_expose_current_head_and_message_parts() {
    let dir = tempdir();
    let path = dir.join("sessions.db");
    let mut db = SessionsDb::open(&path).unwrap();
    db.record_tree(&TreeRequestRecord {
      ts: 100,
      session_id: "sess-1".into(),
      parent_session_id: None,
      request_id: "req-1".into(),
      endpoint: "responses".into(),
      status: Some(200),
      account_id: Some("acct".into()),
      provider_id: Some("prov".into()),
      model: Some("model".into()),
      request_messages: vec![msg("user", "hello")],
      response_messages: vec![msg("assistant", "hi")],
    })
    .unwrap();

    let current = db
      .conn
      .query_row(
        "SELECT session_id, head_request_id, head_endpoint, head_status, account_id, provider_id, model,
                head_reduction_kind, head_request_message_count, head_response_message_count
         FROM session_current",
        [],
        |r| {
          Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
            r.get::<_, i64>(8)?,
            r.get::<_, i64>(9)?,
          ))
        },
      )
      .unwrap();
    assert_eq!(
      current,
      (
        "sess-1".into(),
        "req-1".into(),
        "responses".into(),
        200,
        "acct".into(),
        "prov".into(),
        "model".into(),
        "root_snapshot".into(),
        1,
        1,
      )
    );

    let messages = db
      .conn
      .prepare(
        "SELECT session_id, node_id, request_id, is_head, side, message_seq, role, part_index, part_type, content
         FROM session_messages
         ORDER BY node_ts, side, message_seq, part_index",
      )
      .unwrap()
      .query_map([], |r| {
        Ok((
          r.get::<_, String>(0)?,
          r.get::<_, String>(1)?,
          r.get::<_, String>(2)?,
          r.get::<_, bool>(3)?,
          r.get::<_, String>(4)?,
          r.get::<_, i64>(5)?,
          r.get::<_, String>(6)?,
          r.get::<_, i64>(7)?,
          r.get::<_, String>(8)?,
          String::from_utf8(r.get::<_, Vec<u8>>(9)?).unwrap(),
        ))
      })
      .unwrap()
      .collect::<rusqlite::Result<Vec<_>>>()
      .unwrap();
    assert_eq!(
      messages,
      vec![
        (
          "sess-1".into(),
          "req-1".into(),
          "req-1".into(),
          true,
          "request".into(),
          0,
          "user".into(),
          0,
          "text".into(),
          "hello".into(),
        ),
        (
          "sess-1".into(),
          "req-1".into(),
          "req-1".into(),
          true,
          "response".into(),
          0,
          "assistant".into(),
          0,
          "text".into(),
          "hi".into(),
        ),
      ]
    );
  }

  #[test]
  fn playback_requests_dir_sorts_files_and_verifies_latest_across_all_days() {
    let dir = tempdir();
    let requests_dir = dir.join("requests");
    std::fs::create_dir_all(&requests_dir).unwrap();
    let first_day = requests_dir.join("2026-05-21.db");
    let second_day = requests_dir.join("2026-05-22.db");
    std::fs::write(requests_dir.join("2026-05-23.db.bak"), b"not sqlite").unwrap();
    crate::requests::open_day_db(&second_day).unwrap();
    crate::requests::open_day_db(&first_day).unwrap();
    let first_conn = Connection::open(&first_day).unwrap();
    let second_conn = Connection::open(&second_day).unwrap();
    insert_request_row(
      &second_conn,
      200,
      "req-new",
      "sess-1",
      &json!({
        "input": [
          {"role": "user", "content": [{"type": "input_text", "text": "hello"}]},
          {"role": "user", "content": [{"type": "input_text", "text": "new"}]}
        ]
      }),
      sse_completed("new"),
    );
    insert_request_row(
      &first_conn,
      100,
      "req-old",
      "sess-1",
      &json!({
        "input": [{"role": "user", "content": [{"type": "input_text", "text": "hello"}]}]
      }),
      sse_completed("old"),
    );

    let sessions_path = dir.join("sessions.db");
    let report = playback_requests_source_into_sessions(
      PlaybackSource::Dir(requests_dir),
      &sessions_path,
      PlaybackOptions::default(),
    )
    .unwrap();
    assert_eq!(report.rows_seen, 2);
    assert_eq!(report.rows_recorded, 2);
    assert!(report.latest_mismatches.is_empty());

    let sessions = Connection::open(&sessions_path).unwrap();
    let head: String = sessions
      .query_row(
        "SELECT n.request_id FROM session_heads h JOIN session_nodes n ON n.id = h.node_id WHERE h.session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head, "req-new");
    let parent: Option<String> = sessions
      .query_row(
        "SELECT parent_id FROM session_nodes WHERE request_id = 'req-new'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(parent.as_deref(), Some("req-old"));
  }

  #[test]
  fn playback_requests_accepts_legacy_requests_table_without_idx() {
    let dir = tempdir();
    let requests_path = dir.join("legacy.db");
    let sessions_path = dir.join("sessions.db");
    let conn = Connection::open(&requests_path).unwrap();
    create_legacy_requests_table(&conn);
    insert_legacy_request_row(
      &conn,
      100,
      "req-1",
      "sess-1",
      &json!({
        "input": [{"role": "user", "content": [{"type": "input_text", "text": "hello"}]}]
      }),
      sse_completed("old"),
    );

    let report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(report.rows_seen, 1);
    assert_eq!(report.rows_recorded, 1);
    assert!(report.latest_mismatches.is_empty());

    let sessions = Connection::open(&sessions_path).unwrap();
    let head: String = sessions
      .query_row(
        "SELECT n.request_id FROM session_heads h JOIN session_nodes n ON n.id = h.node_id WHERE h.session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head, "req-1");
    let node_ts: i64 = sessions
      .query_row("SELECT ts FROM session_nodes WHERE request_id = 'req-1'", [], |r| {
        r.get(0)
      })
      .unwrap();
    assert_eq!(node_ts, 100_000);
  }

  #[test]
  fn playback_requests_accepts_raw_json_when_encoding_header_is_stale() {
    let dir = tempdir();
    let requests_path = dir.join("2026-05-22.db");
    let sessions_path = dir.join("sessions.db");
    crate::requests::open_day_db(&requests_path).unwrap();
    let conn = Connection::open(&requests_path).unwrap();
    insert_request_row_with_raw_body(
      &conn,
      100,
      "req-1",
      "sess-1",
      &json!({
        "input": [{"role": "user", "content": [{"type": "input_text", "text": "hello"}]}]
      }),
      sse_completed("ok"),
    );

    let report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(report.rows_seen, 1);
    assert_eq!(report.rows_recorded, 1);
    assert_eq!(report.decode_errors, 0);
    assert!(report.latest_mismatches.is_empty());
  }

  #[test]
  fn playback_requests_rejects_missing_request_body_with_encoded_header() {
    let dir = tempdir();
    let requests_path = dir.join("2026-05-22.db");
    let sessions_path = dir.join("sessions.db");
    crate::requests::open_day_db(&requests_path).unwrap();
    let conn = Connection::open(&requests_path).unwrap();
    insert_request_row_with_missing_encoded_body(&conn, 100, "req-1", "sess-1", sse_completed("ok"));

    let report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(report.rows_seen, 1);
    assert_eq!(report.rows_recorded, 0);
    assert_eq!(report.rows_skipped, 1);
    assert_eq!(report.decode_errors, 1);
    assert!(report.latest_mismatches.is_empty());

    let sessions = Connection::open(&sessions_path).unwrap();
    let node_count: i64 = sessions
      .query_row(
        "SELECT COUNT(*) FROM session_nodes WHERE request_id = 'req-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(node_count, 0);
  }

  #[test]
  fn playback_latest_head_ignores_rows_without_messages() {
    let dir = tempdir();
    let requests_path = dir.join("2026-05-22.db");
    let sessions_path = dir.join("sessions.db");
    crate::requests::open_day_db(&requests_path).unwrap();
    let conn = Connection::open(&requests_path).unwrap();
    insert_request_row(
      &conn,
      100,
      "req-recorded",
      "sess-1",
      &json!({
        "input": [{"role": "user", "content": [{"type": "input_text", "text": "hello"}]}]
      }),
      sse_completed("ok"),
    );
    insert_request_row_with_empty_identity_body_and_status(&conn, 110, "req-empty", "sess-1", 400, "");

    let report = playback_requests_into_sessions(&requests_path, &sessions_path).unwrap();
    assert_eq!(report.rows_seen, 2);
    assert_eq!(report.rows_recorded, 1);
    assert_eq!(report.rows_skipped, 1);
    assert!(report.latest_mismatches.is_empty());

    let sessions = Connection::open(&sessions_path).unwrap();
    let head: String = sessions
      .query_row(
        "SELECT n.request_id FROM session_heads h JOIN session_nodes n ON n.id = h.node_id WHERE h.session_id = 'sess-1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(head, "req-recorded");
  }

  fn msg(role: &str, text: &str) -> MessageRecord {
    MessageRecord {
      role: role.into(),
      status: None,
      parts: vec![PartRecord {
        part_type: "text".into(),
        content: Bytes::from(text.to_string()),
      }],
    }
  }

  fn insert_request_row(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    body: &Value,
    response_body: impl AsRef<[u8]>,
  ) {
    let raw_body = serde_json::to_vec(body).unwrap();
    let encoded_body = zstd::stream::encode_all(raw_body.as_slice(), 0).unwrap();
    let headers = serde_json::to_vec(&json!({
      "Content-Encoding": "zstd",
      "X-Parent-Session-Id": "parent-session"
    }))
    .unwrap();
    conn
      .execute(
        "INSERT INTO request_connection (request_id, ts, ver, endpoint, status)
         VALUES (?1, ?2, 'test', 'responses', 200)",
        params![request_id, ts],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_metadata (request_id, session_id, account_id, provider_id, model)
         VALUES (?1, ?2, 'acct', 'prov', 'model')",
        params![request_id, session_id],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_downstream (request_id, inbound_req_headers, inbound_req_body, inbound_resp_body)
         VALUES (?1, ?2, ?3, ?4)",
        params![request_id, headers, encoded_body, response_body.as_ref()],
      )
      .unwrap();
  }

  fn insert_request_row_with_missing_encoded_body(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    response_body: impl AsRef<[u8]>,
  ) {
    let headers = json!({
      "Content-Encoding": "zstd",
      "Content-Length": "81104"
    });
    insert_request_row_with_empty_body_and_status(conn, ts, request_id, session_id, 200, &headers, response_body);
  }

  fn insert_request_row_with_empty_identity_body_and_status(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    status: u16,
    response_body: impl AsRef<[u8]>,
  ) {
    insert_request_row_with_empty_body_and_status(conn, ts, request_id, session_id, status, &json!({}), response_body);
  }

  fn insert_request_row_with_empty_body_and_status(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    status: u16,
    headers: &Value,
    response_body: impl AsRef<[u8]>,
  ) {
    let headers = serde_json::to_vec(headers).unwrap();
    conn
      .execute(
        "INSERT INTO request_connection (request_id, ts, ver, endpoint, status)
         VALUES (?1, ?2, 'test', 'responses', ?3)",
        params![request_id, ts, status],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_metadata (request_id, session_id, account_id, provider_id, model)
         VALUES (?1, ?2, 'acct', 'prov', 'model')",
        params![request_id, session_id],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_downstream (request_id, inbound_req_headers, inbound_req_body, inbound_resp_body)
         VALUES (?1, ?2, X'', ?3)",
        params![request_id, headers, response_body.as_ref()],
      )
      .unwrap();
  }

  fn insert_request_row_with_raw_body(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    body: &Value,
    response_body: impl AsRef<[u8]>,
  ) {
    let raw_body = serde_json::to_vec(body).unwrap();
    let headers = serde_json::to_vec(&json!({ "Content-Encoding": "zstd" })).unwrap();
    conn
      .execute(
        "INSERT INTO request_connection (request_id, ts, ver, endpoint, status)
         VALUES (?1, ?2, 'test', 'responses', 200)",
        params![request_id, ts],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_metadata (request_id, session_id, account_id, provider_id, model)
         VALUES (?1, ?2, 'acct', 'prov', 'model')",
        params![request_id, session_id],
      )
      .unwrap();
    conn
      .execute(
        "INSERT INTO request_downstream (request_id, inbound_req_headers, inbound_req_body, inbound_resp_body)
         VALUES (?1, ?2, ?3, ?4)",
        params![request_id, headers, raw_body, response_body.as_ref()],
      )
      .unwrap();
  }

  fn create_legacy_requests_table(conn: &Connection) {
    conn
      .execute_batch(
        "CREATE TABLE requests (
          id INTEGER PRIMARY KEY,
          ts INTEGER NOT NULL,
          session_id TEXT,
          request_id TEXT,
          endpoint TEXT NOT NULL,
          account_id TEXT,
          provider_id TEXT,
          model TEXT,
          status INTEGER,
          inbound_req_headers BLOB,
          inbound_req_body BLOB,
          inbound_resp_body BLOB
        );",
      )
      .unwrap();
  }

  fn insert_legacy_request_row(
    conn: &Connection,
    ts: i64,
    request_id: &str,
    session_id: &str,
    body: &Value,
    response_body: impl AsRef<[u8]>,
  ) {
    let body = serde_json::to_vec(body).unwrap();
    let headers = serde_json::to_vec(&json!({})).unwrap();
    conn
      .execute(
        "INSERT INTO requests (
          ts, session_id, request_id, endpoint, account_id, provider_id, model, status,
          inbound_req_headers, inbound_req_body, inbound_resp_body
         )
         VALUES (?1, ?2, ?3, 'responses', 'acct', 'prov', 'model', 200, ?4, ?5, ?6)",
        params![ts, session_id, request_id, headers, body, response_body.as_ref()],
      )
      .unwrap();
  }

  fn sse_completed(text: &str) -> String {
    format!(
      "event: response.completed\ndata: {{\"response\":{{\"output\":[{{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{{\"type\":\"output_text\",\"text\":\"{text}\"}}]}}]}}}}\n\n"
    )
  }

  fn tempdir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("tokn-router-sessions-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&p).unwrap();
    p
  }
}
