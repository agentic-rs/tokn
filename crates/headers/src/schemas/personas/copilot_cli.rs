//! Headers emitted by the GitHub Copilot CLI client (`copilot/<ver>`).
//!
//! Field set verified against one captured request:
//!
//! ```text
//! POST https://api.business.githubcopilot.com/responses
//! user-agent: copilot/1.0.25 (client/sdk win32 v22.19.0) term/unknown
//! copilot-integration-id: copilot-developer-cli
//! ```
//!
//! The Copilot CLI is built on the OpenAI Stainless-generated JS SDK; the
//! `x-stainless-*` family is modelled as Extra because it fingerprints the
//! transport implementation rather than the semantic Copilot request.
//!
//! # Tiers
//! * **Required**: `User-Agent`, `Authorization`.
//! * **Standard**: core request shape, Copilot identity, and per-call
//!   correlation headers.
//! * **Extra**: Stainless fingerprint, body framing, cookies, request id.

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
pub struct CopilotCliHeaders {
  #[serde(rename = "User-Agent", skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<SmolStr>,
  #[serde(rename = "Authorization", skip_serializing_if = "Option::is_none")]
  pub authorization: Option<SmolStr>,
  #[serde(rename = "Content-Type", skip_serializing_if = "Option::is_none")]
  pub content_type: Option<SmolStr>,
  #[serde(rename = "Accept", skip_serializing_if = "Option::is_none")]
  pub accept: Option<SmolStr>,
  #[serde(rename = "Accept-Encoding", skip_serializing_if = "Option::is_none")]
  pub accept_encoding: Option<SmolStr>,
  #[serde(rename = "Accept-Language", skip_serializing_if = "Option::is_none")]
  pub accept_language: Option<SmolStr>,
  #[serde(rename = "Sec-Fetch-Mode", skip_serializing_if = "Option::is_none")]
  pub sec_fetch_mode: Option<SmolStr>,
  #[serde(rename = "Copilot-Integration-Id", skip_serializing_if = "Option::is_none")]
  pub copilot_integration_id: Option<SmolStr>,
  #[serde(rename = "OpenAI-Intent", skip_serializing_if = "Option::is_none")]
  pub openai_intent: Option<SmolStr>,
  #[serde(rename = "X-Initiator", skip_serializing_if = "Option::is_none")]
  pub initiator: Option<SmolStr>,
  #[serde(rename = "X-GitHub-Api-Version", skip_serializing_if = "Option::is_none")]
  pub github_api_version: Option<SmolStr>,
  #[serde(rename = "X-Interaction-Id", skip_serializing_if = "Option::is_none")]
  pub interaction_id: Option<SmolStr>,
  #[serde(rename = "X-Interaction-Type", skip_serializing_if = "Option::is_none")]
  pub interaction_type: Option<SmolStr>,
  #[serde(rename = "X-Client-Session-Id", skip_serializing_if = "Option::is_none")]
  pub client_session_id: Option<SmolStr>,
  #[serde(rename = "X-Agent-Task-Id", skip_serializing_if = "Option::is_none")]
  pub agent_task_id: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Retry-Count", skip_serializing_if = "Option::is_none")]
  pub stainless_retry_count: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Lang", skip_serializing_if = "Option::is_none")]
  pub stainless_lang: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Package-Version", skip_serializing_if = "Option::is_none")]
  pub stainless_package_version: Option<SmolStr>,
  #[serde(rename = "X-Stainless-OS", skip_serializing_if = "Option::is_none")]
  pub stainless_os: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Arch", skip_serializing_if = "Option::is_none")]
  pub stainless_arch: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Runtime", skip_serializing_if = "Option::is_none")]
  pub stainless_runtime: Option<SmolStr>,
  #[serde(rename = "X-Stainless-Runtime-Version", skip_serializing_if = "Option::is_none")]
  pub stainless_runtime_version: Option<SmolStr>,
  #[serde(rename = "Content-Length", skip_serializing_if = "Option::is_none")]
  pub content_length: Option<SmolStr>,
  #[serde(rename = "Cookie", skip_serializing_if = "Option::is_none")]
  pub cookie: Option<SmolStr>,
  #[serde(rename = "X-Request-Id", skip_serializing_if = "Option::is_none")]
  pub request_id: Option<SmolStr>,
}

