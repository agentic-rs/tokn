use crate::{HeaderMap, TemplateVars};
use smol_str::SmolStr;

pub const SESSION_ID_HEADERS: &[&str] = &[
  "x-session-id",
  "x-client-session-id",
  "session-id",
  "session_id",
  "x-session-affinity",
  "x-opencode-session",
];

pub const THREAD_ID_HEADERS: &[&str] = &["thread-id", "thread_id"];

pub const PARENT_THREAD_ID_HEADERS: &[&str] = &[
  "x-codex-parent-thread-id",
  "x-parent-thread-id",
  "parent-thread-id",
  "parent_thread_id",
];

pub const REQUEST_ID_HEADERS: &[&str] = &["x-request-id", "x-interaction-id", "x-opencode-request"];

pub const PROJECT_ID_HEADERS: &[&str] = &["x-opencode-project", "x-project-cwd"];

pub const INTERACTION_ID_HEADERS: &[&str] = &["x-interaction-id"];

pub const ACCOUNT_ID_HEADERS: &[&str] = &["chatgpt-account-id"];

pub fn first_present<'a>(headers: &'a HeaderMap, names: &[&str]) -> Option<&'a str> {
  names.iter().find_map(|name| {
    headers
      .get(*name)
      .map(|value| value.as_str().trim())
      .filter(|value| !value.is_empty())
  })
}

pub fn first_present_smol(headers: &HeaderMap, names: &[&str]) -> Option<SmolStr> {
  first_present(headers, names).map(SmolStr::new)
}

pub fn build_template_vars(headers: &HeaderMap) -> TemplateVars {
  TemplateVars {
    session_id: first_present_smol(headers, SESSION_ID_HEADERS),
    request_id: first_present_smol(headers, REQUEST_ID_HEADERS),
    project_cwd: first_present_smol(headers, PROJECT_ID_HEADERS),
    interaction_id: first_present_smol(headers, INTERACTION_ID_HEADERS),
    account_id: first_present_smol(headers, ACCOUNT_ID_HEADERS),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{HeaderName, HeaderValue};

  fn header_map(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
      headers.insert(HeaderName::new(*name), HeaderValue::from_string((*value).to_string()));
    }
    headers
  }

  #[test]
  fn first_present_uses_priority_and_trims() {
    let headers = header_map(&[
      ("x-session-id", "   "),
      ("x-client-session-id", "   "),
      ("session-id", " sess-2 "),
      ("x-opencode-session", "sess-3"),
    ]);

    assert_eq!(first_present(&headers, SESSION_ID_HEADERS), Some("sess-2"));
  }

  #[test]
  fn template_vars_cover_all_correlation_headers() {
    let headers = header_map(&[
      ("x-session-id", "ses_abc"),
      ("x-request-id", "req_xyz"),
      ("x-project-cwd", "/tmp/work"),
      ("x-interaction-id", "int_9"),
      ("chatgpt-account-id", "acct_42"),
    ]);

    let vars = build_template_vars(&headers);
    assert_eq!(vars.session_id.as_deref(), Some("ses_abc"));
    assert_eq!(vars.request_id.as_deref(), Some("req_xyz"));
    assert_eq!(vars.project_cwd.as_deref(), Some("/tmp/work"));
    assert_eq!(vars.interaction_id.as_deref(), Some("int_9"));
    assert_eq!(vars.account_id.as_deref(), Some("acct_42"));
  }

  #[test]
  fn thread_headers_keep_thread_topology_separate_from_session_identity() {
    let headers = header_map(&[
      ("session-id", "session-root"),
      ("thread-id", "thread-child"),
      ("x-codex-parent-thread-id", "thread-root"),
    ]);

    assert_eq!(first_present(&headers, SESSION_ID_HEADERS), Some("session-root"));
    assert_eq!(first_present(&headers, THREAD_ID_HEADERS), Some("thread-child"));
    assert_eq!(first_present(&headers, PARENT_THREAD_ID_HEADERS), Some("thread-root"));
  }
}
