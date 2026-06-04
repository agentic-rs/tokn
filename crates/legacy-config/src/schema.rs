use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokn_core::account::AccountConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaAction {
  ExtractLegacyAccounts {
    config: PathBuf,
    auth: PathBuf,
    count: usize,
  },
}

const CURRENT_AUTH_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthFile {
  #[serde(default = "default_auth_version")]
  version: u32,
  #[serde(default)]
  accounts: Vec<AccountConfig>,
}

fn default_auth_version() -> u32 {
  CURRENT_AUTH_VERSION
}

#[derive(Deserialize)]
struct LegacyAccounts {
  #[serde(default)]
  accounts: Vec<AccountConfig>,
}

pub fn migrate_home(home: &Path) -> Result<Vec<SchemaAction>> {
  let config = home.join("config.toml");
  let auth = home.join("auth.yaml");
  let mut actions = Vec::new();
  if auth.exists() {
    return Ok(actions);
  }
  if let Some(accounts) = load_legacy_accounts(&config)? {
    write_auth(&auth, &accounts)?;
    actions.push(SchemaAction::ExtractLegacyAccounts {
      config,
      auth,
      count: accounts.len(),
    });
  }
  Ok(actions)
}

pub fn load_legacy_accounts(config_path: &Path) -> Result<Option<Vec<AccountConfig>>> {
  if !config_path.exists() {
    return Ok(None);
  }
  let raw = fs::read_to_string(config_path).with_context(|| format!("reading {}", config_path.display()))?;
  let parsed: LegacyAccounts =
    toml::from_str(&raw).with_context(|| format!("parsing legacy {}", config_path.display()))?;
  if parsed.accounts.is_empty() {
    Ok(None)
  } else {
    Ok(Some(parsed.accounts))
  }
}

fn write_auth(auth_path: &Path, accounts: &[AccountConfig]) -> Result<()> {
  if let Some(parent) = auth_path.parent() {
    fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  let file = AuthFile {
    version: CURRENT_AUTH_VERSION,
    accounts: accounts.to_vec(),
  };
  let yaml = serde_yaml::to_string(&file).with_context(|| "serialising auth.yaml")?;
  write_secured(auth_path, yaml.as_bytes()).with_context(|| format!("writing {}", auth_path.display()))
}

#[cfg(unix)]
fn write_secured(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
  use std::io::Write;
  use std::os::unix::fs::OpenOptionsExt;
  let mut f = fs::OpenOptions::new()
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
  fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn migrate_home_extracts_legacy_accounts_when_auth_missing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
      dir.path().join("config.toml"),
      r#"
[[accounts]]
id = "legacy"
provider = "github-copilot"
enabled = true
"#,
    )
    .unwrap();

    let actions = migrate_home(dir.path()).unwrap();

    assert_eq!(actions.len(), 1);
    assert!(matches!(
      &actions[0],
      SchemaAction::ExtractLegacyAccounts { count: 1, .. }
    ));
    let auth = fs::read_to_string(dir.path().join("auth.yaml")).unwrap();
    assert!(auth.contains("version: 1"));
    assert!(auth.contains("id: legacy"));
  }

  #[test]
  fn migrate_home_keeps_existing_auth() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("auth.yaml"), "version: 1\naccounts: []\n").unwrap();
    fs::write(
      dir.path().join("config.toml"),
      r#"
[[accounts]]
id = "legacy"
provider = "github-copilot"
"#,
    )
    .unwrap();

    let actions = migrate_home(dir.path()).unwrap();

    assert!(actions.is_empty());
    assert_eq!(
      fs::read_to_string(dir.path().join("auth.yaml")).unwrap(),
      "version: 1\naccounts: []\n"
    );
  }
}
