use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum SessionsCmd {
  /// Replay a requests day database into a tree-shaped sessions database.
  Playback(PlaybackArgs),
}

#[derive(Args, Debug)]
pub struct PlaybackArgs {
  /// Source requests/<YYYY-MM-DD>.db file to read. Overrides --requests-dir.
  #[arg(long)]
  pub requests_db: Option<PathBuf>,

  /// Source requests directory to replay in filename order.
  #[arg(long)]
  pub requests_dir: Option<PathBuf>,

  /// Destination sessions.db file to create or update.
  #[arg(long)]
  pub sessions_db: Option<PathBuf>,

  /// Reprocess rows even when their session/request node already exists.
  #[arg(long)]
  pub force: bool,
}

pub async fn run(cmd: SessionsCmd) -> Result<()> {
  match cmd {
    SessionsCmd::Playback(args) => playback(args).await,
  }
}

async fn playback(args: PlaybackArgs) -> Result<()> {
  let requests_dir = args
    .requests_dir
    .map(Ok)
    .unwrap_or_else(crate::config::paths::default_requests_dir)?;
  let sessions_db = args.sessions_db.map(Ok).unwrap_or_else(default_playback_sessions_db)?;
  let source = match args.requests_db {
    Some(path) => crate::db::sessions::PlaybackSource::File(path),
    None => crate::db::sessions::PlaybackSource::Dir(requests_dir),
  };
  let report = crate::db::sessions::playback_requests_source_into_sessions(
    source.clone(),
    &sessions_db,
    crate::db::sessions::PlaybackOptions { force: args.force },
  )?;
  match source {
    crate::db::sessions::PlaybackSource::File(path) => println!("requests_db={}", path.display()),
    crate::db::sessions::PlaybackSource::Dir(path) => println!("requests_dir={}", path.display()),
  }
  println!("sessions_db={}", sessions_db.display());
  println!("force={}", args.force);
  println!("rows_seen={}", report.rows_seen);
  println!("rows_with_session={}", report.rows_with_session);
  println!("rows_recorded={}", report.rows_recorded);
  println!("rows_existing={}", report.rows_existing);
  println!("rows_skipped={}", report.rows_skipped);
  println!("decode_errors={}", report.decode_errors);
  println!("reduction_mismatches={}", report.reduction_mismatches);
  println!("latest_mismatches={}", report.latest_mismatches.len());
  for mismatch in &report.latest_mismatches {
    println!(
      "latest_mismatch session_id={} expected_request_id={} actual_request_id={}",
      mismatch.session_id,
      mismatch.expected_request_id,
      mismatch.actual_request_id.as_deref().unwrap_or("<missing>")
    );
  }
  if !report.latest_mismatches.is_empty() {
    bail!("session latest view did not match requests DB latest rows");
  }
  Ok(())
}

fn default_playback_sessions_db() -> crate::config::Result<PathBuf> {
  Ok(crate::config::paths::data_dir()?.join("sessions.playback.db"))
}
