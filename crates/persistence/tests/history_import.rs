use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};
use tokn_persistence::{history_import::import_history, requests::open_day_db, sessions::SessionsDb, DbPaths, UsageDb};

const DAY: &str = "2026-09-21.db";

struct Fixture {
  root: PathBuf,
  source: PathBuf,
  destination: DbPaths,
}
impl Fixture {
  fn new() -> Self {
    let root = std::env::temp_dir().join(format!("tokn-history-import-{}", uuid::Uuid::new_v4()));
    let source = root.join("source");
    let destination = paths(&root.join("destination"));
    initialize(&paths(&source));
    initialize(&destination);
    let source_paths = paths(&source);
    Connection::open(&source_paths.usage_db)
      .unwrap()
      .execute(
        "INSERT INTO requests (id, ts, request_id, model) VALUES (1, 100, 'trial-request', 'trial-model')",
        [],
      )
      .unwrap();
    Connection::open(&source_paths.sessions_db).unwrap().execute_batch(
      "INSERT INTO sessions (id, first_seen_ts, last_seen_ts, source) VALUES ('trial-session', 100, 100, 'header');
       INSERT INTO part_blobs (hash, part_type, content) VALUES ('part-hash', 'text', X'68656c6c6f');
       INSERT INTO message_tree (id, depth, message_hash, role) VALUES (zeroblob(32), 1, zeroblob(32), 'user');
       INSERT INTO message_parts (message_id, part_index, part_hash) VALUES (zeroblob(32), 0, 'part-hash');
       INSERT INTO session_nodes (id, session_id, request_id, ts, endpoint, reduction_kind, parent_source, message_id)
         VALUES ('trial-node', 'trial-session', 'trial-request', 100, 'responses', 'message_tree', 'none', zeroblob(32));
       INSERT INTO session_heads (session_id, node_id, updated_ts) VALUES ('trial-session', 'trial-node', 100);",
    ).unwrap();
    open_day_db(&source_paths.requests_dir.join(DAY)).unwrap().execute_batch(
      "INSERT INTO request_connection (request_id, ts) VALUES ('trial-request', 100);
       INSERT INTO request_metadata (request_id, session_id, model) VALUES ('trial-request', 'trial-session', 'trial-model');",
    ).unwrap();
    Self {
      root,
      source,
      destination,
    }
  }
  fn error(&self, commit: bool) -> String {
    import_history(&self.source, &self.destination, commit)
      .unwrap_err()
      .to_string()
  }
  fn day(&self) -> PathBuf {
    self.destination.requests_dir.join(DAY)
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}
fn paths(root: &Path) -> DbPaths {
  DbPaths {
    usage_db: root.join("usage.db"),
    sessions_db: root.join("sessions.db"),
    requests_dir: root.join("requests"),
  }
}
fn initialize(paths: &DbPaths) {
  fs::create_dir_all(&paths.requests_dir).unwrap();
  drop(UsageDb::open(&paths.usage_db).unwrap());
  drop(SessionsDb::open(&paths.sessions_db).unwrap());
}
fn count(path: &Path, table: &str) -> i64 {
  Connection::open(path)
    .unwrap()
    .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0))
    .unwrap()
}

#[test]
fn import_remaps_usage_ids_preserves_graph_and_is_idempotent() {
  let fixture = Fixture::new();
  Connection::open(&fixture.destination.usage_db)
    .unwrap()
    .execute(
      "INSERT INTO requests (id, ts, request_id, model) VALUES (1, 99, 'host-request', 'host-model')",
      [],
    )
    .unwrap();
  let first = import_history(&fixture.source, &fixture.destination, true).unwrap();
  assert!(first.committed);
  assert_eq!(first.inserted_total, 9);
  assert_eq!(first.identical_skipped_total, 0);
  assert!(first.databases[2].created);
  let conn = Connection::open(&fixture.destination.usage_db).unwrap();
  let id: i64 = conn
    .query_row(
      "SELECT id FROM requests WHERE request_id = 'trial-request'",
      [],
      |row| row.get(0),
    )
    .unwrap();
  assert_eq!(id, 2);
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 2);
  let sessions = Connection::open(&fixture.destination.sessions_db).unwrap();
  let hash: Vec<u8> = sessions
    .query_row("SELECT message_id FROM session_nodes", [], |row| row.get(0))
    .unwrap();
  assert_eq!(hash, vec![0; 32]);
  assert!(!sessions
    .prepare("PRAGMA foreign_key_check")
    .unwrap()
    .exists([])
    .unwrap());
  let second = import_history(&fixture.source, &fixture.destination, true).unwrap();
  assert_eq!(second.inserted_total, 0);
  assert_eq!(second.identical_skipped_total, first.inserted_total);
  assert!(!second.databases[2].created);
}

