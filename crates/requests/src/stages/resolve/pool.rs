//! Production [`AccountSelector`] backed by [`tokn_accounts::AccountPool`].
//!
//! [`PoolAccountSelector`] bridges the requests [`AccountSelector`] trait
//! to the existing [`AccountPool`] + [`RouteResolver`] machinery. It
//! mirrors what `crates/router/src/pipeline/request.rs::resolve_account`
//! does, but returns a typed [`SelectorOutcome`] instead of poking at an
//! `AppState`.
//!
//! The selector takes ownership of a `RouteResolver` (cheap to clone via
//! `Arc`) and an `AccountPool`, so it can be assembled directly from the
//! CLI / gateway without dragging the legacy `AppState`.

use super::stage::{AccountSelector, SelectorOutcome};
use crate::event::Stage;
use crate::pipeline::ctx::PipelineCtx;
use crate::pipeline::error::{PipelineError, RequestsError};
use crate::pipeline::stages::Extracted;
use async_trait::async_trait;
use smol_str::SmolStr;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokn_accounts::{AccountPool, EndpointAcquire, RouteResolver};

pub struct PoolAccountSelector {
  pool: Arc<AccountPool>,
  resolver: Arc<RouteResolver>,
  allowed_providers: Option<Arc<BTreeSet<String>>>,
  allowed_accounts: Option<Arc<BTreeSet<String>>>,
}

impl PoolAccountSelector {
  pub fn new(pool: Arc<AccountPool>, resolver: Arc<RouteResolver>) -> Self {
    Self {
      pool,
      resolver,
      allowed_providers: None,
      allowed_accounts: None,
    }
  }

  pub fn new_with_providers(
    pool: Arc<AccountPool>,
    resolver: Arc<RouteResolver>,
    allowed_providers: Option<Arc<BTreeSet<String>>>,
  ) -> Self {
    Self {
      pool,
      resolver,
      allowed_providers,
      allowed_accounts: None,
    }
  }

  pub fn new_filtered(
    pool: Arc<AccountPool>,
    resolver: Arc<RouteResolver>,
    allowed_providers: Option<Arc<BTreeSet<String>>>,
    allowed_accounts: Option<Arc<BTreeSet<String>>>,
  ) -> Self {
    Self {
      pool,
      resolver,
      allowed_providers,
      allowed_accounts,
    }
  }
}

#[async_trait]
impl AccountSelector for PoolAccountSelector {
  async fn select(&self, ctx: &PipelineCtx, extracted: &Extracted) -> Result<SelectorOutcome, PipelineError> {
    let request_endpoint = ctx.request_endpoint.resolved().ok_or_else(|| {
      PipelineError::permanent(
        Stage::Resolve,
        RequestsError::MissingResolvedEndpoint {
          request_endpoint: SmolStr::new(ctx.request_endpoint.as_str()),
        },
      )
    })?;
    // Route mode hint comes from the inbound `x-route-mode` header (or
    // equivalent) — `DefaultExtract` parses this into
    // `extracted.route_mode_hint`.
    let route = self
      .resolver
      .resolve(extracted.model.as_str(), extracted.route_mode_hint.as_deref())
      .map_err(|e| PipelineError::permanent(Stage::Resolve, RequestsError::Resolve { source: e }))?;

    match self.pool.acquire_for_route_filtered(
      extracted.session_id.as_deref(),
      &route,
      request_endpoint,
      self.allowed_providers.as_deref(),
      self.allowed_accounts.as_deref(),
    ) {
      EndpointAcquire::Account { acct, endpoint } => {
        let provider_id = SmolStr::from(acct.provider.info().id.as_str());
        let account_id = SmolStr::from(acct.id());
        Ok(SelectorOutcome::Selected {
          account_id,
          provider_id,
          upstream_endpoint: Some(endpoint),
          upstream_model: SmolStr::from(route.upstream_model.as_str()),
          account_handle: acct,
        })
      }
      EndpointAcquire::SessionExpired => Ok(SelectorOutcome::SessionExpired {
        session_id: extracted.session_id.clone().unwrap_or_else(|| SmolStr::new("")),
      }),
      EndpointAcquire::None => Ok(SelectorOutcome::NoAccount),
    }
  }
}
