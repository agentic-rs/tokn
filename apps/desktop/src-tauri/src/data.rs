use anyhow::Result;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
pub struct AccountSummary {
  pub id: String,
  pub provider: String,
  pub label: Option<String>,
  pub enabled: bool,
  pub tier: String,
}

#[derive(Serialize)]
pub struct UsageSummary {
  pub account: Option<String>,
  pub provider: Option<String>,
  pub model: String,
  pub requests: u64,
  pub input_tokens: u64,
  pub output_tokens: u64,
  pub cached_tokens: u64,
}

pub fn accounts() -> Result<Vec<AccountSummary>> {
  Ok(
    tokn_auth::AuthStore::load(None, None)?
      .accounts
      .into_iter()
      .map(|account| AccountSummary {
        id: account.id,
        provider: account.provider,
        label: account.label,
        enabled: account.enabled,
        tier: format!("{:?}", account.tier).to_lowercase(),
      })
      .collect(),
  )
}

pub fn usage() -> Result<Vec<UsageSummary>> {
  let path = crate::config::load()?.persistence().resolve_paths()?.usage_db;
  if !path.exists() {
    return Ok(Vec::new());
  }
  let since = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64 - 86_400_000;
  Ok(
    tokn_persistence::UsageDb::open_readonly(&path)?
      .summary(since, None, None)?
      .into_iter()
      .map(|row| UsageSummary {
        account: row.account,
        provider: row.provider,
        model: row.model,
        requests: row.count,
        input_tokens: row.input_tokens,
        output_tokens: row.output_tokens,
        cached_tokens: row.cached_tokens,
      })
      .collect(),
  )
}
