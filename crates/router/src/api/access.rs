use super::error::ApiError;
use super::{AppState, LiveAppState};
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokn_access::{AccessContext, AuthenticationError};
use tokn_config::RouteMode;

pub async fn authenticate(State(state): State<LiveAppState>, mut request: Request, next: Next) -> Response {
  if !is_client_api_path(request.uri().path()) {
    return next.run(request).await;
  }

  let current = state.current();
  if !current.api_key_enabled || api_request_is_passthrough(&current, request.uri().path()) {
    request.extensions_mut().insert(AccessContext::unrestricted());
    return next.run(request).await;
  }

  match authenticate_managed_request(&current, &mut request) {
    Ok(()) => next.run(request).await,
    Err(error) => unauthorized(error),
  }
}

/// Authenticate one gateway-managed request and remove its gateway credential
/// before downstream routing. A proxy request may already carry a verified
/// context when it is rewritten into the API router; do not authenticate twice.
pub(crate) fn authenticate_managed_request<B>(
  state: &AppState,
  request: &mut http::Request<B>,
) -> Result<(), AuthenticationError> {
  if request.extensions().get::<AccessContext>().is_some() {
    return Ok(());
  }
  let token = bearer_token(request.headers()).or_else(|| api_key_header(request.headers()));
  let context = state.access.authenticate(token)?;

  request.headers_mut().remove(header::AUTHORIZATION);
  request.headers_mut().remove("x-api-key");
  request.extensions_mut().insert(context);
  Ok(())
}

fn api_request_is_passthrough(state: &AppState, path: &str) -> bool {
  let segments = path.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
  let mode = match segments.as_slice() {
    ["v1", ..] => Some(state.default_policy.mode),
    [profile, "v1", ..] => state.profiles.get(*profile).map(|policy| policy.mode),
    _ => None,
  };
  mode == Some(RouteMode::Passthrough)
}

fn is_client_api_path(path: &str) -> bool {
  path.starts_with("/v1/") || path.split('/').filter(|part| !part.is_empty()).nth(1) == Some("v1")
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<&str> {
  let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
  value
    .strip_prefix("Bearer ")
    .or_else(|| value.strip_prefix("bearer "))
    .map(str::trim)
    .filter(|token| !token.is_empty())
}

fn api_key_header(headers: &axum::http::HeaderMap) -> Option<&str> {
  headers
    .get("x-api-key")
    .and_then(|value| value.to_str().ok())
    .map(str::trim)
    .filter(|token| !token.is_empty())
}

pub(crate) fn unauthorized(error: AuthenticationError) -> Response {
  let message = match error {
    AuthenticationError::Missing => "missing API key",
    AuthenticationError::Invalid | AuthenticationError::Revoked => "invalid API key",
  };
  let mut response = ApiError::unauthorized(message).into_response();
  response
    .headers_mut()
    .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
  response
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::HeaderMap;

  #[test]
  fn recognizes_root_and_profile_api_paths() {
    assert!(is_client_api_path("/v1/models"));
    assert!(is_client_api_path("/work/v1/responses"));
    assert!(!is_client_api_path("/healthz"));
    assert!(!is_client_api_path("/admin/config/reload"));
  }

  #[test]
  fn extracts_bearer_and_x_api_key() {
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, "Bearer tokn_key_secret".parse().unwrap());
    assert_eq!(bearer_token(&headers), Some("tokn_key_secret"));
    headers.remove(header::AUTHORIZATION);
    headers.insert("x-api-key", "tokn_other_secret".parse().unwrap());
    assert_eq!(api_key_header(&headers), Some("tokn_other_secret"));
  }
}
