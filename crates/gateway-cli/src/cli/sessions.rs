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
  /// Source requests/<YYYY-MM-DD>.db file to read.
  #[arg(long)]
  pub requests_db: PathBuf,

  /// Destination sessions.db file to create or update.
  #[arg(long)]
  pub sessions_db: PathBuf,
}

pub async fn run(cmd: SessionsCmd) -> Result<()> {
  match cmd {
    SessionsCmd::Playback(args) => playback(args).await,
  }
}

async fn playback(args: PlaybackArgs) -> Result<()> {
  let report = crate::db::sessions::playback_requests_into_sessions(&args.requests_db, &args.sessions_db)?;
  println!("requests_db={}", args.requests_db.display());
  println!("sessions_db={}", args.sessions_db.display());
  println!("rows_seen={}", report.rows_seen);
  println!("rows_with_session={}", report.rows_with_session);
  println!("rows_recorded={}", report.rows_recorded);
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
