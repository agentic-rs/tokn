pub mod locations;
pub mod schema;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Default)]
pub struct MigrationReport {
  pub location_actions: Vec<locations::LocationAction>,
  pub schema_actions: Vec<schema::SchemaAction>,
}

impl MigrationReport {
  pub fn is_empty(&self) -> bool {
    self.location_actions.is_empty() && self.schema_actions.is_empty()
  }
}

/// Bring legacy router state into the current home and then run schema
/// migrations inside that home.
pub fn ensure_latest_home(home: &Path) -> Result<MigrationReport> {
  let location_actions = locations::migrate(home)?;
  let schema_actions = schema::migrate_home(home)?;
  Ok(MigrationReport {
    location_actions,
    schema_actions,
  })
}
