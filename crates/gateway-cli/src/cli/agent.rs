//! Agent migration helpers for local tools that should route through
//! `tokn-router` profiles.

use crate::config::{Account, AuthType, Config};
use crate::util::secret::Secret;
use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use tokn_auth::{default_auth_path, AuthStore};
use tokn_core::account::AccountTier;
use tokn_core::provider::{ID_CODEX, ID_OPENAI};

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
  pub agent: AgentKind,
  #[arg(long)]
  pub profile: String,
  #[arg(long)]
  pub yes: bool,
}

#[derive(Args, Debug)]
pub struct RollbackArgs {
  #[arg(long, value_enum)]
  pub agent: AgentKind,
  /// Timestamp or full manifest path. Defaults to the latest manifest for the agent.
  #[arg(long)]
  pub backup_id: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
  CodexCli,
  Opencode,
}

impl AgentKind {
  fn slug(self) -> &'static str {
    match self {
      Self::CodexCli => "codex-cli",
      Self::Opencode => "opencode",
    }
  }

  fn agent_id(self) -> &'static str {
    match self {
      Self::CodexCli => "codex-cli",
      Self::Opencode => "opencode",
    }
  }

  fn default_provider_id(self) -> &'static str {
    match self {
      Self::CodexCli => ID_CODEX,
      Self::Opencode => ID_OPENAI,
    }
  }
}

#[derive(Debug)]
struct MigrationPlan {
  agent: AgentKind,
  timestamp: String,
  profile: String,
  gateway_config_path: PathBuf,
  gateway_auth_path: PathBuf,
  target_base_url: String,
  imported_accounts: Vec<Account>,
  edits: Vec<PlannedEdit>,
}

#[derive(Debug)]
struct PlannedEdit {
  path: PathBuf,
  kind: EditKind,
}

#[derive(Debug)]
enum EditKind {
  Json(Value),
  Toml(toml_edit::DocumentMut),
}

