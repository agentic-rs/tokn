use crate::cli::config_cmd::RouteModeArg;
use crate::cli::lan_bootstrap;
use crate::config::{Config, ProxyConfig};
use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use std::collections::HashSet;
use std::net::SocketAddr;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tokn_config::RouteMode;

const DEFAULT_CLIENT_NO_PROXY: &[&str] = &["localhost", "127.0.0.1", "::1"];

#[derive(Args, Debug)]
pub struct ProxyArgs {
  /// Route intercepted requests directly to the original upstream with the
  /// client's own credentials.
  #[arg(long, global = true)]
  pub passthrough: bool,
  #[command(subcommand)]
  pub cmd: Option<ProxyCmd>,
}

#[derive(Subcommand, Debug)]
pub enum ProxyCmd {
  /// Run the local MITM forward proxy
  Start(StartArgs),
  /// Print shell environment exports for proxy + CA trust
  Env(EnvArgs),
  /// Enter a shell with proxy + CA env vars set
  Shell(ShellArgs),
  /// Run Codex with proxy + CA env vars set
  Codex(AgentProxyArgs),
  /// Run opencode with proxy + CA env vars set
  Opencode(AgentProxyArgs),
  /// Run pi with proxy + CA env vars set
  Pi(AgentProxyArgs),
  /// Inspect or regenerate the local proxy CA
  Ca(CaArgs),
}

#[derive(Args, Debug, Default)]
pub struct StartArgs {
  #[arg(long)]
  pub host: Option<String>,
  #[arg(long)]
  pub port: Option<u16>,
  #[arg(long, value_enum)]
  pub route_mode: Option<RouteModeArg>,
  #[arg(long)]
  pub ca_dir: Option<PathBuf>,
  /// Allow binding to non-loopback addresses (insecure: there is no client auth in v1).
  #[arg(long)]
  pub insecure_allow_remote: bool,
  /// Skip outbound proxy for this run.
  #[arg(long)]
  pub no_proxy: bool,
}

#[derive(Args, Debug)]
pub struct EnvArgs {
  #[arg(long, value_enum, default_value_t = Shell::Sh)]
  pub shell: Shell,
}

