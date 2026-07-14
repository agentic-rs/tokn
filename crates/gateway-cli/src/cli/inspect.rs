use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct InspectArgs {
  /// Loopback TCP port for the viewer. The default picks an available port.
  #[arg(long, default_value_t = 0)]
  pub port: u16,

  /// Override the configured requests directory.
  #[arg(long)]
  pub requests_dir: Option<PathBuf>,
}

pub async fn run(cfg_path: Option<PathBuf>, args: InspectArgs) -> Result<()> {
  let (cfg, _) = crate::config::Config::load(cfg_path.as_deref())?;
  let mut paths = cfg.db.resolve_paths()?;
  if let Some(requests_dir) = args.requests_dir {
    paths.requests_dir = requests_dir;
  }
  tokn_router_inspect::serve(paths.requests_dir, args.port)
    .await
    .context("run inspect viewer")
}
