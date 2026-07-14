//! CLI bindings for agent account import and binding commands.

use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tokn_agent_migration::{
  apply_reconcile, import_accounts, list_agents, plan_reconcile, show_agent, unlink, AgentStatus, ImportRequest,
  ReconcilePlan, ReconcileRequest, UnlinkRequest,
};
use tokn_config::{AgentAccountSource, Config, RouteMode};
use tokn_core::AgentId;

#[derive(Subcommand, Debug)]
pub enum AgentCmd {
  /// List supported agents and their current binding/import status.
  List,
  /// Show detailed status for one agent.
  Show(AgentTargetArgs),
  /// Import accounts from an agent without changing bindings or rewriting agent config.
  Import(AgentImportArgs),
  /// Bind an agent to the gateway and rewrite the agent's config to use it.
  Link(AgentLinkArgs),
  /// Reconcile agents from the `[agents.*]` source of truth.
  Sync(AgentSyncArgs),
  /// Restore files from the latest or specified bind manifest.
  Unlink(AgentUnlinkArgs),
}

#[derive(Args, Debug)]
pub struct AgentTargetArgs {
  #[arg(value_parser = parse_supported_agent)]
  pub agent: AgentId,
}

#[derive(Args, Debug)]
pub struct AgentImportArgs {
  #[command(flatten)]
  pub target: AgentTargetArgs,
  #[arg(long)]
  pub yes: bool,
}

#[derive(Args, Debug)]
pub struct AgentLinkArgs {
  #[command(flatten)]
  pub target: AgentTargetArgs,
  #[arg(long)]
  pub profile: Option<String>,
  #[arg(long, value_parser = parse_route_mode)]
  pub mode: Option<RouteMode>,
  /// Leave the agent's credentials untouched and use the gateway's existing
  /// main account pool instead of importing agent credentials.
  ///
  /// On an existing link, omitting this flag preserves the current account
  /// source. Unlink before linking again with a different source.
  #[arg(long)]
  pub use_main_accounts: bool,
  /// Provider used by a main-account passthrough or switch link. If omitted,
  /// a prior main-account profile target or `[defaults].default_provider_id` is used.
  #[arg(long, requires = "use_main_accounts")]
  pub provider: Option<String>,
  /// OpenCode provider namespace to redirect to the gateway's main account
  /// pool. May be supplied more than once; defaults to `openai`.
  #[arg(long = "source-provider", requires = "use_main_accounts")]
  pub source_providers: Vec<String>,
  #[arg(long)]
  pub yes: bool,
}

#[derive(Args, Debug)]
pub struct AgentSyncArgs {
  #[arg(value_parser = parse_supported_agent)]
  pub agent: Option<AgentId>,
  #[arg(long)]
  pub all: bool,
  #[arg(long)]
  pub yes: bool,
}

#[derive(Args, Debug)]
pub struct AgentUnlinkArgs {
  #[command(flatten)]
  pub target: AgentTargetArgs,
  /// Timestamp or full manifest path. Defaults to the latest manifest for the agent.
  #[arg(long)]
  pub backup_id: Option<String>,
}

pub async fn run(cfg_path: Option<PathBuf>, cmd: AgentCmd) -> Result<()> {
  match cmd {
    AgentCmd::List => list(cfg_path),
    AgentCmd::Show(args) => show(cfg_path, args),
    AgentCmd::Import(args) => import(cfg_path, args),
    AgentCmd::Link(args) => link(cfg_path, args),
    AgentCmd::Sync(args) => sync(cfg_path, args),
    AgentCmd::Unlink(args) => unlink_cmd(args),
  }
}

fn list(cfg_path: Option<PathBuf>) -> Result<()> {
  let statuses = list_agents(cfg_path.as_deref(), None, None)?;
  for status in statuses {
    print_list_row(&status);
  }
  Ok(())
}

fn show(cfg_path: Option<PathBuf>, args: AgentTargetArgs) -> Result<()> {
  let status = show_agent(cfg_path.as_deref(), None, None, args.agent)?;
  print_status(&status);
  Ok(())
}

fn import(cfg_path: Option<PathBuf>, args: AgentImportArgs) -> Result<()> {
  if !args.yes && !confirm(&format!("Import accounts from {}?", args.target.agent))? {
    println!("Import cancelled.");
    return Ok(());
  }
  let report = import_accounts(ImportRequest {
    agent: args.target.agent,
    gateway_config_path: cfg_path,
    agent_home: None,
  })?;
  println!("Imported into {}", report.gateway_auth_path.display());
  print_string_list("imported_accounts", &report.imported_account_ids);
  print_string_list("disabled_missing_accounts", &report.disabled_account_ids);
  Ok(())
}

