use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokn_core::AgentId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MigrationManifest {
  pub version: u32,
  #[serde(default = "default_completed")]
  pub completed: bool,
  pub agent: AgentId,
  pub timestamp: String,
  pub profile: Option<String>,
  pub target_base_url: String,
  pub imported_account_ids: Vec<String>,
  pub files: Vec<FileBackup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileBackup {
  pub original: PathBuf,
  pub backup: Option<PathBuf>,
  pub existed: bool,
  pub created_by_migration: bool,
}

impl MigrationManifest {
  pub(crate) fn in_progress(mut self) -> Self {
    self.completed = false;
    self
  }

  pub(crate) fn complete(mut self) -> Self {
    self.completed = true;
    self
  }
}

pub(crate) fn manifest_path(timestamp: &str, agent: &AgentId) -> Result<PathBuf> {
  Ok(manifest_dir()?.join(format!("{timestamp}-{}.json", agent.as_str())))
}

pub(crate) fn resolve_manifest(agent: &AgentId, backup_id: Option<&str>) -> Result<PathBuf> {
  if let Some(id) = backup_id {
    let path = PathBuf::from(id);
    if path.exists() {
      return Ok(path);
    }
    let candidate = manifest_dir()?.join(if id.ends_with(".json") {
      id.to_string()
    } else {
      format!("{id}-{}.json", agent.as_str())
    });
    if candidate.exists() {
      return Ok(candidate);
    }
    bail!("backup manifest not found: {id}");
  }

  let dir = manifest_dir()?;
  let suffix = format!("-{}.json", agent.as_str());
  let mut candidates = Vec::new();
  if dir.exists() {
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
      let entry = entry?;
      let path = entry.path();
      if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(&suffix))
        .unwrap_or(false)
      {
        candidates.push(path);
      }
    }
  }
  candidates.sort();
  candidates
    .pop()
    .ok_or_else(|| anyhow!("no migration manifest found for {}", agent.as_str()))
}

pub(crate) fn backup_path_for(path: &Path, timestamp: &str, files: &mut Vec<FileBackup>) -> Result<()> {
  if files.iter().any(|file| file.original == path) {
    return Ok(());
  }
  let existed = path.exists();
  let backup = if existed {
    let backup = adjacent_backup_path(path, timestamp)?;
    std::fs::copy(path, &backup).with_context(|| format!("backing up {} to {}", path.display(), backup.display()))?;
    Some(backup)
  } else {
    None
  };
  files.push(FileBackup {
    original: path.to_path_buf(),
    backup,
    existed,
    created_by_migration: false,
  });
  Ok(())
}

pub(crate) fn mark_created(files: &mut [FileBackup], path: &Path, existed: bool) {
  if !existed {
    if let Some(file) = files.iter_mut().find(|file| file.original == path) {
      file.created_by_migration = true;
    }
  }
}

pub(crate) fn adjacent_backup_path(path: &Path, timestamp: &str) -> Result<PathBuf> {
  let name = path
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| anyhow!("cannot back up path without file name: {}", path.display()))?;
  Ok(path.with_file_name(format!("{name}.bak.{timestamp}")))
}

pub(crate) fn write_manifest(path: &Path, manifest: &MigrationManifest) -> Result<()> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  std::fs::write(path, serde_json::to_vec_pretty(manifest)?).with_context(|| format!("writing {}", path.display()))
}

fn manifest_dir() -> Result<PathBuf> {
  Ok(tokn_config::paths::config_dir()?.join("agent-migrations"))
}

fn default_completed() -> bool {
  true
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn adjacent_backup_keeps_original_name() {
    let path = PathBuf::from("/tmp/auth.json");
    assert_eq!(
      adjacent_backup_path(&path, "20260604T153012Z").unwrap(),
      PathBuf::from("/tmp/auth.json.bak.20260604T153012Z")
    );
  }

  #[test]
  fn backup_path_for_records_missing_file_without_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.json");
    let mut files = Vec::new();

    backup_path_for(&path, "20260604T153012Z", &mut files).unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].original, path);
    assert_eq!(files[0].backup, None);
    assert!(!files[0].existed);
    assert!(!files[0].created_by_migration);
  }

  #[test]
  fn backup_path_for_copies_existing_file_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auth.json");
    std::fs::write(&path, "original").unwrap();
    let mut files = Vec::new();

    backup_path_for(&path, "20260604T153012Z", &mut files).unwrap();
    backup_path_for(&path, "20260604T153012Z", &mut files).unwrap();

    assert_eq!(files.len(), 1);
    let backup = files[0].backup.as_ref().unwrap();
    assert_eq!(backup, &dir.path().join("auth.json.bak.20260604T153012Z"));
    assert_eq!(std::fs::read_to_string(backup).unwrap(), "original");
    assert!(files[0].existed);
  }

  #[test]
  fn mark_created_only_marks_new_files() {
    let existing = PathBuf::from("existing");
    let created = PathBuf::from("created");
    let mut files = vec![
      FileBackup {
        original: existing.clone(),
        backup: None,
        existed: true,
        created_by_migration: false,
      },
      FileBackup {
        original: created.clone(),
        backup: None,
        existed: false,
        created_by_migration: false,
      },
    ];

    mark_created(&mut files, &existing, true);
    mark_created(&mut files, &created, false);

    assert!(!files[0].created_by_migration);
    assert!(files[1].created_by_migration);
  }

  #[test]
  fn resolve_manifest_accepts_full_path_and_rejects_missing_id() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("20260604T153012Z-codex-cli.json");
    std::fs::write(&manifest, "{}").unwrap();

    assert_eq!(
      resolve_manifest(&AgentId::CodexCli, Some(manifest.to_str().unwrap())).unwrap(),
      manifest
    );
    assert!(resolve_manifest(&AgentId::CodexCli, Some("does-not-exist")).is_err());
  }

  #[test]
  fn manifest_without_completed_field_defaults_to_complete() {
    let manifest: MigrationManifest = serde_json::from_str(
      r#"{
        "version": 1,
        "agent": "codex-cli",
        "timestamp": "20260604T153012Z",
        "profile": "codex",
        "target_base_url": "http://127.0.0.1:4141/codex/v1",
        "imported_account_ids": [],
        "files": []
      }"#,
    )
    .unwrap();

    assert!(manifest.completed);
  }
}
