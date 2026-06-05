use crate::agent::AgentKind;
use crate::migration::{EditKind, MigrationPlan, PlannedEdit};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokn_config::{Account, AuthType};
use tokn_core::account::AccountTier;
use tokn_core::util::secret::Secret;

pub(crate) fn plan(timestamp: &str, profile: &str, target_base_url: &str, home: PathBuf) -> Result<MigrationPlan> {
  let config_path = home.join(".config/opencode/opencode.json");
  let mut json = if config_path.exists() {
    let raw = std::fs::read_to_string(&config_path).with_context(|| format!("reading {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", config_path.display()))?
  } else {
    Value::Object(serde_json::Map::new())
  };
  let imported_accounts = accounts_from_json(&json);
  rewrite_config(&mut json, target_base_url);

  Ok(MigrationPlan::new(
    AgentKind::Opencode,
    timestamp,
    profile,
    target_base_url,
    imported_accounts,
    vec![PlannedEdit {
      path: config_path,
      kind: EditKind::Json(json),
    }],
  ))
}

fn rewrite_config(json: &mut Value, target_base_url: &str) {
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

fn accounts_from_json(json: &Value) -> Vec<Account> {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rewrite_preserves_mcp() {
    let mut json = serde_json::json!({"mcp": {"x": true}});
    rewrite_config(&mut json, "http://127.0.0.1:4141/opencode/v1");
    assert_eq!(json["mcp"]["x"], true);
    assert_eq!(json["provider"], "tokn-router");
    assert_eq!(
      json["providers"]["tokn-router"]["options"]["baseURL"],
      "http://127.0.0.1:4141/opencode/v1"
    );
  }

  #[test]
  fn accounts_from_json_finds_nested_api_key() {
    let json = serde_json::json!({
      "provider": {
        "openai": {
          "options": {
            "apiKey": "sk-test"
          }
        }
      }
    });

    let accounts = accounts_from_json(&json);

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "opencode-openai");
    assert_eq!(accounts[0].provider, tokn_core::provider::ID_OPENAI);
    assert_eq!(accounts[0].api_key.as_ref().unwrap().expose(), "sk-test");
  }

  #[test]
  fn accounts_from_json_ignores_router_placeholder_key() {
    let json = serde_json::json!({
      "providers": {
        "tokn-router": {
          "options": {
            "apiKey": "tokn-router"
          }
        }
      }
    });

    assert!(accounts_from_json(&json).is_empty());
  }

  #[test]
  fn plan_reads_existing_config_imports_account_and_rewrites_provider() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".config/opencode/opencode.json");
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(
      &config_path,
      serde_json::json!({
        "mcp": {"x": true},
        "providers": {
          "openai": {
            "options": {
              "apiKey": "sk-test"
            }
          }
        }
      })
      .to_string(),
    )
    .unwrap();

    let plan = plan(
      "20260604T153012Z",
      "opencode",
      "http://127.0.0.1:4141/opencode/v1",
      dir.path().to_path_buf(),
    )
    .unwrap();

    assert_eq!(plan.agent, AgentKind::Opencode);
    assert_eq!(plan.profile, "opencode");
    assert_eq!(plan.imported_accounts.len(), 1);
    assert_eq!(plan.edits.len(), 1);
    assert_eq!(plan.edits[0].path, config_path);
  }
}