fn link(cfg_path: Option<PathBuf>, args: AgentLinkArgs) -> Result<()> {
  let plan = plan_reconcile(ReconcileRequest {
    agent: args.target.agent,
    profile: args.profile,
    mode: args.mode,
    account_source: requested_account_source(args.use_main_accounts),
    default_provider_id: args.provider,
    source_provider_ids: (!args.source_providers.is_empty()).then_some(args.source_providers),
    gateway_config_path: cfg_path,
    agent_home: None,
  })?;
  print_plan("link", &plan);
  if !args.yes && !confirm("Apply this agent link?")? {
    println!("Link cancelled.");
    return Ok(());
  }
  let report = apply_reconcile(plan)?;
  println!("Link complete. Manifest: {}", report.manifest_path.display());
  Ok(())
}

fn sync(cfg_path: Option<PathBuf>, args: AgentSyncArgs) -> Result<()> {
  let agents = resolve_sync_agents(cfg_path.as_deref(), &args)?;
  if agents.is_empty() {
    println!("No synced agents configured.");
    return Ok(());
  }
  println!(
    "Syncing: {}",
    agents.iter().map(AgentId::to_string).collect::<Vec<_>>().join(", ")
  );
  if !args.yes && !confirm("Apply this agent sync?")? {
    println!("Sync cancelled.");
    return Ok(());
  }

  for agent in agents {
    let plan = plan_reconcile(ReconcileRequest {
      agent: agent.clone(),
      profile: None,
      mode: None,
      account_source: None,
      default_provider_id: None,
      source_provider_ids: None,
      gateway_config_path: cfg_path.clone(),
      agent_home: None,
    })?;
    print_plan("sync", &plan);
    let report = apply_reconcile(plan)?;
    println!("synced {} -> {}", agent, report.manifest_path.display());
  }
  Ok(())
}

fn unlink_cmd(args: AgentUnlinkArgs) -> Result<()> {
  let agent = args.target.agent;
  let report = unlink(UnlinkRequest {
    agent: agent.clone(),
    backup_id: args.backup_id,
  })?;
  println!("Rolling back {} from {}", agent, report.timestamp);
  for action in report.actions {
    match action {
      tokn_agent_migration::FileAction::Removed(path) => println!("removed {}", path.display()),
      tokn_agent_migration::FileAction::Restored { original, .. } => println!("restored {}", original.display()),
    }
  }
  Ok(())
}

fn parse_supported_agent(value: &str) -> Result<AgentId, String> {
  let Some(agent) = AgentId::from_slug(value) else {
    return Err(format!("unknown agent '{value}'"));
  };
  match agent {
    AgentId::Opencode | AgentId::CodexCli => Ok(agent),
    _ => Err(format!(
      "agent '{}' is recognized but not yet supported by `agent`; supported: opencode, codex-cli",
      agent.as_str()
    )),
  }
}

fn parse_route_mode(value: &str) -> Result<RouteMode, String> {
  match value.trim() {
    "passthrough" => Ok(RouteMode::Passthrough),
    "switch" => Ok(RouteMode::Switch),
    "exact" => Ok(RouteMode::Exact),
    "route" => Ok(RouteMode::Route),
    "fuzzy" => Ok(RouteMode::Fuzzy),
    _ => Err(format!("unknown route mode '{value}'")),
  }
}

fn resolve_sync_agents(cfg_path: Option<&std::path::Path>, args: &AgentSyncArgs) -> Result<Vec<AgentId>> {
  match (&args.agent, args.all) {
    (Some(agent), false) => Ok(vec![agent.clone()]),
    (None, true) => {
      let (cfg, _) = Config::load(cfg_path)?;
      let mut agents = cfg
        .agents
        .iter()
        .filter(|(_, binding)| binding.sync)
        .filter_map(|(name, _)| AgentId::from_slug(name))
        .collect::<Vec<_>>();
      agents.sort_by(|a, b| a.as_str().cmp(b.as_str()));
      Ok(agents)
    }
    (Some(_), true) => bail!("use either an agent or --all, not both"),
    (None, false) => bail!("sync requires either an <AGENT> or --all"),
  }
}

