use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::HashSet;
use std::path::Path;

use super::super::database::open_readonly;
use super::super::effective_limit;
use super::super::schema::{read_schema_version, SESSION_TREE_SCHEMA_VERSION};
use super::super::value::sqlite_status;
use super::{
  SessionMessage, SessionMessageTruncation, SessionNodeDetail, SessionNodeDetailTruncation, SessionNodeSummary,
  SessionPart, SessionPartContent, SessionPartEncoding, SessionPartOmissionReason, SessionSummary, StoredSessionDetail,
};
use crate::Result;

const MAX_REQUEST_MESSAGES: usize = 200;
const MAX_RESPONSE_MESSAGES: usize = 100;
const MAX_PARTS_PER_SIDE: usize = 256;
const MAX_PART_CONTENT_BYTES: usize = 64 * 1024;
const MAX_INLINE_CONTENT_BYTES_PER_SIDE: usize = 256 * 1024;

const NORMALIZED_FIRST_TS_SQL: &str =
  "CASE WHEN s.first_seen_ts > -10000000000 AND s.first_seen_ts < 10000000000 THEN s.first_seen_ts * 1000 ELSE s.first_seen_ts END";
const NORMALIZED_LAST_TS_SQL: &str =
  "CASE WHEN COALESCE(head.ts, s.last_seen_ts) > -10000000000 AND COALESCE(head.ts, s.last_seen_ts) < 10000000000 THEN COALESCE(head.ts, s.last_seen_ts) * 1000 ELSE COALESCE(head.ts, s.last_seen_ts) END";
const NORMALIZED_NODE_TS_SQL: &str =
  "CASE WHEN n.ts > -10000000000 AND n.ts < 10000000000 THEN n.ts * 1000 ELSE n.ts END";

/// Return the most recently active sessions stored in one `sessions.db` file.
///
/// A missing database is normal when session recording has not been enabled, so
/// it returns an empty list without creating the file. Schema version 2 is the
/// minimum because these queries deliberately use the base tree tables rather
/// than the optional version-3 views.
pub fn list_sessions_from_db(sessions_db: &Path, limit: Option<usize>) -> Result<Vec<SessionSummary>> {
  let Some(conn) = open_readonly(sessions_db)? else {
    return Ok(Vec::new());
  };
  require_tree_schema(&conn)?;

  let limit = effective_limit(limit);
  let sql = format!(
    "SELECT
       s.id,
       s.source,
       {NORMALIZED_FIRST_TS_SQL},
       {NORMALIZED_LAST_TS_SQL},
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
     WHERE s.id <> ''
     ORDER BY {NORMALIZED_LAST_TS_SQL} DESC, s.id ASC
     LIMIT ?1"
  );
  let mut stmt = conn.prepare(&sql)?;
  let sessions = stmt
    .query_map(params![limit as i64], session_summary_from_db_row)?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(sessions)
}

/// Return bounded semantic tree metadata for one stored session.
pub fn get_session_from_db(
  sessions_db: &Path,
  session_id: &str,
  limit: Option<usize>,
) -> Result<Option<StoredSessionDetail>> {
  let Some(mut conn) = open_readonly(sessions_db)? else {
    return Ok(None);
  };
  read_transaction(&mut conn, |conn| {
    require_tree_schema(conn)?;

    let Some((session, head_node_id)) = select_session(conn, session_id)? else {
      return Ok(None);
    };
    let limit = effective_limit(limit);
    let mut nodes = select_bounded_nodes(conn, session_id, limit)?;
    nodes.sort_by(|left, right| left.ts.cmp(&right.ts).then_with(|| left.node_id.cmp(&right.node_id)));
    let nodes_truncated = session.request_count > nodes.len() as u64;

    Ok(Some(StoredSessionDetail {
      session,
      head_node_id,
      nodes,
      nodes_truncated,
    }))
  })
}