#[derive(Args, Debug)]
pub struct ShellArgs {
  #[arg(long)]
  pub shell: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct AgentProxyArgs {
  /// Run via npx instead of a local executable.
  #[arg(long)]
  pub npx: bool,
  /// Arguments forwarded to the agent command.
  #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
  pub args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct CaArgs {
  #[command(subcommand)]
  pub cmd: CaCmd,
}

#[derive(Subcommand, Debug)]
pub enum CaCmd {
  /// Print the CA cert path
  Path,
  /// Print CA details
  Show,
  /// Regenerate the CA and overwrite existing files
  Regenerate,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Shell {
  Sh,
  Fish,
  Pwsh,
  Bash,
  Zsh,
}

pub async fn run(cfg_path: Option<PathBuf>, args: ProxyArgs) -> Result<()> {
  let ProxyArgs { passthrough, cmd } = args;
  match cmd.unwrap_or(ProxyCmd::Start(StartArgs::default())) {
    ProxyCmd::Start(args) => start(cfg_path, args, passthrough).await,
    ProxyCmd::Env(args) => env(cfg_path, args).await,
    ProxyCmd::Shell(args) => shell(cfg_path, args).await,
    ProxyCmd::Codex(args) => agent(cfg_path, AgentKind::Codex, args).await,
    ProxyCmd::Opencode(args) => agent(cfg_path, AgentKind::Opencode, args).await,
    ProxyCmd::Pi(args) => agent(cfg_path, AgentKind::Pi, args).await,
    ProxyCmd::Ca(args) => ca(cfg_path, args).await,
  }
}

#[allow(clippy::result_large_err)]
async fn start(cfg_path: Option<PathBuf>, args: StartArgs, passthrough: bool) -> Result<()> {
  if passthrough && args.route_mode.is_some() {
    anyhow::bail!("--passthrough and --route-mode cannot be used together");
  }
  let (mut cfg, resolved_cfg_path) = Config::load(cfg_path.as_deref())?;
  if args.no_proxy {
    cfg.proxy = ProxyConfig::default();
  }
  let accounts = crate::server_runtime::load_accounts(Some(&resolved_cfg_path))?;

  let host = args.host.unwrap_or_else(|| cfg.proxy_mode.host.clone());
  let port = args.port.unwrap_or(cfg.proxy_mode.port);
  let route_mode = args
    .route_mode
    .map(Into::into)
    .or_else(|| passthrough.then_some(RouteMode::Passthrough))
    .unwrap_or(cfg.proxy_mode.route_mode);
  let ca_dir = args
    .ca_dir
    .clone()
    .map(Ok)
    .unwrap_or_else(|| cfg.proxy_mode.resolved_ca_dir())?;

  let (events, receiver, handlers, archive_runtime) = crate::server_runtime::build_event_bus(&cfg)?;
  let _event_thread = tokn_core::event::spawn_event_loop(receiver, handlers);
  let state = crate::server_runtime::build_proxy_state_for_route_mode(&cfg, &accounts, events.clone(), route_mode)?;
  let n = state.pool.len();
  let addr: SocketAddr = crate::server_runtime::resolve_bind_addr(&host, port, args.insecure_allow_remote)
    .with_context(|| format!("parse bind addr {host}:{port}"))?;

  let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, false)?;
  let ca_fingerprint = ca.fingerprint_sha256();
  let plain_http_handler = if args.insecure_allow_remote {
    let bootstrap = lan_bootstrap::BootstrapState::proxy_only(&ca, port)?;
    Some(lan_bootstrap::proxy_plain_http_handler(bootstrap))
  } else {
    None
  };
  println!("tokn-router proxy listening on http://{addr}");
  println!("CA: {} (sha256:{ca_fingerprint})", ca.cert_path().display());
  println!("Trust this CA, then run: eval \"$(tokn-gateway proxy env)\"");
  if args.insecure_allow_remote {
    println!(
      "LAN proxy bootstrap: {}",
      lan_bootstrap::display_bootstrap_url(&host, port)
    );
  }
  println!("Route mode: {}", route_mode_name(route_mode));
  if let Some(url) = &cfg.proxy.url {
    println!("Outbound proxy: {url}");
    if !cfg.proxy.no_proxy.is_empty() {
      println!("Outbound no_proxy: {}", cfg.proxy.no_proxy.join(","));
    }
  } else if cfg.proxy.system {
    println!("Outbound proxy: system");
  }
  println!("Accounts: {n}");

  let options = tokn_router::proxy::ProxyOptions {
    addr,
    ca_dir,
    intercept_hosts: cfg.proxy_mode.intercept_hosts.clone(),
    passthrough_hosts: cfg.proxy_mode.passthrough_hosts.clone(),
    outbound_proxy: cfg.proxy.to_http_options(),
    plain_http_handler,
  };

  let result = tokn_router::proxy::serve(state, options, async {
    let _ = tokio::signal::ctrl_c().await;
  })
  .await;
  if let Some(archive_runtime) = archive_runtime {
    archive_runtime.shutdown().await;
  }
  events.shutdown().await;
  result
}

async fn env(cfg_path: Option<PathBuf>, args: EnvArgs) -> Result<()> {
  let env = resolved_proxy_env(cfg_path.as_deref())?;
  match args.shell {
    Shell::Sh | Shell::Bash | Shell::Zsh => print_sh(&env),
    Shell::Fish => print_fish(&env),
    Shell::Pwsh => print_pwsh(&env),
  }
  Ok(())
}

async fn shell(cfg_path: Option<PathBuf>, args: ShellArgs) -> Result<()> {
  let env = resolved_proxy_env(cfg_path.as_deref())?;
  let shell = detect_shell(args.shell.as_deref())?;
  println!("Entering proxy shell: {}", shell.path.display());
  println!("HTTPS_PROXY={}", env.get("HTTPS_PROXY").unwrap_or(""));
  println!("SSL_CERT_FILE={}", env.get("SSL_CERT_FILE").unwrap_or(""));
  println!("Type 'exit' to leave this shell.");
  let mut cmd = Command::new(&shell.path);
  cmd.envs(env.vars.iter().map(|(k, v)| (k.as_str(), v.as_str())));
  apply_shell_arg0(&mut cmd, shell.arg0.as_deref());
  let status = cmd
    .status()
    .with_context(|| format!("launch shell {}", shell.path.display()))?;
  if !status.success() {
    anyhow::bail!("shell exited with status {status}");
  }
  Ok(())
}

async fn agent(cfg_path: Option<PathBuf>, kind: AgentKind, args: AgentProxyArgs) -> Result<()> {
  let env = resolved_proxy_env(cfg_path.as_deref())?;
  let spec = agent_command_spec(kind, args.npx, args.args);
  println!("Running {} with proxy env: {}", kind.name(), spec.display());
  println!("HTTPS_PROXY={}", env.get("HTTPS_PROXY").unwrap_or(""));
  println!("SSL_CERT_FILE={}", env.get("SSL_CERT_FILE").unwrap_or(""));

  let mut cmd = Command::new(&spec.program);
  cmd.args(&spec.args);
  cmd.envs(env.vars.iter().map(|(k, v)| (k.as_str(), v.as_str())));
  let status = cmd.status().with_context(|| format!("launch {}", spec.display()))?;
  if !status.success() {
    anyhow::bail!("{} exited with status {status}", kind.name());
  }
  Ok(())
}

async fn ca(cfg_path: Option<PathBuf>, args: CaArgs) -> Result<()> {
  let (cfg, _) = Config::load(cfg_path.as_deref())?;
  let ca_dir = cfg.proxy_mode.resolved_ca_dir()?;
  match args.cmd {
    CaCmd::Path => {
      let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, false)?;
      println!("{}", ca.cert_path().display());
    }
    CaCmd::Show => {
      let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, false)?;
      println!("cert: {}", ca.cert_path().display());
      println!("bundle: {}", ca.ensure_bundle()?.display());
      println!("key: {}", ca.key_path().display());
      println!("sha256: {}", ca.fingerprint_sha256());
    }
    CaCmd::Regenerate => {
      let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, true)?;
      println!("regenerated CA at {}", ca.cert_path().display());
      println!("sha256: {}", ca.fingerprint_sha256());
    }
  }
  Ok(())
}

