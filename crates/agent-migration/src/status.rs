use crate::adapter::{adapter_for, supported_agents};
use crate::jsonc::read_json_or_jsonc;
use crate::reconcile::imported_account_ids;
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
    let drifted = config_points_at_gateway(&config_path, &agent, cfg);
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

fn config_points_at_gateway(config_path: &Path, agent: &AgentId, cfg: &Config) -> bool {
  if !config_path.exists() {
    return false;
  }
  let default_base = format!("http://{}:{}/v1", cfg.server.host, cfg.server.port);
  let Some(binding) = cfg.agents.get(agent.as_str()) else {
    return false;
  };
  let expected = match binding.profile.as_deref() {
    Some(profile) => format!("http://{}:{}/{profile}/v1", cfg.server.host, cfg.server.port),
    None => default_base,
  };
  match agent {
    AgentId::Opencode => read_json_or_jsonc(config_path)
      .ok()
      .and_then(|json| {
        json
          .get("provider")?
          .get("tokn-router")?
          .get("options")?
          .get("baseURL")?
          .as_str()
          .map(str::to_string)
      })
      .map(|value| value == expected)
      .unwrap_or(false),
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
    "tokn-router": {
      "options": {
        "baseURL": "http://127.0.0.1:4141/work/v1",
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
    account.settings.insert("import".into(), toml::Value::Table(import));
    store.upsert(account);
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
    assert_eq!(opencode.imported_account_ids, vec!["opencode-openai"]);
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