/// Materialize one stored semantic node without consulting request history.
pub fn get_session_node_from_db(
  sessions_db: &Path,
  session_id: &str,
  node_id: &str,
) -> Result<Option<SessionNodeDetail>> {
  let Some(mut conn) = open_readonly(sessions_db)? else {
    return Ok(None);
  };
  read_transaction(&mut conn, |conn| {
    require_tree_schema(conn)?;

    let Some(node) = select_node(conn, session_id, node_id)? else {
      return Ok(None);
    };
    let lineage = select_lineage(conn, session_id, node_id)?;
    let active_lineage = active_request_lineage(&lineage);
    let request_stats = select_lineage_stats(conn, active_lineage, "request")?;
    let response_stats = select_side_stats(conn, node_id, "response")?;
    let request_refs = select_request_message_tail(conn, active_lineage, MAX_REQUEST_MESSAGES)?;
    let response_refs = select_node_message_refs(conn, node_id, "response", MAX_RESPONSE_MESSAGES, false)?;

    let mut request_budget = ContentBudget::new();
    let request_messages = load_messages(conn, request_refs, &mut request_budget)?;
    let mut response_budget = ContentBudget::new();
    let response_messages = load_messages(conn, response_refs, &mut response_budget)?;

    let request_messages_returned = request_messages.len() as u64;
    let response_messages_returned = response_messages.len() as u64;
    let parts_total = request_stats.parts.saturating_add(response_stats.parts);
    let parts_returned = request_budget
      .parts_returned
      .saturating_add(response_budget.parts_returned);
    let content_bytes_total = request_stats.content_bytes.saturating_add(response_stats.content_bytes);
    let content_bytes_returned = request_budget
      .content_bytes_returned
      .saturating_add(response_budget.content_bytes_returned);

    Ok(Some(SessionNodeDetail {
      node,
      request_messages,
      response_messages,
      truncation: SessionNodeDetailTruncation {
        request_messages: SessionMessageTruncation {
          messages_total: request_stats.messages,
          messages_returned: request_messages_returned,
          messages_omitted_before: request_stats.messages.saturating_sub(request_messages_returned),
          messages_omitted_after: 0,
        },
        response_messages: SessionMessageTruncation {
          messages_total: response_stats.messages,
          messages_returned: response_messages_returned,
          messages_omitted_before: 0,
          messages_omitted_after: response_stats.messages.saturating_sub(response_messages_returned),
        },
        parts_total,
        parts_returned,
        parts_omitted: parts_total.saturating_sub(parts_returned),
        content_bytes_total,
        content_bytes_returned,
        content_parts_truncated: request_budget
          .content_parts_truncated
          .saturating_add(response_budget.content_parts_truncated),
        binary_parts_elided: request_budget
          .binary_parts_elided
          .saturating_add(response_budget.binary_parts_elided),
      },
    }))
  })
}

fn read_transaction<T>(conn: &mut Connection, operation: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
  let transaction = conn.transaction_with_behavior(TransactionBehavior::Deferred)?;
  let result = operation(&transaction)?;
  transaction.commit()?;
  Ok(result)
}

fn require_tree_schema(conn: &Connection) -> Result<()> {
  let version = read_schema_version(conn)?;
  if version < SESSION_TREE_SCHEMA_VERSION {
    return Err(crate::Error::UnsupportedSessionSchema { version });
  }
  Ok(())
}

fn select_session(conn: &Connection, session_id: &str) -> Result<Option<(SessionSummary, Option<String>)>> {
  let sql = format!(
    "SELECT
       s.id,
       s.source,
       {NORMALIZED_FIRST_TS_SQL},
       {NORMALIZED_LAST_TS_SQL},
       (SELECT COUNT(*) FROM session_nodes AS node_count WHERE node_count.session_id = s.id),
       head.request_id,
       head.endpoint,
       head.status,
       COALESCE(head.account_id, s.account_id),
       COALESCE(head.provider_id, s.provider_id),
       COALESCE(head.model, s.model),
       head_ref.node_id
     FROM sessions AS s
     LEFT JOIN session_heads AS head_ref ON head_ref.session_id = s.id
     LEFT JOIN session_nodes AS head ON head.id = head_ref.node_id
     WHERE s.id = ?1"
  );
  Ok(
    conn
      .query_row(&sql, params![session_id], |row| {
        Ok((session_summary_from_db_row(row)?, row.get(11)?))
      })
      .optional()?,
  )
}

