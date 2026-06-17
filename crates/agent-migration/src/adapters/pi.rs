use crate::adapter::AgentAdapter;
use crate::reconcile::{annotate_imported_account, EditKind, PlannedEdit};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokn_config::{Account, AuthType};
use tokn_core::account::AccountTier;
use tokn_core::provider::ID_OPENAI;
use tokn_core::util::secret::Secret;
use tokn_core::AgentId;

pub(crate) struct PiAdapter;

impl AgentAdapter for PiAdapter {
  fn default_provider_id(&self) -> &'static str {
    ID_OPENAI
  }

  fn auth_path(&self, home: &Path) -> PathBuf {
    pi_agent_dir(home).join("auth.json")
  }

  fn config_path(&self, home: &Path) -> PathBuf {
    pi_agent_dir(home).join("settings.json")
  }

  fn discover_accounts(&self, home: &Path, timestamp: &str) -> Result<Vec<Account>> {
    let auth_path = self.auth_path(home);
    if !auth_path.exists() {
      return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&auth_path).with_context(|| format!("reading {}", auth_path.display()))?;
    let json = serde_json::from_str(&raw).with_context(|| format!("parsing {}", auth_path.display()))?;
    Ok(accounts_from_auth_json(&json, &auth_path, timestamp))
  }

  fn rewrite_config(&self, home: &Path, base_url: &str) -> Result<Vec<PlannedEdit>> {
    let settings_path = self.config_path(home);
    let models_path = pi_agent_dir(home).join("models.json");

    let settings = if settings_path.exists() {
      let raw =
        std::fs::read_to_string(&settings_path).with_context(|| format!("reading {}", settings_path.display()))?;
      serde_json::from_str(&raw).with_context(|| format!("parsing {}", settings_path.display()))?
    } else {
      Value::Object(serde_json::Map::new())
    };
    let default_provider = default_provider(&settings).map(str::to_string);

    let mut models = if models_path.exists() {
      let raw = std::fs::read_to_string(&models_path).with_context(|| format!("reading {}", models_path.display()))?;
      serde_json::from_str(&raw).with_context(|| format!("parsing {}", models_path.display()))?
    } else {
      Value::Object(serde_json::Map::new())
    };
    rewrite_models(&mut models, default_provider.as_deref(), base_url);

    Ok(vec![
      PlannedEdit {
        path: settings_path,
        kind: EditKind::Json(settings),
      },
      PlannedEdit {
        path: models_path,
        kind: EditKind::Json(models),
      },
    ])
  }
}

fn pi_agent_dir(home: &Path) -> PathBuf {
  home.join(".pi/agent")
}

fn default_provider(settings: &Value) -> Option<&str> {
  settings.get("defaultProvider").and_then(Value::as_str)
}

fn rewrite_models(json: &mut Value, provider_name: Option<&str>, target_base_url: &str) {
  let Some(provider_name) = provider_name.filter(|provider| !provider.trim().is_empty()) else {
    return;
  };
  if !json.is_object() {
    *json = Value::Object(serde_json::Map::new());
  }
  let obj = json.as_object_mut().expect("object ensured");
  let providers = obj
    .entry("providers")
    .or_insert_with(|| Value::Object(serde_json::Map::new()));
  if !providers.is_object() {
    *providers = Value::Object(serde_json::Map::new());
  }
  providers.as_object_mut().expect("object ensured").insert(
    provider_name.into(),
    serde_json::json!({
      "baseUrl": target_base_url,
      "apiKey": "tokn-router"
    }),
  );
}

fn accounts_from_auth_json(json: &Value, auth_path: &Path, timestamp: &str) -> Vec<Account> {
  account_from_auth_json(json)
    .map(|account| annotate_imported_account(account, AgentId::Pi, auth_path, "auth.openai", timestamp))
    .into_iter()
    .collect()
}

fn account_from_auth_json(json: &Value) -> Option<Account> {
  let api_key = json
    .get("OPENAI_API_KEY")
    .or_else(|| json.get("openai_api_key"))
    .or_else(|| json.get("api_key"))
    .and_then(Value::as_str)?
    .trim();
  if api_key.is_empty() {
    return None;
  }
  Some(Account {
    id: "pi-openai".into(),
    provider: ID_OPENAI.into(),
    enabled: true,
    tier: AccountTier::Active,
    tags: vec!["agent-migrated".into(), "pi".into()],
    label: Some("Pi migration".into()),
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
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn account_imports_openai_api_key() {
    let account = account_from_auth_json(&serde_json::json!({"OPENAI_API_KEY": "sk-pi"})).unwrap();
    assert_eq!(account.id, "pi-openai");
    assert_eq!(account.provider, ID_OPENAI);
    assert_eq!(account.api_key.unwrap().expose(), "sk-pi");
  }

  #[test]
  fn account_import_ignores_missing_or_empty_key() {
    assert!(account_from_auth_json(&serde_json::json!({})).is_none());
    assert!(account_from_auth_json(&serde_json::json!({"api_key": "  "})).is_none());
  }

  #[test]
  fn default_provider_reads_existing_selection_without_inventing_one() {
    assert_eq!(
      default_provider(&serde_json::json!({"defaultProvider": "openai"})),
      Some("openai")
    );
    assert_eq!(default_provider(&serde_json::json!({})), None);
  }

  #[test]
  fn models_rewrite_points_existing_default_provider_at_router() {
    let mut json = serde_json::json!({"providers": {"other": {"baseUrl": "http://example.test"}}});
    rewrite_models(&mut json, Some("openai"), "http://127.0.0.1:4141/v1");
    assert_eq!(json["providers"]["other"]["baseUrl"], "http://example.test");
    assert_eq!(json["providers"]["openai"]["baseUrl"], "http://127.0.0.1:4141/v1");
    assert_eq!(json["providers"]["openai"]["apiKey"], "tokn-router");
  }

  #[test]
  fn models_rewrite_without_default_provider_is_noop() {
    let mut json = serde_json::json!({"providers": {"other": {"baseUrl": "http://example.test"}}});
    rewrite_models(&mut json, None, "http://127.0.0.1:4141/v1");
    assert!(json["providers"].get("openai").is_none());
  }

  #[test]
  fn adapter_reads_existing_auth_and_config() {
    let dir = tempfile::tempdir().unwrap();
    let adapter = PiAdapter;
    let auth_path = adapter.auth_path(dir.path());
    let settings_path = adapter.config_path(dir.path());
    let models_path = pi_agent_dir(dir.path()).join("models.json");
    std::fs::create_dir_all(auth_path.parent().unwrap()).unwrap();
    std::fs::write(&auth_path, serde_json::json!({"api_key": "sk-pi"}).to_string()).unwrap();
    std::fs::write(
      &settings_path,
      serde_json::json!({"theme": "dark", "defaultProvider": "openai", "defaultModel": "gpt-5"}).to_string(),
    )
    .unwrap();
    std::fs::write(&models_path, serde_json::json!({"providers": {}}).to_string()).unwrap();

    let accounts = adapter.discover_accounts(dir.path(), "20260604T153012Z").unwrap();
    let edits = adapter.rewrite_config(dir.path(), "http://127.0.0.1:4141/v1").unwrap();

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "pi-openai");
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().any(|edit| edit.path == settings_path));
    assert!(edits.iter().any(|edit| edit.path == models_path));
  }
}