impl HeaderSchema for CopilotCliHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: Some(required(map, &keys::USER_AGENT)?),
      authorization: Some(required(map, &keys::AUTHORIZATION)?),
      content_type: optional(map, &keys::CONTENT_TYPE),
      accept: optional(map, &keys::ACCEPT),
      accept_encoding: optional(map, &keys::ACCEPT_ENCODING),
      accept_language: optional(map, &keys::ACCEPT_LANGUAGE),
      sec_fetch_mode: optional(map, &keys::SEC_FETCH_MODE),
      copilot_integration_id: optional(map, &keys::COPILOT_INTEGRATION_ID),
      openai_intent: optional(map, &keys::OPENAI_INTENT),
      initiator: optional(map, &keys::X_INITIATOR),
      github_api_version: optional(map, &keys::X_GITHUB_API_VERSION),
      interaction_id: optional(map, &keys::X_INTERACTION_ID),
      interaction_type: optional(map, &keys::X_INTERACTION_TYPE),
      client_session_id: optional(map, &keys::X_CLIENT_SESSION_ID),
      agent_task_id: optional(map, &keys::X_AGENT_TASK_ID),
      stainless_retry_count: optional(map, &keys::X_STAINLESS_RETRY_COUNT),
      stainless_lang: optional(map, &keys::X_STAINLESS_LANG),
      stainless_package_version: optional(map, &keys::X_STAINLESS_PACKAGE_VERSION),
      stainless_os: optional(map, &keys::X_STAINLESS_OS),
      stainless_arch: optional(map, &keys::X_STAINLESS_ARCH),
      stainless_runtime: optional(map, &keys::X_STAINLESS_RUNTIME),
      stainless_runtime_version: optional(map, &keys::X_STAINLESS_RUNTIME_VERSION),
      content_length: optional(map, &keys::CONTENT_LENGTH),
      cookie: optional(map, &keys::COOKIE),
      request_id: optional(map, &keys::X_REQUEST_ID),
    })
  }

  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::AUTHORIZATION, &self.authorization);
    put_opt(&mut m, &keys::CONTENT_TYPE, &self.content_type);
    put_opt(&mut m, &keys::ACCEPT, &self.accept);
    put_opt(&mut m, &keys::ACCEPT_ENCODING, &self.accept_encoding);
    put_opt(&mut m, &keys::ACCEPT_LANGUAGE, &self.accept_language);
    put_opt(&mut m, &keys::SEC_FETCH_MODE, &self.sec_fetch_mode);
    put_opt(&mut m, &keys::COPILOT_INTEGRATION_ID, &self.copilot_integration_id);
    put_opt(&mut m, &keys::OPENAI_INTENT, &self.openai_intent);
    put_opt(&mut m, &keys::X_INITIATOR, &self.initiator);
    put_opt(&mut m, &keys::X_GITHUB_API_VERSION, &self.github_api_version);
    put_opt(&mut m, &keys::X_INTERACTION_ID, &self.interaction_id);
    put_opt(&mut m, &keys::X_INTERACTION_TYPE, &self.interaction_type);
    put_opt(&mut m, &keys::X_CLIENT_SESSION_ID, &self.client_session_id);
    put_opt(&mut m, &keys::X_AGENT_TASK_ID, &self.agent_task_id);
    put_opt(&mut m, &keys::X_STAINLESS_RETRY_COUNT, &self.stainless_retry_count);
    put_opt(&mut m, &keys::X_STAINLESS_LANG, &self.stainless_lang);
    put_opt(
      &mut m,
      &keys::X_STAINLESS_PACKAGE_VERSION,
      &self.stainless_package_version,
    );
    put_opt(&mut m, &keys::X_STAINLESS_OS, &self.stainless_os);
    put_opt(&mut m, &keys::X_STAINLESS_ARCH, &self.stainless_arch);
    put_opt(&mut m, &keys::X_STAINLESS_RUNTIME, &self.stainless_runtime);
    put_opt(
      &mut m,
      &keys::X_STAINLESS_RUNTIME_VERSION,
      &self.stainless_runtime_version,
    );
    put_opt(&mut m, &keys::CONTENT_LENGTH, &self.content_length);
    put_opt(&mut m, &keys::COOKIE, &self.cookie);
    put_opt(&mut m, &keys::X_REQUEST_ID, &self.request_id);
    m
  }

  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 25] = [
      (&keys::USER_AGENT, Tier::Required),
      (&keys::AUTHORIZATION, Tier::Required),
      (&keys::CONTENT_TYPE, Tier::Standard),
      (&keys::ACCEPT, Tier::Standard),
      (&keys::ACCEPT_ENCODING, Tier::Standard),
      (&keys::ACCEPT_LANGUAGE, Tier::Standard),
      (&keys::SEC_FETCH_MODE, Tier::Standard),
      (&keys::COPILOT_INTEGRATION_ID, Tier::Standard),
      (&keys::OPENAI_INTENT, Tier::Standard),
      (&keys::X_INITIATOR, Tier::Standard),
      (&keys::X_GITHUB_API_VERSION, Tier::Standard),
      (&keys::X_INTERACTION_ID, Tier::Standard),
      (&keys::X_INTERACTION_TYPE, Tier::Standard),
      (&keys::X_CLIENT_SESSION_ID, Tier::Standard),
      (&keys::X_AGENT_TASK_ID, Tier::Standard),
      (&keys::X_STAINLESS_RETRY_COUNT, Tier::Extra),
      (&keys::X_STAINLESS_LANG, Tier::Extra),
      (&keys::X_STAINLESS_PACKAGE_VERSION, Tier::Extra),
      (&keys::X_STAINLESS_OS, Tier::Extra),
      (&keys::X_STAINLESS_ARCH, Tier::Extra),
      (&keys::X_STAINLESS_RUNTIME, Tier::Extra),
      (&keys::X_STAINLESS_RUNTIME_VERSION, Tier::Extra),
      (&keys::CONTENT_LENGTH, Tier::Extra),
      (&keys::COOKIE, Tier::Extra),
      (&keys::X_REQUEST_ID, Tier::Extra),
    ];
    &FIELDS
  }
}

