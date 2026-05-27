use crate::util::secret::Secret;
use crate::HeaderPatchCtx;
use tokn_headers::keys::{
  ACCEPT, AUTHORIZATION, CHATGPT_ACCOUNT_ID, CONTENT_ENCODING, CONTENT_TYPE, OPENAI_BETA, ORIGINATOR, SESSION_ID_LOWER,
  USER_AGENT, VERSION, X_CODEX_TURN_METADATA,
};
use tokn_headers::{HeaderMap, HeaderValue};

pub const CODEX_CLI_USER_AGENT: &str = "codex_cli_rs/0.125.0";
pub const CODEX_CLI_VERSION: &str = "0.125.0";
pub const CODEX_RESPONSES_BETA: &str = "responses=experimental";

pub enum Credential {
  ApiKey(Secret<String>),
  AccessToken(Secret<String>),
}

impl Credential {
  pub fn expose(&self) -> &str {
    match self {
      Credential::ApiKey(secret) | Credential::AccessToken(secret) => secret.expose(),
    }
  }
}

pub fn url(base_url: &str, path: &str) -> String {
  format!("{}{}", base_url.trim_end_matches('/'), path)
}

pub fn inject_openai_credentials(headers: &mut HeaderMap, token: &str) {
  headers.insert(&AUTHORIZATION, HeaderValue::from_string(format!("Bearer {token}")));
}

pub fn inject_codex_credentials(headers: &mut HeaderMap, token: &str, provider_account_id: Option<&str>) {
  inject_openai_credentials(headers, token);
  if let Some(account_id) = provider_account_id.filter(|s| !s.trim().is_empty()) {
    headers.insert(&CHATGPT_ACCOUNT_ID, HeaderValue::from_string(account_id.to_string()));
  }
}

pub fn normalize_openai_platform_headers(headers: &HeaderMap, ctx: &HeaderPatchCtx<'_>) -> HeaderMap {
  let authorization = headers.get(&AUTHORIZATION).map(|value| value.as_str().to_string());
  let mut out = HeaderMap::new();
  if let Some(authorization) = authorization {
    out.insert(&AUTHORIZATION, HeaderValue::from_string(authorization));
  }
  out.insert(&ACCEPT, HeaderValue::from_static(accept_value(ctx.stream)));
  out.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
  if let Some(encoding) = ctx.content_encoding {
    out.insert(&CONTENT_ENCODING, HeaderValue::from_string(encoding.to_string()));
  }
  preserve_allowed(headers, &mut out, &[&USER_AGENT]);
  out
}

pub fn normalize_codex_headers(headers: &HeaderMap, ctx: &HeaderPatchCtx<'_>) -> HeaderMap {
  let originator = first_non_empty(headers, &[ORIGINATOR.as_str()])
    .unwrap_or("codex_cli_rs")
    .to_string();
  let user_agent = first_non_empty(headers, &[USER_AGENT.as_str()])
    .unwrap_or(CODEX_CLI_USER_AGENT)
    .to_string();
  let session_id = first_non_empty(headers, &[SESSION_ID_LOWER.as_str()]).map(str::to_string);
  let turn_metadata = first_non_empty(headers, &[X_CODEX_TURN_METADATA.as_str()]).map(str::to_string);
  let authorization = first_non_empty(headers, &[AUTHORIZATION.as_str()]).map(str::to_string);
  let chatgpt_account_id = first_non_empty(headers, &[CHATGPT_ACCOUNT_ID.as_str()]).map(str::to_string);

  let mut out = HeaderMap::new();
  if let Some(authorization) = authorization {
    out.insert(&AUTHORIZATION, HeaderValue::from_string(authorization));
  }
  out.insert(&ACCEPT, HeaderValue::from_static(accept_value(ctx.stream)));
  out.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
  out.insert(&OPENAI_BETA, HeaderValue::from_static(CODEX_RESPONSES_BETA));
  out.insert(&ORIGINATOR, HeaderValue::from_string(originator));
  out.insert(&VERSION, HeaderValue::from_static(CODEX_CLI_VERSION));
  out.insert(&USER_AGENT, HeaderValue::from_string(user_agent));
  if let Some(chatgpt_account_id) = chatgpt_account_id {
    out.insert(&CHATGPT_ACCOUNT_ID, HeaderValue::from_string(chatgpt_account_id));
  }
  if let Some(session_id) = session_id {
    out.insert(&SESSION_ID_LOWER, HeaderValue::from_string(session_id));
  }
  if let Some(turn_metadata) = turn_metadata {
    out.insert(&X_CODEX_TURN_METADATA, HeaderValue::from_string(turn_metadata));
  }
  if let Some(encoding) = ctx.content_encoding {
    out.insert(&CONTENT_ENCODING, HeaderValue::from_string(encoding.to_string()));
  }
  out
}

fn accept_value(stream: bool) -> &'static str {
  if stream {
    "text/event-stream"
  } else {
    "application/json"
  }
}

fn preserve_allowed(src: &HeaderMap, dst: &mut HeaderMap, allowed: &[&tokn_headers::HeaderName]) {
  for key in allowed {
    if let Some(value) = src.get(*key) {
      dst.insert(*key, HeaderValue::from_string(value.as_str().to_string()));
    }
  }
}

fn first_non_empty<'a>(headers: &'a HeaderMap, keys: &[&str]) -> Option<&'a str> {
  keys
    .iter()
    .filter_map(|key| headers.get(*key).map(|value| value.as_str().trim()))
    .find(|value| !value.is_empty())
}
