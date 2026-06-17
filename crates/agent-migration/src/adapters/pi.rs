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
    home.join(".local/share/pi/auth.json")
  }

  fn config_path(&self, home: &Path) -> PathBuf {
    home.join(".config/pi/config.json")
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
    let config_path = self.config_path(home);
    let mut json = if config_path.exists() {
      let raw = std::fs::read_to_string(&config_path).with_context(|| format!("reading {}", config_path.display()))?;
      serde_json::from_str(&raw).with_context(|| format!("parsing {}", config_path.display()))?
    } else {
      Value::Object(serde_json::Map::new())
    };
    rewrite_config(&mut json, base_url);
    Ok(vec![PlannedEdit {
      path: config_path,
      kind: EditKind::Json(json),
    }])
  }
}

fn rewrite_config(json: &mut Value, target_base_url: &str) {
  if !json.is_object() {
    *json = Value::Object(serde_json::Map::new());
  }
  let obj = json.as_object_mut().expect("object ensured");
  obj.insert("provider".into(), Value::String("tokn-router".into()));
  obj.insert("base_url".into(), Value::String(target_base_url.into()));
  obj.insert("api_key".into(), Value::String("tokn-router".into()));
  obj.insert("env_key".into(), Value::String("OPENAI_API_KEY".into()));

  let providers = obj
    .entry("model_providers")
    .or_insert_with(|| Value::Object(serde_json::Map::new()));
  if !providers.is_object() {
    *providers = Value::Object(serde_json::Map::new());
  }
  providers.as_object_mut().expect("object ensured").insert(
    "tokn-router".into(),
    serde_json::json!({
      "name": "tokn-router",
      "base_url": target_base_url,
      "api_key": "tokn-router",
      "env_key": "OPENAI_API_KEY"
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
  fn config_rewrite_preserves_existing_keys() {
    let mut json = serde_json::json!({"model": "gpt-5"});
    rewrite_config(&mut json, "http://127.0.0.1:4141/pi/v1");
    assert_eq!(json["model"], "gpt-5");
    assert_eq!(json["provider"], "tokn-router");
    assert_eq!(json["base_url"], "http://127.0.0.1:4141/pi/v1");
    assert_eq!(
      json["model_providers"]["tokn-router"]["base_url"],
      "http://127.0.0.1:4141/pi/v1"
    );
  }

  #[test]
  fn adapter_reads_existing_auth_and_config() {
    let dir = tempfile::tempdir().unwrap();
    let adapter = PiAdapter;
    let auth_path = adapter.auth_path(dir.path());
    let config_path = adapter.config_path(dir.path());
    std::fs::create_dir_all(auth_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(&auth_path, serde_json::json!({"api_key": "sk-pi"}).to_string()).unwrap();
    std::fs::write(&config_path, serde_json::json!({"model": "gpt-5"}).to_string()).unwrap();

    let accounts = adapter.discover_accounts(dir.path(), "20260604T153012Z").unwrap();
    let edits = adapter
      .rewrite_config(dir.path(), "http://127.0.0.1:4141/pi/v1")
      .unwrap();

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "pi-openai");
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].path, config_path);
  }
}
