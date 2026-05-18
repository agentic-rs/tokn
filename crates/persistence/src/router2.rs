//! Router2 stage-event persistence.
//!
//! Subscribes to `Event::Router2(Router2Event { request_id, attempt, payload })`
//! and writes one row per `(request_id, attempt)` into the same per-day
//! `requests/<YYYY-MM-DD>.db` files used by the legacy lifecycle writer.
//!
//! Unlike the legacy `DbEventHandler`, this handler **does not buffer** a
//! full record in memory and flush at the end. Instead it follows the
//! incremental pattern from `RequestsDb::started`/`headers`/`parsed`/...:
//! `on_started` performs the only `INSERT`, and every subsequent stage
//! handler runs an `UPDATE … WHERE request_id = ?`. If the row is missing
//! (e.g. `Started` was lost) the update simply warns and drops the event.
//!
//! All SQL lives in this file — we do **not** call into the legacy
//! `RequestsDb` lifecycle methods. The only shared bits are pure helpers:
//! `open_day_db` (opens + migrates a day file) and `headers_json`
//! (serialises a `HeaderMap` to JSON with redaction).

use crate::requests::open_day_db;
use crate::{headers_json, Result};
use llm_core::event::{Event, EventHandler};
use llm_core::router2_event::{Router2EventPayload, Stage, StageEvent};
use rusqlite::{params, Connection};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use time::macros::format_description;

const CACHE_CAP: usize = 3;

/// Compose a row-level `request_id` from the base id and attempt number.
/// Mirrors the legacy convention: attempt 0 keeps the bare id; retries
/// append `:N`.
fn composite_request_id(request_id: &str, attempt: u32) -> String {
  if attempt == 0 {
    request_id.to_string()
  } else {
    format!("{request_id}:{attempt}")
  }
}

fn day_key(ts: i64) -> String {
  let dt = time::OffsetDateTime::from_unix_timestamp(ts).unwrap_or_else(|_| time::OffsetDateTime::now_utc());
  dt.date()
    .format(format_description!("[year]-[month]-[day]"))
    .unwrap_or_else(|_| "1970-01-01".to_string())
}

fn now_unix() -> i64 {
  time::OffsetDateTime::now_utc().unix_timestamp()
}

/// Per-day connection cache (LRU, cap 3) used by [`Router2EventHandler`].
/// Independent from the legacy `RequestsDb` cache so the two handlers do
/// not contend on a shared `Mutex`.
pub struct Router2RequestsWriter {
  dir: PathBuf,
  conns: HashMap<String, Connection>,
  order: VecDeque<String>,
  /// Tracks which day a given composite `request_id` was inserted under, so
  /// subsequent UPDATEs route to the same day file even if events span a
  /// midnight boundary.
  request_day: HashMap<String, String>,
}

impl Router2RequestsWriter {
  pub fn new(dir: PathBuf) -> Result<Self> {
    std::fs::create_dir_all(&dir)?;
    Ok(Self {
      dir,
      conns: HashMap::new(),
      order: VecDeque::new(),
      request_day: HashMap::new(),
    })
  }

  fn conn_for_day(&mut self, key: &str) -> Result<&mut Connection> {
    if !self.conns.contains_key(key) {
      if self.order.len() >= CACHE_CAP {
        if let Some(old) = self.order.pop_front() {
          self.conns.remove(&old);
        }
      }
      let conn = open_day_db(&self.dir.join(format!("{key}.db")))?;
      self.conns.insert(key.to_string(), conn);
    }
    self.order.retain(|k| k != key);
    self.order.push_back(key.to_string());
    Ok(self.conns.get_mut(key).expect("opened router2 requests db"))
  }

  fn conn_for_request(&mut self, request_id: &str) -> Option<&mut Connection> {
    let key = self.request_day.get(request_id).cloned()?;
    self.conn_for_day(&key).ok()
  }

