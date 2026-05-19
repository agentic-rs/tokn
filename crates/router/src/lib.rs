use anyhow::{anyhow, Result};

pub mod accounts;
pub mod api;
pub mod pipeline;
pub mod proxy;
pub mod relay;

pub use tokn_config as config;
pub use tokn_config::profiles;
pub use tokn_convert as convert;
pub use tokn_core::{db, provider, util};

pub fn install_rustls_crypto_provider() -> Result<()> {
  rustls::crypto::ring::default_provider()
    .install_default()
    .map_err(|_| anyhow!("failed to install rustls ring crypto provider"))?;
  Ok(())
}
