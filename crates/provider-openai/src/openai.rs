use crate::util::secret::Secret;
use async_trait::async_trait;
use llm_core::account::AccountConfig;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;
use serde_json::Value;
use snafu::ResultExt;
use std::sync::Arc;
use tracing::{debug, instrument, warn};

use crate::{
  error, AuthKind, Endpoint, HeaderPatchCtx, Provider, ProviderInfo, RequestCtx, Result, ID_CODEX, ID_OPENAI,
};

pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";

enum Credential {
  ApiKey(Secret<String>),
  AccessToken(Secret<String>),
}

pub struct OpenAiProvider {
  pub id: String,
  provider_id: String,
  credential: Credential,
  base_url: String,
  /// Provider-specific account identifier (codex `chatgpt_account_id`).
  /// Set on the outbound `ChatGPT-Account-Id` header for codex requests.
  provider_account_id: Option<String>,
  info: ProviderInfo,
}

impl OpenAiProvider {
  pub fn validate_account(a: &AccountConfig) -> Result<()> {
    match a.provider.as_str() {
      ID_OPENAI => {
        let _ = a
          .api_key
          .clone()
          .filter(|s| !s.expose().trim().is_empty())
          .ok_or(error::Error::MissingCredential {
            account: a.id.clone(),
            what: "api_key",
          })?;
      }
      ID_CODEX => {
        let _ = a
          .access_token
          .clone()
          .or_else(|| a.api_key.clone())
          .filter(|s| !s.expose().trim().is_empty())
          .ok_or(error::Error::MissingCredential {
            account: a.id.clone(),
            what: "access_token",
          })?;
      }
      other => {
        return error::ProviderMismatchSnafu {
          expected: "openai|codex",
          got: other.to_string(),
        }
        .fail()
      }
    }
    Ok(())
  }

  pub fn from_account(a: Arc<AccountConfig>) -> Result<Self> {
    Self::validate_account(&a)?;
    let (credential, base, display_name, auth_kind, models, default_endpoints) = match a.provider.as_str() {
      ID_OPENAI => (
        Credential::ApiKey(a.api_key.clone().expect("validated api_key")),
        a.base_url.clone().unwrap_or_else(|| OPENAI_BASE_URL.to_string()),
        "OpenAI",
        AuthKind::StaticApiKey,
        crate::catalogue::default_models_for(ID_OPENAI),
        crate::DEFAULT_ENDPOINTS_OPENAI,
      ),
      ID_CODEX => (
        Credential::AccessToken(
          a.access_token
            .clone()
            .or_else(|| a.api_key.clone())
            .expect("validated access token"),
        ),
        a.base_url.clone().unwrap_or_else(|| CODEX_BASE_URL.to_string()),
        "ChatGPT Codex",
        AuthKind::OAuthDeviceFlow,
        crate::catalogue::default_models_for(ID_OPENAI),
        crate::DEFAULT_ENDPOINTS_CODEX,
      ),
      _ => unreachable!("validated provider"),
    };
    Ok(Self {
      id: format!("{}:{}", a.provider, a.id),
      provider_id: a.provider.clone(),
      credential,
      base_url: base.clone(),
      provider_account_id: a.provider_account_id.clone(),
      info: ProviderInfo {
        id: a.provider.clone(),
        aliases: &[],
        display_name,
        upstream_url: base,
        auth_kind,
        default_models: models,
        default_endpoints,
        model_cache: Arc::new(llm_core::provider::ModelCache::default()),
      },
    })
  }

  fn token(&self) -> &str {
    match &self.credential {
      Credential::ApiKey(secret) | Credential::AccessToken(secret) => secret.expose(),
    }
  }

  fn url(&self, path: &str) -> String {
    format!("{}{}", self.base_url.trim_end_matches('/'), path)
  }
}

#[async_trait]
impl Provider for OpenAiProvider {
  fn id(&self) -> &str {
    &self.id
  }

  fn info(&self) -> &ProviderInfo {
    &self.info
  }