fn session_summary_from_db_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionSummary> {
  let last_ts: i64 = row.get(3)?;
  Ok(SessionSummary {
    session_id: row.get(0)?,
    source: row.get(1)?,
    first_ts: row.get(2)?,
    last_ts,
    request_count: nonnegative_count(row.get(4)?),
    last_request_day: crate::requests::day_key(last_ts),
    last_request_id: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
    endpoint: row.get(6)?,
    status: sqlite_status(row.get(7)?),
    account_id: row.get(8)?,
    provider_id: row.get(9)?,
    model: row.get(10)?,
  })
}

fn select_bounded_nodes(conn: &Connection, session_id: &str, limit: usize) -> Result<Vec<SessionNodeSummary>> {
  let sql = format!(
    "SELECT
       n.id,
       n.parent_id,
       n.request_id,
       {NORMALIZED_NODE_TS_SQL},
       n.endpoint,
       n.status,
       COALESCE(n.account_id, s.account_id),
       COALESCE(n.provider_id, s.provider_id),
       COALESCE(n.model, s.model),
       n.reduction_kind,
       n.parent_source,
       n.common_prefix_messages,
       n.request_message_count,
       n.response_message_count,
       CASE WHEN head_ref.node_id = n.id THEN 1 ELSE 0 END
     FROM session_nodes AS n
     JOIN sessions AS s ON s.id = n.session_id
     LEFT JOIN session_heads AS head_ref ON head_ref.session_id = n.session_id
     WHERE n.session_id = ?1
     ORDER BY CASE WHEN head_ref.node_id = n.id THEN 0 ELSE 1 END,
              {NORMALIZED_NODE_TS_SQL} DESC,
              n.id DESC
     LIMIT ?2"
  );
  let mut stmt = conn.prepare(&sql)?;
  let nodes = stmt
    .query_map(params![session_id, limit as i64], session_node_from_row)?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(nodes)
}

fn select_node(conn: &Connection, session_id: &str, node_id: &str) -> Result<Option<SessionNodeSummary>> {
  let sql = format!(
    "SELECT
       n.id,
       n.parent_id,
       n.request_id,
       {NORMALIZED_NODE_TS_SQL},
       n.endpoint,
       n.status,
       COALESCE(n.account_id, s.account_id),
       COALESCE(n.provider_id, s.provider_id),
       COALESCE(n.model, s.model),
       n.reduction_kind,
       n.parent_source,
       n.common_prefix_messages,
       n.request_message_count,
       n.response_message_count,
       CASE WHEN head_ref.node_id = n.id THEN 1 ELSE 0 END
     FROM session_nodes AS n
     JOIN sessions AS s ON s.id = n.session_id
     LEFT JOIN session_heads AS head_ref ON head_ref.session_id = n.session_id
     WHERE n.session_id = ?1 AND n.id = ?2"
  );
  Ok(
    conn
      .query_row(&sql, params![session_id, node_id], session_node_from_row)
      .optional()?,
  )
}

fn session_node_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionNodeSummary> {
  Ok(SessionNodeSummary {
    node_id: row.get(0)?,
    parent_node_id: row.get(1)?,
    request_id: row.get(2)?,
    ts: row.get(3)?,
    endpoint: row.get(4)?,
    status: sqlite_status(row.get(5)?),
    account_id: row.get(6)?,
    provider_id: row.get(7)?,
    model: row.get(8)?,
    reduction_kind: row.get(9)?,
    parent_source: row.get(10)?,
    common_prefix_messages: nonnegative_count(row.get(11)?),
    request_message_count: nonnegative_count(row.get(12)?),
    response_message_count: nonnegative_count(row.get(13)?),
    is_head: row.get::<_, i64>(14)? != 0,
  })
}

