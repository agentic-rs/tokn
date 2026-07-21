//! Inbound gateway API-key authentication and provider authorization.
//!
//! This is intentionally separate from `tokn-auth`, which owns credentials
//! used by the gateway to authenticate *to upstream providers*.

use anyhow::{bail, Context, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use time::OffsetDateTime;
use uuid::Uuid;

const KEY_PREFIX: &str = "tokn";
const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderAccess {
  All,
  Only(BTreeSet<String>),
}

impl ProviderAccess {
  pub fn from_provider_ids(provider_ids: Vec<String>) -> Result<Self> {
    if provider_ids.is_empty() {
      return Ok(Self::All);
    }
    if provider_ids.iter().any(|provider| provider == "*") {
      if provider_ids.len() != 1 {
        bail!("provider `*` cannot be combined with specific provider ids");
      }
      return Ok(Self::All);
    }
    let providers = provider_ids
      .into_iter()
      .map(|provider| provider.trim().to_string())
      .collect::<BTreeSet<_>>();
    if providers.iter().any(String::is_empty) {
      bail!("provider ids must not be empty");
    }
    Ok(Self::Only(providers))
  }

  pub fn allows(&self, provider_id: &str) -> bool {
    match self {
      Self::All => true,
      Self::Only(providers) => providers.contains(provider_id),
    }
  }

  pub fn provider_ids(&self) -> Option<&BTreeSet<String>> {
    match self {
      Self::All => None,
      Self::Only(providers) => Some(providers),
    }
  }

  pub fn display(&self) -> String {
    match self {
      Self::All => "*".to_string(),
      Self::Only(providers) => providers.iter().cloned().collect::<Vec<_>>().join(","),
    }
  }

  fn to_json(&self) -> Result<String> {
    let providers = match self {
      Self::All => vec!["*".to_string()],
      Self::Only(providers) => providers.iter().cloned().collect(),
    };
    Ok(serde_json::to_string(&providers)?)
  }

  fn from_json(value: &str) -> Result<Self> {
    Self::from_provider_ids(serde_json::from_str(value).context("decode allowed providers")?)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessContext {
  pub key_id: Option<String>,
  pub key_name: Option<String>,
  pub providers: ProviderAccess,
}

impl AccessContext {
  pub fn unrestricted() -> Self {
    Self {
      key_id: None,
      key_name: None,
      providers: ProviderAccess::All,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedApiKey {
  pub id: String,
  pub name: String,
  pub token: String,
  pub providers: ProviderAccess,
  pub created_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKeySummary {
  pub id: String,
  pub name: String,
  pub providers: ProviderAccess,
  pub created_at: i64,
  pub revoked_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthenticationError {
  Missing,
  Invalid,
  Revoked,
}

/// SQLite-backed store shared by CLI management and HTTP verification.
///
/// Verification is deliberately synchronous and performs one indexed local
/// lookup. The connection mutex also allows a running server to observe keys
/// created or revoked by a separate CLI process without a restart.
pub struct AccessStore {
  path: PathBuf,
  connection: Mutex<Connection>,
}

impl std::fmt::Debug for AccessStore {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.debug_struct("AccessStore").field("path", &self.path).finish()
  }
}

impl AccessStore {
  pub fn disabled() -> Self {
    let connection = Connection::open_in_memory().expect("open in-memory access store");
    migrate(&connection).expect("initialize in-memory access store");
    Self {
      path: PathBuf::from(":memory:"),
      connection: Mutex::new(connection),
    }
  }

  pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref().to_path_buf();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let connection = Connection::open(&path).with_context(|| format!("open {}", path.display()))?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    migrate(&connection)?;
    secure_file(&path)?;
    Ok(Self {
      path,
      connection: Mutex::new(connection),
    })
  }

  pub fn open_default() -> Result<Self> {
    Self::open(default_access_path()?)
  }

  pub fn path(&self) -> &Path {
    &self.path
  }

  /// Authentication switches on permanently when the first key is created.
  /// Revoking the final key therefore fails closed instead of disabling auth.
  pub fn is_enabled(&self) -> Result<bool> {
    let connection = self.connection.lock();
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM api_keys", [], |row| row.get(0))?;
    Ok(count > 0)
  }

  pub fn create_key(&self, name: impl Into<String>, provider_ids: Vec<String>) -> Result<CreatedApiKey> {
    let name = name.into();
    if name.trim().is_empty() {
      bail!("API key name must not be empty");
    }
    let providers = ProviderAccess::from_provider_ids(provider_ids)?;
    let created_at = OffsetDateTime::now_utc().unix_timestamp();

    for _ in 0..4 {
      let id = Uuid::new_v4().simple().to_string()[..16].to_string();
      let secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
      let token = format!("{KEY_PREFIX}_{id}_{secret}");
      let hash = hash_secret(secret.as_bytes());
      let inserted = self.connection.lock().execute(
        "INSERT OR IGNORE INTO api_keys (id, name, secret_hash, allowed_providers, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, hash.as_slice(), providers.to_json()?, created_at],
      )?;
      if inserted == 1 {
        return Ok(CreatedApiKey {
          id,
          name,
          token,
          providers,
          created_at,
        });
      }
    }
    bail!("could not allocate a unique API key id")
  }

  pub fn list_keys(&self) -> Result<Vec<ApiKeySummary>> {
    let connection = self.connection.lock();
    let mut statement = connection.prepare(
      "SELECT id, name, allowed_providers, created_at, revoked_at
       FROM api_keys ORDER BY created_at, id",
    )?;
    let rows = statement.query_map([], |row| {
      Ok((
        row.get::<_, String>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, String>(2)?,
        row.get::<_, i64>(3)?,
        row.get::<_, Option<i64>>(4)?,
      ))
    })?;
    rows
      .map(|row| {
        let (id, name, providers, created_at, revoked_at) = row?;
        Ok(ApiKeySummary {
          id,
          name,
          providers: ProviderAccess::from_json(&providers)?,
          created_at,
          revoked_at,
        })
      })
      .collect()
  }

  pub fn revoke_key(&self, id: &str) -> Result<bool> {
    let revoked_at = OffsetDateTime::now_utc().unix_timestamp();
    let changed = self.connection.lock().execute(
      "UPDATE api_keys SET revoked_at = COALESCE(revoked_at, ?1) WHERE id = ?2",
      params![revoked_at, id],
    )?;
    Ok(changed == 1)
  }

  pub fn authenticate(&self, token: Option<&str>) -> Result<AccessContext, AuthenticationError> {
    let token = token.ok_or(AuthenticationError::Missing)?;
    let (id, secret) = parse_token(token).ok_or(AuthenticationError::Invalid)?;
    let record = self
      .connection
      .lock()
      .query_row(
        "SELECT name, secret_hash, allowed_providers, revoked_at FROM api_keys WHERE id = ?1",
        [id],
        |row| {
          Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
          ))
        },
      )
      .optional()
      .map_err(|_| AuthenticationError::Invalid)?
      .ok_or(AuthenticationError::Invalid)?;
    let (name, expected_hash, allowed_providers, revoked_at) = record;
    if revoked_at.is_some() {
      return Err(AuthenticationError::Revoked);
    }
    let actual_hash = hash_secret(secret.as_bytes());
    if expected_hash.len() != actual_hash.len() || !bool::from(expected_hash.as_slice().ct_eq(actual_hash.as_slice())) {
      return Err(AuthenticationError::Invalid);
    }
    let providers = ProviderAccess::from_json(&allowed_providers).map_err(|_| AuthenticationError::Invalid)?;
    Ok(AccessContext {
      key_id: Some(id.to_string()),
      key_name: Some(name),
      providers,
    })
  }
}

pub fn default_access_path() -> Result<PathBuf> {
  tokn_core::util::paths::router_home()
    .map(|home| home.join("access.db"))
    .context("could not determine the tokn router home")
}

fn parse_token(token: &str) -> Option<(&str, &str)> {
  let mut parts = token.trim().split('_');
  match (parts.next(), parts.next(), parts.next(), parts.next()) {
    (Some(KEY_PREFIX), Some(id), Some(secret), None) if id.len() == 16 && secret.len() == 64 => Some((id, secret)),
    _ => None,
  }
}

fn hash_secret(secret: &[u8]) -> [u8; 32] {
  Sha256::digest(secret).into()
}

fn migrate(connection: &Connection) -> Result<()> {
  let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
  if version > SCHEMA_VERSION {
    bail!("access database schema {version} is newer than supported schema {SCHEMA_VERSION}");
  }
  if version == 0 {
    connection.execute_batch(
      "BEGIN IMMEDIATE;
       CREATE TABLE api_keys (
         id TEXT PRIMARY KEY,
         name TEXT NOT NULL,
         secret_hash BLOB NOT NULL,
         allowed_providers TEXT NOT NULL,
         created_at INTEGER NOT NULL,
         revoked_at INTEGER
       );
       PRAGMA user_version = 1;
       COMMIT;",
    )?;
  }
  Ok(())
}

#[cfg(unix)]
fn secure_file(path: &Path) -> Result<()> {
  use std::os::unix::fs::PermissionsExt;
  let mut permissions = std::fs::metadata(path)?.permissions();
  permissions.set_mode(0o600);
  std::fs::set_permissions(path, permissions)?;
  Ok(())
}

#[cfg(not(unix))]
fn secure_file(_path: &Path) -> Result<()> {
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn store() -> (tempfile::TempDir, AccessStore) {
    let temp = tempfile::tempdir().unwrap();
    let store = AccessStore::open(temp.path().join("access.db")).unwrap();
    (temp, store)
  }

  #[test]
  fn empty_provider_list_defaults_to_all() {
    assert_eq!(
      ProviderAccess::from_provider_ids(Vec::new()).unwrap(),
      ProviderAccess::All
    );
  }

  #[test]
  fn key_round_trip_and_revocation() {
    let (_temp, store) = store();
    assert!(!store.is_enabled().unwrap());
    let created = store
      .create_key("client", vec!["openai".into(), "github-copilot".into()])
      .unwrap();
    assert!(store.is_enabled().unwrap());

    let context = store.authenticate(Some(&created.token)).unwrap();
    assert_eq!(context.key_id.as_deref(), Some(created.id.as_str()));
    assert!(context.providers.allows("openai"));
    assert!(!context.providers.allows("deepseek"));

    assert!(store.revoke_key(&created.id).unwrap());
    assert_eq!(
      store.authenticate(Some(&created.token)),
      Err(AuthenticationError::Revoked)
    );
    assert!(store.is_enabled().unwrap());
  }

  #[test]
  fn wildcard_is_the_default_and_matches_future_providers() {
    let (_temp, store) = store();
    let created = store.create_key("client", Vec::new()).unwrap();
    let context = store.authenticate(Some(&created.token)).unwrap();
    assert!(context.providers.allows("provider-added-later"));
    assert_eq!(created.providers.display(), "*");
  }

  #[test]
  fn malformed_and_unknown_keys_are_rejected() {
    let (_temp, store) = store();
    assert_eq!(store.authenticate(None), Err(AuthenticationError::Missing));
    assert_eq!(store.authenticate(Some("not-a-key")), Err(AuthenticationError::Invalid));
  }
}
