use crate::adapter::{adapter_for, source_provider_id, ProviderRoute};
use crate::manifest::{self, FileBackup, MigrationManifest};
use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use tokn_auth::{default_auth_path, AuthStore};
use tokn_config::{Account, Config, RouteMode};
use tokn_core::AgentId;

#[derive(Debug)]
pub struct ImportRequest {
  pub agent: AgentId,
  pub gateway_config_path: Option<PathBuf>,
  pub agent_home: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ReconcileRequest {
  pub agent: AgentId,
  pub profile: Option<String>,
  pub mode: Option<RouteMode>,
  pub gateway_config_path: Option<PathBuf>,
  pub agent_home: Option<PathBuf>,
}

#[derive(Debug)]
pub struct UnlinkRequest {
  pub agent: AgentId,
  pub backup_id: Option<String>,
}

#[derive(Debug)]
pub struct ReconcilePlan {
  pub agent: AgentId,
  pub timestamp: String,
  pub gateway_config_path: PathBuf,
  pub gateway_auth_path: PathBuf,
  pub agent_auth_path: Option<PathBuf>,
  pub binding_profile: Option<String>,
  pub binding_mode: RouteMode,
  pub target_base_url: String,
  pub imported_accounts: Vec<Account>,
  pub(crate) provider_routes: Vec<ProviderRoute>,
  pub edits: Vec<PlannedEdit>,
  pub(crate) previous_manifest: Option<PathBuf>,
}

#[derive(Debug)]
pub struct PlannedEdit {
  pub path: PathBuf,
  pub(crate) kind: EditKind,
  pub(crate) backup: bool,
}

#[derive(Debug)]
pub(crate) enum EditKind {
  Json(Value),
  Jsonc(String),
  Toml(toml_edit::DocumentMut),
}

#[derive(Debug)]
pub struct ImportReport {
  pub gateway_auth_path: PathBuf,
  pub imported_account_ids: Vec<String>,
  pub disabled_account_ids: Vec<String>,
}

#[derive(Debug)]
pub struct ApplyReport {
  pub manifest_path: PathBuf,
  pub files: Vec<FileBackup>,
}

#[derive(Debug)]
pub struct UnlinkReport {
  pub manifest_path: PathBuf,
  pub timestamp: String,
  pub actions: Vec<FileAction>,
}

#[derive(Debug)]
pub enum FileAction {
  Removed(PathBuf),
  Restored { original: PathBuf, backup: PathBuf },
}

pub fn import_accounts(request: ImportRequest) -> Result<ImportReport> {
  let adapter = adapter_for(&request.agent).ok_or_else(|| anyhow!("unsupported agent {}", request.agent))?;
  let gateway_auth_path = default_gateway_auth_path()?;
  let gateway_config_path = Config::load(request.gateway_config_path.as_deref())?.1;
  let home = resolve_home(request.agent_home)?;
  let timestamp = timestamp()?;
  let imported_accounts = adapter.discover_accounts(&home, &timestamp)?;
  let imported_account_ids = imported_accounts
    .iter()
    .map(|account| account.id.clone())
    .collect::<BTreeSet<_>>();
  let mut store = AuthStore::load(Some(&gateway_auth_path), Some(&gateway_config_path))?;
  let disabled_account_ids = disable_missing_source_accounts(&mut store, &request.agent, &imported_account_ids);
  for account in imported_accounts {
    store.upsert(account);
  }
  store.save()?;
  Ok(ImportReport {
    gateway_auth_path,
    imported_account_ids: imported_account_ids.into_iter().collect(),
    disabled_account_ids,
  })
}

pub fn plan_reconcile(request: ReconcileRequest) -> Result<ReconcilePlan> {
  let gateway_auth_path = default_gateway_auth_path()?;
  plan_reconcile_with_gateway_auth_path(request, gateway_auth_path)
}

fn plan_reconcile_with_gateway_auth_path(
  request: ReconcileRequest,
  gateway_auth_path: PathBuf,
) -> Result<ReconcilePlan> {
  let adapter = adapter_for(&request.agent).ok_or_else(|| anyhow!("unsupported agent {}", request.agent))?;
  let (cfg, gateway_config_path) = Config::load(request.gateway_config_path.as_deref())?;
  let timestamp = timestamp()?;
  let home = resolve_home(request.agent_home)?;
  let discovered_accounts = adapter.discover_accounts(&home, &timestamp)?;
  let imported_accounts = if adapter.transfers_credentials() {
    let store = AuthStore::load(Some(&gateway_auth_path), Some(&gateway_config_path))?;
    merge_transferred_accounts(&store, &request.agent, discovered_accounts)
  } else {
    discovered_accounts
  };
  let existing_binding = cfg.agents.get(request.agent.as_str());
  let binding_profile = resolve_binding_profile(
    request.profile.as_deref(),
    existing_binding,
    &request.agent,
    &imported_accounts,
  )?;
  let binding_mode = request
    .mode
    .or_else(|| existing_binding.and_then(|binding| binding.mode))
    .unwrap_or(RouteMode::Route);
  let target_base_url = gateway_profile_base_url(&cfg, binding_profile.as_deref());
  let provider_routes = provider_routes(
    &cfg,
    binding_profile.as_deref(),
    &imported_accounts,
    adapter.default_provider_id(),
  );
  let edits = adapter.rewrite_config(&home, &target_base_url, &provider_routes)?;
  Ok(ReconcilePlan {
    agent: request.agent,
    timestamp,
    gateway_config_path,
    gateway_auth_path,
    agent_auth_path: adapter.transfers_credentials().then(|| adapter.auth_path(&home)),
    binding_profile,
    binding_mode,
    target_base_url,
    imported_accounts,
    provider_routes,
    edits,
    previous_manifest: None,
  })
}

pub fn apply_reconcile(mut plan: ReconcilePlan) -> Result<ApplyReport> {
  plan.previous_manifest = manifest::latest_active_manifest(&plan.agent)?;
  let manifest_path = manifest::manifest_path(&plan.timestamp, &plan.agent)?;
  apply_reconcile_to_manifest_path(plan, manifest_path)
}

pub fn unlink(request: UnlinkRequest) -> Result<UnlinkReport> {
  let manifest_path = manifest::resolve_manifest(&request.agent, request.backup_id.as_deref())?;
  if let Some(successor) = active_manifest_successor(&manifest_path, &request.agent)? {
    bail!(
      "manifest {} has newer active successor {}; unlink the latest migration instead",
      manifest_path.display(),
      successor.display()
    );
  }
  let mut chain = manifest_chain(&manifest_path, &request.agent)?;
  let latest = chain.first().expect("manifest chain contains selected manifest");
  if latest.1.unlinked {
    bail!("manifest {} has already been unlinked", latest.0.display());
  }
  let timestamp = latest.1.timestamp.clone();

  restore_latest_credentials(&request.agent, &latest.1)?;
  let mut actions = Vec::new();
  for (_, current) in &chain {
    restore_manifest_files(current, &mut actions)?;
  }
  for (path, current) in &mut chain {
    current.unlinked = true;
    manifest::write_manifest(path, current)?;
  }

  Ok(UnlinkReport {
    manifest_path,
    timestamp,
    actions,
  })
}

fn active_manifest_successor(path: &Path, agent: &AgentId) -> Result<Option<PathBuf>> {
  let Some(dir) = path.parent() else {
    return Ok(None);
  };
  let suffix = format!("-{}.json", agent.as_str());
  let mut successors = Vec::new();
  for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
    let candidate = entry?.path();
    if candidate == path
      || !candidate
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(&suffix))
        .unwrap_or(false)
    {
      continue;
    }
    let manifest = manifest::read_manifest(&candidate)?;
    if manifest.agent != *agent || !manifest.completed || manifest.unlinked {
      continue;
    }
    let chain = manifest_chain(&candidate, agent)?;
    if chain.iter().skip(1).any(|(ancestor, _)| same_path(ancestor, path)) {
      successors.push(candidate);
    }
  }
  successors.sort();
  Ok(successors.pop())
}