  fn patch_headers(&self, headers: &mut HeaderMap, ctx: &HeaderPatchCtx<'_>) -> Result<()> {
    headers.insert(
      AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {}", self.token()))
        .context(error::HeaderValueSnafu { name: "authorization" })?,
    );
    if self.provider_id == ID_CODEX {
      if let Some(pid) = self.provider_account_id.as_deref().filter(|s| !s.is_empty()) {
        headers.insert(
          reqwest::header::HeaderName::from_static("chatgpt-account-id"),
          HeaderValue::from_str(pid).context(error::HeaderValueSnafu {
            name: "chatgpt-account-id",
          })?,
        );
      }
    }
    headers.insert(
      ACCEPT,
      HeaderValue::from_static(if ctx.stream {
        "text/event-stream"
      } else {
        "application/json"
      }),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Some(encoding) = ctx.content_encoding {
      headers.insert(
        reqwest::header::CONTENT_ENCODING,
        HeaderValue::from_str(encoding).context(error::HeaderValueSnafu {
          name: "content-encoding",
        })?,
      );
    }
    Ok(())
  }

  async fn list_models(&self, http: &reqwest::Client) -> Result<Value> {
    if self.provider_id == ID_CODEX {
      return Ok(serde_json::json!({ "object": "list", "data": [] }));
    }
    let mut headers = HeaderMap::new();
    self.patch_headers(
      &mut headers,
      &HeaderPatchCtx {
        endpoint: Endpoint::ChatCompletions,
        body: &Value::Null,
        bearer_token: None,
        content_encoding: None,
        stream: false,
        initiator: "user",
        inbound_headers: &HeaderMap::new(),
      },
    )?;
    let resp = crate::util::http::send(
      http,
      Method::GET,
      &self.url("/models"),
      headers,
      None,
      None,
      "openai /models",
    )
    .await?;
    crate::util::http::read_json(resp, "openai /models").await
  }

  #[instrument(name = "openai_chat", skip_all, fields(account = %self.id, stream = ctx.stream))]
  async fn chat(&self, ctx: RequestCtx<'_>) -> Result<reqwest::Response> {
    self.upstream_post(ctx, "/chat/completions", "openai chat").await
  }

  #[instrument(name = "openai_responses", skip_all, fields(account = %self.id, stream = ctx.stream))]
  async fn responses(&self, ctx: RequestCtx<'_>) -> Result<reqwest::Response> {
    let path = if self.provider_id == ID_CODEX {
      "/responses"
    } else {
      "/responses"
    };
    self.upstream_post(ctx, path, "openai responses").await
  }

  fn on_unauthorized(&self) {
    warn!(account = %self.id, "{} upstream returned 401: credential likely revoked or expired", self.provider_id);
  }
}

impl OpenAiProvider {
  async fn upstream_post(&self, ctx: RequestCtx<'_>, path: &str, what: &'static str) -> Result<reqwest::Response> {
    let url = self.url(path);
    debug!(%url, "POST upstream");
    let mut headers = ctx.profile_headers.clone().unwrap_or_default();
    self.patch_headers(
      &mut headers,
      &HeaderPatchCtx {
        endpoint: ctx.endpoint,
        body: ctx.body,
        bearer_token: None,
        content_encoding: ctx.content_encoding,
        stream: ctx.stream,
        initiator: ctx.initiator,
        inbound_headers: ctx.inbound_headers,
      },
    )?;
    let body_bytes = ctx.request_body_bytes();
    crate::util::http::send(
      ctx.http,
      Method::POST,
      &url,
      headers,
      Some(body_bytes),
      ctx.outbound.as_ref(),
      what,
    )
    .await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use llm_core::account::AccountTier;

  fn acct(provider: &str, key: Option<&str>, access: Option<&str>) -> AccountConfig {
    AccountConfig {
      id: "test".into(),
      provider: provider.into(),
      enabled: true,
      tier: AccountTier::Active,
      tags: Vec::new(),
      label: None,
      base_url: None,
      headers: Default::default(),
      auth_type: None,
      username: None,
      api_key: key.map(|s| Secret::new(s.into())),
      api_key_expires_at: None,
      access_token: access.map(|s| Secret::new(s.into())),
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
  fn openai_requires_api_key() {
    let err = OpenAiProvider::from_account(Arc::new(acct(ID_OPENAI, None, None)))
      .err()
      .unwrap();
    assert!(err.to_string().contains("api_key"));
  }

  #[test]
  fn codex_requires_access_token() {
    let err = OpenAiProvider::from_account(Arc::new(acct(ID_CODEX, None, None)))
      .err()
      .unwrap();
    assert!(err.to_string().contains("access_token"));
  }

  fn patch_ctx() -> HeaderPatchCtx<'static> {
    HeaderPatchCtx {
      endpoint: Endpoint::Responses,
      body: &Value::Null,
      bearer_token: None,
      content_encoding: None,
      stream: false,
      initiator: "user",
      inbound_headers: Box::leak(Box::new(HeaderMap::new())),
    }
  }

  #[test]
  fn codex_patch_headers_injects_account_id_when_present() {
    let mut a = acct(ID_CODEX, None, Some("atk-test"));
    a.provider_account_id = Some("acc-77".into());
    let codex = OpenAiProvider::from_account(Arc::new(a)).unwrap();
    let mut h = HeaderMap::new();
    codex.patch_headers(&mut h, &patch_ctx()).unwrap();
    assert_eq!(h.get("authorization").unwrap(), "Bearer atk-test");
    assert_eq!(h.get("chatgpt-account-id").unwrap(), "acc-77");
  }

  #[test]
  fn codex_patch_headers_omits_account_id_when_absent_or_blank() {
    for blank in [None, Some(String::new())] {
      let mut a = acct(ID_CODEX, None, Some("atk-test"));
      a.provider_account_id = blank;
      let codex = OpenAiProvider::from_account(Arc::new(a)).unwrap();
      let mut h = HeaderMap::new();
      codex.patch_headers(&mut h, &patch_ctx()).unwrap();
      assert!(h.get("chatgpt-account-id").is_none());
    }
  }

  #[test]
  fn openai_patch_headers_never_sets_account_id() {
    let mut a = acct(ID_OPENAI, Some("sk-test"), None);
    a.provider_account_id = Some("acc-ignored".into());
    let openai = OpenAiProvider::from_account(Arc::new(a)).unwrap();
    let mut h = HeaderMap::new();
    openai.patch_headers(&mut h, &patch_ctx()).unwrap();
    assert!(h.get("chatgpt-account-id").is_none());
  }

  #[test]
  fn openai_and_codex_construct() {
    let openai = OpenAiProvider::from_account(Arc::new(acct(ID_OPENAI, Some("sk-test"), None))).unwrap();
    assert_eq!(openai.info().upstream_url, OPENAI_BASE_URL);
    let codex = OpenAiProvider::from_account(Arc::new(acct(ID_CODEX, None, Some("atk-test")))).unwrap();
    assert_eq!(codex.info().upstream_url, CODEX_BASE_URL);
    assert!(codex.supports("", Endpoint::Responses));
    assert!(!codex.supports("", Endpoint::ChatCompletions));
  }
}
