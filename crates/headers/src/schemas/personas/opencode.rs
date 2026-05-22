//! Headers emitted by the OpenCode CLI client.
//!
//! Field set derived from the inbound real-world matrix (see
//! `tests/fixtures/inbound_real_world.json`). Required fields are present in
//! ≥99% of captured requests; standard/extra fields are observed but
//! inconsistent.
//!
//! `Authorization` is modelled as required even though its value may be the
//! literal `"<redacted>"` in fixtures: the *header* is universally present,
//! and downstream layers replace its value before transmission.
//!
//! # Tiers
//!
//! * **Required**: `User-Agent`, `Authorization`, `Accept`, `Accept-Encoding`,
//!   `Connection`, `Content-Type`.
//! * **Standard**: `Content-Length`, `X-Session-Affinity`.
//! * **Extra**: `Host`, `X-Parent-Session-Id`. `Host` is NEVER stamped from a
//!   persona default — the persona-default host (e.g. `api.deepseek.com`) is
//!   wrong for any other upstream and caused real 403s when it leaked into
//!   Send. Outbound transport derives `Host` from the URL.

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

/// Inbound headers consistently emitted by the OpenCode CLI persona.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OpencodeHeaders {
  // Required transport
  #[serde(rename = "User-Agent", skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<SmolStr>,
  #[serde(rename = "Authorization", skip_serializing_if = "Option::is_none")]
  pub authorization: Option<SmolStr>,
  #[serde(rename = "Accept", skip_serializing_if = "Option::is_none")]
  pub accept: Option<SmolStr>,
  #[serde(rename = "Accept-Encoding", skip_serializing_if = "Option::is_none")]
  pub accept_encoding: Option<SmolStr>,
  #[serde(rename = "Connection", skip_serializing_if = "Option::is_none")]
  pub connection: Option<SmolStr>,
  #[serde(rename = "Content-Type", skip_serializing_if = "Option::is_none")]
  pub content_type: Option<SmolStr>,

  // Standard
  #[serde(rename = "Content-Length", skip_serializing_if = "Option::is_none")]
  pub content_length: Option<SmolStr>,
  #[serde(rename = "X-Session-Affinity", skip_serializing_if = "Option::is_none")]
  pub session_affinity: Option<SmolStr>,

  // Extra
  #[serde(rename = "Host", skip_serializing_if = "Option::is_none")]
  pub host: Option<SmolStr>,
  #[serde(rename = "X-Parent-Session-Id", skip_serializing_if = "Option::is_none")]
  pub parent_session_id: Option<SmolStr>,
}

impl HeaderSchema for OpencodeHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: Some(required(map, &keys::USER_AGENT)?),
      authorization: Some(required(map, &keys::AUTHORIZATION)?),
      accept: Some(required(map, &keys::ACCEPT)?),
      accept_encoding: Some(required(map, &keys::ACCEPT_ENCODING)?),
      connection: Some(required(map, &keys::CONNECTION)?),
      content_type: Some(required(map, &keys::CONTENT_TYPE)?),
      content_length: optional(map, &keys::CONTENT_LENGTH),
      session_affinity: optional(map, &keys::X_SESSION_AFFINITY),
      host: optional(map, &keys::HOST),
      parent_session_id: optional(map, &keys::X_PARENT_SESSION_ID),
    })
  }

  fn dump(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put_opt(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::AUTHORIZATION, &self.authorization);
    put_opt(&mut m, &keys::ACCEPT, &self.accept);
    put_opt(&mut m, &keys::ACCEPT_ENCODING, &self.accept_encoding);
    put_opt(&mut m, &keys::CONNECTION, &self.connection);
    put_opt(&mut m, &keys::CONTENT_TYPE, &self.content_type);
    put_opt(&mut m, &keys::CONTENT_LENGTH, &self.content_length);
    put_opt(&mut m, &keys::X_SESSION_AFFINITY, &self.session_affinity);
    put_opt(&mut m, &keys::HOST, &self.host);
    put_opt(&mut m, &keys::X_PARENT_SESSION_ID, &self.parent_session_id);
    m
  }

  fn field_tiers() -> &'static [(&'static HeaderName, Tier)] {
    static FIELDS: [(&HeaderName, Tier); 10] = [
      (&keys::USER_AGENT, Tier::Required),
      (&keys::AUTHORIZATION, Tier::Required),
      (&keys::ACCEPT, Tier::Required),
      (&keys::ACCEPT_ENCODING, Tier::Required),
      (&keys::CONNECTION, Tier::Required),
      (&keys::CONTENT_TYPE, Tier::Required),
      (&keys::CONTENT_LENGTH, Tier::Standard),
      (&keys::X_SESSION_AFFINITY, Tier::Standard),
      (&keys::HOST, Tier::Extra),
      (&keys::X_PARENT_SESSION_ID, Tier::Extra),
    ];
    &FIELDS
  }
}