impl CopilotCliHeaders {
  pub fn defaults() -> Self {
    Self {
      user_agent: Some("copilot/1.0.25 (client/sdk win32 v22.19.0) term/unknown".into()),
      authorization: Some("<missing>".into()),
      content_type: Some("application/json".into()),
      accept: Some("application/json".into()),
      accept_encoding: Some("br, gzip, deflate".into()),
      accept_language: Some("*".into()),
      sec_fetch_mode: Some("cors".into()),
      copilot_integration_id: Some("copilot-developer-cli".into()),
      openai_intent: Some("conversation-agent".into()),
      initiator: Some("user".into()),
      github_api_version: Some("2026-01-09".into()),
      interaction_id: Some("00000000-0000-0000-0000-000000000000".into()),
      interaction_type: Some("conversation-user".into()),
      client_session_id: Some("00000000-0000-0000-0000-000000000000".into()),
      agent_task_id: Some("00000000-0000-0000-0000-000000000000".into()),
      ..Default::default()
    }
  }

  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      user_agent: Some(req_inbound_or(inbound, &keys::USER_AGENT, || {
        d.user_agent.clone().expect("default user_agent")
      })),
      authorization: Some(req_inbound_or(inbound, &keys::AUTHORIZATION, || {
        d.authorization.clone().expect("default authorization")
      })),
      content_type: std_loose(inbound, &keys::CONTENT_TYPE),
      accept: std_loose(inbound, &keys::ACCEPT),
      accept_encoding: std_loose(inbound, &keys::ACCEPT_ENCODING),
      accept_language: std_loose(inbound, &keys::ACCEPT_LANGUAGE),
      sec_fetch_mode: std_loose(inbound, &keys::SEC_FETCH_MODE),
      copilot_integration_id: std_loose(inbound, &keys::COPILOT_INTEGRATION_ID),
      openai_intent: std_loose(inbound, &keys::OPENAI_INTENT),
      initiator: std_loose(inbound, &keys::X_INITIATOR),
      github_api_version: std_loose(inbound, &keys::X_GITHUB_API_VERSION),
      interaction_id: vars
        .interaction_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_INTERACTION_ID)),
      interaction_type: std_loose(inbound, &keys::X_INTERACTION_TYPE),
      client_session_id: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_CLIENT_SESSION_ID)),
      agent_task_id: std_loose(inbound, &keys::X_AGENT_TASK_ID),
      stainless_retry_count: extra_loose(inbound, &keys::X_STAINLESS_RETRY_COUNT, || None),
      stainless_lang: extra_loose(inbound, &keys::X_STAINLESS_LANG, || None),
      stainless_package_version: extra_loose(inbound, &keys::X_STAINLESS_PACKAGE_VERSION, || None),
      stainless_os: extra_loose(inbound, &keys::X_STAINLESS_OS, || None),
      stainless_arch: extra_loose(inbound, &keys::X_STAINLESS_ARCH, || None),
      stainless_runtime: extra_loose(inbound, &keys::X_STAINLESS_RUNTIME, || None),
      stainless_runtime_version: extra_loose(inbound, &keys::X_STAINLESS_RUNTIME_VERSION, || None),
      content_length: extra_loose(inbound, &keys::CONTENT_LENGTH, || None),
      cookie: extra_loose(inbound, &keys::COOKIE, || None),
      request_id: vars
        .request_id
        .clone()
        .or_else(|| extra_loose(inbound, &keys::X_REQUEST_ID, || None)),
    })
  }

  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    let mut base = Self::build(vars, inbound)?;
    base.content_type = base.content_type.or_else(|| {
      std_strict(inbound, &keys::CONTENT_TYPE, || {
        d.content_type.clone().expect("default content_type")
      })
    });
    base.accept = base
      .accept
      .or_else(|| std_strict(inbound, &keys::ACCEPT, || d.accept.clone().expect("default accept")));
    base.accept_encoding = base.accept_encoding.or_else(|| {
      std_strict(inbound, &keys::ACCEPT_ENCODING, || {
        d.accept_encoding.clone().expect("default accept_encoding")
      })
    });
    base.accept_language = base.accept_language.or_else(|| {
      std_strict(inbound, &keys::ACCEPT_LANGUAGE, || {
        d.accept_language.clone().expect("default accept_language")
      })
    });
    base.sec_fetch_mode = base.sec_fetch_mode.or_else(|| {
      std_strict(inbound, &keys::SEC_FETCH_MODE, || {
        d.sec_fetch_mode.clone().expect("default sec_fetch_mode")
      })
    });
    base.copilot_integration_id = base.copilot_integration_id.or_else(|| {
      std_strict(inbound, &keys::COPILOT_INTEGRATION_ID, || {
        d.copilot_integration_id
          .clone()
          .expect("default copilot_integration_id")
      })
    });
    base.openai_intent = base.openai_intent.or_else(|| {
      std_strict(inbound, &keys::OPENAI_INTENT, || {
        d.openai_intent.clone().expect("default openai_intent")
      })
    });
    base.initiator = base.initiator.or_else(|| {
      std_strict(inbound, &keys::X_INITIATOR, || {
        d.initiator.clone().expect("default initiator")
      })
    });
    base.github_api_version = base.github_api_version.or_else(|| {
      std_strict(inbound, &keys::X_GITHUB_API_VERSION, || {
        d.github_api_version.clone().expect("default github_api_version")
      })
    });
    base.interaction_id = base.interaction_id.or_else(|| {
      std_strict(inbound, &keys::X_INTERACTION_ID, || {
        d.interaction_id.clone().expect("default interaction_id")
      })
    });
    base.interaction_type = base.interaction_type.or_else(|| {
      std_strict(inbound, &keys::X_INTERACTION_TYPE, || {
        d.interaction_type.clone().expect("default interaction_type")
      })
    });
    base.client_session_id = base.client_session_id.or_else(|| {
      std_strict(inbound, &keys::X_CLIENT_SESSION_ID, || {
        d.client_session_id.clone().expect("default client_session_id")
      })
    });
    base.agent_task_id = base.agent_task_id.or_else(|| {
      std_strict(inbound, &keys::X_AGENT_TASK_ID, || {
        d.agent_task_id.clone().expect("default agent_task_id")
      })
    });
    base.stainless_retry_count = extra_strict(inbound, &keys::X_STAINLESS_RETRY_COUNT);
    base.stainless_lang = extra_strict(inbound, &keys::X_STAINLESS_LANG);
    base.stainless_package_version = extra_strict(inbound, &keys::X_STAINLESS_PACKAGE_VERSION);
    base.stainless_os = extra_strict(inbound, &keys::X_STAINLESS_OS);
    base.stainless_arch = extra_strict(inbound, &keys::X_STAINLESS_ARCH);
    base.stainless_runtime = extra_strict(inbound, &keys::X_STAINLESS_RUNTIME);
    base.stainless_runtime_version = extra_strict(inbound, &keys::X_STAINLESS_RUNTIME_VERSION);
    base.content_length = extra_strict(inbound, &keys::CONTENT_LENGTH);
    base.cookie = extra_strict(inbound, &keys::COOKIE);
    base.request_id = vars
      .request_id
      .clone()
      .or_else(|| extra_strict(inbound, &keys::X_REQUEST_ID));
    Ok(base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn captured() -> CopilotCliHeaders {
    CopilotCliHeaders {
      user_agent: Some("copilot/1.0.25 (client/sdk win32 v22.19.0) term/unknown".into()),
      authorization: Some("Bearer gho_xxxx".into()),
      content_type: Some("application/json".into()),
      accept: Some("application/json".into()),
      accept_encoding: Some("br, gzip, deflate".into()),
      accept_language: Some("*".into()),
      sec_fetch_mode: Some("cors".into()),
      copilot_integration_id: Some("copilot-developer-cli".into()),
      openai_intent: Some("conversation-agent".into()),
      initiator: Some("user".into()),
      github_api_version: Some("2026-01-09".into()),
      interaction_id: Some("6342b4bb-3441-4440-aae6-884912e51b08".into()),
      interaction_type: Some("conversation-user".into()),
      client_session_id: Some("07e363b0-ea39-48e6-a45e-fe5c6066e50d".into()),
      agent_task_id: Some("08a2565a-6ab6-4613-ae6d-3b0814061942".into()),
      stainless_retry_count: Some("0".into()),
      stainless_lang: Some("js".into()),
      stainless_package_version: Some("5.20.1".into()),
      stainless_os: Some("Windows".into()),
      stainless_arch: Some("x64".into()),
      stainless_runtime: Some("node".into()),
      stainless_runtime_version: Some("v22.19.0".into()),
      content_length: Some("58364".into()),
      cookie: None,
      request_id: None,
    }
  }

  #[test]
  fn round_trip_matches_capture() {
    let h = captured();
    assert_eq!(CopilotCliHeaders::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn missing_required_errors() {
    let m = HeaderMap::new();
    assert!(matches!(CopilotCliHeaders::parse(&m), Err(Error::MissingHeader { .. })));
  }

  #[test]
  fn build_with_empty_inbound_uses_required_defaults() {
    let h = CopilotCliHeaders::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(
      h.user_agent.as_deref(),
      Some("copilot/1.0.25 (client/sdk win32 v22.19.0) term/unknown")
    );
    assert_eq!(h.authorization.as_deref(), Some("<missing>"));
    assert!(h.content_type.is_none(), "loose skips Standard when absent");
    assert!(h.stainless_lang.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::USER_AGENT, "copilot/2.0");
    inbound.insert(&keys::AUTHORIZATION, "Bearer gho_abc");
    inbound.insert(&keys::X_STAINLESS_LANG, "js");
    let h = CopilotCliHeaders::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("copilot/2.0"));
    assert_eq!(h.authorization.as_deref(), Some("Bearer gho_abc"));
    assert_eq!(h.stainless_lang.as_deref(), Some("js"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      interaction_id: Some("int_42".into()),
      request_id: Some("req_99".into()),
      ..Default::default()
    };
    let h = CopilotCliHeaders::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.client_session_id.as_deref(), Some("ses_xyz"));
    assert_eq!(h.interaction_id.as_deref(), Some("int_42"));
    assert_eq!(h.request_id.as_deref(), Some("req_99"));
  }

  #[test]
  fn build_strict_fills_standard() {
    let h = CopilotCliHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert!(h.content_type.is_some());
    assert!(h.accept.is_some());
    assert!(h.copilot_integration_id.is_some());
    assert!(h.client_session_id.is_some());
    assert!(h.stainless_lang.is_none());
  }
}
