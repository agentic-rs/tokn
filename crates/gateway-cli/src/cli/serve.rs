use crate::cli::config_cmd::RouteModeArg;
use crate::cli::lan_bootstrap;
use crate::config::Config;
use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tokn_config::{RouteMode, DEFAULT_HOST};
use tokn_core::event::EventBus;

#[derive(Args, Debug)]
pub struct ServeArgs {
  #[arg(long)]
  pub host: Option<String>,
  #[arg(long)]
  pub port: Option<u16>,
  /// Also run the local MITM proxy in the same process.
  #[arg(long)]
  pub with_proxy: bool,
  /// Override the proxy listener's default route mode when `--with-proxy` is enabled.
  #[arg(long, value_enum, requires = "with_proxy")]
  pub proxy_route_mode: Option<RouteModeArg>,
  /// Allow binding to non-loopback addresses (insecure: there is no client auth in v1).
  #[arg(long)]
  pub insecure_allow_remote: bool,
  /// Skip outbound proxy for this run.
  #[arg(long)]
  pub no_proxy: bool,
}

pub async fn run(cfg_path: Option<PathBuf>, args: ServeArgs) -> Result<()> {
  let (mut cfg, resolved_cfg_path) = Config::load(cfg_path.as_deref())?;
  if args.no_proxy {
    cfg.proxy = crate::config::ProxyConfig::default();
  }
  let accounts = crate::server_runtime::load_accounts(Some(&resolved_cfg_path))?;

  let host = args.host.unwrap_or_else(|| cfg.server.host.clone());
  let port = args.port.unwrap_or(cfg.server.port);
  let addr = crate::server_runtime::resolve_bind_addr(&host, port, args.insecure_allow_remote)
    .with_context(|| format!("parse bind addr {host}:{port}"))?;

  let (events, receiver, handlers, archive_runtime) = crate::server_runtime::build_event_bus(&cfg)?;
  let _event_thread = tokn_core::event::spawn_event_loop(receiver, handlers);
  let server_mode = effective_server_mode(&cfg);
  let proxy_route_override = args.proxy_route_mode.map(Into::into);
  let proxy_mode = proxy_route_override.unwrap_or(cfg.proxy_mode.route_mode);
  let app_state = crate::server_runtime::build_state_for_route_mode(&cfg, &accounts, events.clone(), server_mode)?;
  let n = app_state.pool.len();
  let app_live = tokn_router::api::LiveAppState::new(app_state);
  let proxy_live = if args.with_proxy {
    Some(tokn_router::api::LiveAppState::new(
      crate::server_runtime::build_proxy_state_for_route_mode(&cfg, &accounts, events.clone(), proxy_mode)?,
    ))
  } else {
    None
  };
  if !args.insecure_allow_remote {
    install_reload_endpoint(
      &app_live,
      proxy_live.clone(),
      resolved_cfg_path.clone(),
      args.no_proxy,
      args.with_proxy,
      proxy_route_override,
      events.clone(),
    );
  }
  let mut app = tokn_router::api::router_live(app_live);

  tracing::info!(%addr, accounts = n, route_mode = route_mode_name(server_mode), "tokn-router listening");

  let result = if args.with_proxy {
    let proxy_host = proxy_host_for_with_proxy(&host, &cfg.proxy_mode.host, args.insecure_allow_remote);
    let proxy_port = cfg.proxy_mode.port;
    let proxy_addr = crate::server_runtime::resolve_bind_addr(&proxy_host, proxy_port, args.insecure_allow_remote)
      .with_context(|| format!("parse bind addr {proxy_host}:{proxy_port}"))?;
    let ca_dir = cfg.proxy_mode.resolved_ca_dir()?;
    let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, false)?;
    let ca_fingerprint = ca.fingerprint_sha256();
    let bootstrap = if args.insecure_allow_remote {
      Some(lan_bootstrap::BootstrapState::new(&ca, port, proxy_port)?)
    } else {
      None
    };
    let plain_http_handler = bootstrap.clone().map(lan_bootstrap::proxy_plain_http_handler);
    if let Some(bootstrap) = bootstrap {
      app = app.merge(lan_bootstrap::router(bootstrap));
      println!("LAN bootstrap: {}", lan_bootstrap::display_bootstrap_url(&host, port));
      println!(
        "LAN proxy bootstrap: {}",
        lan_bootstrap::display_bootstrap_url(&proxy_host, proxy_port)
      );
      println!("LAN bootstrap CA sha256: {ca_fingerprint}");
    }
    println!("tokn-router proxy listening on http://{proxy_addr}");
    println!("CA: {} (sha256:{ca_fingerprint})", ca.cert_path().display());
    println!("Proxy route mode: {}", route_mode_name(proxy_mode));

    let proxy_state = proxy_live.expect("proxy live state is constructed when --with-proxy is set");
    let proxy_options = tokn_router::proxy::ProxyOptions {
      addr: proxy_addr,
      ca_dir,
      intercept_hosts: cfg.proxy_mode.intercept_hosts.clone(),
      passthrough_hosts: cfg.proxy_mode.passthrough_hosts.clone(),
      outbound_proxy: cfg.proxy.to_http_options(),
      plain_http_handler,
    };
    let shutdown = shutdown_channel();
    tokio::try_join!(
      crate::server_runtime::serve_http(app, addr, wait_for_shutdown(shutdown.clone())),
      tokn_router::proxy::serve_live(proxy_state, proxy_options, wait_for_shutdown(shutdown)),
    )
    .map(|_| ())
  } else {
    crate::server_runtime::serve_http(app, addr, async {
      let _ = tokio::signal::ctrl_c().await;
    })
    .await
  };

  if let Some(archive_runtime) = archive_runtime {
    archive_runtime.shutdown().await;
  }
  events.shutdown().await;
  result
}

