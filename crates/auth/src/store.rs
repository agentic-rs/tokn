//! `auth.yaml` storage.
//!
//! Format (`auth.yaml`):
//!
//! ```yaml
//! version: 1
//! accounts:
//!   - id: clouds56-bot
//!     provider: github-copilot
//!     enabled: true
//!     tier: active
//!     refresh_token: ghu_xxx
//!     access_token: tid_xxx
//!     access_token_expires_at: 1234567890
//!     last_refresh: 1234567890
//!     settings: {}
//! ```
//!
//! Legacy schema conversion is owned by `tokn-router-legacy-config` and
//! should run before latest auth loading.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokn_core::account::AccountConfig;

const CURRENT_VERSION: u32 = 1;
const AUTH_FILE_NAME: &str = "auth.yaml";

/// On-disk schema. Future versions can introduce new top-level keys; the
/// `version` field is mandatory so we can detect format upgrades.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthFile {
  #[serde(default = "default_version")]
  version: u32,
  #[serde(default)]
  accounts: Vec<AccountConfig>,
}

fn default_version() -> u32 {
  CURRENT_VERSION
}

/// In-memory account store backed by `auth.yaml`.
///
/// `path` is the absolute path the store will save to. It is captured at
/// load-time so subsequent saves don't need to re-resolve `XDG_CONFIG_HOME`.
#[derive(Debug, Clone)]
pub struct AuthStore {
  path: PathBuf,
  pub accounts: Vec<AccountConfig>,
}

impl AuthStore {
  /// Load `auth.yaml` from the given path. `auth_path` overrides the default
  /// router auth path. `config_path` is accepted for source compatibility but
  /// is no longer consulted; callers should run `tokn-router-legacy-config`
  /// before latest auth loading.
  pub fn load(auth_path: Option<&Path>, _config_path: Option<&Path>) -> Result<Self> {
    let resolved = match auth_path {
      Some(path) => path.to_path_buf(),
      None => default_auth_path()?,
    };

    if resolved.exists() {
      return load_from_yaml(&resolved);
    }

    // Nothing to load — empty store rooted at the resolved path so a
    // future `save()` creates the file.
    Ok(AuthStore {
      path: resolved,
      accounts: Vec::new(),
    })
  }