fn print_sh(env: &ProxyEnv) {
  for (key, value) in &env.vars {
    println!("export {key}={value}");
  }
}

fn print_fish(env: &ProxyEnv) {
  for (key, value) in &env.vars {
    println!("set -gx {key} {value}");
  }
}

fn print_pwsh(env: &ProxyEnv) {
  for (key, value) in &env.vars {
    println!("$Env:{key} = '{value}'");
  }
}

fn resolved_proxy_env(cfg_path: Option<&Path>) -> Result<ProxyEnv> {
  let (cfg, _) = Config::load(cfg_path)?;
  let ca_dir = cfg.proxy_mode.resolved_ca_dir()?;
  let ca = tokn_router::proxy::load_or_generate_ca(&ca_dir, false)?;
  let proxy_url = format!("http://{}:{}", cfg.proxy_mode.host, cfg.proxy_mode.port);
  let cert = ca.cert_path().display().to_string();
  let bundle = ca.ensure_bundle()?.display().to_string();
  let no_proxy = client_no_proxy_value(&cfg.proxy.no_proxy);
  Ok(ProxyEnv {
    vars: vec![
      ("HTTPS_PROXY".into(), proxy_url.clone()),
      ("HTTP_PROXY".into(), proxy_url),
      ("NO_PROXY".into(), no_proxy),
      ("SSL_CERT_FILE".into(), bundle.clone()),
      ("NODE_EXTRA_CA_CERTS".into(), cert),
      ("CODEX_CA_CERTIFICATE".into(), bundle.clone()),
      ("REQUESTS_CA_BUNDLE".into(), bundle.clone()),
      ("CURL_CA_BUNDLE".into(), bundle.clone()),
      ("GIT_SSL_CAINFO".into(), bundle),
    ],
  })
}

fn client_no_proxy_value(configured: &[String]) -> String {
  let mut seen = HashSet::new();
  DEFAULT_CLIENT_NO_PROXY
    .iter()
    .copied()
    .map(str::to_string)
    .chain(configured.iter().map(|entry| entry.trim().to_string()))
    .filter(|entry| !entry.is_empty())
    .filter(|entry| seen.insert(entry.clone()))
    .collect::<Vec<_>>()
    .join(",")
}

#[derive(Debug)]
struct ProxyEnv {
  vars: Vec<(String, String)>,
}

impl ProxyEnv {
  fn get(&self, key: &str) -> Option<&str> {
    self.vars.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
  }
}

#[derive(Copy, Clone, Debug)]
enum AgentKind {
  Codex,
  Opencode,
  Pi,
}