struct ReloadState {
  generation: u64,
}

#[allow(clippy::too_many_arguments)]
fn install_reload_endpoint(
  app_live: &tokn_router::api::LiveAppState,
  proxy_live: Option<tokn_router::api::LiveAppState>,
  config_path: PathBuf,
  no_proxy: bool,
  with_proxy: bool,
  proxy_route_override: Option<RouteMode>,
  events: Arc<EventBus>,
) {
  let lock = Arc::new(Mutex::new(ReloadState { generation: 0 }));
  let app_live_for_reload = app_live.clone();
  let reloader = tokn_router::api::AdminReloader::new(move || {
    let app_live = app_live_for_reload.clone();
    let proxy_live = proxy_live.clone();
    let config_path = config_path.clone();
    let events = events.clone();
    let lock = lock.clone();
    async move {
      let mut guard = lock.lock().await;
      let (mut cfg, resolved_cfg_path) = Config::load(Some(&config_path)).map_err(|e| e.to_string())?;
      if no_proxy {
        cfg.proxy = crate::config::ProxyConfig::default();
      }
      let accounts = crate::server_runtime::load_accounts(Some(&resolved_cfg_path)).map_err(|e| e.to_string())?;
      let server_mode = effective_server_mode(&cfg);
      let proxy_mode = proxy_route_override.unwrap_or(cfg.proxy_mode.route_mode);
      let app_state = crate::server_runtime::build_state_for_route_mode(&cfg, &accounts, events.clone(), server_mode)
        .map_err(|e| e.to_string())?;
      let proxy_state = if with_proxy {
        Some(
          crate::server_runtime::build_proxy_state_for_route_mode(&cfg, &accounts, events.clone(), proxy_mode)
            .map_err(|e| e.to_string())?,
        )
      } else {
        None
      };

      app_live.swap(app_state);
      if let (Some(live), Some(state)) = (proxy_live, proxy_state) {
        live.swap(state);
      }
      guard.generation = guard.generation.saturating_add(1);
      tracing::info!(
        generation = guard.generation,
        accounts = accounts.len(),
        route_mode = route_mode_name(server_mode),
        "config reloaded"
      );
      Ok(tokn_router::api::ReloadReport {
        status: "reloaded",
        generation: guard.generation,
        accounts: accounts.len(),
        route_mode: route_mode_name(server_mode),
      })
    }
  });
  if app_live.set_admin_reloader(reloader).is_err() {
    tracing::warn!("admin config reload endpoint was already configured");
  }
}

fn effective_server_mode(cfg: &Config) -> RouteMode {
  if cfg.defaults.mode == RouteMode::Route && cfg.server.route_mode != RouteMode::Route {
    cfg.server.route_mode
  } else {
    cfg.defaults.mode
  }
}

#[cfg(test)]
fn shared_route_mode(server_mode: RouteMode, proxy_mode: RouteMode, with_proxy: bool) -> RouteMode {
  if !with_proxy || server_mode != RouteMode::Passthrough {
    server_mode
  } else {
    proxy_mode
  }
}

fn proxy_host_for_with_proxy(server_host: &str, configured_proxy_host: &str, insecure_allow_remote: bool) -> String {
  if insecure_allow_remote && configured_proxy_host == DEFAULT_HOST {
    server_host.to_string()
  } else {
    configured_proxy_host.to_string()
  }
}

fn shutdown_channel() -> watch::Receiver<bool> {
  let (tx, rx) = watch::channel(false);
  tokio::spawn(async move {
    let _ = tokio::signal::ctrl_c().await;
    let _ = tx.send(true);
  });
  rx
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
  if *shutdown.borrow() {
    return;
  }
  let _ = shutdown.changed().await;
}

fn route_mode_name(mode: RouteMode) -> &'static str {
  match mode {
    RouteMode::Passthrough => "passthrough",
    RouteMode::Switch => "switch",
    RouteMode::Exact => "exact",
    RouteMode::Route => "route",
    RouteMode::Fuzzy => "fuzzy",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn shared_mode_prefers_non_passthrough_listener_when_needed() {
    assert_eq!(
      shared_route_mode(RouteMode::Passthrough, RouteMode::Exact, true),
      RouteMode::Exact
    );
    assert_eq!(
      shared_route_mode(RouteMode::Route, RouteMode::Passthrough, true),
      RouteMode::Route
    );
    assert_eq!(
      shared_route_mode(RouteMode::Passthrough, RouteMode::Passthrough, true),
      RouteMode::Passthrough
    );
  }

  #[test]
  fn lan_mode_proxy_host_follows_server_host_when_proxy_host_is_default() {
    assert_eq!(proxy_host_for_with_proxy("0.0.0.0", DEFAULT_HOST, true), "0.0.0.0");
  }

  #[test]
  fn lan_mode_proxy_host_preserves_explicit_proxy_host() {
    assert_eq!(
      proxy_host_for_with_proxy("0.0.0.0", "192.168.1.22", true),
      "192.168.1.22"
    );
  }

  #[test]
  fn local_mode_proxy_host_keeps_default_loopback() {
    assert_eq!(proxy_host_for_with_proxy("0.0.0.0", DEFAULT_HOST, false), DEFAULT_HOST);
  }
}