fn select_lineage(conn: &Connection, session_id: &str, node_id: &str) -> Result<Vec<(String, String)>> {
  let mut lineage = Vec::new();
  let mut seen = HashSet::new();
  let mut current = Some(node_id.to_string());

  while let Some(current_id) = current {
    if !seen.insert(current_id.clone()) {
      return Err(crate::Error::InvalidSessionLineage { node_id: current_id });
    }
    let row = conn
      .query_row(
        "SELECT parent_id, reduction_kind FROM session_nodes WHERE session_id = ?1 AND id = ?2",
        params![session_id, current_id],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?)),
      )
      .optional()?;
    let Some((parent_id, reduction_kind)) = row else {
      return Err(crate::Error::InvalidSessionLineage { node_id: current_id });
    };
    lineage.push((current_id, reduction_kind));
    current = parent_id;
  }

  lineage.reverse();
  Ok(lineage)
}

fn active_request_lineage(lineage: &[(String, String)]) -> &[(String, String)] {
  let start = lineage
    .iter()
    .rposition(|(_, reduction_kind)| matches!(reduction_kind.as_str(), "root_snapshot" | "conflict_snapshot"))
    .unwrap_or(0);
  &lineage[start..]
}

#[derive(Debug, Clone, Copy, Default)]
struct SideStats {
  messages: u64,
  parts: u64,
  content_bytes: u64,
}

impl SideStats {
  fn add(&mut self, other: Self) {
    self.messages = self.messages.saturating_add(other.messages);
    self.parts = self.parts.saturating_add(other.parts);
    self.content_bytes = self.content_bytes.saturating_add(other.content_bytes);
  }
}

fn select_lineage_stats(conn: &Connection, lineage: &[(String, String)], side: &str) -> Result<SideStats> {
  let mut stats = SideStats::default();
  for (node_id, _) in lineage {
    stats.add(select_side_stats(conn, node_id, side)?);
  }
  Ok(stats)
}

fn select_side_stats(conn: &Connection, node_id: &str, side: &str) -> Result<SideStats> {
  let messages = conn.query_row(
    "SELECT COUNT(*) FROM node_messages WHERE node_id = ?1 AND side = ?2",
    params![node_id, side],
    |row| row.get::<_, i64>(0),
  )?;
  let (parts, content_bytes) = conn.query_row(
    "SELECT COUNT(*), COALESCE(SUM(length(b.content)), 0)
     FROM node_messages AS m
     JOIN node_parts AS p ON p.message_id = m.id
     JOIN part_blobs AS b ON b.hash = p.part_hash
     WHERE m.node_id = ?1 AND m.side = ?2",
    params![node_id, side],
    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
  )?;
  Ok(SideStats {
    messages: nonnegative_count(messages),
    parts: nonnegative_count(parts),
    content_bytes: nonnegative_count(content_bytes),
  })
}

#[derive(Debug)]
struct MessageRef {
  message_id: String,
  role: String,
  status: Option<u16>,
}

fn select_request_message_tail(
  conn: &Connection,
  lineage: &[(String, String)],
  limit: usize,
) -> Result<Vec<MessageRef>> {
  let mut remaining = limit;
  let mut messages = Vec::new();
  for (node_id, _) in lineage.iter().rev() {
    if remaining == 0 {
      break;
    }
    let mut node_messages = select_node_message_refs(conn, node_id, "request", remaining, true)?;
    remaining = remaining.saturating_sub(node_messages.len());
    messages.append(&mut node_messages);
  }
  messages.reverse();
  Ok(messages)
}