impl OpencodeHeaders {
  /// Persona-default values for every field that has one. Used by `build` to
  /// satisfy missing Required slots and by `build_strict` to synthesise
  /// Standard fields.
  pub fn defaults() -> Self {
    Self {
      user_agent: Some("opencode/1.14.28 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13".into()),
      authorization: Some("<missing>".into()),
      accept: Some("*/*".into()),
      accept_encoding: Some("gzip, deflate, br, zstd".into()),
      connection: Some("keep-alive".into()),
      content_type: Some("application/json".into()),
      content_length: None,
      session_affinity: None,
      host: None,
      parent_session_id: None,
    }
  }

  /// Build (loose) an [`OpencodeHeaders`] from inbound transport headers and
  /// correlation [`TemplateVars`].
  ///
  /// Required fields use inbound → persona default; absence of both yields
  /// [`Error::MissingHeader`]. Standard fields use inbound only (omitted
  /// when missing). Extra fields are best-effort (inbound → persona default
  /// → omit).
  pub fn build(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    Ok(Self {
      user_agent: Some(req_inbound_or(inbound, &keys::USER_AGENT, || {
        d.user_agent.clone().expect("default user_agent")
      })),
      authorization: Some(req_inbound_or(inbound, &keys::AUTHORIZATION, || {
        d.authorization.clone().expect("default authorization")
      })),
      accept: Some(req_inbound_or(inbound, &keys::ACCEPT, || {
        d.accept.clone().expect("default accept")
      })),
      accept_encoding: Some(req_inbound_or(inbound, &keys::ACCEPT_ENCODING, || {
        d.accept_encoding.clone().expect("default accept_encoding")
      })),
      connection: Some(req_inbound_or(inbound, &keys::CONNECTION, || {
        d.connection.clone().expect("default connection")
      })),
      content_type: Some(req_inbound_or(inbound, &keys::CONTENT_TYPE, || {
        d.content_type.clone().expect("default content_type")
      })),
      content_length: std_loose(inbound, &keys::CONTENT_LENGTH),
      session_affinity: vars
        .session_id
        .clone()
        .or_else(|| std_loose(inbound, &keys::X_SESSION_AFFINITY)),
      host: extra_loose(inbound, &keys::HOST, || None),
      parent_session_id: extra_loose(inbound, &keys::X_PARENT_SESSION_ID, || None),
    })
  }

