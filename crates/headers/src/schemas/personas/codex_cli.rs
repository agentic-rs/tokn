//! Headers emitted by the Codex CLI clients (`codex_exec`, `codex-tui`).
//!
//! Field set derived from the inbound real-world matrix. Codex sends several
//! transport-class headers using lowercase, no-prefix names (`originator`,
//! `version`, `session_id`, `thread_id`); these are kept verbatim rather
//! than canonicalised because that's what the upstream chatgpt.com endpoint
//! expects.
//!
//! The captured matrix contains multiple transport shapes:
//! `chatgpt.com` websocket upgrades, local/router SSE POSTs, and
//! browser-context account calls. A single typed schema therefore needs to
//! accept endpoint-specific optional headers rather than assuming one
//! normalized baseline.
//!
//! # Tiers
//! * **Required**: `User-Agent`, `Authorization`.
//! * **Standard**: `Accept`, `originator`, `version`, `Content-Type`,
//!   `Content-Length`, `session_id`, `thread_id`, `x-client-request-id`.
//! * **Extra**: every endpoint-specific header (Host, Upgrade, Sec-WebSocket-*,
//!   Cookie, x-codex-window-id, x-codex-beta-features, x-codex-turn-metadata,
//!   chatgpt-account-id, OpenAI-Beta, X-Request-Id, Connection).

use crate::error::Error;
use crate::keys;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::schema::{
  extra_loose, extra_strict, optional, put_opt, req_inbound_or, required, std_loose, std_strict, HeaderSchema, Tier,
};
use crate::vars::TemplateVars;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexCliHeaders {
  // Required
  #[serde(rename = "User-Agent", skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<SmolStr>,
  #[serde(rename = "Authorization", skip_serializing_if = "Option::is_none")]
  pub authorization: Option<SmolStr>,

  // Standard
  #[serde(rename = "Accept", skip_serializing_if = "Option::is_none")]
  pub accept: Option<SmolStr>,
  #[serde(rename = "originator", skip_serializing_if = "Option::is_none")]
  pub originator: Option<SmolStr>,
  #[serde(rename = "version", skip_serializing_if = "Option::is_none")]
  pub version: Option<SmolStr>,
  #[serde(rename = "Content-Type", skip_serializing_if = "Option::is_none")]
  pub content_type: Option<SmolStr>,
  #[serde(rename = "Content-Length", skip_serializing_if = "Option::is_none")]
  pub content_length: Option<SmolStr>,
  #[serde(rename = "session_id", skip_serializing_if = "Option::is_none")]
  pub session_id: Option<SmolStr>,
  #[serde(rename = "thread_id", skip_serializing_if = "Option::is_none")]
  pub thread_id: Option<SmolStr>,
  #[serde(rename = "x-client-request-id", skip_serializing_if = "Option::is_none")]
  pub client_request_id: Option<SmolStr>,

  // Extra
  /// NEVER stamped from a persona default: the persona-default host (e.g.
  /// `chatgpt.com`) is wrong for any other upstream.
  #[serde(rename = "Host", skip_serializing_if = "Option::is_none")]
  pub host: Option<SmolStr>,
  #[serde(rename = "Connection", skip_serializing_if = "Option::is_none")]
  pub connection: Option<SmolStr>,
  #[serde(rename = "Upgrade", skip_serializing_if = "Option::is_none")]
  pub upgrade: Option<SmolStr>,
  #[serde(rename = "chatgpt-account-id", skip_serializing_if = "Option::is_none")]
  pub chatgpt_account_id: Option<SmolStr>,
  #[serde(rename = "x-codex-window-id", skip_serializing_if = "Option::is_none")]
  pub codex_window_id: Option<SmolStr>,
  #[serde(rename = "x-codex-beta-features", skip_serializing_if = "Option::is_none")]
  pub codex_beta_features: Option<SmolStr>,
  #[serde(rename = "x-codex-turn-metadata", skip_serializing_if = "Option::is_none")]
  pub codex_turn_metadata: Option<SmolStr>,
  #[serde(rename = "OpenAI-Beta", skip_serializing_if = "Option::is_none")]
  pub openai_beta: Option<SmolStr>,
  #[serde(rename = "X-Request-Id", skip_serializing_if = "Option::is_none")]
  pub request_id: Option<SmolStr>,
  #[serde(rename = "Sec-WebSocket-Extensions", skip_serializing_if = "Option::is_none")]
  pub sec_websocket_extensions: Option<SmolStr>,
  #[serde(rename = "Sec-WebSocket-Key", skip_serializing_if = "Option::is_none")]
  pub sec_websocket_key: Option<SmolStr>,
  #[serde(rename = "Sec-WebSocket-Version", skip_serializing_if = "Option::is_none")]
  pub sec_websocket_version: Option<SmolStr>,
  #[serde(rename = "Cookie", skip_serializing_if = "Option::is_none")]
  pub cookie: Option<SmolStr>,
}