#[test]
fn dry_run_checks_all_rows_and_removes_new_day() {
  let fixture = Fixture::new();
  let source_before = fs::read(fixture.source.join("usage.db")).unwrap();
  let report = import_history(&fixture.source, &fixture.destination, false).unwrap();
  assert!(!report.committed);
  assert_eq!(report.inserted_total, 9);
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert_eq!(count(&fixture.destination.sessions_db, "sessions"), 0);
  assert!(!fixture.day().exists());
  assert_eq!(fs::read(fixture.source.join("usage.db")).unwrap(), source_before);
}

#[test]
fn late_collision_rolls_back_every_database() {
  let fixture = Fixture::new();
  open_day_db(&fixture.day())
    .unwrap()
    .execute(
      "INSERT INTO request_connection (request_id, ts) VALUES ('trial-request', 999)",
      [],
    )
    .unwrap();
  assert!(fixture.error(true).contains("Nonidentical collision"));
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert_eq!(count(&fixture.destination.sessions_db, "sessions"), 0);
  let ts: i64 = Connection::open(fixture.day())
    .unwrap()
    .query_row("SELECT ts FROM request_connection", [], |row| row.get(0))
    .unwrap();
  assert_eq!(ts, 999);
}

#[test]
fn collision_in_usage_dry_run_removes_initialized_day() {
  let fixture = Fixture::new();
  Connection::open(&fixture.destination.usage_db)
    .unwrap()
    .execute(
      "INSERT INTO requests (ts, request_id, model) VALUES (100, 'trial-request', 'different-model')",
      [],
    )
    .unwrap();
  assert!(fixture.error(false).contains("Nonidentical collision"));
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 1);
  assert!(!fixture.day().exists());
}

#[test]
fn schemas_compare_column_names_instead_of_ordinals() {
  let fixture = Fixture::new();
  // This mirrors a migrated database where ALTER TABLE appended `user`.
  let conn = Connection::open(&fixture.destination.usage_db).unwrap();
  conn.execute_batch("DROP INDEX idx_requests_user; ALTER TABLE requests DROP COLUMN user; ALTER TABLE requests ADD COLUMN user TEXT; CREATE INDEX idx_requests_user ON requests(user);").unwrap();
  let report = import_history(&fixture.source, &fixture.destination, true).unwrap();
  assert_eq!(report.inserted_total, 9);
}

#[test]
fn source_with_dangling_foreign_key_is_rejected_even_for_dry_run() {
  let fixture = Fixture::new();
  let conn = Connection::open(fixture.source.join("sessions.db")).unwrap();
  conn.pragma_update(None, "foreign_keys", false).unwrap();
  conn
    .execute(
      "INSERT INTO message_parts (message_id, part_index, part_hash) VALUES (?1, 0, 'missing-part')",
      params![vec![1u8; 32]],
    )
    .unwrap();
  assert!(fixture.error(false).contains("Source foreign key check failed"));
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert!(!fixture.day().exists());
}

#[test]
fn destination_schema_mismatch_rolls_back_and_cleans_new_day() {
  let fixture = Fixture::new();
  Connection::open(&fixture.destination.sessions_db)
    .unwrap()
    .execute(
      "DELETE FROM schema_migrations WHERE version = ?1",
      [tokn_persistence::sessions::latest_version()],
    )
    .unwrap();
  assert!(fixture.error(true).contains("Schema version mismatch"));
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert!(!fixture.day().exists());
}

#[test]
fn archived_destination_day_is_never_recreated() {
  let fixture = Fixture::new();
  fs::write(fixture.destination.requests_dir.join(format!("{DAY}.xz")), b"archive").unwrap();
  assert!(fixture.error(true).contains("has an archive"));
  assert!(!fixture.day().exists());
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
}

#[test]
fn active_source_sidecars_are_rejected() {
  let fixture = Fixture::new();
  for suffix in ["-wal", "-journal"] {
    let sidecar = fixture.source.join(format!("usage.db{suffix}"));
    fs::write(&sidecar, []).unwrap();
    assert!(fixture.error(true).contains("stopped and checkpointed"));
    fs::remove_file(sidecar).unwrap();
  }
}