  /// Build (strict) an [`OpencodeHeaders`]. Required + Standard fields are
  /// always populated (from inbound → persona defaults). Extra fields are
  /// only included when present in inbound.
  pub fn build_strict(vars: &TemplateVars, inbound: &HeaderMap) -> Result<Self, Error> {
    let d = Self::defaults();
    // Required: synthesised the same way as in `build`.
    let mut base = Self::build(vars, inbound)?;
    // Standard: must be filled in strict mode. Prefer vars.session_id for
    // X-Session-Affinity; otherwise inbound; otherwise a synthetic placeholder.
    base.content_length = base
      .content_length
      .or_else(|| std_strict(inbound, &keys::CONTENT_LENGTH, || "0".into()));
    base.session_affinity = base.session_affinity.or_else(|| {
      std_strict(inbound, &keys::X_SESSION_AFFINITY, || {
        d.session_affinity.clone().unwrap_or_else(|| "unknown".into())
      })
    });
    // Extra: strict mode skips when absent — overwrite with pure-inbound view.
    base.host = extra_strict(inbound, &keys::HOST);
    base.parent_session_id = extra_strict(inbound, &keys::X_PARENT_SESSION_ID);
    Ok(base)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample() -> OpencodeHeaders {
    OpencodeHeaders {
      user_agent: Some("opencode/1.14.28 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13".into()),
      authorization: Some("<redacted>".into()),
      host: Some("api.deepseek.com".into()),
      accept: Some("*/*".into()),
      accept_encoding: Some("gzip, deflate, br, zstd".into()),
      connection: Some("keep-alive".into()),
      content_type: Some("application/json".into()),
      content_length: Some("4429".into()),
      session_affinity: Some("ses_1dddd2016ffed1A1u3yj5LmNWC".into()),
      parent_session_id: None,
    }
  }

  #[test]
  fn opencode_round_trip() {
    let h = sample();
    let parsed = OpencodeHeaders::parse(&h.dump()).unwrap();
    assert_eq!(parsed, h);
  }

  #[test]
  fn opencode_optional_fields_omitted_when_none() {
    let mut h = sample();
    h.content_length = None;
    h.session_affinity = None;
    h.parent_session_id = None;
    h.host = None;
    let m = h.dump();
    // 6 required fields, 0 optional written.
    assert_eq!(m.len(), 6);
    assert!(!m.contains_key(&keys::CONTENT_LENGTH));
    assert!(!m.contains_key(&keys::X_SESSION_AFFINITY));
    assert!(!m.contains_key(&keys::HOST));
  }

  #[test]
  fn opencode_missing_required_returns_error() {
    let m = HeaderMap::new();
    assert!(matches!(OpencodeHeaders::parse(&m), Err(Error::MissingHeader { .. })));
  }

  #[test]
  fn build_with_empty_inbound_uses_required_defaults() {
    let h = OpencodeHeaders::build(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    assert_eq!(
      h.user_agent.as_deref(),
      Some("opencode/1.14.28 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13")
    );
    assert_eq!(h.authorization.as_deref(), Some("<missing>"));
    assert!(
      h.host.is_none(),
      "no inbound Host => no persona-default Host (would leak to wire)"
    );
    assert_eq!(h.accept.as_deref(), Some("*/*"));
    assert_eq!(h.accept_encoding.as_deref(), Some("gzip, deflate, br, zstd"));
    assert_eq!(h.connection.as_deref(), Some("keep-alive"));
    assert_eq!(h.content_type.as_deref(), Some("application/json"));
    // Standard: loose mode skips when missing.
    assert!(h.content_length.is_none());
    assert!(h.session_affinity.is_none());
    // Extra: loose mode best-effort (none available here).
    assert!(h.parent_session_id.is_none());
  }

  #[test]
  fn build_passes_through_inbound() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::USER_AGENT, "custom-ua/9.9");
    inbound.insert(&keys::AUTHORIZATION, "Bearer secret");
    inbound.insert(&keys::CONTENT_LENGTH, "1234");
    inbound.insert(&keys::HOST, "api.deepseek.com");
    let h = OpencodeHeaders::build(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.user_agent.as_deref(), Some("custom-ua/9.9"));
    assert_eq!(h.authorization.as_deref(), Some("Bearer secret"));
    assert_eq!(h.content_length.as_deref(), Some("1234"));
    // Extra: inbound provides it, so it passes through.
    assert_eq!(h.host.as_deref(), Some("api.deepseek.com"));
  }

  #[test]
  fn build_uses_vars_for_correlation() {
    let vars = TemplateVars {
      session_id: Some("ses_xyz".into()),
      ..Default::default()
    };
    let h = OpencodeHeaders::build(&vars, &HeaderMap::new()).unwrap();
    assert_eq!(h.session_affinity.as_deref(), Some("ses_xyz"));
  }

  #[test]
  fn build_strict_fills_standard_from_defaults() {
    let h = OpencodeHeaders::build_strict(&TemplateVars::default(), &HeaderMap::new()).unwrap();
    // Required filled.
    assert!(h.user_agent.is_some());
    assert!(h.content_type.is_some());
    // Standard synthesised.
    assert!(h.content_length.is_some(), "strict must synthesize Content-Length");
    assert!(
      h.session_affinity.is_some(),
      "strict must synthesize X-Session-Affinity"
    );
    // Extra skipped in strict mode.
    assert!(h.host.is_none());
    assert!(h.parent_session_id.is_none());
  }

  #[test]
  fn build_strict_passes_through_inbound_extra() {
    let mut inbound = HeaderMap::new();
    inbound.insert(&keys::HOST, "api.deepseek.com");
    inbound.insert(&keys::X_PARENT_SESSION_ID, "parent_42");
    let h = OpencodeHeaders::build_strict(&TemplateVars::default(), &inbound).unwrap();
    assert_eq!(h.host.as_deref(), Some("api.deepseek.com"));
    assert_eq!(h.parent_session_id.as_deref(), Some("parent_42"));
  }

  #[test]
  fn field_tiers_matches_struct() {
    let tiers = OpencodeHeaders::field_tiers();
    assert_eq!(tiers.len(), 10);
    let required = tiers.iter().filter(|(_, t)| *t == Tier::Required).count();
    let standard = tiers.iter().filter(|(_, t)| *t == Tier::Standard).count();
    let extra = tiers.iter().filter(|(_, t)| *t == Tier::Extra).count();
    assert_eq!((required, standard, extra), (6, 2, 2));
  }
}