fn same_path(left: &Path, right: &Path) -> bool {
  left == right
    || left
      .canonicalize()
      .ok()
      .zip(right.canonicalize().ok())
      .map(|(left, right)| left == right)
      .unwrap_or(false)
}

fn manifest_chain(path: &Path, agent: &AgentId) -> Result<Vec<(PathBuf, MigrationManifest)>> {
  let mut chain = Vec::new();
  let mut seen = BTreeSet::new();
  let mut current = Some(path.to_path_buf());
  while let Some(path) = current {
    if !seen.insert(path.clone()) {
      bail!("migration manifest chain contains a cycle at {}", path.display());
    }
    let manifest = manifest::read_manifest(&path)?;
    if manifest.agent != *agent {
      bail!("manifest {} is for {}, not {}", path.display(), manifest.agent, agent);
    }
    current = manifest.previous_manifest.clone();
    chain.push((path, manifest));
  }
  Ok(chain)
}

fn restore_latest_credentials(agent: &AgentId, manifest: &MigrationManifest) -> Result<()> {
  let Some(agent_auth_path) = &manifest.agent_auth_path else {
    return Ok(());
  };
  let Some(gateway_auth_path) = &manifest.gateway_auth_path else {
    bail!("manifest is missing the gateway auth path required to restore transferred credentials");
  };
  let store = AuthStore::load(Some(gateway_auth_path), None)?;
  let accounts = manifest
    .imported_account_ids
    .iter()
    .map(|id| {
      store.get(id).cloned().ok_or_else(|| {
        anyhow!(
          "transferred account '{id}' is missing from {}",
          gateway_auth_path.display()
        )
      })
    })
    .collect::<Result<Vec<_>>>()?;
  adapter_for(agent)
    .ok_or_else(|| anyhow!("unsupported agent {agent}"))?
    .restore_transferred_credentials(agent_auth_path, &accounts)
}

