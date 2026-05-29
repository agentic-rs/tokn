use anyhow::{anyhow, Context, Result};
use axum::extract::{Query, State};
use axum::http::header::{CONTENT_TYPE, HOST};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokn_router::proxy::ProxyCa;

const BOOTSTRAP_JSON_PATH: &str = "/-/lan/bootstrap.json";
const CA_CERT_PATH: &str = "/-/lan/ca.crt";
const ENV_PATH: &str = "/-/lan/env";

#[derive(Clone, Debug)]
pub struct BootstrapState {
  ca_cert_pem: String,
  ca_fingerprint: String,
  api_port: u16,
  proxy_port: u16,
}

impl BootstrapState {
  pub fn new(ca: &ProxyCa, api_port: u16, proxy_port: u16) -> Result<Self> {
    let ca_cert_path = ca.cert_path();
    let ca_cert_pem =
      std::fs::read_to_string(&ca_cert_path).with_context(|| format!("read {}", ca_cert_path.display()))?;
    Ok(Self {
      ca_cert_pem,
      ca_fingerprint: ca.fingerprint_sha256(),
      api_port,
      proxy_port,
    })
  }
}

pub fn router(state: BootstrapState) -> Router {
  Router::new()
    .route(BOOTSTRAP_JSON_PATH, get(bootstrap_json))
    .route(CA_CERT_PATH, get(ca_cert))
    .route(ENV_PATH, get(env_script))
    .with_state(state)
}

pub fn display_bootstrap_url(bind_host: &str, api_port: u16) -> String {
  let host = bind_host.trim();
  if matches!(host, "0.0.0.0" | "::" | "[::]") {
    return format!("http://<server-lan-ip>:{api_port}{BOOTSTRAP_JSON_PATH}");
  }
  format!("http://{}:{api_port}{BOOTSTRAP_JSON_PATH}", url_host(host))
}

#[derive(Serialize)]
struct BootstrapMetadata {
  api_url: String,
  proxy_url: String,
  ca_cert_url: String,
  env_url: String,
  ca_sha256: String,
}

async fn bootstrap_json(
  State(state): State<BootstrapState>,
  headers: HeaderMap,
) -> std::result::Result<Json<BootstrapMetadata>, BootstrapError> {
  let urls = urls_from_headers(&headers, state.api_port, state.proxy_port)?;
  Ok(Json(BootstrapMetadata {
    api_url: format!("{}/v1", urls.api_base),
    proxy_url: urls.proxy_base,
    ca_cert_url: format!("{}{}", urls.api_base, CA_CERT_PATH),
    env_url: format!("{}{}?shell=sh", urls.api_base, ENV_PATH),
    ca_sha256: state.ca_fingerprint,
  }))
}

async fn ca_cert(State(state): State<BootstrapState>) -> Response {
  ([(CONTENT_TYPE, "application/x-pem-file")], state.ca_cert_pem.clone()).into_response()
}

#[derive(Deserialize)]
struct EnvQuery {
  shell: Option<String>,
}

async fn env_script(
  State(state): State<BootstrapState>,
  headers: HeaderMap,
  Query(query): Query<EnvQuery>,
) -> std::result::Result<Response, BootstrapError> {
  let shell = Shell::parse(query.shell.as_deref())?;
  let urls = urls_from_headers(&headers, state.api_port, state.proxy_port)?;
  let script = render_env_script(shell, &urls, &state.ca_fingerprint);
  Ok(([(CONTENT_TYPE, shell.content_type())], script).into_response())
}

#[derive(Debug)]
struct BootstrapError(anyhow::Error);

impl From<anyhow::Error> for BootstrapError {
  fn from(value: anyhow::Error) -> Self {
    Self(value)
  }
}

