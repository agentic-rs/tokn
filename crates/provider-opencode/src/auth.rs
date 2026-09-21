use async_trait::async_trait;
use tokn_auth::{AuthError, ProviderAuth, QuotaSnapshot, RefreshOutcome, Result, VerifyOutcome};
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
    let key = account.api_key.as_ref().ok_or(AuthError::MissingCredential {
      account: account.id.clone(),
      field: "api_key",
    })?;
    let base = account
      .base_url
      .as_deref()
      .unwrap_or(crate::OPENCODE_GO_BASE_URL)
      .trim_end_matches('/');
    let response = client
      .get(format!("{base}/models"))
      .header("authorization", format!("Bearer {}", key.expose()))
      .header("accept", "application/json")
      .header("user-agent", tokn_core::util::version::tokn_router_user_agent())
      .send()
      .await
      .map_err(|error| AuthError::Network(error.to_string()))?;
    if response.status().is_success() {
      return Ok(VerifyOutcome::default());
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(AuthError::Upstream(format!(
      "OpenCode Go rejected the key (HTTP {status}): {}",
      body.chars().take(200).collect::<String>()
    )))
  }

  async fn probe_quota(&self, _client: &reqwest::Client, _account: &AccountConfig) -> Result<QuotaSnapshot> {
    Ok(QuotaSnapshot::default())
  }
}