fn restore_manifest_files(manifest: &MigrationManifest, actions: &mut Vec<FileAction>) -> Result<()> {
  for file in manifest.files.iter().rev() {
    if file.created_by_migration {
      if file.original.exists() {
        std::fs::remove_file(&file.original).with_context(|| format!("removing {}", file.original.display()))?;
        actions.push(FileAction::Removed(file.original.clone()));
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
    actions.push(FileAction::Restored {
      original: file.original.clone(),
      backup: backup.clone(),
    });
  }
  Ok(())
}

fn apply_reconcile_to_manifest_path(plan: ReconcilePlan, manifest_path: PathBuf) -> Result<ApplyReport> {
  let mut files = Vec::new();
  let imported_account_ids = plan
    .imported_accounts
    .iter()
    .map(|account| account.id.clone())
    .collect::<BTreeSet<_>>();

  let gateway_auth_existed = plan.gateway_auth_path.exists();
  manifest::backup_path_for(&plan.gateway_auth_path, &plan.timestamp, &mut files)?;
  let gateway_config_existed = plan.gateway_config_path.exists();
  manifest::backup_path_for(&plan.gateway_config_path, &plan.timestamp, &mut files)?;
  manifest::mark_created(&mut files, &plan.gateway_auth_path, gateway_auth_existed);
  manifest::mark_created(&mut files, &plan.gateway_config_path, gateway_config_existed);

  for edit in &plan.edits {
    if !edit.backup {
      continue;
    }
    let existed = edit.path.exists();
    manifest::backup_path_for(&edit.path, &plan.timestamp, &mut files)?;
    manifest::mark_created(&mut files, &edit.path, existed);
  }

  let manifest = MigrationManifest {
    version: 2,
    completed: true,
    agent: plan.agent.clone(),
    timestamp: plan.timestamp.clone(),
    profile: plan.binding_profile.clone(),
    target_base_url: plan.target_base_url.clone(),
    gateway_auth_path: Some(plan.gateway_auth_path.clone()),
    agent_auth_path: plan.agent_auth_path.clone(),
    provider_routes: plan.provider_routes.clone(),
    previous_manifest: plan.previous_manifest.clone(),
    unlinked: false,
    imported_account_ids: plan
      .imported_accounts
      .iter()
      .map(|account| account.id.clone())
      .collect(),
    files,
  };
  manifest::write_manifest(&manifest_path, &manifest.clone().in_progress())?;

  let mut store = AuthStore::load(Some(&plan.gateway_auth_path), Some(&plan.gateway_config_path))?;
  remove_replaced_gateway_accounts(&mut store, &plan.agent, &plan.imported_accounts);
  disable_missing_source_accounts(&mut store, &plan.agent, &imported_account_ids);
  for account in &plan.imported_accounts {
    store.upsert(account.clone());
  }
  store.save()?;

  let default_provider_id = adapter_for(&plan.agent)
    .expect("supported agent should still have adapter")
    .default_provider_id();
  upsert_agent_and_profiles(
    &plan.gateway_config_path,
    &plan.agent,
    plan.binding_profile.as_deref(),
    plan.binding_mode,
    &plan.imported_accounts,
    &plan.provider_routes,
    default_provider_id,
  )?;

  for edit in &plan.edits {
    write_edit(edit)?;
  }

  let manifest = manifest.complete();
  manifest::write_manifest(&manifest_path, &manifest)?;
  Ok(ApplyReport {
    manifest_path,
    files: manifest.files,
  })
}

pub(crate) fn annotate_imported_account(
  mut account: Account,
  agent: AgentId,
  source_path: &Path,
  source_key: &str,
  imported_at: &str,
) -> Account {
  let source_tag = format!("source:{}", agent.as_str());
  for tag in ["imported", "agent-managed", agent.as_str(), source_tag.as_str()] {
    if !account.tags.iter().any(|existing| existing == tag) {
      account.tags.push(tag.to_string());
    }
  }

  let mut import = toml::Table::new();
  import.insert("source_agent".into(), toml::Value::String(agent.to_string()));
  import.insert(
    "source_path".into(),
    toml::Value::String(source_path.display().to_string()),
  );
  import.insert("source_key".into(), toml::Value::String(source_key.into()));
  import.insert("imported_at".into(), toml::Value::String(imported_at.into()));
  import.insert("last_seen_at".into(), toml::Value::String(imported_at.into()));
  import.insert("sync_managed".into(), toml::Value::Boolean(true));
  import.insert("missing_from_source".into(), toml::Value::Boolean(false));
  account.settings.insert("import".into(), toml::Value::Table(import));
  account.enabled = true;
  account
}

fn merge_transferred_accounts(store: &AuthStore, agent: &AgentId, discovered_accounts: Vec<Account>) -> Vec<Account> {
  let mut accounts = store
    .accounts
    .iter()
    .filter(|account| is_gateway_owned_account(account, agent))
    .map(|account| (transfer_source_provider(account).to_string(), account.clone()))
    .collect::<BTreeMap<_, _>>();
  for account in discovered_accounts {
    let account = mark_gateway_owned(account);
    accounts.insert(transfer_source_provider(&account).to_string(), account);
  }
  accounts.into_values().collect()
}

fn mark_gateway_owned(mut account: Account) -> Account {
  let provider = account.provider.clone();
  if let Some(import) = account.settings.get_mut("import").and_then(toml::Value::as_table_mut) {
    import
      .entry("source_provider")
      .or_insert_with(|| toml::Value::String(provider));
    import.insert("ownership".into(), toml::Value::String("gateway".into()));
    import.insert("sync_managed".into(), toml::Value::Boolean(false));
    import.insert("missing_from_source".into(), toml::Value::Boolean(false));
  }
  account
}

fn remove_replaced_gateway_accounts(store: &mut AuthStore, agent: &AgentId, desired_accounts: &[Account]) {
  let desired = desired_accounts
    .iter()
    .map(|account| (transfer_source_provider(account), account.id.as_str()))
    .collect::<BTreeMap<_, _>>();
  store.accounts.retain(|account| {
    if !is_gateway_owned_account(account, agent) {
      return true;
    }
    desired
      .get(transfer_source_provider(account))
      .map(|desired_id| **desired_id == account.id)
      .unwrap_or(true)
  });
}

fn transfer_source_provider(account: &Account) -> &str {
  source_provider_id(account).unwrap_or(&account.provider)
}

fn is_gateway_owned_account(account: &Account, agent: &AgentId) -> bool {
  is_source_managed_account(account, agent)
    && account
      .settings
      .get("import")
      .and_then(toml::Value::as_table)
      .and_then(|import| import.get("ownership"))
      .and_then(toml::Value::as_str)
      == Some("gateway")
}

fn provider_routes(
  cfg: &Config,
  binding_profile: Option<&str>,
  accounts: &[Account],
  default_provider_id: &str,
) -> Vec<ProviderRoute> {
  if accounts.is_empty() {
    return vec![ProviderRoute {
      source_provider_id: default_provider_id.to_string(),
      gateway_provider_id: default_provider_id.to_string(),
      account_id: String::new(),
      profile: binding_profile.unwrap_or_default().to_string(),
      base_url: gateway_profile_base_url(cfg, binding_profile),
    }];
  }

  let Some(binding_profile) = binding_profile else {
    return Vec::new();
  };
  let mut routes = BTreeMap::new();
  for account in accounts {
    let Some(source_provider_id) = source_provider_id(account) else {
      continue;
    };
    let profile = format!("{binding_profile}-{}", account.provider);
    routes.insert(
      source_provider_id.to_string(),
      ProviderRoute {
        source_provider_id: source_provider_id.to_string(),
        gateway_provider_id: account.provider.clone(),
        account_id: account.id.clone(),
        base_url: gateway_profile_base_url(cfg, Some(&profile)),
        profile,
      },
    );
  }
  routes.into_values().collect()
}

pub(crate) fn imported_account_ids(store: &AuthStore, agent: &AgentId) -> Vec<String> {
  let mut ids = store
    .accounts
    .iter()
    .filter(|account| is_source_managed_account(account, agent))
    .map(|account| account.id.clone())
    .collect::<Vec<_>>();
  ids.sort();
  ids
}

pub(crate) fn disable_missing_source_accounts(
  store: &mut AuthStore,
  agent: &AgentId,
  seen_ids: &BTreeSet<String>,
) -> Vec<String> {
  let mut disabled = Vec::new();
  for account in &mut store.accounts {
    if seen_ids.contains(&account.id) || !is_source_managed_account(account, agent) || !is_sync_managed(account) {
      continue;
    }
    account.enabled = false;
    if !account.tags.iter().any(|tag| tag == "source:missing") {
      account.tags.push("source:missing".into());
    }
    if let Some(import) = account.settings.get_mut("import").and_then(toml::Value::as_table_mut) {
      import.insert("missing_from_source".into(), toml::Value::Boolean(true));
    }
    disabled.push(account.id.clone());
  }
  disabled.sort();
  disabled
}

fn is_sync_managed(account: &Account) -> bool {
  account
    .settings
    .get("import")
    .and_then(toml::Value::as_table)
    .and_then(|import| import.get("sync_managed"))
    .and_then(toml::Value::as_bool)
    .unwrap_or(true)
}

pub(crate) fn is_source_managed_account(account: &Account, agent: &AgentId) -> bool {
  let source_agent = account
    .settings
    .get("import")
    .and_then(toml::Value::as_table)
    .and_then(|import| import.get("source_agent"))
    .and_then(toml::Value::as_str);
  if source_agent == Some(agent.as_str()) {
    return true;
  }
  let source_tag = format!("source:{}", agent.as_str());
  account.tags.iter().any(|tag| tag == &source_tag)
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
    EditKind::Jsonc(raw) => {
      std::fs::write(&edit.path, raw).with_context(|| format!("writing {}", edit.path.display()))?;
    }
    EditKind::Toml(doc) => {
      std::fs::write(&edit.path, doc.to_string()).with_context(|| format!("writing {}", edit.path.display()))?;
    }
  }
  Ok(())
}

fn upsert_agent_and_profiles(
  path: &Path,
  agent: &AgentId,
  profile: Option<&str>,
  mode: RouteMode,
  accounts: &[Account],
  provider_routes: &[ProviderRoute],
  default_provider_id: &str,
) -> Result<()> {
  Ok(Config::edit_in_place(path, |doc| {
    let previous_profile = existing_agent_profile(doc, agent);
    upsert_agent(doc, agent, profile, mode);
    if let Some(previous_profile) = previous_profile.as_deref() {
      if Some(previous_profile) != profile {
        remove_materialized_profile(doc, previous_profile, agent);
      }
    }
    if let Some(profile) = profile {
      upsert_profile_item(doc, profile, agent, mode, accounts, default_provider_id);
      if !provider_routes.is_empty() {
        remove_agent_profiles(doc, profile, agent);
        upsert_provider_route_profiles(doc, agent, mode, provider_routes);
      } else if mode == RouteMode::Switch {
        upsert_switch_profiles(doc, profile, agent, accounts);
      } else {
        remove_agent_profiles(doc, profile, agent);
      }
    }
    Ok(())
  })?)
}

fn existing_agent_profile(doc: &toml_edit::DocumentMut, agent: &AgentId) -> Option<String> {
  doc
    .get("agents")
    .and_then(toml_edit::Item::as_table_like)
    .and_then(|agents| agents.get(agent.as_str()))
    .and_then(toml_edit::Item::as_table_like)
    .and_then(|agent| agent.get("profile"))
    .and_then(toml_edit::Item::as_str)
    .map(str::to_string)
}

fn upsert_agent(doc: &mut toml_edit::DocumentMut, agent: &AgentId, profile: Option<&str>, mode: RouteMode) {
  let agents = doc["agents"].or_insert(toml_edit::table());
  let agent_item = agents[agent.as_str()].or_insert(toml_edit::table());
  agent_item["mode"] = toml_edit::value(route_mode_as_str(mode));
  if let Some(profile) = profile {
    agent_item["profile"] = toml_edit::value(profile);
  } else if let Some(table) = agent_item.as_table_mut() {
    table.remove("profile");
  }
  agent_item["sync"] = toml_edit::value(true);
}

fn upsert_profile_item(
  doc: &mut toml_edit::DocumentMut,
  profile: &str,
  agent: &AgentId,
  mode: RouteMode,
  accounts: &[Account],
  default_provider_id: &str,
) {
  let account_ids = accounts.iter().map(|account| account.id.clone()).collect::<Vec<_>>();
  let mut providers = accounts
    .iter()
    .map(|account| account.provider.clone())
    .collect::<Vec<_>>();
  if providers.is_empty() {
    providers.push(default_provider_id.to_string());
  }
  providers.sort();
  providers.dedup();
  let profiles = doc["profiles"].or_insert(toml_edit::table());
  let profile_item = profiles[profile].or_insert(toml_edit::table());
  profile_item["mode"] = toml_edit::value(route_mode_as_str(mode));
  profile_item["agent_id"] = toml_edit::value(agent.as_str());
  profile_item["providers"] = array_value(&providers);
  if accounts.is_empty() {
    profile_item.as_table_mut().map(|table| table.remove("accounts"));
  } else {
    profile_item["accounts"] = array_value(&account_ids);
  }
}

fn upsert_switch_profiles(doc: &mut toml_edit::DocumentMut, profile: &str, agent: &AgentId, accounts: &[Account]) {
  let mut by_provider: BTreeMap<String, Vec<String>> = BTreeMap::new();
  for account in accounts {
    by_provider
      .entry(account.provider.clone())
      .or_default()
      .push(account.id.clone());
  }
  let profiles = doc["profiles"].or_insert(toml_edit::table());
  for (provider, account_ids) in by_provider {
    let synthetic_profile = format!("{profile}-{provider}");
    let item = profiles[synthetic_profile.as_str()].or_insert(toml_edit::table());
    item["mode"] = toml_edit::value("switch");
    item["agent_id"] = toml_edit::value(agent.as_str());
    item["providers"] = array_value(std::slice::from_ref(&provider));
    item["accounts"] = array_value(&account_ids);
  }
}

fn upsert_provider_route_profiles(
  doc: &mut toml_edit::DocumentMut,
  agent: &AgentId,
  mode: RouteMode,
  routes: &[ProviderRoute],
) {
  let profiles = doc["profiles"].or_insert(toml_edit::table());
  for route in routes {
    if route.account_id.is_empty() || route.profile.is_empty() {
      continue;
    }
    let item = profiles[route.profile.as_str()].or_insert(toml_edit::table());
    item["mode"] = toml_edit::value(route_mode_as_str(mode));
    item["agent_id"] = toml_edit::value(agent.as_str());
    item["providers"] = array_value(std::slice::from_ref(&route.gateway_provider_id));
    item["accounts"] = array_value(std::slice::from_ref(&route.account_id));
  }
}

fn remove_materialized_profile(doc: &mut toml_edit::DocumentMut, profile: &str, agent: &AgentId) {
  if let Some(table) = doc["profiles"].as_table_mut() {
    table.remove(profile);
  }
  remove_agent_profiles(doc, profile, agent);
}

fn remove_agent_profiles(doc: &mut toml_edit::DocumentMut, profile: &str, agent: &AgentId) {
  let Some(table) = doc["profiles"].as_table_mut() else {
    return;
  };
  let prefix = format!("{profile}-");
  let keys = table
    .iter()
    .filter(|(key, item)| {
      key.starts_with(&prefix)
        && item
          .as_table_like()
          .and_then(|profile| profile.get("agent_id"))
          .and_then(toml_edit::Item::as_str)
          == Some(agent.as_str())
    })
    .map(|(key, _)| key.to_string())
    .collect::<Vec<_>>();
  for key in keys {
    table.remove(&key);
  }
}

fn array_value(values: &[String]) -> toml_edit::Item {
  let mut arr = toml_edit::Array::new();
  for value in values {
    arr.push(value.as_str());
  }
  toml_edit::value(arr)
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

fn resolve_binding_profile(
  explicit_profile: Option<&str>,
  existing_binding: Option<&tokn_config::AgentConfig>,
  agent: &AgentId,
  imported_accounts: &[Account],
) -> Result<Option<String>> {
  if let Some(profile) = explicit_profile {
    validate_profile_name(profile)?;
    return Ok(Some(profile.to_string()));
  }
  if let Some(profile) = existing_binding.and_then(|binding| binding.profile.as_deref()) {
    validate_profile_name(profile)?;
    return Ok(Some(profile.to_string()));
  }
  if imported_accounts.is_empty() {
    return Ok(None);
  }
  Ok(Some(agent.as_str().to_string()))
}

fn validate_profile_name(profile: &str) -> Result<()> {
  if profile.trim().is_empty() || profile.contains('/') {
    bail!("profile name must be non-empty and must not contain '/'");
  }
  Ok(())
}

fn gateway_profile_base_url(cfg: &Config, profile: Option<&str>) -> String {
  match profile {
    Some(profile) => format!("http://{}:{}/{profile}/v1", cfg.server.host, cfg.server.port),
    None => format!("http://{}:{}/v1", cfg.server.host, cfg.server.port),
  }
}

fn timestamp() -> Result<String> {
  let now = time::OffsetDateTime::now_utc();
  let rfc3339 = now.format(&Rfc3339)?;
  Ok(compact_rfc3339_timestamp(&rfc3339))
}

fn compact_rfc3339_timestamp(rfc3339: &str) -> String {
  let compact = rfc3339
    .replace(['-', ':'], "")
    .replace('.', "")
    .trim_end_matches('Z')
    .to_string();
  format!("{compact}Z")
}

fn resolve_home(home: Option<PathBuf>) -> Result<PathBuf> {
  match home {
    Some(home) => Ok(home),
    None => directories::BaseDirs::new()
      .map(|dirs| dirs.home_dir().to_path_buf())
      .ok_or_else(|| anyhow!("cannot resolve home directory")),
  }
}

fn default_gateway_auth_path() -> Result<PathBuf> {
  default_auth_path()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_account(id: &str, provider: &str) -> Account {
    Account {
      id: id.into(),
      provider: provider.into(),
      enabled: true,
      tier: tokn_core::account::AccountTier::Active,
      tags: Vec::new(),
      label: None,
      base_url: None,
      headers: Default::default(),
      auth_type: None,
      username: None,
      api_key: None,
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

  #[test]
  fn default_gateway_auth_path_uses_auth_store_default() {
    assert_eq!(default_gateway_auth_path().unwrap(), default_auth_path().unwrap());
  }

  #[test]
  fn compact_timestamp_preserves_fractional_seconds() {
    assert_eq!(
      compact_rfc3339_timestamp("2026-06-05T10:11:12.123456789Z"),
      "20260605T101112123456789Z"
    );
  }

  #[test]
  fn plan_reconcile_uses_explicit_agent_home() {
    let dir = tempfile::tempdir().unwrap();
    let gateway_config_path = dir.path().join("config.toml");
    let agent_home = dir.path().join("agent-home");
    let opencode_auth_path = agent_home.join(".local/share/opencode/auth.json");
    let opencode_config_path = agent_home.join(".config/opencode/opencode.jsonc");
    std::fs::create_dir_all(opencode_auth_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(opencode_config_path.parent().unwrap()).unwrap();
    std::fs::write(
      &gateway_config_path,
      r#"
[server]
host = "127.0.0.1"
port = 4141
"#,
    )
    .unwrap();
    std::fs::write(
      &opencode_auth_path,
      serde_json::json!({
        "openai": {
          "type": "api",
          "key": "sk-test"
        }
      })
      .to_string(),
    )
    .unwrap();
    std::fs::write(&opencode_config_path, "{\n  // user config\n  \"mcp\": {},\n}\n").unwrap();

    let plan = plan_reconcile(ReconcileRequest {
      agent: AgentId::Opencode,
      profile: None,
      mode: None,
      gateway_config_path: Some(gateway_config_path.clone()),
      agent_home: Some(agent_home),
    })
    .unwrap();

    assert_eq!(plan.agent, AgentId::Opencode);
    assert_eq!(plan.gateway_config_path, gateway_config_path);
    assert_eq!(plan.target_base_url, "http://127.0.0.1:4141/opencode/v1");
    assert_eq!(plan.binding_profile.as_deref(), Some("opencode"));
    assert_eq!(plan.imported_accounts.len(), 1);
    assert_eq!(plan.agent_auth_path.as_deref(), Some(opencode_auth_path.as_path()));
    assert_eq!(plan.edits[0].path, opencode_config_path);
  }

  #[test]
  fn plan_reconcile_falls_back_to_defaults_when_no_accounts_exist() {
    let dir = tempfile::tempdir().unwrap();
    let gateway_config_path = dir.path().join("config.toml");
    let agent_home = dir.path().join("agent-home");
    let opencode_config_path = agent_home.join(".config/opencode/opencode.json");
    std::fs::create_dir_all(opencode_config_path.parent().unwrap()).unwrap();
    std::fs::write(&gateway_config_path, "").unwrap();
    std::fs::write(&opencode_config_path, serde_json::json!({}).to_string()).unwrap();

    let plan = plan_reconcile(ReconcileRequest {
      agent: AgentId::Opencode,
      profile: None,
      mode: None,
      gateway_config_path: Some(gateway_config_path),
      agent_home: Some(agent_home),
    })
    .unwrap();

    assert_eq!(plan.binding_profile, None);
    assert_eq!(plan.target_base_url, "http://127.0.0.1:4141/v1");
  }

  #[test]
  fn validate_profile_name_rejects_empty_and_path_like_names() {
    assert!(validate_profile_name("").is_err());
    assert!(validate_profile_name("   ").is_err());
    assert!(validate_profile_name("agent/profile").is_err());
    assert!(validate_profile_name("agent").is_ok());
  }

  #[test]
  fn upsert_agent_and_profiles_without_imported_accounts_scopes_to_agent_provider() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    upsert_agent_and_profiles(
      &path,
      &AgentId::Opencode,
      Some("opencode"),
      RouteMode::Route,
      &[],
      &[],
      tokn_core::provider::ID_OPENAI,
    )
    .unwrap();

    let (cfg, _) = Config::load(Some(&path)).unwrap();
    let agent = cfg.agents.get("opencode").unwrap();
    assert_eq!(agent.mode, Some(RouteMode::Route));
    assert_eq!(agent.profile.as_deref(), Some("opencode"));
    assert!(agent.sync);
    let profile = cfg.profiles.get("opencode").unwrap();
    assert_eq!(profile.agent_id, Some(AgentId::Opencode));
    assert_eq!(
      profile.providers.as_deref(),
      Some(&[tokn_core::provider::ID_OPENAI.to_string()][..])
    );
    assert_eq!(profile.accounts, None);
  }

  #[test]
  fn upsert_agent_and_profiles_with_imported_accounts_scopes_to_accounts_and_providers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let accounts = vec![sample_account("codex-cli-codex", tokn_core::provider::ID_CODEX)];

    upsert_agent_and_profiles(
      &path,
      &AgentId::CodexCli,
      Some("codex"),
      RouteMode::Route,
      &accounts,
      &[],
      tokn_core::provider::ID_CODEX,
    )
    .unwrap();

    let (cfg, _) = Config::load(Some(&path)).unwrap();
    let agent = cfg.agents.get("codex-cli").unwrap();
    assert_eq!(agent.profile.as_deref(), Some("codex"));
    let profile = cfg.profiles.get("codex").unwrap();
    assert_eq!(profile.agent_id, Some(AgentId::CodexCli));
    assert_eq!(
      profile.providers.as_deref(),
      Some(&[tokn_core::provider::ID_CODEX.to_string()][..])
    );
    assert_eq!(profile.accounts.as_deref(), Some(&["codex-cli-codex".to_string()][..]));
  }

  #[test]
  fn upsert_agent_and_profiles_respects_switch_mode_with_synthetic_profiles() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut openai = sample_account("opencode-openai", tokn_core::provider::ID_OPENAI);
    let mut codex = sample_account("opencode-codex", tokn_core::provider::ID_CODEX);
    openai.tags.push("source:opencode".into());
    codex.tags.push("source:opencode".into());

    upsert_agent_and_profiles(
      &path,
      &AgentId::Opencode,
      Some("opencode"),
      RouteMode::Switch,
      &[openai, codex],
      &[],
      tokn_core::provider::ID_OPENAI,
    )
    .unwrap();

    let (cfg, _) = Config::load(Some(&path)).unwrap();
    assert_eq!(cfg.agents["opencode"].mode, Some(RouteMode::Switch));
    assert_eq!(cfg.profiles["opencode"].mode, Some(RouteMode::Switch));
    assert_eq!(
      cfg.profiles["opencode-openai"].accounts.as_deref(),
      Some(&["opencode-openai".to_string()][..])
    );
    assert_eq!(
      cfg.profiles["opencode-codex"].accounts.as_deref(),
      Some(&["opencode-codex".to_string()][..])
    );
  }

  #[test]
  fn disable_missing_source_accounts_disables_previously_imported_accounts() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.yaml");
    let mut store = AuthStore::load(Some(&auth_path), None).unwrap();
    store.accounts = vec![
      annotate_imported_account(
        sample_account("opencode-openai", tokn_core::provider::ID_OPENAI),
        AgentId::Opencode,
        Path::new("/tmp/opencode-auth.json"),
        "auth.openai",
        "20260604T153012Z",
      ),
      sample_account("manual-openai", tokn_core::provider::ID_OPENAI),
    ];

    disable_missing_source_accounts(&mut store, &AgentId::Opencode, &BTreeSet::new());

    let imported = store.get("opencode-openai").unwrap();
    assert!(!imported.enabled);
    assert!(imported.tags.iter().any(|tag| tag == "source:missing"));
    assert!(store.get("manual-openai").unwrap().enabled);
  }

  #[test]
  fn transferred_account_replaces_an_old_credential_for_the_same_source_provider() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.yaml");
    let source_path = dir.path().join("opencode-auth.json");
    let mut old = annotate_imported_account(
      sample_account("opencode-codex", tokn_core::provider::ID_CODEX),
      AgentId::Opencode,
      &source_path,
      "auth.openai",
      "20260604T153012Z",
    );
    old
      .settings
      .get_mut("import")
      .and_then(toml::Value::as_table_mut)
      .unwrap()
      .insert("source_provider".into(), toml::Value::String("openai".into()));
    let old = mark_gateway_owned(old);
    let mut store = AuthStore::load(Some(&auth_path), None).unwrap();
    store.accounts = vec![old];

    let mut replacement = annotate_imported_account(
      sample_account("opencode-openai", tokn_core::provider::ID_OPENAI),
      AgentId::Opencode,
      &source_path,
      "auth.openai",
      "20260604T153013Z",
    );
    replacement
      .settings
      .get_mut("import")
      .and_then(toml::Value::as_table_mut)
      .unwrap()
      .insert("source_provider".into(), toml::Value::String("openai".into()));

    let desired = merge_transferred_accounts(&store, &AgentId::Opencode, vec![replacement]);
    remove_replaced_gateway_accounts(&mut store, &AgentId::Opencode, &desired);

    assert_eq!(desired.len(), 1);
    assert_eq!(desired[0].id, "opencode-openai");
    assert!(store.accounts.is_empty());
  }

  #[test]
  fn write_edit_creates_parent_directories_for_json_and_toml() {
    let dir = tempfile::tempdir().unwrap();
    let json_path = dir.path().join("nested/auth.json");
    let toml_path = dir.path().join("nested/config.toml");
    let mut doc = toml_edit::DocumentMut::new();
    doc["model_provider"] = toml_edit::value("tokn-router");

    write_edit(&PlannedEdit {
      path: json_path.clone(),
      kind: EditKind::Json(serde_json::json!({"auth_mode": "api_key"})),
      backup: true,
    })
    .unwrap();
    write_edit(&PlannedEdit {
      path: toml_path.clone(),
      kind: EditKind::Toml(doc),
      backup: true,
    })
    .unwrap();

    let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(json_path).unwrap()).unwrap();
    assert_eq!(json["auth_mode"], "api_key");
    assert!(std::fs::read_to_string(toml_path).unwrap().contains("model_provider"));
  }

  #[test]
  fn apply_reconcile_writes_gateway_state_agent_edits_and_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let gateway_config_path = dir.path().join("gateway/config.toml");
    let gateway_auth_path = dir.path().join("gateway/auth.yaml");
    let agent_config_path = dir.path().join("agent/config.json");
    let manifest_path = dir.path().join("manifest.json");
    let mut account = sample_account("opencode-openai", tokn_core::provider::ID_OPENAI);
    account.api_key = Some(tokn_core::util::secret::Secret::new("sk-test".to_string()));
    let plan = ReconcilePlan {
      agent: AgentId::Opencode,
      timestamp: "20260604T153012Z".into(),
      gateway_config_path: gateway_config_path.clone(),
      gateway_auth_path: gateway_auth_path.clone(),
      agent_auth_path: Some(dir.path().join("agent/auth.json")),
      binding_profile: Some("opencode".into()),
      binding_mode: RouteMode::Route,
      target_base_url: "http://127.0.0.1:4141/opencode/v1".into(),
      imported_accounts: vec![account],
      provider_routes: Vec::new(),
      edits: vec![PlannedEdit {
        path: agent_config_path.clone(),
        kind: EditKind::Json(serde_json::json!({"provider": "tokn-router"})),
        backup: true,
      }],
      previous_manifest: None,
    };

    let report = apply_reconcile_to_manifest_path(plan, manifest_path.clone()).unwrap();

    assert_eq!(report.manifest_path, manifest_path);
    assert!(gateway_config_path.exists());
    assert!(gateway_auth_path.exists());
    assert_eq!(
      serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(agent_config_path).unwrap()).unwrap()
        ["provider"],
      "tokn-router"
    );
    let manifest: MigrationManifest = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
    assert!(manifest.completed);
    assert_eq!(manifest.imported_account_ids, vec!["opencode-openai"]);
    assert_eq!(manifest.profile.as_deref(), Some("opencode"));
    assert!(report.files.iter().any(|file| file.original == gateway_config_path));
    assert!(report.files.iter().any(|file| file.original == gateway_auth_path));
  }

  #[test]
  fn apply_reconcile_leaves_in_progress_manifest_if_later_edit_fails() {
    let dir = tempfile::tempdir().unwrap();
    let gateway_config_path = dir.path().join("gateway/config.toml");
    let gateway_auth_path = dir.path().join("gateway/auth.yaml");
    let edit_path = dir.path().join("agent/config.json");
    let manifest_path = dir.path().join("manifest.json");
    std::fs::write(dir.path().join("agent"), "not a directory").unwrap();
    let mut account = sample_account("opencode-openai", tokn_core::provider::ID_OPENAI);
    account.api_key = Some(tokn_core::util::secret::Secret::new("sk-test".to_string()));
    let plan = ReconcilePlan {
      agent: AgentId::Opencode,
      timestamp: "20260604T153012Z".into(),
      gateway_config_path,
      gateway_auth_path,
      agent_auth_path: Some(dir.path().join("agent/auth.json")),
      binding_profile: Some("opencode".into()),
      binding_mode: RouteMode::Route,
      target_base_url: "http://127.0.0.1:4141/opencode/v1".into(),
      imported_accounts: vec![account],
      provider_routes: Vec::new(),
      edits: vec![PlannedEdit {
        path: edit_path.clone(),
        kind: EditKind::Json(serde_json::json!({"provider": "tokn-router"})),
        backup: true,
      }],
      previous_manifest: None,
    };

    let err = apply_reconcile_to_manifest_path(plan, manifest_path.clone()).unwrap_err();

    assert!(format!("{err:#}").contains("creating"));
    let manifest: MigrationManifest = serde_json::from_str(&std::fs::read_to_string(manifest_path).unwrap()).unwrap();
    assert!(!manifest.completed);
    assert!(manifest.files.iter().any(|file| file.original == edit_path));
  }

  #[test]
  fn opencode_transfer_survives_sync_and_unlink_exports_latest_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let agent_home = dir.path().join("home");
    let gateway_config_path = dir.path().join("gateway/config.toml");
    let gateway_auth_path = dir.path().join("gateway/auth.yaml");
    let opencode_config_path = agent_home.join(".config/opencode/opencode.jsonc");
    let opencode_auth_path = agent_home.join(".local/share/opencode/auth.json");
    let first_manifest_path = dir.path().join("20260604T153012Z-opencode.json");
    let second_manifest_path = dir.path().join("20260604T153013Z-opencode.json");
    std::fs::create_dir_all(opencode_config_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(opencode_auth_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(gateway_config_path.parent().unwrap()).unwrap();

    let original_gateway_config = r#"[server]
host = "127.0.0.1"
port = 4141

[profiles.existing]
providers = ["openai"]

[profiles.opencode-user]
providers = ["anthropic"]
"#;
    let original_opencode_config = r#"{
  // Preserve the user's global model choice.
  "model": "openai/gpt-5",
  "provider": {
    "anthropic": {
      "options": { "apiKey": "leave-alone" },
    },
  },
}
"#;
    std::fs::write(&gateway_config_path, original_gateway_config).unwrap();
    std::fs::write(&opencode_config_path, original_opencode_config).unwrap();
    std::fs::write(
      &opencode_auth_path,
      serde_json::to_vec_pretty(&serde_json::json!({
        "openai": {
          "type": "api",
          "key": "sk-original"
        },
        "github-copilot": {
          "type": "oauth",
          "refresh": "ghu-original",
          "access": "tid-original",
          "expires": 0
        },
        "anthropic": {
          "type": "api",
          "key": "anthropic-keep"
        }
      }))
      .unwrap(),
    )
    .unwrap();

    let request = || ReconcileRequest {
      agent: AgentId::Opencode,
      profile: None,
      mode: None,
      gateway_config_path: Some(gateway_config_path.clone()),
      agent_home: Some(agent_home.clone()),
    };
    let mut first_plan = plan_reconcile_with_gateway_auth_path(request(), gateway_auth_path.clone()).unwrap();
    first_plan.timestamp = "20260604T153012Z".into();
    assert_eq!(first_plan.imported_accounts.len(), 2);
    assert_eq!(first_plan.provider_routes.len(), 2);
    apply_reconcile_to_manifest_path(first_plan, first_manifest_path.clone()).unwrap();

    let linked_auth: Value = serde_json::from_str(&std::fs::read_to_string(&opencode_auth_path).unwrap()).unwrap();
    assert!(linked_auth.get("openai").is_none());
    assert!(linked_auth.get("github-copilot").is_none());
    assert_eq!(linked_auth["anthropic"]["key"], "anthropic-keep");

    let linked_config = crate::jsonc::read_jsonc(&opencode_config_path).unwrap();
    assert_eq!(linked_config["model"], "openai/gpt-5");
    assert_eq!(
      linked_config["provider"]["openai"]["options"]["baseURL"],
      "http://127.0.0.1:4141/opencode-openai/v1"
    );
    assert_eq!(
      linked_config["provider"]["github-copilot"]["options"]["baseURL"],
      "http://127.0.0.1:4141/opencode-github-copilot/v1"
    );
    assert!(std::fs::read_to_string(&opencode_config_path)
      .unwrap()
      .contains("Preserve the user's global model choice"));

    let (linked_gateway_config, _) = Config::load(Some(&gateway_config_path)).unwrap();
    assert!(linked_gateway_config.profiles.contains_key("opencode-user"));
    assert_eq!(
      linked_gateway_config.profiles["opencode-openai"].accounts.as_deref(),
      Some(&["opencode-openai".to_string()][..])
    );
    assert_eq!(
      linked_gateway_config.profiles["opencode-github-copilot"]
        .accounts
        .as_deref(),
      Some(&["opencode-github-copilot".to_string()][..])
    );

    let mut store = AuthStore::load(Some(&gateway_auth_path), None).unwrap();
    assert_eq!(
      store.get("opencode-openai").unwrap().api_key.as_ref().unwrap().expose(),
      "sk-original"
    );
    assert_eq!(
      store
        .get("opencode-github-copilot")
        .unwrap()
        .refresh_token
        .as_ref()
        .unwrap()
        .expose(),
      "ghu-original"
    );
    store.get_mut("opencode-openai").unwrap().api_key = Some(tokn_core::util::secret::Secret::new("sk-latest".into()));
    let copilot = store.get_mut("opencode-github-copilot").unwrap();
    copilot.refresh_token = Some(tokn_core::util::secret::Secret::new("ghu-latest".into()));
    copilot.access_token = Some(tokn_core::util::secret::Secret::new("tid-latest".into()));
    copilot.access_token_expires_at = Some(222);
    store.save().unwrap();

    let mut second_plan = plan_reconcile_with_gateway_auth_path(request(), gateway_auth_path.clone()).unwrap();
    second_plan.timestamp = "20260604T153013Z".into();
    second_plan.previous_manifest = Some(first_manifest_path.clone());
    assert_eq!(second_plan.imported_accounts.len(), 2);
    assert!(second_plan.imported_accounts.iter().all(|account| account.enabled));
    apply_reconcile_to_manifest_path(second_plan, second_manifest_path.clone()).unwrap();

    let synced_store = AuthStore::load(Some(&gateway_auth_path), None).unwrap();
    assert_eq!(synced_store.accounts.len(), 2);
    assert_eq!(
      synced_store
        .get("opencode-github-copilot")
        .unwrap()
        .refresh_token
        .as_ref()
        .unwrap()
        .expose(),
      "ghu-latest"
    );
    let second_manifest = manifest::read_manifest(&second_manifest_path).unwrap();
    assert_eq!(
      second_manifest.previous_manifest.as_deref(),
      Some(first_manifest_path.as_path())
    );

    let err = unlink(UnlinkRequest {
      agent: AgentId::Opencode,
      backup_id: Some(first_manifest_path.display().to_string()),
    })
    .unwrap_err();
    assert!(err.to_string().contains("newer active successor"));

    unlink(UnlinkRequest {
      agent: AgentId::Opencode,
      backup_id: Some(second_manifest_path.display().to_string()),
    })
    .unwrap();

    assert_eq!(
      std::fs::read_to_string(&opencode_config_path).unwrap(),
      original_opencode_config
    );
    assert_eq!(
      std::fs::read_to_string(&gateway_config_path).unwrap(),
      original_gateway_config
    );
    assert!(!gateway_auth_path.exists());
    let restored_auth: Value = serde_json::from_str(&std::fs::read_to_string(&opencode_auth_path).unwrap()).unwrap();
    assert_eq!(restored_auth["openai"]["type"], "api");
    assert_eq!(restored_auth["openai"]["key"], "sk-latest");
    assert_eq!(restored_auth["github-copilot"]["type"], "oauth");
    assert_eq!(restored_auth["github-copilot"]["refresh"], "ghu-latest");
    assert_eq!(restored_auth["github-copilot"]["access"], "ghu-latest");
    assert_eq!(restored_auth["github-copilot"]["expires"], 0);
    assert_eq!(restored_auth["anthropic"]["key"], "anthropic-keep");
    assert!(manifest::read_manifest(&first_manifest_path).unwrap().unlinked);
    assert!(manifest::read_manifest(&second_manifest_path).unwrap().unlinked);
  }

  #[test]
  fn unlink_restores_file_from_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("config.toml");
    let backup = dir.path().join("config.toml.bak.20260604T153012Z");
    let manifest_path = dir.path().join("20260604T153012Z-codex-cli.json");
    std::fs::write(&original, "mutated").unwrap();
    std::fs::write(&backup, "original").unwrap();
    let manifest = MigrationManifest {
      version: 1,
      completed: true,
      agent: AgentId::CodexCli,
      timestamp: "20260604T153012Z".into(),
      profile: Some("codex".into()),
      target_base_url: "http://127.0.0.1:4141/codex/v1".into(),
      gateway_auth_path: None,
      agent_auth_path: None,
      provider_routes: Vec::new(),
      previous_manifest: None,
      unlinked: false,
      imported_account_ids: vec!["codex-cli-codex".into()],
      files: vec![FileBackup {
        original: original.clone(),
        backup: Some(backup),
        existed: true,
        created_by_migration: false,
      }],
    };
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let report = unlink(UnlinkRequest {
      agent: AgentId::CodexCli,
      backup_id: Some(manifest_path.display().to_string()),
    })
    .unwrap();

    assert_eq!(std::fs::read_to_string(original).unwrap(), "original");
    assert_eq!(report.actions.len(), 1);
  }

  #[test]
  fn unlink_removes_files_created_by_migration() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("created.toml");
    let manifest_path = dir.path().join("20260604T153012Z-opencode.json");
    std::fs::write(&original, "created").unwrap();
    let manifest = MigrationManifest {
      version: 1,
      completed: true,
      agent: AgentId::Opencode,
      timestamp: "20260604T153012Z".into(),
      profile: Some("opencode".into()),
      target_base_url: "http://127.0.0.1:4141/opencode/v1".into(),
      gateway_auth_path: None,
      agent_auth_path: None,
      provider_routes: Vec::new(),
      previous_manifest: None,
      unlinked: false,
      imported_account_ids: vec!["opencode-openai".into()],
      files: vec![FileBackup {
        original: original.clone(),
        backup: None,
        existed: false,
        created_by_migration: true,
      }],
    };
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let report = unlink(UnlinkRequest {
      agent: AgentId::Opencode,
      backup_id: Some(manifest_path.display().to_string()),
    })
    .unwrap();

    assert!(!original.exists());
    assert!(matches!(report.actions.as_slice(), [FileAction::Removed(path)] if path == &original));
  }

  #[test]
  fn unlink_rejects_manifest_for_different_agent() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("20260604T153012Z-codex-cli.json");
    let manifest = MigrationManifest {
      version: 1,
      completed: true,
      agent: AgentId::CodexCli,
      timestamp: "20260604T153012Z".into(),
      profile: Some("codex".into()),
      target_base_url: "http://127.0.0.1:4141/codex/v1".into(),
      gateway_auth_path: None,
      agent_auth_path: None,
      provider_routes: Vec::new(),
      previous_manifest: None,
      unlinked: false,
      imported_account_ids: vec!["codex-cli-codex".into()],
      files: Vec::new(),
    };
    std::fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let err = unlink(UnlinkRequest {
      agent: AgentId::Opencode,
      backup_id: Some(manifest_path.display().to_string()),
    })
    .unwrap_err();

    assert!(err.to_string().contains("not opencode"));
  }
}
