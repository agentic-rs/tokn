use crate::config::CopilotHeaders;
use crate::provider::{error, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use snafu::ResultExt;

/// Headers for the Copilot API token exchange (`api.github.com`).
pub fn token_exchange_headers(github_token: &str, h: &CopilotHeaders) -> Result<HeaderMap> {
  let mut m = HeaderMap::new();
  m.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("token {github_token}"))
      .context(error::HeaderValueSnafu { name: "authorization" })?,
  );
  m.insert(ACCEPT, HeaderValue::from_static("application/json"));
  m.insert(
    USER_AGENT,
    HeaderValue::from_str(&h.user_agent).context(error::HeaderValueSnafu { name: "user-agent" })?,
  );
  insert_str(&mut m, "editor-version", &h.editor_version)?;
  insert_str(&mut m, "editor-plugin-version", &h.editor_plugin_version)?;
  Ok(m)
}

/// Headers for upstream Copilot API requests (chat / models).
///
/// `initiator` must be "user" or "agent". It is sent as `X-Initiator` and is
/// what GitHub's billing pipeline uses to attribute premium-request charges to
/// a single user-initiated turn rather than to every tool-call follow-up.
pub fn copilot_request_headers(
  api_token: &str,
  h: &CopilotHeaders,
  streaming: bool,
  initiator: &str,
) -> Result<HeaderMap> {
  let mut m = HeaderMap::new();
  m.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {api_token}")).context(error::HeaderValueSnafu { name: "authorization" })?,
  );
  m.insert(
    ACCEPT,
    HeaderValue::from_static(if streaming {
      "text/event-stream"
    } else {
      "application/json"
    }),
  );
  m.insert(
    USER_AGENT,
    HeaderValue::from_str(&h.user_agent).context(error::HeaderValueSnafu { name: "user-agent" })?,
  );
  insert_str(&mut m, "editor-version", &h.editor_version)?;
  insert_str(&mut m, "editor-plugin-version", &h.editor_plugin_version)?;
  insert_str(&mut m, "copilot-integration-id", &h.copilot_integration_id)?;
  insert_str(&mut m, "openai-intent", &h.openai_intent)?;
  insert_str(&mut m, "x-initiator", initiator)?;

  // Extra (free-form) headers — applied last, overriding earlier values.
  for (k, v) in &h.extra_headers {
    let name = HeaderName::from_bytes(k.as_bytes()).context(error::HeaderNameSnafu { name: k.clone() })?;
    let val = HeaderValue::from_str(v).context(error::HeaderValueSnafu { name: k.clone() })?;
    m.insert(name, val);
  }
  Ok(m)
}

fn insert_str(m: &mut HeaderMap, name: &'static str, value: &str) -> Result<()> {
  let n = HeaderName::from_static(name);
  let v = HeaderValue::from_str(value).context(error::HeaderValueSnafu { name: name.to_string() })?;
  m.insert(n, v);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn defaults() -> CopilotHeaders {
    CopilotHeaders::default()
  }

  #[test]
  fn copilot_request_headers_includes_editor_metadata_and_intent() {
    let h = copilot_request_headers("api-tok", &defaults(), false, "user").unwrap();
    assert_eq!(h.get("authorization").unwrap(), "Bearer api-tok");
    assert_eq!(h.get("accept").unwrap(), "application/json");
    assert_eq!(h.get("user-agent").unwrap(), "GitHubCopilotChat/0.20.0");
    assert_eq!(h.get("editor-version").unwrap(), "vscode/1.95.0");
    assert_eq!(h.get("editor-plugin-version").unwrap(), "copilot-chat/0.20.0");
    assert_eq!(h.get("copilot-integration-id").unwrap(), "vscode-chat");
    assert_eq!(h.get("openai-intent").unwrap(), "conversation-panel");
    assert_eq!(h.get("x-initiator").unwrap(), "user");
    let names: Vec<_> = h.keys().map(|k| k.as_str().to_string()).collect();
    assert_eq!(names.len(), 8, "unexpected extra headers: {names:?}");
  }

  #[test]
  fn copilot_request_headers_streaming_toggles_accept() {
    let h = copilot_request_headers("api-tok", &defaults(), true, "user").unwrap();
    assert_eq!(h.get("accept").unwrap(), "text/event-stream");
  }

  #[test]
  fn copilot_request_headers_x_initiator_round_trips_user_and_agent() {
    let user = copilot_request_headers("t", &defaults(), false, "user").unwrap();
    let agent = copilot_request_headers("t", &defaults(), false, "agent").unwrap();
    assert_eq!(user.get("x-initiator").unwrap(), "user");
    assert_eq!(agent.get("x-initiator").unwrap(), "agent");
  }

  #[test]
  fn copilot_request_headers_extra_headers_override_defaults_last() {
    let mut h_cfg = defaults();
    // Same key as a default to prove last-wins; plus a brand-new key.
    h_cfg
      .extra_headers
      .insert("editor-version".into(), "neovim/0.10.0".into());
    h_cfg.extra_headers.insert("x-custom".into(), "yes".into());
    let h = copilot_request_headers("t", &h_cfg, false, "user").unwrap();
    assert_eq!(h.get("editor-version").unwrap(), "neovim/0.10.0");
    assert_eq!(h.get("x-custom").unwrap(), "yes");
  }

  #[test]
  fn token_exchange_headers_shape_is_stable() {
    let h = token_exchange_headers("gh-pat", &defaults()).unwrap();
    assert_eq!(h.get("authorization").unwrap(), "token gh-pat");
    assert_eq!(h.get("accept").unwrap(), "application/json");
    assert_eq!(h.get("user-agent").unwrap(), "GitHubCopilotChat/0.20.0");
    assert_eq!(h.get("editor-version").unwrap(), "vscode/1.95.0");
    assert_eq!(h.get("editor-plugin-version").unwrap(), "copilot-chat/0.20.0");
    let names: Vec<_> = h.keys().map(|k| k.as_str().to_string()).collect();
    assert_eq!(names.len(), 5, "unexpected extra headers: {names:?}");
  }
}
