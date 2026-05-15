//! Persona-specific header schemas (one struct per known client tool).
//!
//! Each persona owns the headers that identify it as the originating tool —
//! `User-Agent`, editor metadata, correlation IDs supplied by the client, etc.

use crate::error::Error;
use crate::keys;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::schema::{optional, put, put_opt, required, HeaderSchema};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Headers emitted by the OpenCode CLI client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpencodeHeaders {
  #[serde(rename = "User-Agent")]
  pub user_agent: SmolStr,
  #[serde(rename = "X-Session-Id")]
  pub session_id: Option<SmolStr>,
  #[serde(rename = "X-Request-Id")]
  pub request_id: Option<SmolStr>,
  #[serde(rename = "X-Initiator")]
  pub initiator: Option<SmolStr>,
}

impl HeaderSchema for OpencodeHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: required(map, &keys::USER_AGENT)?,
      session_id: optional(map, &keys::X_SESSION_ID),
      request_id: optional(map, &keys::X_REQUEST_ID),
      initiator: optional(map, &keys::X_INITIATOR),
    })
  }
  fn build(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    put_opt(&mut m, &keys::X_REQUEST_ID, &self.request_id);
    put_opt(&mut m, &keys::X_INITIATOR, &self.initiator);
    m
  }
  fn known_names() -> &'static [&'static HeaderName] {
    static NAMES: [&HeaderName; 4] =
      [&keys::USER_AGENT, &keys::X_SESSION_ID, &keys::X_REQUEST_ID, &keys::X_INITIATOR];
    &NAMES
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn opencode_round_trip() {
    let h = OpencodeHeaders {
      user_agent: "opencode/1.14.28 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.13".into(),
      session_id: Some("ses_42".into()),
      request_id: Some("req_99".into()),
      initiator: Some("agent".into()),
    };
    let m = h.build();
    let parsed = OpencodeHeaders::parse(&m).unwrap();
    assert_eq!(parsed, h);
  }

  #[test]
  fn opencode_optional_fields_omitted_when_none() {
    let h = OpencodeHeaders {
      user_agent: "opencode/1.0".into(),
      session_id: None,
      request_id: None,
      initiator: None,
    };
    let m = h.build();
    assert_eq!(m.len(), 1);
    assert!(m.contains_key(&keys::USER_AGENT));
  }

  #[test]
  fn opencode_missing_required_returns_error() {
    let m = HeaderMap::new();
    let err = OpencodeHeaders::parse(&m).unwrap_err();
    assert!(matches!(err, Error::MissingHeader { .. }));
  }
}