impl HeaderSchema for CodexCliHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: Some(required(map, &keys::USER_AGENT)?),
      authorization: Some(required(map, &keys::AUTHORIZATION)?),
      accept: optional(map, &keys::ACCEPT),
      originator: optional(map, &keys::ORIGINATOR),
      version: optional(map, &keys::VERSION),
      content_type: optional(map, &keys::CONTENT_TYPE),
      content_length: optional(map, &keys::CONTENT_LENGTH),
      session_id: optional(map, &keys::SESSION_ID_LOWER),
      thread_id: optional(map, &keys::THREAD_ID),
      client_request_id: optional(map, &keys::X_CLIENT_REQUEST_ID),
      host: optional(map, &keys::HOST),
      connection: optional(map, &keys::CONNECTION),
      upgrade: optional(map, &keys::UPGRADE),
      chatgpt_account_id: optional(map, &keys::CHATGPT_ACCOUNT_ID),
      codex_window_id: optional(map, &keys::X_CODEX_WINDOW_ID),
      codex_beta_features: optional(map, &keys::X_CODEX_BETA_FEATURES),
      codex_turn_metadata: optional(map, &keys::X_CODEX_TURN_METADATA),
      openai_beta: optional(map, &keys::OPENAI_BETA),
      request_id: optional(map, &keys::X_REQUEST_ID),
      sec_websocket_extensions: optional(map, &keys::SEC_WEBSOCKET_EXTENSIONS),
      sec_websocket_key: optional(map, &keys::SEC_WEBSOCKET_KEY),
      sec_websocket_version: optional(map, &keys::SEC_WEBSOCKET_VERSION),
      cookie: optional(map, &keys::COOKIE),
    })
  }
  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::AUTHORIZATION, &self.authorization);
    put_opt(&mut m, &keys::ACCEPT, &self.accept);
    put_opt(&mut m, &keys::ORIGINATOR, &self.originator);
    put_opt(&mut m, &keys::VERSION, &self.version);
    put_opt(&mut m, &keys::CONTENT_TYPE, &self.content_type);
    put_opt(&mut m, &keys::CONTENT_LENGTH, &self.content_length);
    put_opt(&mut m, &keys::SESSION_ID_LOWER, &self.session_id);
    put_opt(&mut m, &keys::THREAD_ID, &self.thread_id);
    put_opt(&mut m, &keys::X_CLIENT_REQUEST_ID, &self.client_request_id);
    put_opt(&mut m, &keys::HOST, &self.host);
    put_opt(&mut m, &keys::CONNECTION, &self.connection);
    put_opt(&mut m, &keys::UPGRADE, &self.upgrade);
    put_opt(&mut m, &keys::CHATGPT_ACCOUNT_ID, &self.chatgpt_account_id);
    put_opt(&mut m, &keys::X_CODEX_WINDOW_ID, &self.codex_window_id);
    put_opt(&mut m, &keys::X_CODEX_BETA_FEATURES, &self.codex_beta_features);
    put_opt(&mut m, &keys::X_CODEX_TURN_METADATA, &self.codex_turn_metadata);
    put_opt(&mut m, &keys::OPENAI_BETA, &self.openai_beta);
    put_opt(&mut m, &keys::X_REQUEST_ID, &self.request_id);
    put_opt(&mut m, &keys::SEC_WEBSOCKET_EXTENSIONS, &self.sec_websocket_extensions);
    put_opt(&mut m, &keys::SEC_WEBSOCKET_KEY, &self.sec_websocket_key);
    put_opt(&mut m, &keys::SEC_WEBSOCKET_VERSION, &self.sec_websocket_version);
    put_opt(&mut m, &keys::COOKIE, &self.cookie);
    m
  }
  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 23] = [
      (&keys::USER_AGENT, Tier::Required),
      (&keys::AUTHORIZATION, Tier::Required),
      (&keys::ACCEPT, Tier::Standard),
      (&keys::ORIGINATOR, Tier::Standard),
      (&keys::VERSION, Tier::Standard),
      (&keys::CONTENT_TYPE, Tier::Standard),
      (&keys::CONTENT_LENGTH, Tier::Standard),
      (&keys::SESSION_ID_LOWER, Tier::Standard),
      (&keys::THREAD_ID, Tier::Standard),
      (&keys::X_CLIENT_REQUEST_ID, Tier::Standard),
      (&keys::HOST, Tier::Extra),
      (&keys::CONNECTION, Tier::Extra),
      (&keys::UPGRADE, Tier::Extra),
      (&keys::CHATGPT_ACCOUNT_ID, Tier::Extra),
      (&keys::X_CODEX_WINDOW_ID, Tier::Extra),
      (&keys::X_CODEX_BETA_FEATURES, Tier::Extra),
      (&keys::X_CODEX_TURN_METADATA, Tier::Extra),
      (&keys::OPENAI_BETA, Tier::Extra),
      (&keys::X_REQUEST_ID, Tier::Extra),
      (&keys::SEC_WEBSOCKET_EXTENSIONS, Tier::Extra),
      (&keys::SEC_WEBSOCKET_KEY, Tier::Extra),
      (&keys::SEC_WEBSOCKET_VERSION, Tier::Extra),
      (&keys::COOKIE, Tier::Extra),
    ];
    &FIELDS
  }
}