  /// Persist the current state to `auth.yaml`. Creates parent directories
  /// as needed; writes with mode 0600 on Unix to keep tokens off prying
  /// eyes.
  pub fn save(&self) -> Result<()> {
    if let Some(parent) = self.path.parent() {
      std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let file = AuthFile {
      version: CURRENT_VERSION,
      accounts: self.accounts.clone(),
    };
    let yaml = serde_yaml::to_string(&file).with_context(|| "serialising auth.yaml")?;
    write_secured(&self.path, yaml.as_bytes()).with_context(|| format!("writing {}", self.path.display()))
  }

  /// Insert or replace an account by id.
  pub fn upsert(&mut self, account: AccountConfig) {
    if let Some(slot) = self.accounts.iter_mut().find(|a| a.id == account.id) {
      *slot = account;
    } else {
      self.accounts.push(account);
    }
  }

  /// Remove the account with the given id, returning the removed value if
  /// any.
  pub fn remove(&mut self, id: &str) -> Option<AccountConfig> {
    let idx = self.accounts.iter().position(|a| a.id == id)?;
    Some(self.accounts.remove(idx))
  }

  /// Borrow the account with the given id, if any.
  pub fn get(&self, id: &str) -> Option<&AccountConfig> {
    self.accounts.iter().find(|a| a.id == id)
  }

  /// Mutably borrow the account with the given id, if any.
  pub fn get_mut(&mut self, id: &str) -> Option<&mut AccountConfig> {
    self.accounts.iter_mut().find(|a| a.id == id)
  }

  /// The path this store will save to.
  pub fn path(&self) -> &Path {
    &self.path
  }
}

fn load_from_yaml(path: &Path) -> Result<AuthStore> {
  let raw = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
  let parsed: AuthFile =
    serde_yaml::from_str(&raw).with_context(|| format!("parsing {} (expected `version: 1` schema)", path.display()))?;
  if parsed.version != CURRENT_VERSION {
    anyhow::bail!(
      "{}: unsupported version {} (this build understands {})",
      path.display(),
      parsed.version,
      CURRENT_VERSION
    );
  }
  Ok(AuthStore {
    path: path.to_path_buf(),
    accounts: parsed.accounts,
  })
}

/// Default path: the gateway config directory's `auth.yaml`.
pub fn default_auth_path() -> Result<PathBuf> {
  Ok(tokn_config::paths::config_dir()?.join(AUTH_FILE_NAME))
}

#[cfg(unix)]
fn write_secured(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
  use std::io::Write;
  use std::os::unix::fs::OpenOptionsExt;
  let mut f = std::fs::OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .mode(0o600)
    .open(path)?;
  f.write_all(bytes)?;
  Ok(())
}

#[cfg(not(unix))]
fn write_secured(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
  std::fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
  use super::*;
  use tokn_core::account::{AccountTier, AuthType};

  fn sample_account(id: &str) -> AccountConfig {
    AccountConfig {
      id: id.into(),
      provider: "github-copilot".into(),
      enabled: true,
      tier: AccountTier::Active,
      tags: vec![],
      label: None,
      base_url: None,
      headers: Default::default(),
      auth_type: Some(AuthType::Bearer),
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
  fn roundtrip_yaml_preserves_accounts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.yaml");
    let store = AuthStore {
      path: path.clone(),
      accounts: vec![sample_account("a1"), sample_account("a2")],
    };
    store.save().unwrap();
    let loaded = AuthStore::load(Some(&path), None).unwrap();
    assert_eq!(loaded.accounts.len(), 2);
    assert_eq!(loaded.accounts[0].id, "a1");
    assert_eq!(loaded.accounts[1].id, "a2");
  }

  #[test]
  fn default_auth_path_uses_gateway_config_dir() {
    let expected = tokn_config::paths::config_dir().unwrap().join(AUTH_FILE_NAME);
    assert_eq!(default_auth_path().unwrap(), expected);
  }

  #[test]
  fn upsert_replaces_by_id() {
    let mut store = AuthStore {
      path: PathBuf::from("/dev/null"),
      accounts: vec![sample_account("a1")],
    };
    let mut updated = sample_account("a1");
    updated.label = Some("renamed".into());
    store.upsert(updated);
    assert_eq!(store.accounts.len(), 1);
    assert_eq!(store.accounts[0].label.as_deref(), Some("renamed"));
  }

  #[test]
  fn remove_returns_extracted_account() {
    let mut store = AuthStore {
      path: PathBuf::from("/dev/null"),
      accounts: vec![sample_account("a1"), sample_account("a2")],
    };
    let popped = store.remove("a1").unwrap();
    assert_eq!(popped.id, "a1");
    assert_eq!(store.accounts.len(), 1);
    assert!(store.remove("ghost").is_none());
  }

  #[test]
  fn missing_yaml_ignores_legacy_config_after_schema_migration_split() {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join("auth.yaml");
    let cfg_path = dir.path().join("config.toml");
    std::fs::write(
      &cfg_path,
      r#"
[[accounts]]
id = "legacy"
provider = "github-copilot"
enabled = true
"#,
    )
    .unwrap();

    let store = AuthStore::load(Some(&yaml_path), Some(&cfg_path)).unwrap();
    assert!(store.accounts.is_empty());
    assert!(!yaml_path.exists());
  }

  #[test]
  fn missing_both_yields_empty_store() {
    let dir = tempfile::tempdir().unwrap();
    let yaml_path = dir.path().join("auth.yaml");
    let store = AuthStore::load(Some(&yaml_path), None).unwrap();
    assert!(store.accounts.is_empty());
  }

  #[test]
  fn unknown_version_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.yaml");
    std::fs::write(&path, "version: 99\naccounts: []\n").unwrap();
    let err = AuthStore::load(Some(&path), None).unwrap_err();
    assert!(err.to_string().contains("unsupported version 99"));
  }
}