  /// Single INSERT for a fresh request. All subsequent stage handlers are
  /// UPDATEs that assume this row exists.
  pub fn on_started(&mut self, request_id: &str, attempt: u32, ts: i64, endpoint: &str) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let key = day_key(ts);
    let conn = self.conn_for_day(&key)?;
    conn.execute(
      "INSERT INTO requests (request_id, ts, endpoint, account_id, provider_id, model, initiator)
       VALUES (?1, ?2, ?3, '', '', '', '')",
      params![id, ts, endpoint],
    )?;
    self.request_day.insert(id, key);
    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub fn on_extract(
    &mut self,
    request_id: &str,
    attempt: u32,
    model: &str,
    stream: bool,
    session_id: Option<&str>,
    initiator: &str,
    inbound_req_headers: &llm_headers::HeaderMap,
    inbound_req_body: &bytes::Bytes,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let hdr_json = headers_json(inbound_req_headers);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 Extract without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET
         model = ?2,
         stream = ?3,
         session_id = COALESCE(?4, session_id),
         initiator = ?5,
         inbound_req_headers = ?6,
         inbound_req_body = ?7
       WHERE request_id = ?1",
      params![id, model, stream as i64, session_id, initiator, hdr_json.as_ref(), inbound_req_body.as_ref()],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 Extract UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_resolve(
    &mut self,
    request_id: &str,
    attempt: u32,
    account_id: &str,
    provider_id: &str,
    upstream_endpoint: &str,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 Resolve without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET
         account_id = ?2,
         provider_id = ?3,
         endpoint = ?4
       WHERE request_id = ?1",
      params![id, account_id, provider_id, upstream_endpoint],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 Resolve UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_build_headers(
    &mut self,
    request_id: &str,
    attempt: u32,
    outbound_req_headers: &llm_headers::HeaderMap,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let hdr_json = headers_json(outbound_req_headers);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 BuildHeaders without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET outbound_req_headers = ?2 WHERE request_id = ?1",
      params![id, hdr_json.as_ref()],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 BuildHeaders UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_convert_request(
    &mut self,
    request_id: &str,
    attempt: u32,
    outbound_req_body: &bytes::Bytes,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 ConvertRequest without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET outbound_req_body = ?2 WHERE request_id = ?1",
      params![id, outbound_req_body.as_ref()],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 ConvertRequest UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_send(
    &mut self,
    request_id: &str,
    attempt: u32,
    ts_now: i64,
    status: u16,
    outbound_resp_headers: &llm_headers::HeaderMap,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let hdr_json = headers_json(outbound_resp_headers);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 Send without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET
         outbound_resp_status = ?2,
         outbound_resp_headers = ?3,
         status = ?2,
         latency_header_ms = (?4 - ts) * 1000
       WHERE request_id = ?1",
      params![id, status as i64, hdr_json.as_ref(), ts_now],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 Send UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_convert_response(
    &mut self,
    request_id: &str,
    attempt: u32,
    status: u16,
    inbound_resp_headers: &llm_headers::HeaderMap,
    inbound_resp_body: &bytes::Bytes,
  ) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let hdr_json = headers_json(inbound_resp_headers);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 ConvertResponse without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET
         inbound_resp_status = ?2,
         inbound_resp_headers = ?3,
         inbound_resp_body = ?4
       WHERE request_id = ?1",
      params![id, status as i64, hdr_json.as_ref(), inbound_resp_body.as_ref()],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 ConvertResponse UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_error(&mut self, request_id: &str, attempt: u32, stage: Stage, message: &str) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let formatted = format!("{}: {message}", stage.as_str());
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 Error without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET request_error = ?2 WHERE request_id = ?1",
      params![id, formatted],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 Error UPDATE matched no row");
    }
    Ok(())
  }

  pub fn on_completed(&mut self, request_id: &str, attempt: u32, ts_now: i64) -> Result<()> {
    let id = composite_request_id(request_id, attempt);
    let Some(conn) = self.conn_for_request(&id) else {
      tracing::warn!(request_id = %id, "router2 Completed without prior Started");
      return Ok(());
    };
    let updated = conn.execute(
      "UPDATE requests SET latency_ms = (?2 - ts) * 1000 WHERE request_id = ?1",
      params![id, ts_now],
    )?;
    if updated == 0 {
      tracing::warn!(request_id = %id, "router2 Completed UPDATE matched no row");
    }
    Ok(())
  }
}

/// `EventHandler` that persists router2 stage events into the requests DB.
/// Construct once and register alongside the legacy `DbEventHandler` —
/// both run in the same `spawn_event_loop` and each maintains its own
/// per-day connection cache.
pub struct Router2EventHandler {
  writer: Router2RequestsWriter,
}

impl Router2EventHandler {
  pub fn new(requests_dir: PathBuf) -> Result<Self> {
    Ok(Self {
      writer: Router2RequestsWriter::new(requests_dir)?,
    })
  }
}

impl EventHandler for Router2EventHandler {
  fn handle(&mut self, event: &Event) {
    let Event::Router2(r2) = event else {
      return;
    };
    let request_id = r2.request_id.as_str();
    let attempt = r2.attempt;
    let stage = match &r2.payload {
      Router2EventPayload::Known(stage) => stage,
      Router2EventPayload::Custom(_) => return,
    };
    let result = match stage {
      StageEvent::Started { endpoint } => self
        .writer
        .on_started(request_id, attempt, now_unix(), endpoint.as_str()),
      StageEvent::Extract(s) => self.writer.on_extract(
        request_id,
        attempt,
        s.model.as_str(),
        s.stream,
        s.session_id.as_deref(),
        s.initiator.as_str(),
        &s.headers,
        &s.raw_body,
      ),
      StageEvent::Resolve(s) => self.writer.on_resolve(
        request_id,
        attempt,
        s.account_id.as_str(),
        s.provider_id.as_str(),
        s.upstream_endpoint.as_str(),
      ),
      StageEvent::BuildHeaders(s) => self.writer.on_build_headers(request_id, attempt, &s.headers),
      StageEvent::ConvertRequest(s) => {
        self
          .writer
          .on_convert_request(request_id, attempt, &s.upstream_wire_body)
      }
      StageEvent::Send(s) => self
        .writer
        .on_send(request_id, attempt, now_unix(), s.status, &s.headers),
      StageEvent::ConvertResponse(s) => {
        let body_bytes = s
          .body
          .as_ref()
          .map(|v| bytes::Bytes::from(serde_json::to_vec(v.as_ref()).unwrap_or_default()))
          .unwrap_or_default();
        self
          .writer
          .on_convert_response(request_id, attempt, s.status, &s.headers, &body_bytes)
      }
      StageEvent::Error { stage, message, .. } => self.writer.on_error(request_id, attempt, *stage, message.as_str()),
      StageEvent::Completed { .. } => self.writer.on_completed(request_id, attempt, now_unix()),
    };
    if let Err(e) = result {
      tracing::warn!(error = %e, request_id, attempt, "router2 persistence write failed");
    }
  }
}