impl CodexCliHeaders {
  /// Persona defaults derived from real captured traffic.
  pub fn defaults() -> Self {
    Self {
      user_agent: Some("codex_exec/0.130.0 (Ubuntu 24.4.0; x86_64) unknown (codex_exec; 0.130.0)".into()),
      authorization: Some("<missing>".into()),
      accept: Some("text/event-stream".into()),
      originator: Some("codex_exec".into()),
      version: Some("0.130.0".into()),
      content_type: Some("application/json".into()),
      content_length: None,
      session_id: None,
      thread_id: None,
      client_request_id: None,
      ..Default::default()
    }
  }

  /// Build (loose).
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      user_agent: Some(req_inbound_or(inbound, &keys::USER_AGENT, || {
        d.user_agent.clone().expect("default user_agent")
      })),
      authorization: Some(req_inbound_or(inbound, &keys::AUTHORIZATION, || {
        d.authorization.clone().expect("default authorization")
      })),
      accept: std_loose(inbound, &keys::ACCEPT),
      originator: std_loose(inbound, &keys::ORIGINATOR),
      version: std_loose(inbound, &keys::VERSION),
      content_type: std_loose(inbound, &keys::CONTENT_TYPE),
      content_length: std_loose(inbound, &keys::CONTENT_LENGTH),
      session_id: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::SESSION_ID_LOWER)),
      thread_id: std_loose(inbound, &keys::THREAD_ID),
      client_request_id: vars
        .request_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_CLIENT_REQUEST_ID)),
      host: extra_loose(inbound, &keys::HOST, || None),
      connection: extra_loose(inbound, &keys::CONNECTION, || None),
      upgrade: extra_loose(inbound, &keys::UPGRADE, || None),
      chatgpt_account_id: vars
        .account_id
        .clone()
        .or_else(|| extra_loose(inbound, &keys::CHATGPT_ACCOUNT_ID, || None)),
      codex_window_id: extra_loose(inbound, &keys::X_CODEX_WINDOW_ID, || None),
      codex_beta_features: extra_loose(inbound, &keys::X_CODEX_BETA_FEATURES, || None),
      codex_turn_metadata: extra_loose(inbound, &keys::X_CODEX_TURN_METADATA, || None),
      openai_beta: extra_loose(inbound, &keys::OPENAI_BETA, || None),
      request_id: vars
        .request_id
        .clone()
        .or_else(|| extra_loose(inbound, &keys::X_REQUEST_ID, || None)),
      sec_websocket_extensions: extra_loose(inbound, &keys::SEC_WEBSOCKET_EXTENSIONS, || None),
      sec_websocket_key: extra_loose(inbound, &keys::SEC_WEBSOCKET_KEY, || None),
      sec_websocket_version: extra_loose(inbound, &keys::SEC_WEBSOCKET_VERSION, || None),
      cookie: extra_loose(inbound, &keys::COOKIE, || None),
    })
  }

  /// Build (strict).
  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    let mut base = Self::build(vars, inbound)?;
    base.accept = base.accept.or_else(|| {
      std_strict(inbound, &keys::ACCEPT, || {
        d.accept.clone().unwrap_or_else(|| "text/event-stream".into())
      })
    });
    base.originator = base.originator.or_else(|| {
      std_strict(inbound, &keys::ORIGINATOR, || {
        d.originator.clone().unwrap_or_else(|| "codex_exec".into())
      })
    });
    base.version = base.version.or_else(|| {
      std_strict(inbound, &keys::VERSION, || {
        d.version.clone().unwrap_or_else(|| "0.130.0".into())
      })
    });
    base.content_type = base.content_type.or_else(|| {
      std_strict(inbound, &keys::CONTENT_TYPE, || {
        d.content_type.clone().unwrap_or_else(|| "application/json".into())
      })
    });
    base.content_length = base
      .content_length
      .or_else(|| std_strict(inbound, &keys::CONTENT_LENGTH, || "0".into()));
    base.session_id = base
      .session_id
      .or_else(|| std_strict(inbound, &keys::SESSION_ID_LOWER, || "unknown".into()));
    base.thread_id = base
      .thread_id
      .or_else(|| std_strict(inbound, &keys::THREAD_ID, || "unknown".into()));
    base.client_request_id = base
      .client_request_id
      .or_else(|| std_strict(inbound, &keys::X_CLIENT_REQUEST_ID, || "unknown".into()));
    // Extra: strict mode passes through inbound only.
    base.host = extra_strict(inbound, &keys::HOST);
    base.connection = extra_strict(inbound, &keys::CONNECTION);
    base.upgrade = extra_strict(inbound, &keys::UPGRADE);
    base.chatgpt_account_id = vars
      .account_id
      .clone()
      .or_else(|| extra_strict(inbound, &keys::CHATGPT_ACCOUNT_ID));
    base.codex_window_id = extra_strict(inbound, &keys::X_CODEX_WINDOW_ID);
    base.codex_beta_features = extra_strict(inbound, &keys::X_CODEX_BETA_FEATURES);
    base.codex_turn_metadata = extra_strict(inbound, &keys::X_CODEX_TURN_METADATA);
    base.openai_beta = extra_strict(inbound, &keys::OPENAI_BETA);
    base.request_id = vars
      .request_id
      .clone()
      .or_else(|| extra_strict(inbound, &keys::X_REQUEST_ID));
    base.sec_websocket_extensions = extra_strict(inbound, &keys::SEC_WEBSOCKET_EXTENSIONS);
    base.sec_websocket_key = extra_strict(inbound, &keys::SEC_WEBSOCKET_KEY);
    base.sec_websocket_version = extra_strict(inbound, &keys::SEC_WEBSOCKET_VERSION);
    base.cookie = extra_strict(inbound, &keys::COOKIE);
    Ok(base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn responses_sample() -> CodexCliHeaders {
    CodexCliHeaders {
      user_agent: Some("codex_exec/0.130.0 (Ubuntu 24.4.0; x86_64) unknown (codex_exec; 0.130.0)".into()),
      authorization: Some("<redacted>".into()),
      host: Some("chatgpt.com".into()),
      accept: Some("text/event-stream".into()),
      connection: None,
      upgrade: None,
      originator: Some("codex_exec".into()),
      chatgpt_account_id: Some("<redacted>".into()),
      version: Some("0.130.0".into()),
      content_type: Some("application/json".into()),
      content_length: Some("45273".into()),
      session_id: Some("019e271b-4023-7081-be3e-7a69d97138a2".into()),
      thread_id: Some("019e271b-4023-7081-be3e-7a69d97138a2".into()),
      client_request_id: Some("019e271b-4023-7081-be3e-7a69d97138a2".into()),
      codex_window_id: Some("019e271b-4023-7081-be3e-7a69d97138a2:0".into()),
      codex_beta_features: Some("terminal_resize_reflow".into()),
      codex_turn_metadata: Some("{\"session_id\":\"019e271b\"}".into()),
      openai_beta: None,
      request_id: None,
      sec_websocket_extensions: None,
      sec_websocket_key: None,
      sec_websocket_version: None,
      cookie: None,
    }
  }

  #[test]
  fn responses_round_trip() {
    let h = responses_sample();
    assert_eq!(CodexCliHeaders::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn missing_required_errors() {
    let m = HeaderMap::new();
    assert!(matches!(CodexCliHeaders::parse(&m), Err(Error::MissingHeader { .. })));
  }

  #[test]
  fn build_with_empty_inbound_uses_required_defaults() {
    let h = CodexCliHeaders::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(
      h.user_agent.as_deref(),
      Some("codex_exec/0.130.0 (Ubuntu 24.4.0; x86_64) unknown (codex_exec; 0.130.0)")
    );
    assert_eq!(h.authorization.as_deref(), Some("<missing>"));
    assert!(h.host.is_none());
    // Loose skips Standard fields not in inbound.
    assert!(h.accept.is_none());
    assert!(h.originator.is_none());
    assert!(h.version.is_none());
    assert!(h.content_type.is_none());
    assert!(h.session_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::USER_AGENT, "codex_exec/9.9.9");
    inbound.insert(&keys::AUTHORIZATION, "Bearer abc");
    inbound.insert(&keys::OPENAI_BETA, "responses=v1");
    inbound.insert(&keys::HOST, "chatgpt.com");
    let h = CodexCliHeaders::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("codex_exec/9.9.9"));
    assert_eq!(h.authorization.as_deref(), Some("Bearer abc"));
    assert_eq!(h.openai_beta.as_deref(), Some("responses=v1"));
    assert_eq!(h.host.as_deref(), Some("chatgpt.com"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      request_id: Some("req_42".into()),
      account_id: Some("acct_z".into()),
      ..Default::default()
    };
    let h = CodexCliHeaders::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.session_id.as_deref(), Some("ses_xyz"));
    assert_eq!(h.client_request_id.as_deref(), Some("req_42"));
    assert_eq!(h.request_id.as_deref(), Some("req_42"));
    assert_eq!(h.chatgpt_account_id.as_deref(), Some("acct_z"));
  }

  #[test]
  fn build_strict_fills_standard() {
    let h = CodexCliHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert!(h.accept.is_some());
    assert!(h.originator.is_some());
    assert!(h.version.is_some());
    assert!(h.content_type.is_some());
    assert!(h.session_id.is_some());
    assert!(h.thread_id.is_some());
    // Extra still skipped.
    assert!(h.host.is_none());
    assert!(h.openai_beta.is_none());
  }
}
