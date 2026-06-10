//! Per-agent adapters: discovery of credentials and rewriting of an agent's own
//! config so it routes through a gateway profile. Everything agent-specific
//! lives behind the [`AgentAdapter`] trait; the rest of the crate is generic.

use crate::adapters::{codex::CodexAdapter, opencode::OpencodeAdapter};
use crate::reconcile::PlannedEdit;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokn_config::Account;
use tokn_core::AgentId;

/// A local agent the gateway can bind to: import credentials from it and point
/// its own config at a gateway profile.
pub trait AgentAdapter {
  /// Provider id used to scope a materialised profile that has no imported
  /// accounts of its own.
  fn default_provider_id(&self) -> &'static str;

  /// Absolute path to the agent's credential store for a given home dir.
  fn auth_path(&self, home: &Path) -> PathBuf;

  /// Absolute path to the agent's own config file for a given home dir.
  fn config_path(&self, home: &Path) -> PathBuf;

  /// Best-effort credential discovery. An empty vec means "nothing importable
  /// found" and is **not** an error.
  fn discover_accounts(&self, home: &Path, timestamp: &str) -> Result<Vec<Account>>;

  /// Produce the edits that point the agent's own config at `base_url`.
  fn rewrite_config(&self, home: &Path, base_url: &str) -> Result<Vec<PlannedEdit>>;
}

/// Resolve the adapter for an agent, if the agent is supported.
pub fn adapter_for(agent: &AgentId) -> Option<Box<dyn AgentAdapter>> {
  match agent {
    AgentId::Opencode => Some(Box::new(OpencodeAdapter)),
    AgentId::CodexCli => Some(Box::new(CodexAdapter)),
    _ => None,
  }
}

/// All agents with a built-in adapter, in stable display order.
pub fn supported_agents() -> Vec<AgentId> {
  vec![AgentId::Opencode, AgentId::CodexCli]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn supported_agents_have_adapters() {
    for agent in supported_agents() {
      assert!(adapter_for(&agent).is_some(), "{} should be supported", agent.as_str());
    }
  }

  #[test]
  fn unknown_agents_are_unsupported() {
    assert!(adapter_for(&AgentId::ClaudeCode).is_none());
    assert!(adapter_for(&AgentId::from("bespoke")).is_none());
  }
}
