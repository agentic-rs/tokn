use super::Result;
use std::path::PathBuf;

pub fn config_path() -> Result<PathBuf> {
  Ok(config_dir()?.join("config.toml"))
}

pub fn config_dir() -> Result<PathBuf> {
  tokn_core::util::paths::config_dir().ok_or(super::Error::NoProjectDirs)
}

pub fn data_dir() -> Result<PathBuf> {
  tokn_core::util::paths::data_dir().ok_or(super::Error::NoProjectDirs)
}

pub fn cache_dir() -> Result<PathBuf> {
  tokn_core::util::paths::cache_dir().ok_or(super::Error::NoProjectDirs)
}

pub fn default_usage_db() -> Result<PathBuf> {
  Ok(data_dir()?.join("usage.db"))
}

pub fn default_sessions_db() -> Result<PathBuf> {
  Ok(data_dir()?.join("sessions.db"))
}

pub fn default_requests_dir() -> Result<PathBuf> {
  Ok(data_dir()?.join("requests"))
}

pub fn default_logs_dir() -> Result<PathBuf> {
  Ok(data_dir()?.join("logs"))
}

pub fn default_ca_dir() -> Result<PathBuf> {
  Ok(config_dir()?.join("ca"))
}
