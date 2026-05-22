//! Codex (ChatGPT account) transport overlay.
//!
//! Headers required when targeting the ChatGPT-account Codex backend on top
//! of a base persona.
//!
//! SCOPE: this overlay models **outbound** headers the router injects /
//! validates when forwarding to `chatgpt.com`. The codex-cli-native
//! inbound headers (`originator`, `version`, `session_id`, `thread_id`,
//! `x-codex-*`) are modelled directly on `CodexCliHeaders`.

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexOverlay {
  #[serde(rename = "OpenAI-Beta", skip_serializing_if = "Option::is_none")]
  pub openai_beta: Option<SmolStr>,
  #[serde(rename = "OpenAI-Intent", skip_serializing_if = "Option::is_none")]
  pub openai_intent: Option<SmolStr>,
  #[serde(rename = "chatgpt-account-id", skip_serializing_if = "Option::is_none")]
  pub chatgpt_account_id: Option<SmolStr>,
  #[serde(rename = "X-Session-Id", skip_serializing_if = "Option::is_none")]
  pub session_id: Option<SmolStr>,
}

impl HeaderSchema for CodexOverlay {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      openai_beta: Some(required(map, &keys::OPENAI_BETA)?),
      openai_intent: optional(map, &keys::OPENAI_INTENT),
      chatgpt_account_id: optional(map, &keys::CHATGPT_ACCOUNT_ID),
      session_id: optional(map, &keys::X_SESSION_ID),
    })
  }
  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::OPENAI_BETA, &self.openai_beta);
    put_opt(&mut m, &keys::OPENAI_INTENT, &self.openai_intent);
    put_opt(&mut m, &keys::CHATGPT_ACCOUNT_ID, &self.chatgpt_account_id);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    m
  }
  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 4] = [
      (&keys::OPENAI_BETA, Tier::Required),
      (&keys::X_SESSION_ID, Tier::Standard),
      (&keys::OPENAI_INTENT, Tier::Extra),
      (&keys::CHATGPT_ACCOUNT_ID, Tier::Extra),
    ];
    &FIELDS
  }
}

impl CodexOverlay {
  pub fn defaults() -> Self {
    Self {
      openai_beta: Some("responses=v1".into()),
      openai_intent: None,
      chatgpt_account_id: None,
      session_id: None,
    }
  }

  /// Build a [`CodexOverlay`] from inbound transport headers and
  /// correlation [`TemplateVars`].
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      openai_beta: Some(req_inbound_or(inbound, &keys::OPENAI_BETA, || {
        d.openai_beta.clone().expect("default openai_beta")
      })),
      openai_intent: extra_loose(inbound, &keys::OPENAI_INTENT, || d.openai_intent.clone()),
      chatgpt_account_id: vars
        .account_id
        .clone()
        .or_else(|| extra_loose(inbound, &keys::CHATGPT_ACCOUNT_ID, || d.chatgpt_account_id.clone())),
      session_id: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_SESSION_ID)),
    })
  }

  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let mut base = Self::build(vars, inbound)?;
    base.session_id = base
      .session_id
      .or_else(|| std_strict(inbound, &keys::X_SESSION_ID, || "unknown".into()));
    base.openai_intent = extra_strict(inbound, &keys::OPENAI_INTENT);
    base.chatgpt_account_id = vars
      .account_id
      .clone()
      .or_else(|| extra_strict(inbound, &keys::CHATGPT_ACCOUNT_ID));
    Ok(base)
  }

  /// Apply this overlay onto an outbound [`HeaderMap`]. `OpenAI-Beta` is
  /// always overridden (the gateway requires it). Optional fields are filled
  /// in only when the overlay has a value AND the header is not already
  /// present on the map.
  pub fn apply_to(&self, map: &mut HeaderMap, _vars: &TemplateVars) {
    if let Some(v) = &self.openai_beta {
      map.insert(&keys::OPENAI_BETA, v.to_string());
    }
    if let Some(v) = &self.openai_intent {
      if !map.contains_key(&keys::OPENAI_INTENT) {
        map.insert(&keys::OPENAI_INTENT, v.to_string());
      }
    }
    if let Some(v) = &self.chatgpt_account_id {
      if !map.contains_key(&keys::CHATGPT_ACCOUNT_ID) {
        map.insert(&keys::CHATGPT_ACCOUNT_ID, v.to_string());
      }
    }
    if let Some(v) = &self.session_id {
      if !map.contains_key(&keys::X_SESSION_ID) {
        map.insert(&keys::X_SESSION_ID, v.to_string());
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let h = CodexOverlay {
      openai_beta: Some("responses=v1".into()),
      openai_intent: Some("assistants".into()),
      chatgpt_account_id: Some("acct_99".into()),
      session_id: Some("ses_codex".into()),
    };
    assert_eq!(CodexOverlay::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn build_with_empty_inbound_uses_defaults() {
    let h = CodexOverlay::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(h.openai_beta.as_deref(), Some("responses=v1"));
    assert!(h.openai_intent.is_none());
    assert!(h.chatgpt_account_id.is_none());
    assert!(h.session_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::OPENAI_BETA, "responses=v2");
    inbound.insert(&keys::OPENAI_INTENT, "assistants");
    let h = CodexOverlay::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.openai_beta.as_deref(), Some("responses=v2"));
    assert_eq!(h.openai_intent.as_deref(), Some("assistants"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      account_id: Some("acct_abc".into()),
      ..Default::default()
    };
    let h = CodexOverlay::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.session_id.as_deref(), Some("ses_xyz"));
    assert_eq!(h.chatgpt_account_id.as_deref(), Some("acct_abc"));
  }

  #[test]
  fn apply_to_overrides_managed_fields_and_skips_optionals_when_none() {
    let mut map = HeaderMap::new();
    map.insert(&keys::OPENAI_BETA, "stale=v0");
    map.insert(&keys::X_SESSION_ID, "preexisting");

    let overlay = CodexOverlay {
      openai_beta: Some("responses=v1".into()),
      openai_intent: None,
      chatgpt_account_id: Some("acct_abc".into()),
      session_id: Some("ses_xyz".into()),
    };
    overlay.apply_to(&mut map, &TemplateVars::default());

    assert_eq!(map.get(&keys::OPENAI_BETA).unwrap().as_str(), "responses=v1");
    // session_id was already present — overlay must not clobber it.
    assert_eq!(map.get(&keys::X_SESSION_ID).unwrap().as_str(), "preexisting");
    // chatgpt-account-id absent originally — overlay fills it in.
    assert_eq!(map.get(&keys::CHATGPT_ACCOUNT_ID).unwrap().as_str(), "acct_abc");
    // None-valued optional not inserted.
    assert!(!map.contains_key(&keys::OPENAI_INTENT));
  }
}
