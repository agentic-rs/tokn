use crate::adapter::{source_provider_id, AgentAdapter, ProviderRoute};
use crate::jsonc::{parse_cst, set_property};
use crate::reconcile::{annotate_imported_account, EditKind, PlannedEdit};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tokn_config::{Account, AuthType};
use tokn_core::account::AccountTier;
use tokn_core::util::secret::Secret;
use tokn_core::AgentId;

pub(crate) struct OpencodeAdapter;

const OPENCODE_CONFIG_JSON: &str = ".config/opencode/opencode.json";
const OPENCODE_CONFIG_JSONC: &str = ".config/opencode/opencode.jsonc";

impl AgentAdapter for OpencodeAdapter {
  fn default_provider_id(&self) -> &'static str {
    tokn_core::provider::ID_OPENAI
  }

  fn auth_path(&self, home: &Path) -> PathBuf {
    home.join(".local/share/opencode/auth.json")
  }

  fn config_path(&self, home: &Path) -> PathBuf {
    opencode_config_path(home)
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

  fn transfers_credentials(&self) -> bool {
    true
  }

  fn rewrite_config(&self, home: &Path, _base_url: &str, routes: &[ProviderRoute]) -> Result<Vec<PlannedEdit>> {
    let config_path = self.config_path(home);
    let raw = if config_path.exists() {
      std::fs::read_to_string(&config_path).with_context(|| format!("reading {}", config_path.display()))?
    } else {
      "{}\n".to_string()
    };
    let root = parse_cst(&raw, &config_path)?;
    rewrite_config(&root, routes)?;

    let mut edits = vec![PlannedEdit {
      path: config_path,
      kind: EditKind::Jsonc(root.to_string()),
      backup: true,
    }];
    if let Some(auth_edit) = remove_transferred_credentials(&self.auth_path(home), routes)? {
      edits.push(auth_edit);
    }
    Ok(edits)
  }

  fn restore_transferred_credentials(&self, auth_path: &Path, accounts: &[Account]) -> Result<()> {
    restore_transferred_credentials(auth_path, accounts)
  }
}

fn opencode_config_path(home: &Path) -> PathBuf {
  let jsonc = home.join(OPENCODE_CONFIG_JSONC);
  if jsonc.exists() {
    return jsonc;
  }
  let json = home.join(OPENCODE_CONFIG_JSON);
  if json.exists() {
    return json;
  }
  jsonc
}

fn rewrite_config(root: &jsonc_parser::cst::CstRootNode, routes: &[ProviderRoute]) -> Result<()> {
  let Some(obj) = root.object_value() else {
    bail!("OpenCode config must contain a JSON object");
  };
  if obj.get("$schema").is_none() {
    set_property(&obj, "$schema", "https://opencode.ai/config.json");
  }
  let providers = obj.object_value_or_set("provider");
  for route in routes {
    let provider = providers.object_value_or_set(&route.source_provider_id);
    set_property(&provider, "name", format!("tokn-router ({})", route.source_provider_id));
    set_property(&provider, "npm", "@ai-sdk/openai-compatible");
    let options = provider.object_value_or_set("options");
    set_property(&options, "baseURL", route.base_url.clone());
    set_property(&options, "apiKey", "tokn-router");
  }
  Ok(())
}

fn remove_transferred_credentials(auth_path: &Path, routes: &[ProviderRoute]) -> Result<Option<PlannedEdit>> {
  if !auth_path.exists() {
    return Ok(None);
  }
  let raw = std::fs::read_to_string(auth_path).with_context(|| format!("reading {}", auth_path.display()))?;
  let mut json: Value = serde_json::from_str(&raw).with_context(|| format!("parsing {}", auth_path.display()))?;
  let Some(auth) = json.as_object_mut() else {
    bail!("{} must contain a JSON object", auth_path.display());
  };
  let providers = routes
    .iter()
    .filter(|route| route.transfer_source_auth)
    .map(|route| route.source_provider_id.as_str())
    .collect::<BTreeSet<_>>();
  let mut changed = false;
  for provider in providers {
    changed |= auth.remove(provider).is_some();
  }
  Ok(changed.then(|| PlannedEdit {
    path: auth_path.to_path_buf(),
    kind: EditKind::Json(json),
    // The gateway-owned credentials are the rollback source of truth. Avoid
    // leaving adjacent plaintext token backups behind.
    backup: false,
  }))
}

