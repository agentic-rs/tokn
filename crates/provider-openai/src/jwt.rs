//! Minimal JWT *claim parser*. Does not verify signatures — codex's
//! ChatGPT OAuth flow trusts the issuer over TLS and only uses the
//! `id_token` to surface the `chatgpt_account_id` for outbound headers.
//!
//! Mirrors `parseJwtClaims` / `extractAccountIdFromClaims` from
//! `opencode/src/plugin/codex.ts`.

use base64::Engine;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct IdTokenClaims {
  pub chatgpt_account_id: Option<String>,
  pub email: Option<String>,
  pub organizations: Option<Vec<Organization>>,
  #[serde(rename = "https://api.openai.com/auth")]
  pub openai_auth: Option<OpenAiAuthClaim>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct OpenAiAuthClaim {
  pub chatgpt_account_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Organization {
  pub id: String,
}

/// Parse the unverified payload of a JWT. Returns `None` for malformed
/// inputs (wrong number of segments, base64 decode failure, JSON decode
/// failure).
pub fn parse_jwt_claims(token: &str) -> Option<IdTokenClaims> {
  let mut parts = token.split('.');
  let _header = parts.next()?;
  let payload = parts.next()?;
  let _sig = parts.next();
  // No fourth segment.
  if parts.next().is_some() {
    return None;
  }
  let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
    .decode(payload)
    .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(payload))
    .ok()?;
  serde_json::from_slice(&decoded).ok()
}

/// Pick the first available account id, mirroring opencode's precedence:
/// top-level `chatgpt_account_id` → namespaced
/// `https://api.openai.com/auth.chatgpt_account_id` → `organizations[0].id`.
pub fn extract_account_id(claims: &IdTokenClaims) -> Option<String> {
  claims
    .chatgpt_account_id
    .clone()
    .or_else(|| claims.openai_auth.as_ref().and_then(|a| a.chatgpt_account_id.clone()))
    .or_else(|| {
      claims
        .organizations
        .as_ref()
        .and_then(|o| o.first().map(|o| o.id.clone()))
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use base64::Engine;

  fn jwt(payload: &serde_json::Value) -> String {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"{\"alg\":\"none\"}");
    let body = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
    format!("{header}.{body}.")
  }

  #[test]
  fn parses_top_level_account_id() {
    let token = jwt(&serde_json::json!({"chatgpt_account_id": "acc-1", "email": "a@b"}));
    let claims = parse_jwt_claims(&token).unwrap();
    assert_eq!(extract_account_id(&claims).as_deref(), Some("acc-1"));
  }

  #[test]
  fn parses_namespaced_account_id() {
    let token = jwt(&serde_json::json!({
      "https://api.openai.com/auth": {"chatgpt_account_id": "acc-2"},
    }));
    let claims = parse_jwt_claims(&token).unwrap();
    assert_eq!(extract_account_id(&claims).as_deref(), Some("acc-2"));
  }

  #[test]
  fn falls_back_to_first_organization() {
    let token = jwt(&serde_json::json!({"organizations": [{"id": "org-x"}, {"id": "org-y"}]}));
    let claims = parse_jwt_claims(&token).unwrap();
    assert_eq!(extract_account_id(&claims).as_deref(), Some("org-x"));
  }

  #[test]
  fn missing_account_id_returns_none() {
    let token = jwt(&serde_json::json!({"email": "x"}));
    let claims = parse_jwt_claims(&token).unwrap();
    assert_eq!(extract_account_id(&claims), None);
  }

  #[test]
  fn malformed_jwt_returns_none() {
    assert!(parse_jwt_claims("not-a-jwt").is_none());
    assert!(parse_jwt_claims("a.b.c.d").is_none());
    assert!(parse_jwt_claims("a.!notbase64.c").is_none());
  }
}
