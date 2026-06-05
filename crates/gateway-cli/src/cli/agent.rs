//! CLI bindings for agent migration commands.

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;
use tokn_agent_migration::{
  apply_migration, plan_migration, rollback_migration, AgentKind, FileAction, MigrateRequest, MigrationPlan,
  RollbackRequest,
};

#[derive(Subcommand, Debug)]
pub enum AgentCmd {
  /// Import credentials and point an agent at a gateway profile.
  Migrate(MigrateArgs),
  /// Restore files from a previous agent migration backup.
  Rollback(RollbackArgs),
}

#[derive(Args, Debug)]
pub struct MigrateArgs {
  #[arg(long, value_enum)]
  pub agent: CliAgentKind,
  #[arg(long)]
  pub profile: String,
  #[arg(long)]
  pub yes: bool,
}

#[derive(Args, Debug)]
pub struct RollbackArgs {
  #[arg(long, value_enum)]
  pub agent: CliAgentKind,
  /// Timestamp or full manifest path. Defaults to the latest manifest for the agent.
  #[arg(long)]
  pub backup_id: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum CliAgentKind {
  CodexCli,
  Opencode,
}

impl From<CliAgentKind> for AgentKind {
  fn from(value: CliAgentKind) -> Self {
    match value {
      CliAgentKind::CodexCli => Self::CodexCli,
      CliAgentKind::Opencode => Self::Opencode,
    }
  }
}

pub async fn run(cfg_path: Option<PathBuf>, cmd: AgentCmd) -> Result<()> {
  match cmd {
    AgentCmd::Migrate(args) => migrate(cfg_path, args),
    AgentCmd::Rollback(args) => rollback(args),
  }
}

fn migrate(cfg_path: Option<PathBuf>, args: MigrateArgs) -> Result<()> {
  let plan = plan_migration(MigrateRequest {
    agent: args.agent.into(),
    profile: args.profile,
    gateway_config_path: cfg_path,
    agent_home: None,
  })?;

  print_plan(&plan);
  if !args.yes && !confirm("Apply this migration?")? {
    println!("Migration cancelled.");
    return Ok(());
  }

  let report = apply_migration(plan)?;
  println!("Migration complete. Manifest: {}", report.manifest_path.display());
  Ok(())
}

fn rollback(args: RollbackArgs) -> Result<()> {
  let agent = AgentKind::from(args.agent);
  let report = rollback_migration(RollbackRequest {
    agent,
    backup_id: args.backup_id,
  })?;
  println!("Rolling back {} from {}", agent.slug(), report.timestamp);
  for action in report.actions {
    match action {
      FileAction::Removed(path) => println!("removed {}", path.display()),
      FileAction::Restored { original, .. } => println!("restored {}", original.display()),
    }
  }
  Ok(())
}

fn print_plan(plan: &MigrationPlan) {
  println!("Agent migration plan");
  println!("agent: {}", plan.agent.slug());
  println!("profile: {}", plan.profile);
  println!("target_base_url: {}", plan.target_base_url);
  println!("gateway_config: {}", plan.gateway_config_path.display());
  println!("gateway_auth: {}", plan.gateway_auth_path.display());
  if plan.imported_accounts.is_empty() {
    println!("imported_accounts: (none discovered)");
  } else {
    println!("imported_accounts:");
    for account in &plan.imported_accounts {
      println!("  - {} ({})", account.id, account.provider);
    }
  }
  println!("edits:");
  println!("  - {}", plan.gateway_config_path.display());
  println!("  - {}", plan.gateway_auth_path.display());
  for edit in &plan.edits {
    println!("  - {}", edit.path.display());
  }
}

fn confirm(prompt: &str) -> Result<bool> {
  inquire::Confirm::new(prompt)
    .with_default(false)
    .prompt()
    .context("confirmation prompt cancelled")
}
