use crate::util::secret::Secret;
use crate::{error, HeaderPatchCtx, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use snafu::ResultExt;

pub enum Credential {
  ApiKey(Secret<String>),
  AccessToken(Secret<String>),
}

impl Credential {
  pub fn expose(&self) -> &str {
    match self {
      Credential::ApiKey(secret) | Credential::AccessToken(secret) => secret.expose(),
    }
  }
}

pub fn url(base_url: &str, path: &str) -> String {
  format!("{}{}", base_url.trim_end_matches('/'), path)
}

pub fn patch_openai_headers(headers: &mut HeaderMap, token: &str, ctx: &HeaderPatchCtx<'_>) -> Result<()> {
  headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {token}"))
      .context(error::HeaderValueSnafu { name: "authorization" })?,
  );
  headers.insert(
    ACCEPT,
    HeaderValue::from_static(if ctx.stream {
      "text/event-stream"
    } else {
      "application/json"
    }),
  );
  headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
  if let Some(encoding) = ctx.content_encoding {
    headers.insert(
      reqwest::header::CONTENT_ENCODING,
      HeaderValue::from_str(encoding).context(error::HeaderValueSnafu {
        name: "content-encoding",
      })?,
    );
  }
  Ok(())
}
