use crate::config::Config;
use crate::db::{DbEventHandler, DbOptions, DbPaths, DbStore};
use anyhow::Result;
use llm_core::event::{EventBus, EventHandler, EventReceiver};
use std::sync::Arc;

pub fn build_db(cfg: &Config) -> Result<Option<Arc<DbStore>>> {
  if !cfg.db.enabled {
    return Ok(None);
  }
  let paths = cfg.db.resolve_paths()?;
  Ok(Some(Arc::new(DbStore::spawn(DbOptions {
    paths: DbPaths {
      usage_db: paths.usage_db,
      sessions_db: paths.sessions_db,
      requests_dir: paths.requests_dir,
    },
    queue_capacity: cfg.db.write_queue_capacity,
    body_max_bytes: cfg.db.body_max_bytes,
  })?)))
}

/// Build the event bus and its handlers. The DB event handler is included
/// when usage recording is enabled.
pub fn build_event_bus(cfg: &Config) -> Result<(Arc<EventBus>, EventReceiver, Vec<Box<dyn EventHandler>>)> {
  let capacity = cfg.db.write_queue_capacity.max(256);
  let (bus, receiver) = EventBus::new(capacity);
  let mut handlers: Vec<Box<dyn EventHandler>> = Vec::new();

  if cfg.db.enabled {
    let paths = cfg.db.resolve_paths()?;
    let db_handler = DbEventHandler::new(DbPaths {
      usage_db: paths.usage_db,
      sessions_db: paths.sessions_db,
      requests_dir: paths.requests_dir,
    })?;
    handlers.push(Box::new(db_handler));
  }

  Ok((Arc::new(bus), receiver, handlers))
}

pub fn build_state(cfg: &Config, db: &Option<Arc<DbStore>>, events: Arc<EventBus>) -> Result<llm_router::server::AppState> {
  llm_router::server::build_state(cfg, db.clone().map(|db| db as Arc<dyn llm_core::db::DbStore>), events)
}

pub async fn shutdown_db(db: Option<Arc<DbStore>>) -> Result<()> {
  if let Some(db) = db {
    db.shutdown().await?;
  }
  Ok(())
}

pub fn is_loopback(host: &str) -> bool {
  matches!(host, "127.0.0.1" | "::1" | "localhost")
    || host
      .parse::<std::net::IpAddr>()
      .map(|ip| ip.is_loopback())
      .unwrap_or(false)
}

pub fn ensure_bind_host(host: &str, allow_remote: bool) -> Result<()> {
  if !allow_remote && !is_loopback(host) {
    anyhow::bail!("refusing to bind to non-loopback host '{host}' without --allow-remote (no client auth in v1)");
  }
  Ok(())
}
