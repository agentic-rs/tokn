//! Agent migration helpers for local tools that should route through
//! `tokn-router` profiles.

mod agent;
mod codex;
mod manifest;
mod migration;
mod opencode;

pub use agent::AgentKind;
pub use manifest::FileBackup;
pub use migration::{
  apply_migration, plan_migration, rollback_migration, ApplyReport, FileAction, MigrateRequest, MigrationPlan,
  PlannedEdit, RollbackReport, RollbackRequest,
};