impl IntoResponse for BootstrapError {
  fn into_response(self) -> Response {
    (StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Shell {
  Sh,
  Bash,
  Zsh,
  Fish,
  Pwsh,
}

impl Shell {
  fn parse(value: Option<&str>) -> Result<Self> {
    match value.unwrap_or("sh") {
      "sh" => Ok(Self::Sh),
      "bash" => Ok(Self::Bash),
      "zsh" => Ok(Self::Zsh),
      "fish" => Ok(Self::Fish),
      "pwsh" => Ok(Self::Pwsh),
      other => Err(anyhow!("unsupported shell '{other}'; expected sh|bash|zsh|fish|pwsh")),
    }
  }

  fn content_type(self) -> &'static str {
    match self {
      Self::Pwsh => "text/plain; charset=utf-8",
      _ => "text/x-shellscript; charset=utf-8",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapUrls {
  api_base: String,
  proxy_base: String,
  ca_cert_url: String,
  host_for_no_proxy: String,
}

fn urls_from_headers(headers: &HeaderMap, api_port: u16, proxy_port: u16) -> Result<BootstrapUrls> {
  let raw = headers
    .get(HOST)
    .ok_or_else(|| anyhow!("missing Host header"))?
    .to_str()
    .context("Host header must be valid ASCII")?;
  urls_from_host(raw, api_port, proxy_port)
}

fn urls_from_host(raw: &str, api_port: u16, proxy_port: u16) -> Result<BootstrapUrls> {
  let raw = raw.trim();
  if raw.is_empty() || raw.contains('@') {
    return Err(anyhow!("invalid Host header"));
  }
  let authority: http::uri::Authority = raw.parse().context("invalid Host header authority")?;
  let host = authority.host();
  validate_host(host)?;
  let url_host = url_host(host);
  let api_base = format!("http://{url_host}:{api_port}");
  Ok(BootstrapUrls {
    ca_cert_url: format!("{api_base}{CA_CERT_PATH}"),
    api_base,
    proxy_base: format!("http://{url_host}:{proxy_port}"),
    host_for_no_proxy: no_proxy_host(host),
  })
}

fn validate_host(host: &str) -> Result<()> {
  if host.is_empty() {
    return Err(anyhow!("Host header host cannot be empty"));
  }
  let valid = host
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':' | '[' | ']'));
  if !valid {
    return Err(anyhow!("Host header contains unsupported characters"));
  }
  Ok(())
}

fn url_host(host: &str) -> String {
  let host = host.trim_matches(['[', ']']);
  if host.contains(':') {
    format!("[{host}]")
  } else {
    host.to_string()
  }
}

fn no_proxy_host(host: &str) -> String {
  host.trim_matches(['[', ']']).to_string()
}

fn render_env_script(shell: Shell, urls: &BootstrapUrls, fingerprint: &str) -> String {
  match shell {
    Shell::Sh | Shell::Bash | Shell::Zsh => render_sh_env(urls, fingerprint),
    Shell::Fish => render_fish_env(urls, fingerprint),
    Shell::Pwsh => render_pwsh_env(urls, fingerprint),
  }
}

fn no_proxy_value(host: &str) -> String {
  format!("localhost,127.0.0.1,::1,{host}")
}

fn render_sh_env(urls: &BootstrapUrls, fingerprint: &str) -> String {
  let no_proxy = no_proxy_value(&urls.host_for_no_proxy);
  let api_url = format!("{}/v1", urls.api_base);
  format!(
    r#"# Verify this CA fingerprint before trusting it: {fingerprint}
TOKN_ROUTER_CA_DIR="${{XDG_CONFIG_HOME:-$HOME/.config}}/tokn-router/lan"
TOKN_ROUTER_CA_CERT="$TOKN_ROUTER_CA_DIR/ca.crt"
TOKN_ROUTER_CA_BUNDLE="$TOKN_ROUTER_CA_DIR/ca-bundle.crt"
mkdir -p "$TOKN_ROUTER_CA_DIR"
curl -fsSL {ca_url} -o "$TOKN_ROUTER_CA_CERT"
TOKN_ROUTER_SYSTEM_CA=""
for candidate in /etc/ssl/certs/ca-certificates.crt /etc/pki/tls/certs/ca-bundle.crt /etc/ssl/ca-bundle.pem /etc/pki/tls/cacert.pem /etc/ssl/cert.pem; do
  if [ -f "$candidate" ]; then
    TOKN_ROUTER_SYSTEM_CA="$candidate"
    break
  fi
done
if [ -n "$TOKN_ROUTER_SYSTEM_CA" ]; then
  cat "$TOKN_ROUTER_SYSTEM_CA" "$TOKN_ROUTER_CA_CERT" > "$TOKN_ROUTER_CA_BUNDLE"
else
  cp "$TOKN_ROUTER_CA_CERT" "$TOKN_ROUTER_CA_BUNDLE"
fi
export OPENAI_BASE_URL={api_url}
export HTTP_PROXY={proxy_url}
export HTTPS_PROXY={proxy_url}
export NO_PROXY={no_proxy}
export NODE_EXTRA_CA_CERTS="$TOKN_ROUTER_CA_CERT"
export SSL_CERT_FILE="$TOKN_ROUTER_CA_BUNDLE"
export REQUESTS_CA_BUNDLE="$TOKN_ROUTER_CA_BUNDLE"
export CURL_CA_BUNDLE="$TOKN_ROUTER_CA_BUNDLE"
export GIT_SSL_CAINFO="$TOKN_ROUTER_CA_BUNDLE"
"#,
    ca_url = sh_quote(&urls.ca_cert_url),
    api_url = sh_quote(&api_url),
    proxy_url = sh_quote(&urls.proxy_base),
    no_proxy = sh_quote(&no_proxy),
  )
}

fn render_fish_env(urls: &BootstrapUrls, fingerprint: &str) -> String {
  let no_proxy = no_proxy_value(&urls.host_for_no_proxy);
  let api_url = format!("{}/v1", urls.api_base);
  format!(
    r#"# Verify this CA fingerprint before trusting it: {fingerprint}
set -q XDG_CONFIG_HOME; or set XDG_CONFIG_HOME "$HOME/.config"
set -gx TOKN_ROUTER_CA_DIR "$XDG_CONFIG_HOME/tokn-router/lan"
set -gx TOKN_ROUTER_CA_CERT "$TOKN_ROUTER_CA_DIR/ca.crt"
set -gx TOKN_ROUTER_CA_BUNDLE "$TOKN_ROUTER_CA_DIR/ca-bundle.crt"
mkdir -p "$TOKN_ROUTER_CA_DIR"
curl -fsSL {ca_url} -o "$TOKN_ROUTER_CA_CERT"
set -l system_ca
for candidate in /etc/ssl/certs/ca-certificates.crt /etc/pki/tls/certs/ca-bundle.crt /etc/ssl/ca-bundle.pem /etc/pki/tls/cacert.pem /etc/ssl/cert.pem
  if test -f "$candidate"
    set system_ca "$candidate"
    break
  end
end
if test -n "$system_ca"
  cat "$system_ca" "$TOKN_ROUTER_CA_CERT" > "$TOKN_ROUTER_CA_BUNDLE"
else
  cp "$TOKN_ROUTER_CA_CERT" "$TOKN_ROUTER_CA_BUNDLE"
end
set -gx OPENAI_BASE_URL {api_url}
set -gx HTTP_PROXY {proxy_url}
set -gx HTTPS_PROXY {proxy_url}
set -gx NO_PROXY {no_proxy}
set -gx NODE_EXTRA_CA_CERTS "$TOKN_ROUTER_CA_CERT"
set -gx SSL_CERT_FILE "$TOKN_ROUTER_CA_BUNDLE"
set -gx REQUESTS_CA_BUNDLE "$TOKN_ROUTER_CA_BUNDLE"
set -gx CURL_CA_BUNDLE "$TOKN_ROUTER_CA_BUNDLE"
set -gx GIT_SSL_CAINFO "$TOKN_ROUTER_CA_BUNDLE"
"#,
    ca_url = sh_quote(&urls.ca_cert_url),
    api_url = sh_quote(&api_url),
    proxy_url = sh_quote(&urls.proxy_base),
    no_proxy = sh_quote(&no_proxy),
  )
}

fn render_pwsh_env(urls: &BootstrapUrls, fingerprint: &str) -> String {
  let no_proxy = no_proxy_value(&urls.host_for_no_proxy);
  let api_url = format!("{}/v1", urls.api_base);
  format!(
    r#"# Verify this CA fingerprint before trusting it: {fingerprint}
$configHome = if ($Env:XDG_CONFIG_HOME) {{ $Env:XDG_CONFIG_HOME }} else {{ Join-Path $HOME ".config" }}
$caDir = Join-Path $configHome "tokn-router/lan"
$caCert = Join-Path $caDir "ca.crt"
$caBundle = Join-Path $caDir "ca-bundle.crt"
New-Item -ItemType Directory -Force -Path $caDir | Out-Null
Invoke-WebRequest -UseBasicParsing -Uri {ca_url} -OutFile $caCert
$systemCa = @("/etc/ssl/certs/ca-certificates.crt", "/etc/pki/tls/certs/ca-bundle.crt", "/etc/ssl/ca-bundle.pem", "/etc/pki/tls/cacert.pem", "/etc/ssl/cert.pem") | Where-Object {{ Test-Path $_ }} | Select-Object -First 1
if ($systemCa) {{
  Get-Content $systemCa, $caCert | Set-Content $caBundle
}} else {{
  Copy-Item $caCert $caBundle
}}
$Env:OPENAI_BASE_URL = {api_url}
$Env:HTTP_PROXY = {proxy_url}
$Env:HTTPS_PROXY = {proxy_url}
$Env:NO_PROXY = {no_proxy}
$Env:NODE_EXTRA_CA_CERTS = $caCert
$Env:SSL_CERT_FILE = $caBundle
$Env:REQUESTS_CA_BUNDLE = $caBundle
$Env:CURL_CA_BUNDLE = $caBundle
$Env:GIT_SSL_CAINFO = $caBundle
"#,
    ca_url = pwsh_quote(&urls.ca_cert_url),
    api_url = pwsh_quote(&api_url),
    proxy_url = pwsh_quote(&urls.proxy_base),
    no_proxy = pwsh_quote(&no_proxy),
  )
}

fn sh_quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "'\\''"))
}