fn print_list_row(status: &AgentStatus) {
  let binding = status
    .binding
    .as_ref()
    .map(|binding| match binding.profile.as_deref() {
      Some(profile) => format!(
        "{} ({}; accounts={})",
        profile,
        route_mode_as_str(binding.mode),
        account_source_as_str(binding.account_source)
      ),
      None => format!(
        "defaults ({}; accounts={})",
        route_mode_as_str(binding.mode),
        account_source_as_str(binding.account_source)
      ),
    })
    .unwrap_or_else(|| "unbound".into());
  let detected = if status.detected { "detected" } else { "missing" };
  let drift = if status.binding.is_some() && !status.drifted {
    "drifted"
  } else {
    "ok"
  };
  println!(
    "{}\tdetected={}\tbinding={}\timported={}\tconfig={}",
    status.agent,
    detected,
    binding,
    status.imported_account_ids.len(),
    drift
  );
}

fn print_status(status: &AgentStatus) {
  println!("agent: {}", status.agent);
  println!("supported: {}", status.supported);
  println!("detected: {}", status.detected);
  println!("auth_path: {}", status.auth_path.display());
  println!("config_path: {}", status.config_path.display());
  match &status.binding {
    Some(binding) => {
      println!("binding:");
      println!("  profile: {}", binding.profile.as_deref().unwrap_or("(defaults)"));
      println!("  mode: {}", route_mode_as_str(binding.mode));
      println!("  account_source: {}", account_source_as_str(binding.account_source));
      if binding.account_source == AgentAccountSource::Main {
        println!(
          "  source_providers: {}",
          binding
            .source_providers
            .as_deref()
            .filter(|providers| !providers.is_empty())
            .map(|providers| providers.join(", "))
            .unwrap_or_else(|| "openai".into())
        );
      }
      println!("  sync: {}", binding.sync);
    }
    None => println!("binding: (none)"),
  }
  println!("imported_accounts:");
  if status.imported_account_ids.is_empty() {
    println!("  (none)");
  } else {
    for id in &status.imported_account_ids {
      println!("  - {id}");
    }
  }
  println!("config_in_sync: {}", status.drifted);
}

fn print_plan(kind: &str, plan: &ReconcilePlan) {
  println!("Agent {kind} plan");
  println!("agent: {}", plan.agent);
  println!("profile: {}", plan.binding_profile.as_deref().unwrap_or("(defaults)"));
  println!("mode: {}", route_mode_as_str(plan.binding_mode));
  println!("account_source: {}", account_source_as_str(plan.account_source));
  println!("target_base_url: {}", plan.target_base_url);
  println!("gateway_config: {}", plan.gateway_config_path.display());
  println!(
    "gateway_config_fragment: {}",
    plan.gateway_config_fragment_path.display()
  );
  if let Some(auth_shard) = &plan.gateway_auth_shard_path {
    println!("gateway_auth_root: unchanged");
    println!("gateway_auth_fragment: {}", auth_shard.display());
  } else {
    println!("gateway_auth: unchanged");
  }
  if plan.account_source == AgentAccountSource::Main {
    print_string_list("source_providers", &plan.source_provider_ids);
  }
  print_string_list(
    "imported_accounts",
    &plan
      .imported_accounts
      .iter()
      .map(|account| account.id.clone())
      .collect::<Vec<_>>(),
  );
  println!("edits:");
  for edit in &plan.edits {
    println!("  - {}", edit.path.display());
  }
}

fn print_string_list(label: &str, values: &[String]) {
  if values.is_empty() {
    println!("{label}: (none)");
    return;
  }
  println!("{label}:");
  for value in values {
    println!("  - {value}");
  }
}

fn route_mode_as_str(mode: RouteMode) -> &'static str {
  match mode {
    RouteMode::Passthrough => "passthrough",
    RouteMode::Switch => "switch",
    RouteMode::Exact => "exact",
    RouteMode::Route => "route",
    RouteMode::Fuzzy => "fuzzy",
  }
}

fn requested_account_source(use_main_accounts: bool) -> Option<AgentAccountSource> {
  use_main_accounts.then_some(AgentAccountSource::Main)
}

fn account_source_as_str(source: AgentAccountSource) -> &'static str {
  match source {
    AgentAccountSource::Agent => "agent",
    AgentAccountSource::Main => "main",
  }
}

fn confirm(prompt: &str) -> Result<bool> {
  inquire::Confirm::new(prompt)
    .with_default(false)
    .prompt()
    .context("confirmation prompt cancelled")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn use_main_accounts_only_requests_a_source_when_present() {
    assert_eq!(requested_account_source(false), None);
    assert_eq!(requested_account_source(true), Some(AgentAccountSource::Main));
  }
}
