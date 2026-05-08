pub mod migrate;
pub mod requests;
pub mod sessions;
pub mod usage;

use bytes::Bytes;
use snafu::Snafu;

pub use llm_core::db::{CallRecord, DbPaths, HttpSnapshot, MessageRecord, PartRecord};
pub use usage::UsageDb;

#[cfg(test)]
pub use llm_core::db::SessionSource;

/// Serialise an HTTP header map to JSON bytes, redacting values whose name
/// is sensitive (`authorization`, `proxy-authorization`, `cookie`, anything
/// containing `api-key`). Public so both inbound (server::forward) and
/// outbound (db::requests) capture paths share the same redaction policy.
pub fn headers_json(headers: &reqwest::header::HeaderMap) -> Bytes {
  use serde_json::{Map, Value};
  let mut out = Map::new();
  for (name, value) in headers {
    let key = name.as_str().to_ascii_lowercase();
    let value = if is_sensitive_header(&key) {
      "<redacted>".to_string()
    } else {
      value.to_str().unwrap_or("<non-utf8>").to_string()
    };
    out.insert(key, Value::String(value));
  }
  serde_json::to_vec(&Value::Object(out)).unwrap_or_default().into()
}

pub fn is_sensitive_header(name: &str) -> bool {
  matches!(name, "authorization" | "proxy-authorization" | "cookie") || name.contains("api-key")
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
  #[snafu(display("db io"))]
  Io { source: std::io::Error },

  #[snafu(display("sqlite"))]
  Sqlite { source: rusqlite::Error },

  #[snafu(display("db writer channel closed"))]
  ChannelClosed,
}

impl From<std::io::Error> for Error {
  fn from(source: std::io::Error) -> Self {
    Error::Io { source }
  }
}

impl From<rusqlite::Error> for Error {
  fn from(source: rusqlite::Error) -> Self {
    Error::Sqlite { source }
  }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn write_record(
  usage: &mut usage::UsageDb,
  requests: &mut requests::RequestsDb,
  sessions: &mut Option<sessions::SessionsDb>,
  record: &CallRecord,
) {
  if let Err(e) = usage.record(record) {
    tracing::warn!(error = %e, "failed to write usage db row");
  }
  if let Err(e) = requests.record(record) {
    tracing::warn!(error = %e, "failed to write requests db row");
  }
  if let Some(s) = sessions.as_mut() {
    if let Err(e) = s.record(record) {
      tracing::warn!(error = %e, session_id = %record.session_id, "failed to write sessions db row");
    }
  }
}

// --- Event bus integration ---

use llm_core::event::{Event, EventHandler};

/// Database writer that implements `EventHandler` for use with the event bus.
/// Processes `RequestCompleted` events by writing to usage, requests, and sessions DBs.
pub struct DbEventHandler {
  usage: usage::UsageDb,
  requests: requests::RequestsDb,
  sessions: Option<sessions::SessionsDb>,
}

impl DbEventHandler {
  pub fn new(paths: DbPaths) -> Result<Self> {
    let usage = usage::UsageDb::open(&paths.usage_db)?;
    let requests = requests::RequestsDb::new(paths.requests_dir)?;
    let sessions = match sessions::SessionsDb::open(&paths.sessions_db) {
      Ok(s) => Some(s),
      Err(e) => {
        tracing::error!(error = %e, path = %paths.sessions_db.display(), "sessions.db open failed; continuing without per-message capture");
        None
      }
    };
    Ok(Self { usage, requests, sessions })
  }
}

impl EventHandler for DbEventHandler {
  fn handle(&mut self, event: &Event) {
    match event {
      Event::RequestCompleted { record } => {
        write_record(&mut self.usage, &mut self.requests, &mut self.sessions, record);
      }
      // Other events are ignored by the DB handler
      _ => {}
    }
  }
}