fn pwsh_quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::body::to_bytes;
  use axum::http::Request;
  use tower::ServiceExt;

  fn test_state() -> BootstrapState {
    BootstrapState {
      ca_cert_pem: "-----BEGIN CERTIFICATE-----\npublic\n-----END CERTIFICATE-----\n".into(),
      ca_fingerprint: "abc123".into(),
      api_port: 4141,
      proxy_port: 4142,
    }
  }

  #[test]
  fn concrete_host_produces_api_and_proxy_urls() {
    let urls = urls_from_host("192.168.1.10:4141", 4141, 4142).unwrap();
    assert_eq!(urls.api_base, "http://192.168.1.10:4141");
    assert_eq!(urls.proxy_base, "http://192.168.1.10:4142");
    assert_eq!(urls.ca_cert_url, "http://192.168.1.10:4141/-/lan/ca.crt");
  }

  #[test]
  fn wildcard_display_uses_lan_ip_template() {
    assert_eq!(
      display_bootstrap_url("0.0.0.0", 4141),
      "http://<server-lan-ip>:4141/-/lan/bootstrap.json"
    );
    assert_eq!(
      display_bootstrap_url("[::]", 4141),
      "http://<server-lan-ip>:4141/-/lan/bootstrap.json"
    );
    assert_eq!(
      display_bootstrap_url("lan-router.local", 4141),
      "http://lan-router.local:4141/-/lan/bootstrap.json"
    );
  }

  #[test]
  fn request_host_drives_urls_even_for_wildcard_bind() {
    let urls = urls_from_host("lan-router.local:4141", 4141, 4142).unwrap();
    assert_eq!(urls.api_base, "http://lan-router.local:4141");
    assert_eq!(urls.proxy_base, "http://lan-router.local:4142");
  }

  #[test]
  fn ipv6_host_formats_urls_and_no_proxy_host() {
    let urls = urls_from_host("[fd00::10]:4141", 4141, 4142).unwrap();
    assert_eq!(urls.api_base, "http://[fd00::10]:4141");
    assert_eq!(urls.proxy_base, "http://[fd00::10]:4142");
    assert_eq!(urls.host_for_no_proxy, "fd00::10");
  }

  #[test]
  fn rejects_shell_injection_host() {
    let err = urls_from_host("lan.local;touch /tmp/nope:4141", 4141, 4142).expect_err("host should be rejected");
    assert!(err.to_string().contains("invalid Host header"));
  }

  #[test]
  fn rejects_empty_and_userinfo_hosts() {
    assert!(urls_from_host("", 4141, 4142).is_err());
    assert!(urls_from_host("user@lan.local:4141", 4141, 4142).is_err());
  }

  #[test]
  fn shell_parsing_and_content_types_are_explicit() {
    assert_eq!(Shell::parse(None).unwrap(), Shell::Sh);
    assert_eq!(Shell::parse(Some("bash")).unwrap(), Shell::Bash);
    assert_eq!(Shell::parse(Some("zsh")).unwrap(), Shell::Zsh);
    assert_eq!(Shell::parse(Some("fish")).unwrap(), Shell::Fish);
    assert_eq!(Shell::parse(Some("pwsh")).unwrap(), Shell::Pwsh);
    assert!(Shell::parse(Some("cmd")).is_err());
    assert_eq!(Shell::Sh.content_type(), "text/x-shellscript; charset=utf-8");
    assert_eq!(Shell::Pwsh.content_type(), "text/plain; charset=utf-8");
  }

  #[test]
  fn env_includes_server_host_in_no_proxy() {
    let urls = urls_from_host("lan-router.local:4141", 4141, 4142).unwrap();
    let script = render_env_script(Shell::Sh, &urls, "abc123");
    assert!(script.contains("NO_PROXY='localhost,127.0.0.1,::1,lan-router.local'"));
  }

  #[test]
  fn env_renderers_quote_shell_values() {
    let urls = BootstrapUrls {
      api_base: "http://lan-router.local:4141".into(),
      proxy_base: "http://lan-router.local:4142".into(),
      ca_cert_url: "http://lan-router.local:4141/-/lan/ca.crt".into(),
      host_for_no_proxy: "lan-router.local".into(),
    };
    let fish = render_env_script(Shell::Fish, &urls, "abc123");
    assert!(fish.contains("set -gx OPENAI_BASE_URL 'http://lan-router.local:4141/v1'"));
    assert!(fish.contains("set -gx HTTPS_PROXY 'http://lan-router.local:4142'"));

    let pwsh = render_env_script(Shell::Pwsh, &urls, "abc123");
    assert!(pwsh.contains("$Env:OPENAI_BASE_URL = 'http://lan-router.local:4141/v1'"));
    assert!(pwsh.contains("$Env:HTTPS_PROXY = 'http://lan-router.local:4142'"));
  }

  #[test]
  fn quote_helpers_escape_single_quotes() {
    assert_eq!(sh_quote("a'b"), "'a'\\''b'");
    assert_eq!(pwsh_quote("a'b"), "'a''b'");
  }

  #[test]
  fn bootstrap_state_loads_public_ca_from_generated_ca() {
    let dir = tempfile::tempdir().unwrap();
    let ca = tokn_router::proxy::load_or_generate_ca(dir.path(), false).unwrap();
    let state = BootstrapState::new(&ca, 4141, 4142).unwrap();
    assert!(state.ca_cert_pem.contains("BEGIN CERTIFICATE"));
    assert_eq!(state.ca_fingerprint, ca.fingerprint_sha256());
  }

  #[tokio::test]
  async fn bootstrap_json_endpoint_uses_request_host() {
    let response = router(test_state())
      .oneshot(
        Request::builder()
          .uri(BOOTSTRAP_JSON_PATH)
          .header(HOST, "lan-router.local:4141")
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["api_url"], "http://lan-router.local:4141/v1");
    assert_eq!(json["proxy_url"], "http://lan-router.local:4142");
    assert_eq!(json["ca_cert_url"], "http://lan-router.local:4141/-/lan/ca.crt");
    assert_eq!(json["env_url"], "http://lan-router.local:4141/-/lan/env?shell=sh");
    assert_eq!(json["ca_sha256"], "abc123");
  }

  #[tokio::test]
  async fn env_endpoint_renders_requested_shell() {
    let response = router(test_state())
      .oneshot(
        Request::builder()
          .uri("/-/lan/env?shell=pwsh")
          .header(HOST, "lan-router.local:4141")
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      response.headers().get(CONTENT_TYPE).unwrap(),
      "text/plain; charset=utf-8"
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    assert!(body.contains("$Env:OPENAI_BASE_URL = 'http://lan-router.local:4141/v1'"));
    assert!(body.contains("Invoke-WebRequest"));
  }

  #[tokio::test]
  async fn bootstrap_endpoints_reject_bad_host_or_shell() {
    let missing_host = router(test_state())
      .oneshot(
        Request::builder()
          .uri(BOOTSTRAP_JSON_PATH)
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(missing_host.status(), StatusCode::BAD_REQUEST);

    let bad_shell = router(test_state())
      .oneshot(
        Request::builder()
          .uri("/-/lan/env?shell=cmd")
          .header(HOST, "lan-router.local:4141")
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(bad_shell.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn ca_endpoint_serves_only_public_certificate() {
    let response = router(test_state())
      .oneshot(
        Request::builder()
          .uri(CA_CERT_PATH)
          .header(HOST, "lan-router.local:4141")
          .body(axum::body::Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = std::str::from_utf8(&body).unwrap();
    assert!(body.contains("BEGIN CERTIFICATE"));
    assert!(!body.contains("PRIVATE KEY"));
  }
}