fn restore_transferred_credentials(auth_path: &Path, accounts: &[Account]) -> Result<()> {
  if accounts.is_empty() {
    return Ok(());
  }
  let mut json = if auth_path.exists() {
    let raw = std::fs::read_to_string(auth_path).with_context(|| format!("reading {}", auth_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", auth_path.display()))?
  } else {
    Value::Object(serde_json::Map::new())
  };
  let Some(auth) = json.as_object_mut() else {
    bail!("{} must contain a JSON object", auth_path.display());
  };
  for account in accounts {
    let provider = source_provider_id(account)
      .with_context(|| format!("transferred account '{}' is missing its OpenCode provider", account.id))?;
    if auth.contains_key(provider) {
      continue;
    }
    auth.insert(provider.to_string(), opencode_auth_record(provider, account)?);
  }
  write_sensitive_json(auth_path, &json)
}

fn opencode_auth_record(source_provider: &str, account: &Account) -> Result<Value> {
  if let Some(api_key) = &account.api_key {
    return Ok(serde_json::json!({
      "type": "api",
      "key": api_key.expose()
    }));
  }
  let refresh = account
    .refresh_token
    .as_ref()
    .with_context(|| format!("transferred account '{}' has no refresh token", account.id))?;
  let mut record = serde_json::Map::new();
  record.insert("type".into(), Value::String("oauth".into()));
  record.insert("refresh".into(), Value::String(refresh.expose().to_string()));
  if source_provider == tokn_core::provider::ID_GITHUB_COPILOT {
    record.insert("access".into(), Value::String(refresh.expose().to_string()));
    record.insert("expires".into(), Value::Number(0.into()));
  } else {
    let access = account
      .access_token
      .as_ref()
      .with_context(|| format!("transferred account '{}' has no access token", account.id))?;
    let expires = account
      .access_token_expires_at
      .with_context(|| format!("transferred account '{}' has no access-token expiry", account.id))?;
    if expires < 0 {
      bail!(
        "transferred account '{}' has a negative access-token expiry",
        account.id
      );
    }
    record.insert("access".into(), Value::String(access.expose().to_string()));
    record.insert("expires".into(), Value::Number(expires.saturating_mul(1_000).into()));
  }
  if let Some(account_id) = &account.provider_account_id {
    record.insert("accountId".into(), Value::String(account_id.clone()));
  }
  Ok(Value::Object(record))
}

fn write_sensitive_json(path: &Path, value: &Value) -> Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  let bytes = serde_json::to_vec_pretty(value)?;
  write_sensitive(path, &bytes).with_context(|| format!("writing {}", path.display()))
}

#[cfg(unix)]
fn write_sensitive(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
  use std::io::Write;
  use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
  let mut file = std::fs::OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .mode(0o600)
    .open(path)?;
  file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
  file.write_all(bytes)
}

#[cfg(not(unix))]
fn write_sensitive(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
  std::fs::write(path, bytes)
}

fn accounts_from_auth_json(json: &Value, auth_path: &Path, timestamp: &str) -> Vec<Account> {
  let Some(providers) = json.as_object() else {
    return Vec::new();
  };

  providers
    .iter()
    .filter_map(|(provider, auth)| {
      account_from_provider_auth(provider, auth).map(|account| {
        let mut account = annotate_imported_account(
          account,
          AgentId::Opencode,
          auth_path,
          &format!("auth.{provider}"),
          timestamp,
        );
        account
          .settings
          .get_mut("import")
          .and_then(toml::Value::as_table_mut)
          .expect("import metadata inserted")
          .insert("source_provider".into(), toml::Value::String(provider.clone()));
        account
      })
    })
    .collect()
}

fn account_from_provider_auth(provider: &str, auth: &Value) -> Option<Account> {
  let auth = auth.as_object()?;
  match (provider, auth.get("type").and_then(Value::as_str)?) {
    (tokn_core::provider::ID_OPENAI, "api") => {
      let api_key = auth.get("key").and_then(Value::as_str)?.trim();
      if api_key.is_empty() {
        return None;
      }
      Some(openai_account_from_key(api_key))
    }
    (tokn_core::provider::ID_OPENAI, "oauth") => oauth_account_from_auth(
      "opencode-codex",
      tokn_core::provider::ID_CODEX,
      "opencode Codex migration",
      Some(tokn_provider_openai::codex::CODEX_BASE_URL),
      Some(tokn_provider_openai::CODEX_OAUTH_TOKEN_URL),
      auth,
      auth
        .get("accountId")
        .or_else(|| auth.get("account_id"))
        .and_then(Value::as_str),
    ),
    (tokn_core::provider::ID_GITHUB_COPILOT, "oauth") if !has_enterprise_url(auth) => oauth_account_from_auth(
      "opencode-github-copilot",
      tokn_core::provider::ID_GITHUB_COPILOT,
      "opencode GitHub Copilot migration",
      Some(tokn_provider_copilot::COPILOT_BASE_URL),
      Some(tokn_provider_copilot::COPILOT_TOKEN_EXCHANGE_URL),
      auth,
      None,
    ),
    _ => None,
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

fn oauth_account_from_auth(
  id: &str,
  provider: &str,
  label: &str,
  base_url: Option<&str>,
  refresh_url: Option<&str>,
  auth: &serde_json::Map<String, Value>,
  provider_account_id: Option<&str>,
) -> Option<Account> {
  let refresh = auth.get("refresh").and_then(Value::as_str)?.trim();
  if refresh.is_empty() {
    return None;
  }
  let access = auth.get("access").and_then(Value::as_str).and_then(non_empty_string)?;
  let expires = auth.get("expires").and_then(expires_at)?;
  Some(Account {
    id: id.into(),
    provider: provider.into(),
    enabled: true,
    tier: AccountTier::Active,
    tags: vec!["agent-migrated".into(), "opencode".into()],
    label: Some(label.into()),
    base_url: base_url.map(str::to_string),
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: None,
    api_key_expires_at: None,
    access_token: Some(Secret::new(access)),
    access_token_expires_at: Some(expires),
    id_token: None,
    refresh_token: Some(Secret::new(refresh.to_string())),
    provider_account_id: provider_account_id.and_then(non_empty_string),
    extra: Default::default(),
    refresh_url: refresh_url.map(str::to_string),
    last_refresh: None,
    settings: toml::Table::new(),
  })
}

fn non_empty_string(value: &str) -> Option<String> {
  let value = value.trim();
  (!value.is_empty()).then(|| value.to_string())
}

fn has_enterprise_url(auth: &serde_json::Map<String, Value>) -> bool {
  auth
    .get("enterpriseUrl")
    .or_else(|| auth.get("enterprise_url"))
    .and_then(Value::as_str)
    .is_some_and(|value| !value.trim().is_empty())
}

fn expires_at(value: &Value) -> Option<i64> {
  let expires = match value {
    Value::Number(n) => n
      .as_i64()
      .or_else(|| n.as_u64().and_then(|value| i64::try_from(value).ok())),
    Value::String(s) => s.trim().parse().ok(),
    _ => None,
  }?;
  if expires < 0 {
    return None;
  }
  Some(if expires > 10_000_000_000 {
    expires / 1_000
  } else {
    expires
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn route(source: &str, provider: &str, account: &str, base_url: &str) -> ProviderRoute {
    ProviderRoute {
      source_provider_id: source.into(),
      gateway_provider_id: provider.into(),
      account_id: account.into(),
      profile: format!("opencode-{provider}"),
      base_url: base_url.into(),
      transfer_source_auth: true,
    }
  }

  #[test]
  fn rewrite_preserves_comments_and_model_selections() {
    let path = Path::new("opencode.jsonc");
    let root = parse_cst(
      r#"{
  // project defaults remain readable.
  "model": "openai/gpt-5",
  "mcp": {"x": true},
}
"#,
      path,
    )
    .unwrap();
    rewrite_config(
      &root,
      &[route(
        "openai",
        "codex",
        "opencode-codex",
        "http://127.0.0.1:4141/opencode-codex/v1",
      )],
    )
    .unwrap();

    let output = root.to_string();
    let json = crate::jsonc::parse_jsonc(&output, path).unwrap();
    assert!(output.contains("// project defaults remain readable."));
    assert_eq!(json["model"], "openai/gpt-5");
    assert_eq!(json["mcp"]["x"], true);
    assert_eq!(json["provider"]["openai"]["npm"], "@ai-sdk/openai-compatible");
    assert_eq!(
      json["provider"]["openai"]["options"]["baseURL"],
      "http://127.0.0.1:4141/opencode-codex/v1"
    );
  }

  #[test]
  fn accounts_from_auth_json_imports_openai_api_key() {
    let json = serde_json::json!({"openai": {"type": "api", "key": "sk-test"}});
    let accounts = accounts_from_auth_json(
      &json,
      std::path::Path::new("/tmp/opencode-auth.json"),
      "20260604T153012Z",
    );

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "opencode-openai");
    assert_eq!(accounts[0].provider, tokn_core::provider::ID_OPENAI);
    assert_eq!(accounts[0].api_key.as_ref().unwrap().expose(), "sk-test");
    assert_eq!(source_provider_id(&accounts[0]), Some("openai"));
  }

  #[test]
  fn accounts_from_auth_json_imports_oauth_records() {
    let json = serde_json::json!({
      "github-copilot": {"type": "oauth", "access": "at", "refresh": "ghu_rt", "expires": 0},
      "openai": {"type": "oauth", "access": "codex_at", "refresh": "codex_rt", "expires": "1800000000000", "accountId": "acc"}
    });

    let accounts = accounts_from_auth_json(
      &json,
      std::path::Path::new("/tmp/opencode-auth.json"),
      "20260604T153012Z",
    );

    assert_eq!(accounts.len(), 2);
    let copilot = accounts
      .iter()
      .find(|account| account.id == "opencode-github-copilot")
      .unwrap();
    assert_eq!(copilot.provider, tokn_core::provider::ID_GITHUB_COPILOT);
    assert_eq!(copilot.refresh_token.as_ref().unwrap().expose(), "ghu_rt");
    let codex = accounts.iter().find(|account| account.id == "opencode-codex").unwrap();
    assert_eq!(codex.provider, tokn_core::provider::ID_CODEX);
    assert_eq!(codex.access_token_expires_at, Some(1_800_000_000));
    assert_eq!(codex.provider_account_id.as_deref(), Some("acc"));
    assert_eq!(source_provider_id(codex), Some("openai"));
  }

  #[test]
  fn codex_oauth_expiry_roundtrips_between_opencode_milliseconds_and_gateway_seconds() {
    let json = serde_json::json!({
      "openai": {
        "type": "oauth",
        "access": "codex_at",
        "refresh": "codex_rt",
        "expires": 1800000000000_i64
      }
    });
    let account = accounts_from_auth_json(&json, Path::new("/tmp/opencode-auth.json"), "20260604T153012Z")
      .pop()
      .unwrap();

    assert_eq!(account.access_token_expires_at, Some(1_800_000_000));
    assert_eq!(
      opencode_auth_record("openai", &account).unwrap()["expires"],
      1_800_000_000_000_i64
    );
  }

  #[test]
  fn accounts_from_auth_json_ignores_unsupported_and_incomplete_records() {
    let json = serde_json::json!({
      "anthropic": {"type": "oauth", "access": "at", "refresh": "rt"},
      "openai": {"type": "oauth", "access": "at"},
      "github-copilot": {
        "type": "oauth",
        "access": "github-token",
        "refresh": "github-token",
        "expires": 0,
        "enterpriseUrl": "company.ghe.com"
      }
    });

    assert!(accounts_from_auth_json(
      &json,
      std::path::Path::new("/tmp/opencode-auth.json"),
      "20260604T153012Z",
    )
    .is_empty());
  }

  #[test]
  fn oauth_import_requires_the_complete_opencode_auth_shape() {
    for auth in [
      serde_json::json!({"type": "oauth", "refresh": "rt", "expires": 1_800_000_000_000_i64}),
      serde_json::json!({"type": "oauth", "refresh": "rt", "access": "at"}),
      serde_json::json!({"type": "oauth", "refresh": "rt", "access": "at", "expires": -1}),
    ] {
      assert!(account_from_provider_auth(tokn_core::provider::ID_OPENAI, &auth).is_none());
    }
  }

  #[test]
  fn adapter_plans_config_patch_and_credential_removal() {
    let dir = tempfile::tempdir().unwrap();
    let adapter = OpencodeAdapter;
    let auth_path = adapter.auth_path(dir.path());
    let config_path = dir.path().join(OPENCODE_CONFIG_JSONC);
    std::fs::create_dir_all(auth_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(
      &auth_path,
      serde_json::json!({
        "openai": {"type": "api", "key": "sk-test"},
        "anthropic": {"type": "api", "key": "keep-me"}
      })
      .to_string(),
    )
    .unwrap();
    std::fs::write(&config_path, "{\n  // keep me\n}\n").unwrap();
    let routes = [route(
      "openai",
      "openai",
      "opencode-openai",
      "http://127.0.0.1:4141/opencode-openai/v1",
    )];

    let edits = adapter
      .rewrite_config(dir.path(), "http://127.0.0.1:4141/opencode/v1", &routes)
      .unwrap();

    assert_eq!(edits.len(), 2);
    let config = edits.iter().find(|edit| edit.path == config_path).unwrap();
    assert!(matches!(&config.kind, EditKind::Jsonc(raw) if raw.contains("// keep me")));
    let auth = edits.iter().find(|edit| edit.path == auth_path).unwrap();
    assert!(!auth.backup);
    assert!(
      matches!(&auth.kind, EditKind::Json(json) if json.get("openai").is_none() && json.get("anthropic").is_some())
    );
  }

  #[test]
  fn retained_gateway_route_does_not_remove_an_untransferred_credential() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    std::fs::write(
      &auth_path,
      serde_json::json!({
        "openai": {"type": "oauth", "access": "missing-refresh-token"}
      })
      .to_string(),
    )
    .unwrap();
    let mut retained = route(
      "openai",
      "openai",
      "opencode-openai",
      "http://127.0.0.1:4141/opencode-openai/v1",
    );
    retained.transfer_source_auth = false;

    assert!(remove_transferred_credentials(&auth_path, &[retained])
      .unwrap()
      .is_none());
    assert!(std::fs::read_to_string(auth_path)
      .unwrap()
      .contains("missing-refresh-token"));
  }

  #[test]
  fn restore_exports_latest_api_and_oauth_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    std::fs::write(&auth_path, r#"{"anthropic":{"type":"api","key":"keep"}}"#).unwrap();
    let mut api = openai_account_from_key("sk-latest");
    api.settings.insert(
      "import".into(),
      toml::Value::Table(toml::toml! { source_provider = "openai" }),
    );
    let mut oauth = oauth_account_from_auth(
      "opencode-github-copilot",
      tokn_core::provider::ID_GITHUB_COPILOT,
      "copilot",
      None,
      None,
      serde_json::json!({"refresh": "rt-latest", "access": "at-latest", "expires": 42})
        .as_object()
        .unwrap(),
      None,
    )
    .unwrap();
    oauth.settings.insert(
      "import".into(),
      toml::Value::Table(toml::toml! { source_provider = "github-copilot" }),
    );

    restore_transferred_credentials(&auth_path, &[api, oauth]).unwrap();

    let restored: Value = serde_json::from_str(&std::fs::read_to_string(auth_path).unwrap()).unwrap();
    assert_eq!(restored["anthropic"]["key"], "keep");
    assert_eq!(restored["openai"]["key"], "sk-latest");
    assert_eq!(restored["github-copilot"]["refresh"], "rt-latest");
    assert_eq!(restored["github-copilot"]["access"], "rt-latest");
    assert_eq!(restored["github-copilot"]["expires"], 0);
  }

  #[test]
  fn restore_preserves_auth_recreated_while_linked() {
    let dir = tempfile::tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    std::fs::write(
      &auth_path,
      serde_json::to_vec_pretty(&serde_json::json!({
        "github-copilot": {
          "type": "oauth",
          "refresh": "enterprise-token",
          "access": "enterprise-token",
          "expires": 0,
          "enterpriseUrl": "company.ghe.com"
        }
      }))
      .unwrap(),
    )
    .unwrap();
    let mut account = oauth_account_from_auth(
      "opencode-github-copilot",
      tokn_core::provider::ID_GITHUB_COPILOT,
      "copilot",
      None,
      None,
      serde_json::json!({"refresh": "gateway-token", "access": "gateway-token", "expires": 0})
        .as_object()
        .unwrap(),
      None,
    )
    .unwrap();
    account.settings.insert(
      "import".into(),
      toml::Value::Table(toml::toml! { source_provider = "github-copilot" }),
    );

    restore_transferred_credentials(&auth_path, &[account]).unwrap();

    let restored: Value = serde_json::from_str(&std::fs::read_to_string(auth_path).unwrap()).unwrap();
    assert_eq!(restored["github-copilot"]["refresh"], "enterprise-token");
    assert_eq!(restored["github-copilot"]["enterpriseUrl"], "company.ghe.com");
  }
}
