use crate::config::Config;
use crate::db::archive::{ArchiveEventHandler, ArchiveRuntime};
use crate::progress::{ArchiveProgressEventHandler, ProgressEventHandler, ProgressLogEventHandler};
use anyhow::Result;
use axum::Router;
use std::future::Future;
use std::io::IsTerminal;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokn_auth::AuthStore;
use tokn_config::RouteMode;
use tokn_core::account::AccountConfig;
use tokn_core::event::{EventBus, EventHandler};

type EventBusParts = (
  Arc<EventBus>,
  broadcast::Receiver<Arc<tokn_core::event::Event>>,
  Vec<Box<dyn EventHandler>>,
  Option<ArchiveRuntime>,
);

/// Build the event bus and its handlers. The requests-event persistence
/// handler is included when usage recording is enabled. A TTY progress handler is attached
/// automatically when stdout is a terminal.
pub fn build_event_bus(cfg: &Config) -> Result<EventBusParts> {
  let capacity = cfg.db.write_queue_capacity.max(256);
  let bus = EventBus::new(capacity);
  let receiver = bus.subscribe();
  let mut handlers: Vec<Box<dyn EventHandler>> = Vec::new();
  let mut archive_handlers: Vec<Box<dyn ArchiveEventHandler>> = Vec::new();
  let tty_progress = std::io::stdout().is_terminal();

  if cfg.db.enabled {
    let paths = cfg.db.resolve_paths()?;
    let request_handler = tokn_persistence::RequestEventHandler::new(paths.requests_dir)?;
    let usage_handler = tokn_persistence::UsageEventHandler::new(paths.usage_db)?;
    handlers.push(Box::new(request_handler));
    handlers.push(Box::new(usage_handler));
  }

  match crate::logging::resolve_logs_dir(&cfg.logging) {
    Ok(dir) => match ProgressLogEventHandler::new(&dir) {
      Ok(handler) => handlers.push(Box::new(handler)),
      Err(e) => tracing::warn!(path = %dir.display(), error = %e, "progress log disabled"),
    },
    Err(e) => tracing::warn!(error = %e, "progress log disabled"),
  }

  if tty_progress {
    handlers.push(Box::new(ProgressEventHandler::new()));
    archive_handlers.push(Box::new(ArchiveProgressEventHandler::new()));
  }

  let archive_runtime = if cfg.db.enabled {
    let paths = cfg.db.resolve_paths()?;
    crate::db::archive::start_request_archive_worker(
      paths.requests_dir,
      cfg.db.archive_extension.as_deref(),
      archive_handlers,
    )
  } else {
    None
  };

  Ok((Arc::new(bus), receiver, handlers, archive_runtime))
}

/// Load accounts from the root `auth.yaml` and any `auth.d` fragments.
///
/// `config_path` is accepted for compatibility with call sites that already
/// have the effective config path; legacy schema migration runs before latest
/// config/auth loading.
pub fn load_accounts(config_path: Option<&Path>) -> Result<Vec<AccountConfig>> {
  let store = AuthStore::load(None, config_path)?;
  Ok(store.accounts)
}

pub fn build_state(
  cfg: &Config,
  accounts: &[AccountConfig],
  events: Arc<EventBus>,
) -> Result<tokn_router::api::AppState> {
  tokn_router::api::build_state(cfg, accounts, events)
}

pub fn build_state_for_route_mode(
  cfg: &Config,
  accounts: &[AccountConfig],
  events: Arc<EventBus>,
  route_mode: RouteMode,
) -> Result<tokn_router::api::AppState> {
  let mut cfg = cfg.clone();
  cfg.server.route_mode = route_mode;
  cfg.defaults.mode = route_mode;
  build_state(&cfg, accounts, events)
}

pub fn build_proxy_state_for_route_mode(
  cfg: &Config,
  accounts: &[AccountConfig],
  events: Arc<EventBus>,
  route_mode: RouteMode,
) -> Result<tokn_router::api::AppState> {
  let mut cfg = cfg.clone();
  cfg.server.route_mode = route_mode;
  cfg.defaults.mode = route_mode;
  tokn_router::api::build_proxy_state(&cfg, accounts, events)
}

pub fn resolve_bind_addr(host: &str, port: u16, insecure_allow_remote: bool) -> Result<SocketAddr> {
  ensure_bind_host(host, insecure_allow_remote)?;
  Ok(format!("{host}:{port}").parse()?)
}

pub async fn serve_http<F>(app: Router, addr: SocketAddr, shutdown: F) -> Result<()>
where
  F: Future<Output = ()> + Send + 'static,
{
  let listener = tokio::net::TcpListener::bind(addr).await?;
  tracing::info!(%addr, "tokn-router listening");
  axum::serve(listener, app).with_graceful_shutdown(shutdown).await?;
  Ok(())
}

pub fn is_loopback(host: &str) -> bool {
  matches!(host, "127.0.0.1" | "::1" | "localhost")
    || host
      .parse::<std::net::IpAddr>()
      .map(|ip| ip.is_loopback())
      .unwrap_or(false)
}

pub fn ensure_bind_host(host: &str, insecure_allow_remote: bool) -> Result<()> {
  if !insecure_allow_remote && !is_loopback(host) {
    anyhow::bail!(
      "refusing to bind to non-loopback host '{host}' without --insecure-allow-remote (no client auth in v1)"
    );
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_non_loopback_without_insecure_allow_remote() {
    let err = ensure_bind_host("0.0.0.0", false).expect_err("remote bind should be rejected");
    assert!(
      err.to_string().contains("--insecure-allow-remote"),
      "unexpected error: {err}"
    );
  }

  #[test]
  fn accepts_non_loopback_with_insecure_allow_remote() {
    ensure_bind_host("0.0.0.0", true).expect("remote bind should be allowed");
  }

  #[test]
  fn accepts_loopback_without_insecure_allow_remote() {
    ensure_bind_host("127.0.0.1", false).expect("loopback bind should be allowed");
  }
}