impl AgentKind {
  fn name(self) -> &'static str {
    match self {
      Self::Codex => "codex",
      Self::Opencode => "opencode",
      Self::Pi => "pi",
    }
  }

  fn npx_package(self) -> &'static str {
    match self {
      Self::Codex => "@openai/codex",
      Self::Opencode => "opencode-ai",
      Self::Pi => "@earendil-works/pi-coding-agent",
    }
  }
}

#[derive(Debug, Eq, PartialEq)]
struct CommandSpec {
  program: String,
  args: Vec<String>,
}

impl CommandSpec {
  fn display(&self) -> String {
    std::iter::once(self.program.as_str())
      .chain(self.args.iter().map(String::as_str))
      .collect::<Vec<_>>()
      .join(" ")
  }
}

fn agent_command_spec(kind: AgentKind, npx: bool, forwarded_args: Vec<String>) -> CommandSpec {
  if npx {
    CommandSpec {
      program: "npx".into(),
      args: ["-y".into(), kind.npx_package().into()]
        .into_iter()
        .chain(forwarded_args)
        .collect(),
    }
  } else {
    CommandSpec {
      program: kind.name().into(),
      args: forwarded_args,
    }
  }
}

#[derive(Debug)]
struct ShellExec {
  path: PathBuf,
  arg0: Option<String>,
}

fn detect_shell(explicit: Option<&Path>) -> Result<ShellExec> {
  if let Some(path) = explicit {
    return Ok(ShellExec {
      path: path.to_path_buf(),
      arg0: shell_arg0(path),
    });
  }

  if let Some(shell) = std::env::var_os("SHELL") {
    let path = PathBuf::from(shell);
    return Ok(ShellExec {
      arg0: shell_arg0(&path),
      path,
    });
  }

  if let Some(comspec) = std::env::var_os("COMSPEC") {
    let path = PathBuf::from(comspec);
    return Ok(ShellExec {
      arg0: shell_arg0(&path),
      path,
    });
  }

  #[cfg(windows)]
  let path = PathBuf::from("cmd.exe");
  #[cfg(not(windows))]
  let path = PathBuf::from("/bin/sh");
  Ok(ShellExec {
    arg0: shell_arg0(&path),
    path,
  })
}

fn shell_arg0(path: &Path) -> Option<String> {
  path.file_name().and_then(|name| name.to_str()).map(|s| s.to_string())
}

#[cfg(unix)]
fn apply_shell_arg0(cmd: &mut Command, arg0: Option<&str>) {
  if let Some(arg0) = arg0 {
    cmd.arg0(arg0);
  }
}

#[cfg(not(unix))]
fn apply_shell_arg0(_cmd: &mut Command, _arg0: Option<&str>) {}

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
  fn client_no_proxy_includes_configured_entries() {
    let configured = vec!["internal.local".into(), "10.0.0.0/8".into()];

    assert_eq!(
      client_no_proxy_value(&configured),
      "localhost,127.0.0.1,::1,internal.local,10.0.0.0/8"
    );
  }

  #[test]
  fn client_no_proxy_deduplicates_defaults_and_skips_empty_entries() {
    let configured = vec![
      "localhost".into(),
      " ".into(),
      "::1".into(),
      "internal.local".into(),
      "internal.local".into(),
    ];

    assert_eq!(
      client_no_proxy_value(&configured),
      "localhost,127.0.0.1,::1,internal.local"
    );
  }

  #[test]
  fn local_agent_command_uses_agent_binary_and_forwards_args() {
    assert_eq!(
      agent_command_spec(AgentKind::Codex, false, vec!["--model".into(), "gpt-5".into()]),
      CommandSpec {
        program: "codex".into(),
        args: vec!["--model".into(), "gpt-5".into()],
      }
    );
  }

  #[test]
  fn npx_agent_command_uses_agent_package_and_forwards_args() {
    assert_eq!(
      agent_command_spec(AgentKind::Opencode, true, vec!["run".into()]),
      CommandSpec {
        program: "npx".into(),
        args: vec!["-y".into(), "opencode-ai".into(), "run".into()],
      }
    );
    assert_eq!(
      agent_command_spec(AgentKind::Pi, true, Vec::new()),
      CommandSpec {
        program: "npx".into(),
        args: vec!["-y".into(), "@earendil-works/pi-coding-agent".into()],
      }
    );
  }
}
