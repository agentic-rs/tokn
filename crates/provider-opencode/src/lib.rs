pub mod auth;
mod opencode_go;

pub use opencode_go::*;
pub use tokn_catalogue as catalogue;
pub use tokn_core::provider::{
  error, AuthKind, Endpoint, EndpointRule, HeaderPatchCtx, Provider, ProviderInfo, ProviderRequestKind, RequestCtx,
  Result, TemplateVars, ID_OPENCODE_GO,
};
pub use tokn_core::util;

use std::sync::Arc;
use tokn_auth::descriptor::{EndpointSpec, PathRewrite, ProviderDescriptor};
use tokn_auth::provider::CredentialFlavor;
use tokn_core::provider::ProviderTarget;

pub const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";

pub static DEFAULT_ENDPOINTS: &[Endpoint] = &[Endpoint::ChatCompletions, Endpoint::Responses, Endpoint::Messages];

/// Startup fallback for models documented by OpenCode Go. Runtime routing
/// prefers the current models.dev adapter metadata, which is refreshed by the
/// gateway independently of the live `/models` identity list.
pub static MODEL_ENDPOINT_RULES: &[EndpointRule] = &[
  EndpointRule {
    pattern: "gpt-*",
    endpoints: &[Endpoint::Responses],
  },
  EndpointRule {
    pattern: "grok-*",
    endpoints: &[Endpoint::Responses],
  },
  EndpointRule {
    pattern: "muse-*",
    endpoints: &[Endpoint::Responses],
  },
  EndpointRule {
    pattern: "minimax-*",
    endpoints: &[Endpoint::Messages],
  },
  EndpointRule {
    pattern: "qwen*",
    endpoints: &[Endpoint::Messages],
  },
];

pub(crate) fn operation_url(target: &ProviderTarget, endpoint: Endpoint) -> Result<reqwest::Url> {
  let segments = match endpoint {
    Endpoint::ChatCompletions => &["chat", "completions"][..],
    Endpoint::Responses => &["responses"][..],
    Endpoint::Messages => &["messages"][..],
  };
  Ok(target.base_url().operation_url(segments.iter().copied())?)
}

pub static DESCRIPTOR: ProviderDescriptor = ProviderDescriptor {
  id: ID_OPENCODE_GO,
  display_name: "OpenCode Go",
  hosts: &["opencode.ai"],
  base_url: OPENCODE_GO_BASE_URL,
  credentials: &[CredentialFlavor::ApiKey],
  endpoints: &[
    EndpointSpec {
      endpoint: Endpoint::ChatCompletions,
      method: "POST",
      path: "/v1/chat/completions",
      aliases: &["/zen/go/v1/chat/completions"],
    },
    EndpointSpec {
      endpoint: Endpoint::Responses,
      method: "POST",
      path: "/v1/responses",
      aliases: &["/zen/go/v1/responses"],
    },
    EndpointSpec {
      endpoint: Endpoint::Messages,
      method: "POST",
      path: "/v1/messages",
      aliases: &["/zen/go/v1/messages"],
    },
  ],
  model_endpoint_rules: Some(MODEL_ENDPOINT_RULES),
  operation_url,
  rewrites: &[PathRewrite {
    method: "GET",
    src: "/zen/go/v1/models",
    path: "/v1/models",
  }],
  auth_urls: &[],
  matches_url,
  validate,
  build,
  build_auth: Some(crate::auth::provider_auth),
};

pub fn matches_url(host: &str, path: &str, _id: &'static str) -> bool {
  host == "opencode.ai" && (path.is_empty() || path.starts_with("/zen/go/v1"))
}

pub fn validate(account: &tokn_core::account::AccountConfig) -> Result<()> {
  OpenCodeGoProvider::validate_account(account)
}

pub fn build(
  account: Arc<tokn_core::account::AccountConfig>,
  target: ProviderTarget,
) -> Result<Arc<dyn tokn_core::provider::Provider>> {
  Ok(Arc::new(OpenCodeGoProvider::from_account_at(account, target)?))
}
