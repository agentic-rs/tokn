use super::error::ApiError;
use super::LiveAppState;
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokn_access::{AccessContext, AuthenticationError};

pub async fn authenticate(State(state): State<LiveAppState>, mut request: Request, next: Next) -> Response {
  if !is_client_api_path(request.uri().path()) {
    return next.run(request).await;
  }

  let current = state.current();
  let enabled = match current.access.is_enabled() {
    Ok(enabled) => enabled,
    Err(error) => {
      tracing::error!(%error, "client authentication store unavailable");
      return ApiError::internal("client authentication unavailable").into_response();
    }
  };
  let context = if enabled {
    let token = bearer_token(request.headers()).or_else(|| api_key_header(request.headers()));
    match current.access.authenticate(token) {
      Ok(context) => context,
      Err(error) => return unauthorized(error),
    }
  } else {
    AccessContext::unrestricted()
  };

  // A gateway credential must never be forwarded to an upstream provider.
  // When authentication is disabled, preserve existing passthrough behavior.
  if enabled {
    request.headers_mut().remove(header::AUTHORIZATION);
    request.headers_mut().remove("x-api-key");
  }
  request.extensions_mut().insert(context);
  next.run(request).await
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

fn unauthorized(error: AuthenticationError) -> Response {
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
