//! Headers emitted by the Claude Code CLI client.
//!
//! NOTE: not yet verified against real-world inbound captures — no
//! `claude-cli` traffic was observed in the mined request logs. Field set is
//! a best-effort outbound model and may need refinement once captures
//! become available.
//!
//! # Tiers
//! * **Required**: `User-Agent`.
//! * **Standard**: `Anthropic-Version`, `X-Session-Id`.
//! * **Extra**: `Anthropic-Beta`, `X-Interaction-Id`.

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
pub struct ClaudeCodeHeaders {
  #[serde(rename = "User-Agent", skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<SmolStr>,
  #[serde(rename = "Anthropic-Version", skip_serializing_if = "Option::is_none")]
  pub anthropic_version: Option<SmolStr>,
  #[serde(rename = "X-Session-Id", skip_serializing_if = "Option::is_none")]
  pub session_id: Option<SmolStr>,
  #[serde(rename = "Anthropic-Beta", skip_serializing_if = "Option::is_none")]
  pub anthropic_beta: Option<SmolStr>,
  #[serde(rename = "X-Interaction-Id", skip_serializing_if = "Option::is_none")]
  pub interaction_id: Option<SmolStr>,
}

impl HeaderSchema for ClaudeCodeHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: Some(required(map, &keys::USER_AGENT)?),
      anthropic_version: optional(map, &keys::ANTHROPIC_VERSION),
      session_id: optional(map, &keys::X_SESSION_ID),
      anthropic_beta: optional(map, &keys::ANTHROPIC_BETA),
      interaction_id: optional(map, &keys::X_INTERACTION_ID),
    })
  }
  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::ANTHROPIC_VERSION, &self.anthropic_version);
    put_opt(&mut m, &keys::ANTHROPIC_BETA, &self.anthropic_beta);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    put_opt(&mut m, &keys::X_INTERACTION_ID, &self.interaction_id);
    m
  }
  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 5] = [
      (&keys::USER_AGENT, Tier::Required),
      (&keys::ANTHROPIC_VERSION, Tier::Standard),
      (&keys::X_SESSION_ID, Tier::Standard),
      (&keys::ANTHROPIC_BETA, Tier::Extra),
      (&keys::X_INTERACTION_ID, Tier::Extra),
    ];
    &FIELDS
  }
}

impl ClaudeCodeHeaders {
  /// Persona defaults.
  pub fn defaults() -> Self {
    Self {
      user_agent: Some("claude-cli/1.0.0".into()),
      anthropic_version: Some("2023-06-01".into()),
      anthropic_beta: None,
      session_id: None,
      interaction_id: None,
    }
  }

  /// Build (loose).
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      user_agent: Some(req_inbound_or(inbound, &keys::USER_AGENT, || {
        d.user_agent.clone().expect("default user_agent")
      })),
      anthropic_version: std_loose(inbound, &keys::ANTHROPIC_VERSION),
      session_id: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_SESSION_ID)),
      anthropic_beta: extra_loose(inbound, &keys::ANTHROPIC_BETA, || None),
      interaction_id: vars
        .interaction_id
        .clone()
        .or_else(|| extra_loose(inbound, &keys::X_INTERACTION_ID, || None)),
    })
  }

  /// Build (strict). Standard fields are synthesised when missing; Extra
  /// fields are passed through from inbound only.
  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    let mut base = Self::build(vars, inbound)?;
    base.anthropic_version = base.anthropic_version.or_else(|| {
      std_strict(inbound, &keys::ANTHROPIC_VERSION, || {
        d.anthropic_version.clone().unwrap_or_else(|| "2023-06-01".into())
      })
    });
    base.session_id = base
      .session_id
      .or_else(|| std_strict(inbound, &keys::X_SESSION_ID, || "unknown".into()));
    base.anthropic_beta = extra_strict(inbound, &keys::ANTHROPIC_BETA);
    base.interaction_id = vars
      .interaction_id
      .clone()
      .or_else(|| extra_strict(inbound, &keys::X_INTERACTION_ID));
    Ok(base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let h = ClaudeCodeHeaders {
      user_agent: Some("claude-code/1.2.3".into()),
      anthropic_version: Some("2023-06-01".into()),
      anthropic_beta: Some("messages-2023-12-15".into()),
      session_id: Some("ses_cc".into()),
      interaction_id: Some("int_99".into()),
    };
    assert_eq!(ClaudeCodeHeaders::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn build_with_empty_inbound_uses_defaults() {
    let h = ClaudeCodeHeaders::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("claude-cli/1.0.0"));
    assert!(h.anthropic_version.is_none(), "loose skips Standard when absent");
    assert!(h.session_id.is_none());
    assert!(h.anthropic_beta.is_none());
    assert!(h.interaction_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::USER_AGENT, "claude-cli/2.0");
    inbound.insert(&keys::ANTHROPIC_VERSION, "2023-06-01");
    let h = ClaudeCodeHeaders::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("claude-cli/2.0"));
    assert_eq!(h.anthropic_version.as_deref(), Some("2023-06-01"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      interaction_id: Some("int_42".into()),
      ..Default::default()
    };
    let h = ClaudeCodeHeaders::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.session_id.as_deref(), Some("ses_xyz"));
    assert_eq!(h.interaction_id.as_deref(), Some("int_42"));
  }

  #[test]
  fn build_strict_synthesizes_standard() {
    let h = ClaudeCodeHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert!(h.anthropic_version.is_some());
    assert!(h.session_id.is_some());
    // Extra still skipped.
    assert!(h.anthropic_beta.is_none());
    assert!(h.interaction_id.is_none());
  }
}