#[derive(Debug, Serialize, Deserialize)]
struct MigrationManifest {
  version: u32,
  agent: AgentKind,
  timestamp: String,
  profile: String,
  target_base_url: String,
  imported_account_ids: Vec<String>,
  files: Vec<FileBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileBackup {
  original: PathBuf,
  backup: Option<PathBuf>,
  existed: bool,
  created_by_migration: bool,
}

pub async fn run(cfg_path: Option<PathBuf>, cmd: AgentCmd) -> Result<()> {
  match cmd {
    AgentCmd::Migrate(args) => migrate(cfg_path, args).await,
    AgentCmd::Rollback(args) => rollback(args),
  }
}

async fn migrate(cfg_path: Option<PathBuf>, args: MigrateArgs) -> Result<()> {
  validate_profile_name(&args.profile)?;
  let (cfg, gateway_config_path) = Config::load(cfg_path.as_deref())?;
  let gateway_auth_path = default_gateway_auth_path()?;
  let timestamp = timestamp()?;
  let target_base_url = gateway_profile_base_url(&cfg, &args.profile);
  let mut plan = match args.agent {
    AgentKind::CodexCli => plan_codex_cli(&timestamp, &args.profile, &target_base_url)?,
    AgentKind::Opencode => plan_opencode(&timestamp, &args.profile, &target_base_url)?,
  };
  plan.gateway_config_path = gateway_config_path;
  plan.gateway_auth_path = gateway_auth_path;
  validate_migration_plan(&plan)?;

  print_plan(&plan);
  if !args.yes && !confirm("Apply this migration?")? {
    println!("Migration cancelled.");
    return Ok(());
  }

  apply_migration(plan)
}

fn validate_migration_plan(plan: &MigrationPlan) -> Result<()> {
  if plan.imported_accounts.is_empty() {
    bail!(
      "no credentials were discovered for {}; refusing to rewrite agent config without an account to route",
      plan.agent.slug()
    );
  }
  Ok(())
}

fn plan_codex_cli(timestamp: &str, profile: &str, target_base_url: &str) -> Result<MigrationPlan> {
  let home = home_dir()?;
  let auth_path = home.join(".codex/auth.json");
  let config_path = home.join(".codex/config.toml");

  let mut imported_accounts = Vec::new();
  let mut edits = Vec::new();

  if auth_path.exists() {
    let raw = std::fs::read_to_string(&auth_path).with_context(|| format!("reading {}", auth_path.display()))?;
    let mut json: Value = serde_json::from_str(&raw).with_context(|| format!("parsing {}", auth_path.display()))?;
    if let Some(account) = codex_account_from_auth_json(&json) {
      imported_accounts.push(account);
    }
    if let Some(obj) = json.as_object_mut() {
      obj.insert("auth_mode".into(), Value::String("api_key".into()));
      obj.insert("OPENAI_API_KEY".into(), Value::String("tokn-router".into()));
    }
    edits.push(PlannedEdit {
      path: auth_path,
      kind: EditKind::Json(json),
    });
  }

  let doc = if config_path.exists() {
    std::fs::read_to_string(&config_path)
      .with_context(|| format!("reading {}", config_path.display()))?
      .parse::<toml_edit::DocumentMut>()
      .with_context(|| format!("parsing {}", config_path.display()))?
  } else {
    toml_edit::DocumentMut::new()
  };
  let mut doc = doc;
  rewrite_codex_config(&mut doc, target_base_url);
  edits.push(PlannedEdit {
    path: config_path,
    kind: EditKind::Toml(doc),
  });

  Ok(MigrationPlan {
    agent: AgentKind::CodexCli,
    timestamp: timestamp.to_string(),
    profile: profile.to_string(),
    gateway_config_path: PathBuf::new(),
    gateway_auth_path: PathBuf::new(),
    target_base_url: target_base_url.to_string(),
    imported_accounts,
    edits,
  })
}

fn plan_opencode(timestamp: &str, profile: &str, target_base_url: &str) -> Result<MigrationPlan> {
  let home = home_dir()?;
  let config_path = home.join(".config/opencode/opencode.json");
  let mut imported_accounts = Vec::new();
  let mut edits = Vec::new();

  let mut json = if config_path.exists() {
    let raw = std::fs::read_to_string(&config_path).with_context(|| format!("reading {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", config_path.display()))?
  } else {
    Value::Object(serde_json::Map::new())
  };
  imported_accounts.extend(opencode_accounts_from_json(&json));
  rewrite_opencode_config(&mut json, target_base_url);
  edits.push(PlannedEdit {
    path: config_path,
    kind: EditKind::Json(json),
  });

  Ok(MigrationPlan {
    agent: AgentKind::Opencode,
    timestamp: timestamp.to_string(),
    profile: profile.to_string(),
    gateway_config_path: PathBuf::new(),
    gateway_auth_path: PathBuf::new(),
    target_base_url: target_base_url.to_string(),
    imported_accounts,
    edits,
  })
}

fn apply_migration(plan: MigrationPlan) -> Result<()> {
  let manifest_path = manifest_path(&plan.timestamp, plan.agent)?;
  let mut files = Vec::new();

  let gateway_auth_existed = plan.gateway_auth_path.exists();
  backup_path_for(&plan.gateway_auth_path, &plan.timestamp, &mut files)?;
  let mut store = AuthStore::load(Some(&plan.gateway_auth_path), Some(&plan.gateway_config_path))?;
  for account in &plan.imported_accounts {
    store.upsert(account.clone());
  }
  store.save()?;

  let gateway_config_existed = plan.gateway_config_path.exists();
  backup_path_for(&plan.gateway_config_path, &plan.timestamp, &mut files)?;
  upsert_profile(
    &plan.gateway_config_path,
    &plan.profile,
    plan.agent,
    &plan.imported_accounts,
  )?;
  mark_created(&mut files, &plan.gateway_auth_path, gateway_auth_existed);
  mark_created(&mut files, &plan.gateway_config_path, gateway_config_existed);

  for edit in &plan.edits {
    let existed = edit.path.exists();
    backup_path_for(&edit.path, &plan.timestamp, &mut files)?;
    write_edit(edit)?;
    mark_created(&mut files, &edit.path, existed);
  }

  let manifest = MigrationManifest {
    version: 1,
    agent: plan.agent,
    timestamp: plan.timestamp,
    profile: plan.profile,
    target_base_url: plan.target_base_url,
    imported_account_ids: plan
      .imported_accounts
      .iter()
      .map(|account| account.id.clone())
      .collect(),
    files,
  };
  if let Some(parent) = manifest_path.parent() {
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
    .with_context(|| format!("writing {}", manifest_path.display()))?;
  println!("Migration complete. Manifest: {}", manifest_path.display());
  Ok(())
}

fn rollback(args: RollbackArgs) -> Result<()> {
  let manifest_path = resolve_manifest(args.agent, args.backup_id.as_deref())?;
  let raw = std::fs::read_to_string(&manifest_path).with_context(|| format!("reading {}", manifest_path.display()))?;
  let manifest: MigrationManifest =
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", manifest_path.display()))?;
  if manifest.agent != args.agent {
    bail!(
      "manifest {} is for {}, not {}",
      manifest_path.display(),
      manifest.agent.slug(),
      args.agent.slug()
    );
  }
  println!("Rolling back {} from {}", args.agent.slug(), manifest.timestamp);
  for file in manifest.files.iter().rev() {
    if file.created_by_migration {
      if file.original.exists() {
        std::fs::remove_file(&file.original).with_context(|| format!("removing {}", file.original.display()))?;
        println!("removed {}", file.original.display());
      }
      continue;
    }
    let Some(backup) = &file.backup else {
      continue;
    };
    if let Some(parent) = file.original.parent() {
      std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::copy(backup, &file.original)
      .with_context(|| format!("restoring {} from {}", file.original.display(), backup.display()))?;
    println!("restored {}", file.original.display());
  }
  Ok(())
}

fn backup_path_for(path: &Path, timestamp: &str, files: &mut Vec<FileBackup>) -> Result<()> {
  if files.iter().any(|file| file.original == path) {
    return Ok(());
  }
  let existed = path.exists();
  let backup = if existed {
    let backup = adjacent_backup_path(path, timestamp)?;
    std::fs::copy(path, &backup).with_context(|| format!("backing up {} to {}", path.display(), backup.display()))?;
    Some(backup)
  } else {
    None
  };
  files.push(FileBackup {
    original: path.to_path_buf(),
    backup,
    existed,
    created_by_migration: false,
  });
  Ok(())
}

fn mark_created(files: &mut [FileBackup], path: &Path, existed: bool) {
  if !existed {
    if let Some(file) = files.iter_mut().find(|file| file.original == path) {
      file.created_by_migration = true;
    }
  }
}

fn adjacent_backup_path(path: &Path, timestamp: &str) -> Result<PathBuf> {
  let name = path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| anyhow!("cannot back up path without file name: {}", path.display()))?;
  Ok(path.with_file_name(format!("{name}.bak.{timestamp}")))
}

fn write_edit(edit: &PlannedEdit) -> Result<()> {
  if let Some(parent) = edit.path.parent() {
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  match &edit.kind {
    EditKind::Json(value) => {
      std::fs::write(&edit.path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("writing {}", edit.path.display()))?;
    }
    EditKind::Toml(doc) => {
      std::fs::write(&edit.path, doc.to_string()).with_context(|| format!("writing {}", edit.path.display()))?;
    }
  }
  Ok(())
}

fn upsert_profile(path: &Path, profile: &str, agent: AgentKind, accounts: &[Account]) -> Result<()> {
  Ok(Config::edit_in_place(path, |doc| {
    let account_ids = accounts.iter().map(|account| account.id.clone()).collect::<Vec<_>>();
    let mut providers = accounts
      .iter()
      .map(|account| account.provider.clone())
      .collect::<Vec<_>>();
    if providers.is_empty() {
      providers.push(agent.default_provider_id().to_string());
    }
    providers.sort();
    providers.dedup();
    let profiles = doc["profiles"].or_insert(toml_edit::table());
    let profile_item = profiles[profile].or_insert(toml_edit::table());
    profile_item["mode"] = toml_edit::value("route");
    profile_item["agent_id"] = toml_edit::value(agent.agent_id());
    profile_item["providers"] = array_value(&providers);
    if accounts.is_empty() {
      profile_item.as_table_mut().map(|table| table.remove("accounts"));
    } else {
      profile_item["accounts"] = array_value(&account_ids);
    }
    Ok(())
  })?)
}

fn rewrite_codex_config(doc: &mut toml_edit::DocumentMut, target_base_url: &str) {
  doc["model_provider"] = toml_edit::value("tokn-router");
  let providers = doc["model_providers"].or_insert(toml_edit::table());
  let provider = providers["tokn-router"].or_insert(toml_edit::table());
  provider["name"] = toml_edit::value("tokn-router");
  provider["base_url"] = toml_edit::value(target_base_url);
  provider["env_key"] = toml_edit::value("OPENAI_API_KEY");
  provider["wire_api"] = toml_edit::value("responses");
}

fn rewrite_opencode_config(json: &mut Value, target_base_url: &str) {
  if !json.is_object() {
    *json = Value::Object(serde_json::Map::new());
  }
  let obj = json.as_object_mut().expect("object ensured");
  obj.insert("provider".into(), Value::String("tokn-router".into()));
  let provider = obj
    .entry("providers")
    .or_insert_with(|| Value::Object(serde_json::Map::new()));
  if !provider.is_object() {
    *provider = Value::Object(serde_json::Map::new());
  }
  provider.as_object_mut().expect("object ensured").insert(
    "tokn-router".into(),
    serde_json::json!({
      "name": "tokn-router",
      "npm": "@ai-sdk/openai-compatible",
      "options": {
        "baseURL": target_base_url,
        "apiKey": "tokn-router"
      }
    }),
  );
}

fn codex_account_from_auth_json(json: &Value) -> Option<Account> {
  let tokens = json.get("tokens")?;
  let refresh = tokens.get("refresh_token").and_then(Value::as_str)?.trim();
  if refresh.is_empty() {
    return None;
  }
  let access = tokens
    .get("access_token")
    .and_then(Value::as_str)
    .filter(|s| !s.trim().is_empty());
  let account_id = tokens
    .get("account_id")
    .and_then(Value::as_str)
    .filter(|s| !s.trim().is_empty());
  let username = tokens
    .get("email")
    .and_then(Value::as_str)
    .filter(|s| !s.trim().is_empty());
  Some(Account {
    id: "codex-cli".into(),
    provider: ID_CODEX.into(),
    enabled: true,
    tier: AccountTier::Active,
    tags: vec!["agent-migrated".into(), "codex-cli".into()],
    label: Some("Codex CLI migration".into()),
    base_url: Some(tokn_provider_openai::codex::CODEX_BASE_URL.into()),
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: username.map(str::to_string),
    api_key: None,
    api_key_expires_at: None,
    access_token: access.map(|value| Secret::new(value.to_string())),
    access_token_expires_at: None,
    id_token: tokens
      .get("id_token")
      .and_then(Value::as_str)
      .filter(|s| !s.trim().is_empty())
      .map(|value| Secret::new(value.to_string())),
    refresh_token: Some(Secret::new(refresh.to_string())),
    provider_account_id: account_id.map(str::to_string),
    extra: Default::default(),
    refresh_url: Some(tokn_provider_openai::CODEX_OAUTH_TOKEN_URL.into()),
    last_refresh: None,
    settings: toml::Table::new(),
  })
}

fn opencode_accounts_from_json(json: &Value) -> Vec<Account> {
  let mut accounts = Vec::new();
  scan_api_keys(json, &mut accounts);
  accounts
}

fn scan_api_keys(value: &Value, accounts: &mut Vec<Account>) {
  match value {
    Value::Object(map) => {
      for (key, value) in map {
        if matches!(key.as_str(), "apiKey" | "api_key" | "OPENAI_API_KEY") {
          if let Some(api_key) = value.as_str().filter(|s| !s.trim().is_empty() && *s != "tokn-router") {
            accounts.push(openai_account_from_key(api_key));
            return;
          }
        }
        scan_api_keys(value, accounts);
        if !accounts.is_empty() {
          return;
        }
      }
    }
    Value::Array(values) => {
      for value in values {
        scan_api_keys(value, accounts);
        if !accounts.is_empty() {
          return;
        }
      }
    }
    _ => {}
  }
}

fn openai_account_from_key(api_key: &str) -> Account {
  Account {
    id: "opencode-openai".into(),
    provider: tokn_core::provider::ID_OPENAI.into(),
    enabled: true,
    tier: AccountTier::Active,
    tags: vec!["agent-migrated".into(), "opencode".into()],
    label: Some("opencode migration".into()),
    base_url: Some(tokn_provider_openai::openai::OPENAI_BASE_URL.into()),
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: Some(Secret::new(api_key.to_string())),
    api_key_expires_at: None,
    access_token: None,
    access_token_expires_at: None,
    id_token: None,
    refresh_token: None,
    provider_account_id: None,
    extra: Default::default(),
    refresh_url: None,
    last_refresh: None,
    settings: toml::Table::new(),
  }
}

fn array_value(values: &[String]) -> toml_edit::Item {
  let mut arr = toml_edit::Array::new();
  for value in values {
    arr.push(value.as_str());
  }
  toml_edit::value(arr)
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

fn validate_profile_name(profile: &str) -> Result<()> {
  if profile.trim().is_empty() || profile.contains('/') {
    bail!("profile name must be non-empty and must not contain '/'");
  }
  Ok(())
}

fn gateway_profile_base_url(cfg: &Config, profile: &str) -> String {
  format!("http://{}:{}/{profile}/v1", cfg.server.host, cfg.server.port)
}

fn timestamp() -> Result<String> {
  let now = time::OffsetDateTime::now_utc();
  let rfc3339 = now.format(&Rfc3339)?;
  let compact = rfc3339
    .replace(['-', ':'], "")
    .split('.')
    .next()
    .unwrap_or("backup")
    .trim_end_matches('Z')
    .to_string();
  Ok(format!("{compact}Z"))
}

fn home_dir() -> Result<PathBuf> {
  directories::BaseDirs::new()
    .map(|dirs| dirs.home_dir().to_path_buf())
    .ok_or_else(|| anyhow!("cannot resolve home directory"))
}

fn default_gateway_auth_path() -> Result<PathBuf> {
  default_auth_path()
}

fn manifest_dir() -> Result<PathBuf> {
  Ok(tokn_config::paths::config_dir()?.join("agent-migrations"))
}

fn manifest_path(timestamp: &str, agent: AgentKind) -> Result<PathBuf> {
  Ok(manifest_dir()?.join(format!("{timestamp}-{}.json", agent.slug())))
}

fn resolve_manifest(agent: AgentKind, backup_id: Option<&str>) -> Result<PathBuf> {
  if let Some(id) = backup_id {
    let path = PathBuf::from(id);
    if path.exists() {
      return Ok(path);
    }
    let candidate = manifest_dir()?.join(if id.ends_with(".json") {
      id.to_string()
    } else {
      format!("{id}-{}.json", agent.slug())
    });
    if candidate.exists() {
      return Ok(candidate);
    }
    bail!("backup manifest not found: {id}");
  }

  let dir = manifest_dir()?;
  let suffix = format!("-{}.json", agent.slug());
  let mut candidates = Vec::new();
  if dir.exists() {
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
      let entry = entry?;
      let path = entry.path();
      if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(&suffix))
        .unwrap_or(false)
      {
        candidates.push(path);
      }
    }
  }
  candidates.sort();
  candidates
    .pop()
    .ok_or_else(|| anyhow!("no migration manifest found for {}", agent.slug()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn codex_account_imports_refresh_and_account_id() {
    let account = codex_account_from_auth_json(&serde_json::json!({
      "tokens": {
        "refresh_token": "rt",
        "access_token": "at",
        "id_token": "id",
        "account_id": "acc"
      }
    }))
    .unwrap();
    assert_eq!(account.id, "codex-cli");
    assert_eq!(account.provider, "codex");
    assert_eq!(account.provider_account_id.as_deref(), Some("acc"));
    assert_eq!(account.refresh_token.unwrap().expose(), "rt");
  }

  #[test]
  fn codex_config_rewrite_preserves_existing_keys() {
    let mut doc: toml_edit::DocumentMut = "model = \"gpt-5\"\n".parse().unwrap();
    rewrite_codex_config(&mut doc, "http://127.0.0.1:4141/codex/v1");
    assert_eq!(doc["model"].as_str(), Some("gpt-5"));
    assert_eq!(doc["model_provider"].as_str(), Some("tokn-router"));
    assert_eq!(
      doc["model_providers"]["tokn-router"]["base_url"].as_str(),
      Some("http://127.0.0.1:4141/codex/v1")
    );
  }

  #[test]
  fn opencode_rewrite_preserves_mcp() {
    let mut json = serde_json::json!({"mcp": {"x": true}});
    rewrite_opencode_config(&mut json, "http://127.0.0.1:4141/opencode/v1");
    assert_eq!(json["mcp"]["x"], true);
    assert_eq!(json["provider"], "tokn-router");
    assert_eq!(
      json["providers"]["tokn-router"]["options"]["baseURL"],
      "http://127.0.0.1:4141/opencode/v1"
    );
  }

  #[test]
  fn default_gateway_auth_path_uses_auth_store_default() {
    assert_eq!(default_gateway_auth_path().unwrap(), default_auth_path().unwrap());
  }

  #[test]
  fn migration_plan_rejects_missing_imported_accounts() {
    let plan = MigrationPlan {
      agent: AgentKind::Opencode,
      timestamp: "20260604T153012Z".into(),
      profile: "opencode".into(),
      gateway_config_path: PathBuf::from("config.toml"),
      gateway_auth_path: PathBuf::from("auth.yaml"),
      target_base_url: "http://127.0.0.1:4141/opencode/v1".into(),
      imported_accounts: Vec::new(),
      edits: Vec::new(),
    };

    let err = validate_migration_plan(&plan).unwrap_err();
    assert!(err.to_string().contains("no credentials were discovered"));
  }

  #[test]
  fn upsert_profile_without_imported_accounts_scopes_to_agent_provider() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    upsert_profile(&path, "opencode", AgentKind::Opencode, &[]).unwrap();

    let (cfg, _) = Config::load(Some(&path)).unwrap();
    let profile = cfg.profiles.get("opencode").unwrap();
    assert_eq!(profile.agent_id, Some(tokn_core::AgentId::Opencode));
    assert_eq!(profile.providers.as_deref(), Some(&[ID_OPENAI.to_string()][..]));
    assert_eq!(profile.accounts, None);
  }

  #[test]
  fn upsert_profile_with_imported_accounts_scopes_to_accounts_and_providers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let accounts = vec![codex_account_from_auth_json(&serde_json::json!({
      "tokens": {
        "refresh_token": "rt"
      }
    }))
    .unwrap()];

    upsert_profile(&path, "codex", AgentKind::CodexCli, &accounts).unwrap();

    let (cfg, _) = Config::load(Some(&path)).unwrap();
    let profile = cfg.profiles.get("codex").unwrap();
    assert_eq!(profile.agent_id, Some(tokn_core::AgentId::CodexCli));
    assert_eq!(profile.providers.as_deref(), Some(&[ID_CODEX.to_string()][..]));
    assert_eq!(profile.accounts.as_deref(), Some(&["codex-cli".to_string()][..]));
  }

  #[test]
  fn adjacent_backup_keeps_original_name() {
    let path = PathBuf::from("/tmp/auth.json");
    assert_eq!(
      adjacent_backup_path(&path, "20260604T153012Z").unwrap(),
      PathBuf::from("/tmp/auth.json.bak.20260604T153012Z")
    );
  }

  #[test]
  fn rollback_restores_file_from_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("config.toml");
    let backup = dir.path().join("config.toml.bak.20260604T153012Z");
    let manifest_path = dir.path().join("20260604T153012Z-codex-cli.json");
    std::fs::write(&original, "mutated").unwrap();
    std::fs::write(&backup, "original").unwrap();
    let manifest = MigrationManifest {
      version: 1,
      agent: AgentKind::CodexCli,
      timestamp: "20260604T153012Z".into(),
      profile: "codex".into(),
      target_base_url: "http://127.0.0.1:4141/codex/v1".into(),
      imported_account_ids: vec!["codex-cli".into()],
      files: vec![FileBackup {
        original: original.clone(),
        backup: Some(backup),
        existed: true,
        created_by_migration: false,
      }],
    };
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    rollback(RollbackArgs {
      agent: AgentKind::CodexCli,
      backup_id: Some(manifest_path.display().to_string()),
    })
    .unwrap();

    assert_eq!(std::fs::read_to_string(original).unwrap(), "original");
  }
}
