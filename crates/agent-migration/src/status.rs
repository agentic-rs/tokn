use crate::adapter::{adapter_for, source_provider_id, supported_agents};
use crate::jsonc::read_jsonc;
use crate::reconcile::{imported_account_ids, is_source_managed_account};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokn_auth::AuthStore;
use tokn_config::{Config, RouteMode};
use tokn_core::AgentId;

#[derive(Debug, Clone)]
pub struct AgentStatus {
  pub agent: AgentId,
  pub supported: bool,
  pub detected: bool,
  pub auth_path: PathBuf,
  pub config_path: PathBuf,
  pub binding: Option<AgentBindingStatus>,
  pub imported_account_ids: Vec<String>,
  pub drifted: bool,
}

#[derive(Debug, Clone)]
pub struct AgentBindingStatus {
  pub profile: Option<String>,
  pub mode: RouteMode,
  pub sync: bool,
}

pub fn list_agents(
  gateway_config_path: Option<&Path>,
  gateway_auth_path: Option<&Path>,
  agent_home: Option<&Path>,
) -> Result<Vec<AgentStatus>> {
  let (cfg, config_path) = Config::load(gateway_config_path)?;
  let auth_path = resolve_gateway_auth_path(gateway_auth_path)?;
  let store = AuthStore::load(Some(&auth_path), Some(&config_path))?;
  let home = resolve_home(agent_home)?;
  let mut statuses = supported_agents()
    .into_iter()
    .map(|agent| status_for_agent(&cfg, &store, &home, agent))
    .collect::<Result<Vec<_>>>()?;
  statuses.sort_by(|a, b| a.agent.as_str().cmp(b.agent.as_str()));
  Ok(statuses)
}

pub fn show_agent(
  gateway_config_path: Option<&Path>,
  gateway_auth_path: Option<&Path>,
  agent_home: Option<&Path>,
  agent: AgentId,
) -> Result<AgentStatus> {
  let (cfg, config_path) = Config::load(gateway_config_path)?;
  let auth_path = resolve_gateway_auth_path(gateway_auth_path)?;
  let store = AuthStore::load(Some(&auth_path), Some(&config_path))?;
  let home = resolve_home(agent_home)?;
  status_for_agent(&cfg, &store, &home, agent)
}

fn status_for_agent(cfg: &Config, store: &AuthStore, home: &Path, agent: AgentId) -> Result<AgentStatus> {
  let supported = adapter_for(&agent).is_some();
  let adapter = adapter_for(&agent);
  let (auth_path, config_path, detected, drifted) = if let Some(adapter) = adapter {
    let auth_path = adapter.auth_path(home);
    let config_path = adapter.config_path(home);
    let detected = auth_path.exists() || config_path.exists();
    let drifted = config_points_at_gateway(&config_path, &agent, cfg, store);
    (auth_path, config_path, detected, drifted)
  } else {
    let base = home.join(format!(".unsupported/{}", agent.as_str()));
    (base.join("auth"), base.join("config"), false, false)
  };
  let binding = cfg.agents.get(agent.as_str()).map(|binding| AgentBindingStatus {
    profile: binding.profile.clone(),
    mode: binding.mode.unwrap_or(RouteMode::Route),
    sync: binding.sync,
  });
  let imported_account_ids = imported_account_ids(store, &agent);
  Ok(AgentStatus {
    agent,
    supported,
    detected,
    auth_path,
    config_path,
    binding,
    imported_account_ids,
    drifted,
  })
}

fn config_points_at_gateway(config_path: &Path, agent: &AgentId, cfg: &Config, store: &AuthStore) -> bool {
  if !config_path.exists() {
    return false;
  }
  let default_base = format!("http://{}:{}/v1", cfg.server.host, cfg.server.port);
  let Some(binding) = cfg.agents.get(agent.as_str()) else {
    return false;
  };
  let expected = match binding.profile.as_deref() {
    Some(profile) => format!("http://{}:{}/{profile}/v1", cfg.server.host, cfg.server.port),
    None => default_base.clone(),
  };
  match agent {
    AgentId::Opencode => {
      let Ok(json) = read_jsonc(config_path) else {
        return false;
      };
      let accounts = binding
        .profile
        .as_deref()
        .and_then(|profile| cfg.profiles.get(profile))
        .and_then(|profile| profile.accounts.as_deref())
        .into_iter()
        .flatten()
        .filter_map(|id| store.get(id))
        .filter(|account| is_source_managed_account(account, agent))
        .collect::<Vec<_>>();
      if accounts.is_empty() {
        return provider_base_url(&json, tokn_core::provider::ID_OPENAI) == Some(expected.as_str());
      }
      accounts.iter().all(|account| {
        let Some(source_provider) = source_provider_id(account) else {
          return false;
        };
        let expected = binding
          .profile
          .as_deref()
          .map(|profile| {
            format!(
              "http://{}:{}/{profile}-{}/v1",
              cfg.server.host, cfg.server.port, account.provider
            )
          })
          .unwrap_or_else(|| default_base.clone());
        provider_base_url(&json, source_provider) == Some(expected.as_str())
      })
    }
    AgentId::CodexCli => std::fs::read_to_string(config_path)
      .ok()
      .and_then(|raw| raw.parse::<toml_edit::DocumentMut>().ok())
      .and_then(|doc| {
        doc["model_providers"]["tokn-router"]["base_url"]
          .as_str()
          .map(str::to_string)
      })
      .map(|value| value == expected)
      .unwrap_or(false),
    _ => false,
  }
}

