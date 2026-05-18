//! Headers emitted by the Claude Code CLI client.
//!
//! NOTE: not yet verified against real-world inbound captures — no
//! `claude-cli` traffic was observed in the mined request logs. Field set is
//! a best-effort outbound model and may need refinement once captures
//! become available.

use crate::error::Error;
use crate::keys;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::schema::{from_inbound_or, opt_from_inbound, optional, put, put_opt, required, HeaderSchema};
use crate::vars::TemplateVars;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeCodeHeaders {
  #[serde(rename = "User-Agent")]
  pub user_agent: SmolStr,
  #[serde(rename = "Anthropic-Version")]
  pub anthropic_version: Option<SmolStr>,
  #[serde(rename = "Anthropic-Beta")]
  pub anthropic_beta: Option<SmolStr>,
  #[serde(rename = "X-Session-Id")]
  pub session_id: Option<SmolStr>,
  #[serde(rename = "X-Interaction-Id")]
  pub interaction_id: Option<SmolStr>,
}

impl HeaderSchema for ClaudeCodeHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: required(map, &keys::USER_AGENT)?,
      anthropic_version: optional(map, &keys::ANTHROPIC_VERSION),
      anthropic_beta: optional(map, &keys::ANTHROPIC_BETA),
      session_id: optional(map, &keys::X_SESSION_ID),
      interaction_id: optional(map, &keys::X_INTERACTION_ID),
    })
  }
  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::ANTHROPIC_VERSION, &self.anthropic_version);
    put_opt(&mut m, &keys::ANTHROPIC_BETA, &self.anthropic_beta);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    put_opt(&mut m, &keys::X_INTERACTION_ID, &self.interaction_id);
    m
  }
  fn known_names() -> &'static [&'static HeaderName] {
    static NAMES: [&HeaderName; 5] = [
      &keys::USER_AGENT,
      &keys::ANTHROPIC_VERSION,
      &keys::ANTHROPIC_BETA,
      &keys::X_SESSION_ID,
      &keys::X_INTERACTION_ID,
    ];
    &NAMES
  }
}

impl ClaudeCodeHeaders {
  /// Build a [`ClaudeCodeHeaders`] from inbound transport headers and
  /// correlation [`TemplateVars`].
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Self {
    Self {
      user_agent: from_inbound_or(inbound, &keys::USER_AGENT, || "claude-cli/1.0.0".into()),
      anthropic_version: opt_from_inbound(inbound, &keys::ANTHROPIC_VERSION),
      anthropic_beta: opt_from_inbound(inbound, &keys::ANTHROPIC_BETA),
      session_id: vars
        .session_id
        .clone()
        .or_else(|| opt_from_inbound(inbound, &keys::X_SESSION_ID)),
      interaction_id: vars
        .interaction_id
        .clone()
        .or_else(|| opt_from_inbound(inbound, &keys::X_INTERACTION_ID)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let h = ClaudeCodeHeaders {
      user_agent: "claude-code/1.2.3".into(),
      anthropic_version: Some("2023-06-01".into()),
      anthropic_beta: Some("messages-2023-12-15".into()),
      session_id: Some("ses_cc".into()),
      interaction_id: Some("int_99".into()),
    };
    assert_eq!(ClaudeCodeHeaders::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn build_with_empty_inbound_uses_defaults() {
    let h = ClaudeCodeHeaders::build(&TemplateVars::default(), &HeaderMap::new());
    assert_eq!(h.user_agent.as_str(), "claude-cli/1.0.0");
    assert!(h.anthropic_version.is_none());
    assert!(h.anthropic_beta.is_none());
    assert!(h.session_id.is_none());
    assert!(h.interaction_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(keys::USER_AGENT.clone(), "claude-cli/2.0");
    inbound.insert(keys::ANTHROPIC_VERSION.clone(), "2023-06-01");
    let h = ClaudeCodeHeaders::build(&TemplateVars::default(), &inbound);
    assert_eq!(h.user_agent.as_str(), "claude-cli/2.0");
    assert_eq!(h.anthropic_version.as_deref(), Some("2023-06-01"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      interaction_id: Some("int_42".into()),
      ..Default::default()
    };
    let h = ClaudeCodeHeaders::build(&vars, &HeaderMap::new());
    assert_eq!(h.session_id.as_deref(), Some("ses_xyz"));
    assert_eq!(h.interaction_id.as_deref(), Some("int_42"));
  }
}
