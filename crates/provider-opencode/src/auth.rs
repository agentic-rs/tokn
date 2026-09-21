use async_trait::async_trait;
use tokn_auth::{ProviderAuth, QuotaSnapshot, RefreshOutcome, Result, VerifyOutcome};
use tokn_core::account::AccountConfig;

pub struct OpenCodeGoAuth;

pub fn provider_auth() -> &'static dyn ProviderAuth {
  &OpenCodeGoAuth
}

#[async_trait]
impl ProviderAuth for OpenCodeGoAuth {
  fn id(&self) -> &'static str {
    crate::ID_OPENCODE_GO
  }

  fn supports_static_key(&self) -> bool {
    true
  }

  fn default_base_url(&self) -> Option<&'static str> {
    Some(crate::OPENCODE_GO_BASE_URL)
  }

  async fn refresh_credential(&self, _client: &reqwest::Client, _account: &AccountConfig) -> Result<RefreshOutcome> {
    Ok(RefreshOutcome::NotApplicable)
  }

  async fn verify_credential(&self, client: &reqwest::Client, account: &AccountConfig) -> Result<VerifyOutcome> {
    crate::quota::verify(client, account).await?;
    Ok(VerifyOutcome::default())
  }

  async fn probe_quota(&self, client: &reqwest::Client, account: &AccountConfig) -> Result<QuotaSnapshot> {
    crate::quota::fetch(client, account).await
  }
}
