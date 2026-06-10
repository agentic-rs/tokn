use super::{migrate, MessageRecord, PartRecord, Result};
use bytes::Bytes;
use flate2::read::GzDecoder;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use tokn_core::db::SessionSource;
use tokn_core::request_event::RequestEndpoint;
use tracing::{debug, trace};

pub struct SessionRecord<'a> {
  pub ts: i64,
  pub session_id: &'a str,
  pub session_source: SessionSource,
  pub endpoint: &'a RequestEndpoint,
  pub account_id: &'a str,
  pub provider_id: &'a str,
  pub model: &'a str,
  pub messages: &'a [MessageRecord],
}

const BOOTSTRAP: &str = include_str!("../schemas/snapshot/sessions/v0.2.0.sql");
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
  pub rows_skipped: u64,
  pub decode_errors: u64,
  pub reduction_mismatches: u64,
  pub latest_mismatches: Vec<LatestMismatch>,
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
      "INSERT INTO sessions (id, first_seen_ts, last_seen_ts, source, account_id, provider_id, model, message_count, part_count)
       VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, 0, 0)
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
       ON CONFLICT(session_id) DO UPDATE SET node_id = excluded.node_id, updated_ts = excluded.updated_ts",
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

  /// Append all messages of a single inbound call to the session log. Each
  /// `MessageRecord` becomes one logical "message" (a contiguous group of
  /// `session_parts` rows sharing `message_seq`); each `PartRecord` becomes
  /// one row, with the blob deduplicated in `part_blobs`.
  pub fn record(&mut self, r: &SessionRecord<'_>) -> Result<()> {
    if r.messages.is_empty() {
      debug!(session_id = %r.session_id, "sessions.record: no messages, skipping");
      return Ok(());
    }
    trace!(
      session_id = %r.session_id,
      source = r.session_source.as_str(),
      message_count = r.messages.len(),
      "sessions.record: begin",
    );

    let tx = self.conn.transaction()?;

    // Resolve the next free part_seq / message_seq for this session up
    // front so we can interleave appends without races (we hold the
    // sqlite write lock for the whole transaction).
    let (mut next_part_seq, mut next_message_seq) = next_seqs(&tx, r.session_id)?;

    let new_message_count = r.messages.len() as i64;
    let new_part_count: i64 = r.messages.iter().map(|m| m.parts.len() as i64).sum();

    tx.execute(
      "INSERT INTO sessions (id, first_seen_ts, last_seen_ts, source, account_id, provider_id, model, message_count, part_count)
       VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
       ON CONFLICT(id) DO UPDATE SET
         last_seen_ts  = excluded.last_seen_ts,
         account_id    = excluded.account_id,
         provider_id   = excluded.provider_id,
         model         = excluded.model,
         message_count = message_count + excluded.message_count,
         part_count    = part_count + excluded.part_count",
      params![
        r.session_id,
        r.ts,
        r.session_source.as_str(),
        r.account_id,
        r.provider_id,
        r.model,
        new_message_count,
        new_part_count,
      ],
    )?;

    for m in r.messages {
      append_message(&tx, r, m, &mut next_part_seq, next_message_seq)?;
      next_message_seq += 1;
    }
    tx.commit()?;
    trace!(session_id = %r.session_id, "sessions.record: committed");
    Ok(())
  }
}

