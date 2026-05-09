use super::ca::DynamicResolver;
use super::passthrough::proxy_passthrough;
use super::{extract_proxy_auth_mode, rewrite_target, split_authority, HostPolicy, ProxyCa};
use crate::api::{error::ApiError, AppState};
use crate::api::routing::RouteResolver;
use anyhow::{Context, Result};
use axum::body::Body;
use axum::http::{Method, Request, Response, Uri};
use axum::response::IntoResponse;
use axum::Router;
use http::header::{HeaderValue, HOST};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use llm_config::RouteMode;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

const CONNECT_OK: &[u8] = b"HTTP/1.1 200 Connection Established\r\n\r\n";
const BAD_CONNECT: &[u8] = b"HTTP/1.1 405 Method Not Allowed\r\ncontent-length: 0\r\n\r\n";

pub(super) async fn handle_client(
  stream: TcpStream,
  peer: SocketAddr,
  state: Arc<AppState>,
  router: Router,
  ca: Arc<ProxyCa>,
  host_policy: HostPolicy,
  route_resolver: Arc<RouteResolver>,
  http: reqwest::Client,
) -> Result<()> {
  let mut reader = BufReader::new(stream);
  let mut request_line = String::new();
  if reader.read_line(&mut request_line).await? == 0 {
    return Ok(());
  }
  let request_line = request_line.trim_end_matches(['\r', '\n']);
  let mut parts = request_line.split_whitespace();
  let method = parts.next().unwrap_or_default();
  let authority = parts.next().unwrap_or_default();
  let _version = parts.next().unwrap_or_default();

  let mut proxy_route_mode: Option<String> = None;
  loop {
    let mut header_line = String::new();
    if reader.read_line(&mut header_line).await? == 0 {
      break;
    }
    if header_line == "\r\n" || header_line == "\n" {
      break;
    }
    if let Some(value) = header_line
      .strip_prefix("Proxy-Authorization:")
      .or_else(|| header_line.strip_prefix("proxy-authorization:"))
    {
      if let Some(mode) = extract_proxy_auth_mode(value.trim().trim_end_matches(['\r', '\n'])) {
        proxy_route_mode = Some(mode);
      }
    }
  }

  let mut stream = reader.into_inner();
  if method != Method::CONNECT.as_str() {
    stream.write_all(BAD_CONNECT).await?;
    return Ok(());
  }

  let (host, port) = split_authority(authority)?;
  let intercept = port == 443 && host_policy.should_intercept(&host);
  tracing::info!(%peer, host = %host, port, intercept, proxy_route_mode = ?proxy_route_mode, "proxy_connect");

  if intercept {
    stream.write_all(CONNECT_OK).await?;
    stream.flush().await?;
    intercept_tls(stream, &host, state, router, ca, route_resolver, http, proxy_route_mode).await
  } else {
    tunnel(stream, &host, port).await
  }
}

async fn tunnel(mut client: TcpStream, host: &str, port: u16) -> Result<()> {
  let mut upstream = TcpStream::connect((host, port))
    .await
    .with_context(|| format!("connect upstream {host}:{port}"))?;
  client.write_all(CONNECT_OK).await?;
  client.flush().await?;
  tokio::io::copy_bidirectional(&mut client, &mut upstream).await?;
  Ok(())
}

async fn intercept_tls(
  stream: TcpStream,
  host: &str,
  state: Arc<AppState>,
  router: Router,
  ca: Arc<ProxyCa>,
  route_resolver: Arc<RouteResolver>,
  http: reqwest::Client,
  proxy_route_mode: Option<String>,
) -> Result<()> {
  let resolver = Arc::new(DynamicResolver {
    ca,
    fallback_host: host.to_string(),
  });
  let tls = TlsAcceptor::from(Arc::new(
    rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_cert_resolver(resolver),
  ));
  let tls_stream = tls.accept(stream).await.context("TLS handshake failed")?;
  let mut http1_builder = http1::Builder::new();
  http1_builder.keep_alive(true).title_case_headers(true);

  let service = service_fn(move |req| {
    route_intercepted_request(
      state.clone(),
      router.clone(),
      route_resolver.clone(),
      http.clone(),
      req,
      proxy_route_mode.clone(),
    )
  });
  http1_builder
    .serve_connection(TokioIo::new(tls_stream), service)
    .await
    .context("serve intercepted HTTP connection")?;
  Ok(())
}

async fn route_intercepted_request(
  state: Arc<AppState>,
  router: Router,
  route_resolver: Arc<RouteResolver>,
  http: reqwest::Client,
  mut req: Request<hyper::body::Incoming>,
  proxy_route_mode: Option<String>,
) -> Result<Response<Body>, std::convert::Infallible> {
  if let Some(ref mode) = proxy_route_mode {
    if !req.headers().contains_key(RouteResolver::mode_header()) {
      if let Ok(val) = HeaderValue::from_str(mode) {
        req
          .headers_mut()
          .insert(http::header::HeaderName::from_static("x-route-mode"), val);
      }
    }
  }

  let host = req
    .headers()
    .get(HOST)
    .and_then(|v| v.to_str().ok())
    .map(|s| s.split(':').next().unwrap_or(s).to_string())
    .unwrap_or_default();
  let path = req.uri().path().to_string();
  let method = req.method().clone();

  let route_mode = req
    .headers()
    .get(RouteResolver::mode_header())
    .and_then(|v| v.to_str().ok());

  let resolved_mode = route_resolver.resolve_mode(route_mode);
  if matches!(resolved_mode, Ok(RouteMode::Passthrough)) {
    return Ok(
      proxy_passthrough(state.as_ref(), &http, &host, req)
        .await
        .unwrap_or_else(|err| ApiError::bad_gateway(err.to_string()).into_response()),
    );
  }
  if let Err(err) = resolved_mode {
    return Ok(ApiError::bad_request(err.to_string()).into_response());
  }

  let rewritten = if let Some(rewritten) = rewrite_target(&host, &path, &method) {
    rewritten
  } else {
    return Ok(ApiError::not_implemented(path, host).into_response());
  };

  let path_and_query = req.uri().path_and_query().map(|v| v.as_str()).unwrap_or(&path);
  let rewritten_path_and_query = path_and_query.replacen(&path, rewritten, 1);
  let uri = Uri::builder()
    .path_and_query(rewritten_path_and_query.as_str())
    .build()
    .unwrap_or_else(|_| Uri::from_static("/"));

  let (parts, body) = req.into_parts();
  let mut builder = Request::builder().method(method).uri(uri).version(parts.version);
  for (key, value) in &parts.headers {
    if key != HOST {
      builder = builder.header(key, value);
    }
  }
  builder = builder.header(
    HOST,
    HeaderValue::from_str(&host).unwrap_or_else(|_| HeaderValue::from_static("localhost")),
  );
  let body = Body::new(body);
  let request = builder.body(body).unwrap_or_else(|_| Request::new(Body::empty()));

  use tower::ServiceExt;
  let response = router
    .oneshot(request)
    .await
    .unwrap_or_else(|err| ApiError::bad_gateway(err.to_string()).into_response());
  Ok(response)
}