fn select_node_message_refs(
  conn: &Connection,
  node_id: &str,
  side: &str,
  limit: usize,
  descending: bool,
) -> Result<Vec<MessageRef>> {
  let order = if descending { "DESC" } else { "ASC" };
  let sql = format!(
    "SELECT id, role, status
     FROM node_messages
     WHERE node_id = ?1 AND side = ?2
     ORDER BY message_seq {order}, id {order}
     LIMIT ?3"
  );
  let mut stmt = conn.prepare(&sql)?;
  let messages = stmt
    .query_map(params![node_id, side, limit as i64], |row| {
      Ok(MessageRef {
        message_id: row.get(0)?,
        role: row.get(1)?,
        status: sqlite_status(row.get(2)?),
      })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
  Ok(messages)
}

#[derive(Debug)]
struct ContentBudget {
  parts_remaining: usize,
  content_bytes_remaining: usize,
  parts_returned: u64,
  content_bytes_returned: u64,
  content_parts_truncated: u64,
  binary_parts_elided: u64,
}

impl ContentBudget {
  fn new() -> Self {
    Self {
      parts_remaining: MAX_PARTS_PER_SIDE,
      content_bytes_remaining: MAX_INLINE_CONTENT_BYTES_PER_SIDE,
      parts_returned: 0,
      content_bytes_returned: 0,
      content_parts_truncated: 0,
      binary_parts_elided: 0,
    }
  }
}

fn load_messages(
  conn: &Connection,
  message_refs: Vec<MessageRef>,
  budget: &mut ContentBudget,
) -> Result<Vec<SessionMessage>> {
  message_refs
    .into_iter()
    .map(|message_ref| {
      let (parts_total, parts) = load_message_parts(conn, &message_ref.message_id, budget)?;
      Ok(SessionMessage {
        role: message_ref.role,
        status: message_ref.status,
        parts,
        parts_total,
      })
    })
    .collect()
}

fn load_message_parts(
  conn: &Connection,
  message_id: &str,
  budget: &mut ContentBudget,
) -> Result<(u64, Vec<SessionPart>)> {
  let parts_total = conn.query_row(
    "SELECT COUNT(*) FROM node_parts WHERE message_id = ?1",
    params![message_id],
    |row| row.get::<_, i64>(0),
  )?;
  if budget.parts_remaining == 0 {
    return Ok((nonnegative_count(parts_total), Vec::new()));
  }

  let mut stmt = conn.prepare(
    "SELECT b.part_type, length(b.content), substr(b.content, 1, ?2)
     FROM node_parts AS p
     JOIN part_blobs AS b ON b.hash = p.part_hash
     WHERE p.message_id = ?1
     ORDER BY p.part_index
     LIMIT ?3",
  )?;
  let mut rows = stmt.query(params![
    message_id,
    MAX_PART_CONTENT_BYTES as i64,
    budget.parts_remaining as i64
  ])?;
  let mut parts = Vec::new();
  while let Some(row) = rows.next()? {
    let part_type = row.get::<_, String>(0)?;
    let byte_length = nonnegative_count(row.get(1)?);
    let content_prefix = row.get::<_, Vec<u8>>(2)?;
    let decoded = decode_part_content(&part_type, &content_prefix, byte_length, budget.content_bytes_remaining);
    budget.parts_remaining = budget.parts_remaining.saturating_sub(1);
    budget.content_bytes_remaining = budget.content_bytes_remaining.saturating_sub(decoded.returned_bytes);
    budget.parts_returned = budget.parts_returned.saturating_add(1);
    budget.content_bytes_returned = budget
      .content_bytes_returned
      .saturating_add(decoded.returned_bytes as u64);
    if decoded.truncated {
      budget.content_parts_truncated = budget.content_parts_truncated.saturating_add(1);
    }
    if decoded.binary_elided {
      budget.binary_parts_elided = budget.binary_parts_elided.saturating_add(1);
    }
    parts.push(SessionPart {
      part_type,
      byte_length,
      content: decoded.content,
    });
  }
  Ok((nonnegative_count(parts_total), parts))
}

struct DecodedPart {
  content: SessionPartContent,
  returned_bytes: usize,
  truncated: bool,
  binary_elided: bool,
}

fn decode_part_content(
  part_type: &str,
  content_prefix: &[u8],
  byte_length: u64,
  aggregate_bytes_remaining: usize,
) -> DecodedPart {
  if is_explicit_binary_part_type(part_type) {
    return binary_part(byte_length);
  }

  let content_is_complete = byte_length <= content_prefix.len() as u64;
  let utf8 = match std::str::from_utf8(content_prefix) {
    Ok(text) => Some(text),
    Err(error) if !content_is_complete && error.error_len().is_none() => {
      std::str::from_utf8(&content_prefix[..error.valid_up_to()]).ok()
    }
    Err(_) => None,
  };
  let Some(text) = utf8 else {
    return binary_part(byte_length);
  };

  let parsed_json = content_is_complete.then(|| serde_json::from_str(text).ok()).flatten();
  if parsed_json.as_ref().is_some_and(json_contains_embedded_binary) || is_data_url(text) {
    return binary_part(byte_length);
  }

  if part_type.eq_ignore_ascii_case("text") {
    return decode_text_content(content_prefix, byte_length, aggregate_bytes_remaining);
  }

  if let Some(value) = parsed_json {
    if byte_length as usize <= aggregate_bytes_remaining {
      return DecodedPart {
        content: SessionPartContent::Json { value },
        returned_bytes: byte_length as usize,
        truncated: false,
        binary_elided: false,
      };
    }
    return omitted_part(
      SessionPartEncoding::Json,
      byte_length,
      SessionPartOmissionReason::AggregateLimit,
    );
  }

  if is_media_part_type(part_type) {
    return binary_part(byte_length);
  }

  if content_is_complete {
    return decode_text_content(content_prefix, byte_length, aggregate_bytes_remaining);
  }

  let original_encoding = if is_json_part_type(part_type) {
    SessionPartEncoding::Json
  } else {
    SessionPartEncoding::Unknown
  };
  omitted_part(original_encoding, byte_length, SessionPartOmissionReason::PartLimit)
}

fn decode_text_content(content_prefix: &[u8], byte_length: u64, aggregate_bytes_remaining: usize) -> DecodedPart {
  let allowed = content_prefix.len().min(aggregate_bytes_remaining);
  let Some(value) = valid_utf8_prefix(content_prefix, allowed) else {
    return binary_part(byte_length);
  };
  let returned_bytes = value.len();
  if returned_bytes == 0 && byte_length > 0 {
    let reason = if byte_length > MAX_PART_CONTENT_BYTES as u64 {
      SessionPartOmissionReason::PartLimit
    } else {
      SessionPartOmissionReason::AggregateLimit
    };
    return omitted_part(SessionPartEncoding::Text, byte_length, reason);
  }
  let truncated = returned_bytes as u64 != byte_length;
  DecodedPart {
    content: SessionPartContent::Text {
      value: value.to_string(),
      truncated,
    },
    returned_bytes,
    truncated,
    binary_elided: false,
  }
}

fn valid_utf8_prefix(content: &[u8], max_bytes: usize) -> Option<&str> {
  let prefix = &content[..content.len().min(max_bytes)];
  match std::str::from_utf8(prefix) {
    Ok(text) => Some(text),
    Err(error) if error.error_len().is_none() => std::str::from_utf8(&prefix[..error.valid_up_to()]).ok(),
    Err(_) => None,
  }
}

fn is_json_part_type(part_type: &str) -> bool {
  part_type.eq_ignore_ascii_case("json") || part_type.to_ascii_lowercase().ends_with("_json")
}

fn is_explicit_binary_part_type(part_type: &str) -> bool {
  let part_type = part_type.trim().to_ascii_lowercase().replace('-', "_");
  matches!(
    part_type.as_str(),
    "base64" | "binary" | "blob" | "data_url" | "file_data"
  ) || part_type == "application/octet_stream"
}

fn is_media_part_type(part_type: &str) -> bool {
  let part_type = part_type.trim().to_ascii_lowercase().replace('-', "_");
  matches!(
    part_type.as_str(),
    "audio"
      | "audio_url"
      | "computer_screenshot"
      | "document"
      | "file"
      | "image"
      | "image_generation_call"
      | "image_url"
      | "input_audio"
      | "input_file"
      | "input_image"
      | "input_video"
      | "output_audio"
      | "output_file"
      | "output_image"
      | "output_video"
      | "video"
      | "video_url"
  ) || part_type.starts_with("audio/")
    || part_type.starts_with("image/")
    || part_type.starts_with("video/")
    || part_type == "application/pdf"
}

fn is_binary_part_type(part_type: &str) -> bool {
  is_explicit_binary_part_type(part_type) || is_media_part_type(part_type)
}

fn json_contains_embedded_binary(value: &serde_json::Value) -> bool {
  json_contains_embedded_binary_with_context(value, false)
}

fn json_contains_embedded_binary_with_context(value: &serde_json::Value, binary_context: bool) -> bool {
  match value {
    serde_json::Value::String(value) => is_data_url(value),
    serde_json::Value::Array(values) => values
      .iter()
      .any(|value| json_contains_embedded_binary_with_context(value, binary_context)),
    serde_json::Value::Object(object) => {
      let object_is_binary = binary_context
        || object
          .get("type")
          .and_then(serde_json::Value::as_str)
          .is_some_and(|part_type| part_type.eq_ignore_ascii_case("base64") || is_binary_part_type(part_type));

      object.iter().any(|(key, value)| {
        let key = normalized_json_key(key);
        if value.as_str().is_some_and(|value| {
          is_data_url(value)
            || (is_explicit_binary_payload_key(&key) && !value.is_empty())
            || (object_is_binary && key == "data" && !value.is_empty())
        }) {
          return true;
        }

        let child_is_binary = object_is_binary || is_binary_container_key(&key);
        json_contains_embedded_binary_with_context(value, child_is_binary)
      })
    }
    _ => false,
  }
}

fn normalized_json_key(key: &str) -> String {
  key
    .chars()
    .filter(|character| character.is_ascii_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

fn is_explicit_binary_payload_key(key: &str) -> bool {
  matches!(
    key,
    "b64json"
      | "base64"
      | "binary"
      | "binarydata"
      | "blob"
      | "blobdata"
      | "encryptedcontent"
      | "filedata"
      | "inlinedata"
      | "rawbytes"
  ) || key.ends_with("base64")
    || key.ends_with("b64")
}

fn is_binary_container_key(key: &str) -> bool {
  matches!(
    key,
    "audio"
      | "audiourl"
      | "document"
      | "file"
      | "image"
      | "imageurl"
      | "inlinedata"
      | "inputaudio"
      | "inputfile"
      | "inputimage"
      | "inputvideo"
      | "outputaudio"
      | "outputfile"
      | "outputimage"
      | "outputvideo"
      | "source"
      | "video"
      | "videourl"
  )
}

fn is_data_url(value: &str) -> bool {
  let value = value.trim();
  value
    .get(..5)
    .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
    && value.get(5..).is_some_and(|payload| payload.contains(','))
}

fn binary_part(byte_length: u64) -> DecodedPart {
  DecodedPart {
    content: SessionPartContent::Binary { byte_length },
    returned_bytes: 0,
    truncated: false,
    binary_elided: true,
  }
}

fn omitted_part(
  original_encoding: SessionPartEncoding,
  byte_length: u64,
  reason: SessionPartOmissionReason,
) -> DecodedPart {
  DecodedPart {
    content: SessionPartContent::Omitted {
      original_encoding,
      reason,
    },
    returned_bytes: 0,
    truncated: byte_length > 0,
    binary_elided: matches!(original_encoding, SessionPartEncoding::Binary),
  }
}

fn nonnegative_count(value: i64) -> u64 {
  value.max(0) as u64
}

#[cfg(test)]
mod tests {
  use super::{read_transaction, Connection};

  #[test]
  fn read_transaction_keeps_one_snapshot_across_dependent_queries() {
    let path = std::env::temp_dir().join(format!("tokn-viewer-snapshot-{}.db", uuid::Uuid::new_v4()));
    let writer = Connection::open(&path).unwrap();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    writer
      .execute_batch("CREATE TABLE values_table (value INTEGER NOT NULL); INSERT INTO values_table VALUES (1);")
      .unwrap();
    let mut reader = super::super::super::database::open_readonly(&path).unwrap().unwrap();

    read_transaction(&mut reader, |snapshot| {
      let before: i64 = snapshot
        .query_row("SELECT COUNT(*) FROM values_table", [], |row| row.get(0))
        .unwrap();
      assert_eq!(before, 1);

      writer.execute("INSERT INTO values_table VALUES (2)", []).unwrap();

      let during: i64 = snapshot
        .query_row("SELECT COUNT(*) FROM values_table", [], |row| row.get(0))
        .unwrap();
      assert_eq!(during, 1);
      Ok(())
    })
    .unwrap();

    let after: i64 = reader
      .query_row("SELECT COUNT(*) FROM values_table", [], |row| row.get(0))
      .unwrap();
    assert_eq!(after, 2);
  }
}