fn next_seqs(tx: &rusqlite::Transaction<'_>, session_id: &str) -> Result<(i64, i64)> {
  let row: (Option<i64>, Option<i64>) = tx
    .prepare("SELECT MAX(part_seq), MAX(message_seq) FROM session_parts WHERE session_id = ?1")?
    .query_row(params![session_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
  Ok((row.0.map(|v| v + 1).unwrap_or(0), row.1.map(|v| v + 1).unwrap_or(0)))
}

fn append_message(
  tx: &rusqlite::Transaction<'_>,
  r: &SessionRecord<'_>,
  m: &MessageRecord,
  next_part_seq: &mut i64,
  message_seq: i64,
) -> Result<()> {
  for (idx, part) in m.parts.iter().enumerate() {
    upsert_part_blob(tx, part)?;
    tx.execute(
      "INSERT INTO session_parts
         (session_id, part_seq, message_seq, part_index, ts, endpoint, role, status, part_hash)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
      params![
        r.session_id,
        *next_part_seq,
        message_seq,
        idx as i64,
        r.ts,
        r.endpoint.as_str(),
        m.role,
        m.status.map(|v| v as i64),
        hash_part(&part.part_type, part.content.as_ref()),
      ],
    )?;
    *next_part_seq += 1;
  }
  Ok(())
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
  let requests = Connection::open_with_flags(requests_db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
  let mut sessions = SessionsDb::open(sessions_db)?;
  let mut report = PlaybackReport::default();
  let mut stmt = requests.prepare(
    "SELECT ts, session_id, request_id, endpoint, account_id, provider_id, model, status,
            inbound_req_headers, inbound_req_body, inbound_resp_body
     FROM requests
     WHERE session_id IS NOT NULL
     ORDER BY ts, idx",
  )?;
  let mut rows = stmt.query([])?;
  while let Some(row) = rows.next()? {
    report.rows_seen += 1;
    report.rows_with_session += 1;
    let headers = row.get::<_, Option<Vec<u8>>>(8)?.unwrap_or_default();
    let body = row.get::<_, Option<Vec<u8>>>(9)?.unwrap_or_default();
    let response_body = row.get::<_, Option<Vec<u8>>>(10)?.unwrap_or_default();
    let header_json = parse_json_bytes(&headers).unwrap_or(Value::Null);
    let endpoint: String = row.get(3)?;
    let decoded = match decode_request_body(&header_json, &body) {
      Ok(decoded) => decoded,
      Err(e) => {
        tracing::warn!(error = %e, "request playback decode failed");
        report.decode_errors += 1;
        report.rows_skipped += 1;
        continue;
      }
    };
    let body_json = match serde_json::from_slice::<Value>(&decoded) {
      Ok(value) => value,
      Err(e) => {
        tracing::warn!(error = %e, "request playback json parse failed");
        report.decode_errors += 1;
        report.rows_skipped += 1;
        continue;
      }
    };
    let request_messages = request_messages_from_json(&endpoint, &body_json);
    let response_messages = response_messages_from_body(&response_body);
    if request_messages.is_empty() && response_messages.is_empty() {
      report.rows_skipped += 1;
      continue;
    }
    let record = TreeRequestRecord {
      ts: row.get(0)?,
      session_id: row.get(1)?,
      parent_session_id: header_str(&header_json, "x-parent-session-id").map(str::to_string),
      request_id: row.get(2)?,
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
  }
  report.latest_mismatches = check_latest_heads(&requests, &sessions.conn)?;
  Ok(report)
}

fn check_latest_heads(requests: &Connection, sessions: &Connection) -> Result<Vec<LatestMismatch>> {
  let mut stmt = requests.prepare(
    "SELECT session_id, request_id
     FROM (
       SELECT session_id, request_id, ROW_NUMBER() OVER (PARTITION BY session_id ORDER BY ts DESC, idx DESC) AS rn
       FROM requests
       WHERE session_id IS NOT NULL
     )
     WHERE rn = 1
     ORDER BY session_id",
  )?;
  let rows = stmt
    .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  let mut out = Vec::new();
  for (session_id, expected_request_id) in rows {
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
        session_id,
        expected_request_id,
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
      decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("gzip decode failed: {e}"))?;
      Ok(out)
    }
    "zstd" => zstd::stream::decode_all(body).map_err(|e| format!("zstd decode failed: {e}")),
    other => Err(format!("unsupported content-encoding: {other}")),
  }
}

fn header_str<'a>(headers: &'a Value, name: &str) -> Option<&'a str> {
  headers
    .as_object()?
    .get(name)
    .or_else(|| headers.as_object()?.get(&name.to_ascii_lowercase()))?
    .as_str()
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
  use tokn_core::provider::Endpoint;

  fn rec(parts: Vec<(String, Bytes)>) -> Vec<MessageRecord> {
    vec![MessageRecord {
      role: "user".into(),
      status: None,
      parts: parts
        .into_iter()
        .map(|(t, c)| PartRecord {
          part_type: t,
          content: c,
        })
        .collect(),
    }]
  }

  #[test]
  fn dedupes_identical_parts_across_messages() {
    let dir = tempdir();
    let path = dir.join("sessions.db");
    let mut db = SessionsDb::open(&path).unwrap();
    let part = ("text".to_string(), Bytes::from_static(b"hello"));
    let messages1 = rec(vec![part.clone()]);
    let messages2 = rec(vec![part.clone()]);
    db.record(&SessionRecord {
      ts: 100,
      session_id: "s1",
      session_source: SessionSource::Header,
      endpoint: &Endpoint::ChatCompletions.into(),
      account_id: "a",
      provider_id: "p",
      model: "m",
      messages: &messages1,
    })
    .unwrap();
    db.record(&SessionRecord {
      ts: 100,
      session_id: "s2",
      session_source: SessionSource::Header,
      endpoint: &Endpoint::ChatCompletions.into(),
      account_id: "a",
      provider_id: "p",
      model: "m",
      messages: &messages2,
    })
    .unwrap();
    let blobs: i64 = db
      .conn
      .query_row("SELECT COUNT(*) FROM part_blobs", [], |r| r.get(0))
      .unwrap();
    assert_eq!(blobs, 1);
    let parts: i64 = db
      .conn
      .query_row("SELECT COUNT(*) FROM session_parts", [], |r| r.get(0))
      .unwrap();
    assert_eq!(parts, 2);
  }

  #[test]
  fn appending_advances_part_seq() {
    let dir = tempdir();
    let path = dir.join("sessions.db");
    let mut db = SessionsDb::open(&path).unwrap();
    let messages1 = rec(vec![
      ("text".into(), Bytes::from_static(b"hello")),
      ("text".into(), Bytes::from_static(b"world")),
    ]);
    db.record(&SessionRecord {
      ts: 100,
      session_id: "s1",
      session_source: SessionSource::Header,
      endpoint: &Endpoint::ChatCompletions.into(),
      account_id: "a",
      provider_id: "p",
      model: "m",
      messages: &messages1,
    })
    .unwrap();
    let messages2 = rec(vec![("text".into(), Bytes::from_static(b"again"))]);
    db.record(&SessionRecord {
      ts: 100,
      session_id: "s1",
      session_source: SessionSource::Header,
      endpoint: &Endpoint::ChatCompletions.into(),
      account_id: "a",
      provider_id: "p",
      model: "m",
      messages: &messages2,
    })
    .unwrap();
    let max_part_seq: i64 = db
      .conn
      .query_row(
        "SELECT MAX(part_seq) FROM session_parts WHERE session_id = 's1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(max_part_seq, 2);
    let max_msg_seq: i64 = db
      .conn
      .query_row(
        "SELECT MAX(message_seq) FROM session_parts WHERE session_id = 's1'",
        [],
        |r| r.get(0),
      )
      .unwrap();
    assert_eq!(max_msg_seq, 1);
    let (mc, pc): (i64, i64) = db
      .conn
      .query_row(
        "SELECT message_count, part_count FROM sessions WHERE id = 's1'",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
      )
      .unwrap();
    assert_eq!(mc, 2);
    assert_eq!(pc, 3);
  }

  fn tempdir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("tokn-router-sessions-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&p).unwrap();
    p
  }
}