#[test]
fn wal_destination_mode_is_rejected_without_changing_it() {
  let fixture = Fixture::new();
  let conn = Connection::open(&fixture.destination.usage_db).unwrap();
  conn.pragma_update(None, "journal_mode", "wal").unwrap();
  assert!(fixture.error(true).contains("requires DELETE journal mode"));
  let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0)).unwrap();
  assert_eq!(mode, "wal");
  assert!(!fixture.day().exists());
}

#[test]
fn hardlinked_source_and_destination_are_rejected() {
  let fixture = Fixture::new();
  fs::remove_file(&fixture.destination.usage_db).unwrap();
  fs::hard_link(fixture.source.join("usage.db"), &fixture.destination.usage_db).unwrap();
  assert!(fixture.error(true).contains("alias each other"));
  assert!(!fixture.day().exists());
}

#[cfg(unix)]
#[test]
fn symlinked_source_or_destination_is_rejected() {
  use std::os::unix::fs::symlink;
  let fixture = Fixture::new();
  let original = fixture.source.join("original.db");
  fs::rename(fixture.source.join("usage.db"), &original).unwrap();
  symlink(&original, fixture.source.join("usage.db")).unwrap();
  assert!(fixture.error(true).contains("non-symlink database"));
  fs::remove_file(fixture.source.join("usage.db")).unwrap();
  fs::rename(original, fixture.source.join("usage.db")).unwrap();
  let original = fixture.root.join("original-target.db");
  fs::rename(&fixture.destination.usage_db, &original).unwrap();
  symlink(&original, &fixture.destination.usage_db).unwrap();
  assert!(fixture.error(true).contains("non-symlink database"));
}

#[test]
fn archived_source_days_cannot_be_silently_omitted() {
  let fixture = Fixture::new();
  for extension in ["xz", "zstd", "zst", "lzma"] {
    // A different day simulates an older archived day alongside the current raw day.
    let archive = fixture.source.join(format!("requests/2026-09-20.db.{extension}"));
    fs::write(&archive, b"archive").unwrap();
    assert!(fixture
      .error(true)
      .contains("Archived source request days are not supported"));
    assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
    assert!(!fixture.day().exists());
    fs::remove_file(archive).unwrap();
  }
}

#[test]
fn orphan_source_request_sidecars_are_rejected() {
  let fixture = Fixture::new();
  for suffix in ["-wal", "-journal", "-shm"] {
    let sidecar = fixture.source.join(format!("requests/2026-09-20.db{suffix}"));
    fs::write(&sidecar, []).unwrap();
    assert!(fixture.error(true).contains("unexpected request sidecar"));
    assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
    assert!(!fixture.day().exists());
    fs::remove_file(sidecar).unwrap();
  }
}

#[test]
fn source_request_scan_ignores_only_the_regular_maintenance_lock() {
  let fixture = Fixture::new();
  fs::write(fixture.source.join("requests/.tokn-requests-maintenance.lock"), []).unwrap();
  assert_eq!(
    import_history(&fixture.source, &fixture.destination, false)
      .unwrap()
      .inserted_total,
    9
  );
  let unexpected = fixture.source.join("requests/scratch.txt");
  fs::write(&unexpected, b"scratch").unwrap();
  assert!(fixture.error(true).contains("Unexpected source request entry"));
  fs::remove_file(unexpected).unwrap();
  fs::create_dir(fixture.source.join("requests/nested-capture")).unwrap();
  assert!(fixture.error(true).contains("regular, non-symlink database"));
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert!(!fixture.day().exists());
}

#[cfg(unix)]
#[test]
fn source_request_symlinks_are_rejected_regardless_of_filename() {
  let fixture = Fixture::new();
  for name in ["ignored.txt", ".tokn-requests-maintenance.lock", "2026-09-20.db.xz"] {
    let link = fixture.source.join("requests").join(name);
    std::os::unix::fs::symlink(fixture.source.join("usage.db"), &link).unwrap();
    assert!(fixture.error(true).contains("regular, non-symlink database"));
    fs::remove_file(link).unwrap();
  }
  assert_eq!(count(&fixture.destination.usage_db, "requests"), 0);
  assert!(!fixture.day().exists());
}
