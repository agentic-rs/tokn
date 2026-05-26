use crate::util::secret::Secret;
use crate::{HeaderPatchCtx, Result};
use tokn_headers::keys::{
  ACCEPT, AUTHORIZATION, CHATGPT_ACCOUNT_ID, CONTENT_ENCODING, CONTENT_TYPE, OPENAI_BETA, ORIGINATOR, SESSION_ID_LOWER,
  USER_AGENT, VERSION, X_API_KEY, X_CODEX_TURN_METADATA, X_GOOG_API_KEY,
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

pub fn patch_openai_headers(headers: &mut HeaderMap, token: &str, ctx: &HeaderPatchCtx<'_>) -> Result<()> {
  headers.insert(&AUTHORIZATION, HeaderValue::from_string(format!("Bearer {token}")));
  headers.insert(
    &ACCEPT,
    HeaderValue::from_static(if ctx.stream {
      "text/event-stream"
    } else {
      "application/json"
    }),
  );
  headers.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
  if let Some(encoding) = ctx.content_encoding {
    headers.insert(&CONTENT_ENCODING, HeaderValue::from_string(encoding.to_string()));
  }
  Ok(())
}

pub fn normalize_openai_platform_headers(headers: &mut HeaderMap, token: &str, ctx: &HeaderPatchCtx<'_>) {
  remove_auth_residue(headers);
  remove_codex_only_headers(headers);
  let mut out = HeaderMap::new();
  out.insert(&AUTHORIZATION, HeaderValue::from_string(format!("Bearer {token}")));
  out.insert(&ACCEPT, HeaderValue::from_static(accept_value(ctx.stream)));
  out.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
  if let Some(encoding) = ctx.content_encoding {
    out.insert(&CONTENT_ENCODING, HeaderValue::from_string(encoding.to_string()));
  }
  preserve_allowed(headers, &mut out, &[&USER_AGENT]);
  *headers = out;
}

pub fn normalize_codex_headers(
  headers: &mut HeaderMap,
  token: &str,
  provider_account_id: Option<&str>,
  ctx: &HeaderPatchCtx<'_>,
) {
  remove_auth_residue(headers);

  let originator = first_non_empty(headers, &[ORIGINATOR.as_str()]).unwrap_or("codex_cli_rs");
  let user_agent = first_non_empty(headers, &[USER_AGENT.as_str()]).unwrap_or(CODEX_CLI_USER_AGENT);
  let session_id = first_non_empty(headers, &[SESSION_ID_LOWER.as_str()]);
  let turn_metadata = first_non_empty(headers, &[X_CODEX_TURN_METADATA.as_str()]);

  let mut out = HeaderMap::new();
  out.insert(&AUTHORIZATION, HeaderValue::from_string(format!("Bearer {token}")));
  out.insert(&ACCEPT, HeaderValue::from_static(accept_value(ctx.stream)));
  out.insert(&CONTENT_TYPE, HeaderValue::from_static("application/json"));
  out.insert(&OPENAI_BETA, HeaderValue::from_static(CODEX_RESPONSES_BETA));
  out.insert(&ORIGINATOR, HeaderValue::from_string(originator.to_string()));
  out.insert(&VERSION, HeaderValue::from_static(CODEX_CLI_VERSION));
  out.insert(&USER_AGENT, HeaderValue::from_string(user_agent.to_string()));
  if let Some(account_id) = provider_account_id.filter(|s| !s.trim().is_empty()) {
    out.insert(&CHATGPT_ACCOUNT_ID, HeaderValue::from_string(account_id.to_string()));
  }
  if let Some(session_id) = session_id {
    out.insert(&SESSION_ID_LOWER, HeaderValue::from_string(session_id.to_string()));
  }
  if let Some(turn_metadata) = turn_metadata {
    out.insert(
      &X_CODEX_TURN_METADATA,
      HeaderValue::from_string(turn_metadata.to_string()),
    );
  }
  if let Some(encoding) = ctx.content_encoding {
    out.insert(&CONTENT_ENCODING, HeaderValue::from_string(encoding.to_string()));
  }
  *headers = out;
}

fn accept_value(stream: bool) -> &'static str {
  if stream {
    "text/event-stream"
  } else {
    "application/json"
  }
}

fn remove_auth_residue(headers: &mut HeaderMap) {
  headers.remove(&AUTHORIZATION);
  headers.remove(&X_API_KEY);
  headers.remove(&X_GOOG_API_KEY);
}

fn remove_codex_only_headers(headers: &mut HeaderMap) {
  for key in [
    OPENAI_BETA.as_str(),
    ORIGINATOR.as_str(),
    SESSION_ID_LOWER.as_str(),
    "conversation_id",
    "x-codex-turn-state",
    X_CODEX_TURN_METADATA.as_str(),
    CHATGPT_ACCOUNT_ID.as_str(),
    VERSION.as_str(),
  ] {
    headers.remove(key);
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
