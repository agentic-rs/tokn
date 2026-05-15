//! Headers emitted by the Codex CLI client.

use crate::error::Error;
use crate::keys;
use crate::map::HeaderMap;
use crate::name::HeaderName;
use crate::schema::{optional, put, put_opt, required, HeaderSchema};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexCliHeaders {
  #[serde(rename = "User-Agent")]
  pub user_agent: SmolStr,
  #[serde(rename = "X-Session-Id")]
  pub session_id: Option<SmolStr>,
  #[serde(rename = "X-Project-Cwd")]
  pub project_cwd: Option<SmolStr>,
}

impl HeaderSchema for CodexCliHeaders {
  fn parse(map: &HeaderMap) -> Result<Self, Error> {
    Ok(Self {
      user_agent: required(map, &keys::USER_AGENT)?,
      session_id: optional(map, &keys::X_SESSION_ID),
      project_cwd: optional(map, &keys::X_PROJECT_CWD),
    })
  }
  fn build(&self) -> HeaderMap {
    let mut m = HeaderMap::new();
    put(&mut m, &keys::USER_AGENT, &self.user_agent);
    put_opt(&mut m, &keys::X_SESSION_ID, &self.session_id);
    put_opt(&mut m, &keys::X_PROJECT_CWD, &self.project_cwd);
    m
  }
  fn known_names() -> &'static [&'static HeaderName] {
    static NAMES: [&HeaderName; 3] = [&keys::USER_AGENT, &keys::X_SESSION_ID, &keys::X_PROJECT_CWD];
    &NAMES
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip() {
    let h = CodexCliHeaders {
      user_agent: "codex/0.5.0".into(),
      session_id: Some("ses_x".into()),
      project_cwd: Some("/work/proj".into()),
    };
    assert_eq!(CodexCliHeaders::parse(&h.build()).unwrap(), h);
  }
}
