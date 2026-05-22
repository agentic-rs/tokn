//! Headers emitted by the Cline CLI client.
//!
//! NOTE: not yet verified against real-world inbound captures — no `cline`
//! traffic was observed in the mined request logs. Field set is a
//! best-effort outbound model and may need refinement once captures
//! become available.
//!
//! # Tiers
//! * **Required**: `User-Agent`.
//! * **Standard**: `X-Session-Id`.

use crate::error::Error;
use crate::keys;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::schema::{optional, put_opt, req_inbound_or, required, std_loose, std_strict, HeaderSchema, Tier};
use crate::vars::TemplateVars;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClineHeaders {
  #[serde(rename = "User-Agent", skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<SmolStr>,
  #[serde(rename = "X-Session-Id", skip_serializing_if = "Option::is_none")]
  pub session_id: Option<SmolStr>,
}

impl HeaderSchema for ClineHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: Some(required(map, &keys::USER_AGENT)?),
      session_id: optional(map, &keys::X_SESSION_ID),
    })
  }
  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    m
  }
  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 2] = [
      (&keys::USER_AGENT, Tier::Required),
      (&keys::X_SESSION_ID, Tier::Standard),
    ];
    &FIELDS
  }
}

impl ClineHeaders {
  /// Persona defaults for every field that has a known default.
  pub fn defaults() -> Self {
    Self {
      user_agent: Some("cline/3.0.0".into()),
      session_id: None,
    }
  }

  /// Build (loose) a [`ClineHeaders`] from inbound transport headers and
  /// correlation [`TemplateVars`].
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      user_agent: Some(req_inbound_or(inbound, &keys::USER_AGENT, || {
        d.user_agent.clone().expect("default user_agent")
      })),
      session_id: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_SESSION_ID)),
    })
  }

  /// Build (strict). Standard fields are synthesised from defaults when
  /// absent from inbound.
  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let mut base = Self::build(vars, inbound)?;
    base.session_id = base
      .session_id
      .or_else(|| std_strict(inbound, &keys::X_SESSION_ID, || "unknown".into()));
    Ok(base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let h = ClineHeaders {
      user_agent: Some("cline/3.0.0".into()),
      session_id: Some("ses_cli".into()),
    };
    assert_eq!(ClineHeaders::parse(&h.dump()).unwrap(), h);
  }

  #[test]
  fn build_with_empty_inbound_uses_defaults() {
    let h = ClineHeaders::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("cline/3.0.0"));
    assert!(h.session_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::USER_AGENT, "cline/9.9");
    let h = ClineHeaders::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("cline/9.9"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      ..Default::default()
    };
    let h = ClineHeaders::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.session_id.as_deref(), Some("ses_xyz"));
  }

  #[test]
  fn build_strict_fills_standard() {
    let h = ClineHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert!(h.session_id.is_some());
  }
}