fn provider_base_url<'a>(json: &'a serde_json::Value, provider: &str) -> Option<&'a str> {
  json
    .get("provider")?
    .get(provider)?
    .get("options")?
    .get("baseURL")?
    .as_str()
}

fn resolve_home(agent_home: Option<&Path>) -> Result<PathBuf> {
  match agent_home {
    Some(home) => Ok(home.to_path_buf()),
    None => directories::BaseDirs::new()
      .map(|dirs| dirs.home_dir().to_path_buf())
      .ok_or_else(|| anyhow::anyhow!("cannot resolve home directory")),
  }
}

fn resolve_gateway_auth_path(gateway_auth_path: Option<&Path>) -> Result<PathBuf> {
  match gateway_auth_path {
    Some(path) => Ok(path.to_path_buf()),
    None => tokn_auth::default_auth_path(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_account(id: &str, provider: &str) -> tokn_config::Account {
    tokn_config::Account {
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
  fn list_agents_reports_binding_detection_and_imported_accounts() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let auth_path = dir.path().join("auth.yaml");
    let home = dir.path().join("home");
    std::fs::write(
      &config_path,
      r#"
[agents.opencode]
profile = "work"
mode = "route"
sync = true

[profiles.work]
agent_id = "opencode"
providers = ["openai"]
accounts = ["opencode-openai"]
"#,
    )
    .unwrap();
    let opencode_config = home.join(".config/opencode/opencode.jsonc");
    std::fs::create_dir_all(opencode_config.parent().unwrap()).unwrap();
    std::fs::write(
      &opencode_config,
      r#"
{
  // opencode may store this as JSONC.
  "provider": {
    "openai": {
      "options": {
        "baseURL": "http://127.0.0.1:4141/work-openai/v1",
      },
    },
  },
}
"#,
    )
    .unwrap();
    let mut store = AuthStore::load(Some(&auth_path), Some(&config_path)).unwrap();
    let mut account = sample_account("opencode-openai", "openai");
    account.tags.push("source:opencode".into());
    let mut import = toml::Table::new();
    import.insert("source_agent".into(), toml::Value::String("opencode".into()));
    import.insert("source_provider".into(), toml::Value::String("openai".into()));
    account.settings.insert("import".into(), toml::Value::Table(import));
    store.upsert(account);
    let mut historical = sample_account("opencode-codex", "codex");
    historical.enabled = false;
    historical.tags.push("source:opencode".into());
    let mut historical_import = toml::Table::new();
    historical_import.insert("source_agent".into(), toml::Value::String("opencode".into()));
    historical_import.insert("source_provider".into(), toml::Value::String("openai".into()));
    historical
      .settings
      .insert("import".into(), toml::Value::Table(historical_import));
    store.upsert(historical);
    store.save().unwrap();

    let statuses = list_agents(Some(&config_path), Some(&auth_path), Some(&home)).unwrap();
    let opencode = statuses
      .iter()
      .find(|status| status.agent == AgentId::Opencode)
      .unwrap();
    assert!(opencode.detected);
    assert_eq!(opencode.config_path, opencode_config);
    assert!(opencode.drifted);
    assert_eq!(opencode.binding.as_ref().unwrap().profile.as_deref(), Some("work"));
    assert_eq!(opencode.imported_account_ids, vec!["opencode-codex", "opencode-openai"]);
  }

  #[test]
  fn show_agent_reports_unbound_defaults_case() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let auth_path = dir.path().join("auth.yaml");
    let home = dir.path().join("home");
    let status = show_agent(Some(&config_path), Some(&auth_path), Some(&home), AgentId::CodexCli).unwrap();
    assert_eq!(status.agent, AgentId::CodexCli);
    assert!(status.binding.is_none());
    assert!(status.imported_account_ids.is_empty());
  }

  #[test]
  fn imported_account_helper_matches_source_markers() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.yaml");
    let mut store = AuthStore::load(Some(&auth_path), None).unwrap();
    let mut account = sample_account("x", "openai");
    account.tags.push("source:opencode".into());
    let mut import = toml::Table::new();
    import.insert("source_agent".into(), toml::Value::String("opencode".into()));
    account.settings.insert("import".into(), toml::Value::Table(import));
    store.upsert(account.clone());
    assert!(crate::reconcile::is_source_managed_account(
      &account,
      &AgentId::Opencode
    ));
    assert_eq!(imported_account_ids(&store, &AgentId::Opencode), vec!["x"]);
  }
}
