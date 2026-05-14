use super::OutputFormat;
use crate::cli::config_cmd::RouteModeArg;
use crate::config::Config;
use crate::provider::Endpoint;
use anyhow::{anyhow, Result};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue};
use axum::response::Response;
use clap::Args;
use llm_config::RouteMode;
use llm_router::api::AppState;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum EndpointArg {
  ChatCompletions,
  Responses,
  Messages,
}

impl From<EndpointArg> for Endpoint {
  fn from(val: EndpointArg) -> Self {
    match val {
      EndpointArg::ChatCompletions => Endpoint::ChatCompletions,
      EndpointArg::Responses => Endpoint::Responses,
      EndpointArg::Messages => Endpoint::Messages,
    }
  }
}

#[derive(Args, Debug)]
pub struct SendArgs {
  /// Route mode (defaults to the serve route-mode from config).
  #[arg(long, value_enum)]
  pub route: Option<RouteModeArg>,

  /// Constrain account selection to this provider.
  #[arg(long)]
  pub provider: Option<String>,

  /// Pick a specific account by id (requires --provider).
  #[arg(long, requires = "provider")]
  pub account: Option<String>,

  /// Model to use for the smoke request.
  #[arg(long)]
  pub model: Option<String>,

  /// API endpoint to test.
  #[arg(long, value_enum, default_value_t = EndpointArg::ChatCompletions)]
  pub endpoint: EndpointArg,

  /// Request streaming SSE response.
  #[arg(long)]
  pub stream: bool,

  /// Build and print outbound headers/body without sending the request.
  #[arg(long)]
  pub dry_run: bool,

  /// Output format.
  #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
  pub format: OutputFormat,

  /// Read the raw JSON request body from a file (or `-` for stdin) instead of
  /// the auto-built body. When set, `message` is optional and `--model`
  /// defaults to whatever the body declares.
  #[arg(long)]
  pub body_file: Option<PathBuf>,

  /// Inject a custom inbound header (`name=value`). Repeatable. Last wins per
  /// header name. Useful for replaying captured requests that depend on
  /// `accept`, `originator`, etc.
  #[arg(long = "header", value_parser = parse_header_kv, num_args = 0..)]
  pub headers: Vec<(String, String)>,

  /// Message to send. Optional when `--body-file` is provided.
  pub message: Option<String>,
}

fn parse_header_kv(raw: &str) -> std::result::Result<(String, String), String> {
  let (k, v) = raw
    .split_once('=')
    .or_else(|| raw.split_once(':').map(|(a, b)| (a, b.trim_start())))
    .ok_or_else(|| format!("expected `name=value` or `name: value`, got `{raw}`"))?;
  let k = k.trim().to_string();
  let v = v.trim().to_string();
  if k.is_empty() {
    return Err("header name must not be empty".into());
  }
  Ok((k, v))
}

pub async fn run(cfg_path: Option<PathBuf>, args: SendArgs) -> Result<()> {
  let (mut cfg, resolved_cfg_path) = Config::load(cfg_path.as_deref())?;
  let mut accounts = crate::server_runtime::load_accounts(Some(&resolved_cfg_path))?;

  let route_mode = args.route.map(RouteMode::from).unwrap_or(cfg.server.route_mode);
  cfg.server.route_mode = route_mode;

  if route_mode == RouteMode::Passthrough {
    anyhow::bail!("passthrough mode requires the proxy; use a different --route mode");
  }

  filter_accounts(&mut accounts, args.provider.as_deref(), args.account.as_deref())?;

  let (events, receiver, handlers, archive_runtime) = crate::server_runtime::build_event_bus(&cfg)?;
  let _event_thread = llm_core::event::spawn_event_loop(receiver, handlers);

  let state = crate::server_runtime::build_state(&cfg, &accounts, events.clone())?;

  let custom_body: Option<serde_json::Value> = match args.body_file.as_deref() {
    Some(path) => Some(load_body_file(path)?),
    None => None,
  };

  let model = match (&args.model, custom_body.as_ref()) {
    (Some(m), _) => m.clone(),
    (None, Some(body)) => body
      .get("model")
      .and_then(|v| v.as_str())
      .map(str::to_string)
      .ok_or_else(|| anyhow!("--body-file does not contain a `model` field; pass --model"))?,
    (None, None) => pick_default_model(&state, args.provider.as_deref())?,
  };

  let endpoint: Endpoint = args.endpoint.into();

  if custom_body.is_none() && args.message.is_none() {
    anyhow::bail!("missing message: pass either a positional message or --body-file");
  }

  let route = state.route.resolve(&model, None).map_err(|e| anyhow!("{e}"))?;

  if args.format == OutputFormat::Text {
    println!("provider: {}", args.provider.as_deref().unwrap_or("(any)"));
    println!("account:  {}", args.account.as_deref().unwrap_or("(any)"));
    println!("model:    {} -> {}", route.requested_model, route.upstream_model);
    println!("endpoint: {}", endpoint);
    println!("route:    {}", route_mode_name(route_mode));
    println!("stream:   {}", args.stream);
    if args.body_file.is_some() {
      println!("body:     {}", args.body_file.as_ref().unwrap().display());
    }
    println!();
  }

  let body_value = match custom_body {
    Some(mut v) => {
      if let Some(obj) = v.as_object_mut() {
        if args.model.is_some() {
          obj.insert("model".into(), serde_json::Value::String(model.clone()));
        }
        if args.stream {
          obj.insert("stream".into(), serde_json::Value::Bool(true));
        }
      }
      v
    }
    None => build_request_body(
      endpoint,
      &route.upstream_model,
      args.message.as_deref().unwrap_or(""),
      args.stream,
    ),
  };
  let body_bytes = Bytes::from(serde_json::to_vec(&body_value)?);
  let headers = build_headers(&args.headers)?;

  if args.dry_run {
    let dry_endpoint = match endpoint {
      Endpoint::ChatCompletions => llm_router::pipeline::DryRunEndpoint::ChatCompletions,
      Endpoint::Responses => llm_router::pipeline::DryRunEndpoint::Responses,
      Endpoint::Messages => llm_router::pipeline::DryRunEndpoint::Messages,
    };
    let dry = llm_router::pipeline::dry_run_request(
      &state,
      dry_endpoint,
      headers.clone(),
      body_value.clone(),
      body_bytes.clone(),
      body_bytes.clone(),
      None,
    )
    .map_err(|e| anyhow!("dry-run failed: {e}"))?;
    print_dry_run(&dry, args.format)?;
    events.shutdown().await;
    if let Some(archive_runtime) = archive_runtime {
      archive_runtime.shutdown().await;
    }
    return Ok(());
  }

  let resp_result: Result<Response> = match endpoint {
    Endpoint::ChatCompletions => {
      llm_router::api::endpoints::chat_completions(State(state.clone()), headers, body_bytes)
        .await
        .map_err(|e| anyhow!("{e}"))
    }
    Endpoint::Responses => llm_router::api::endpoints::responses(State(state.clone()), headers, body_bytes)
      .await
      .map_err(|e| anyhow!("{e}")),
    Endpoint::Messages => llm_router::api::endpoints::messages(State(state.clone()), headers, body_bytes)
      .await
      .map_err(|e| anyhow!("{e}")),
  };

  let resp = resp_result?;
  let status = resp.status();
  let resp_body = axum::body::to_bytes(resp.into_body(), usize::MAX)
    .await
    .map_err(|e| anyhow!("read response body: {e}"))?;

  if let Some(archive_runtime) = archive_runtime {
    archive_runtime.shutdown().await;
  }
  events.shutdown().await;

  if args.format == OutputFormat::Json {
    print_json_response(status, &resp_body, args.stream)?;
  } else {
    print_text_response(status, &resp_body, args.stream)?;
  }

  if !status.is_success() {
    std::process::exit(1);
  }
  Ok(())
}

pub(super) fn filter_accounts(
  accounts: &mut Vec<llm_core::account::AccountConfig>,
  provider: Option<&str>,
  account: Option<&str>,
) -> Result<()> {
  if provider.is_none() && account.is_none() {
    return Ok(());
  }
  let before = accounts.len();
  accounts.retain(|a| {
    if let Some(p) = provider {
      if a.provider != p {
        return false;
      }
    }
    if let Some(id) = account {
      if a.id != id {
        return false;
      }
    }
    true
  });
  if accounts.is_empty() {
    anyhow::bail!(
      "no accounts match the requested filters (provider={:?}, account={:?}); had {before} configured",
      provider,
      account
    );
  }
  Ok(())
}

fn build_request_body(endpoint: Endpoint, model: &str, message: &str, stream: bool) -> serde_json::Value {
  match endpoint {
    Endpoint::ChatCompletions | Endpoint::Messages => serde_json::json!({
      "model": model,
      "stream": stream,
      "messages": [{"role": "user", "content": message}],
    }),
    Endpoint::Responses => serde_json::json!({
      "model": model,
      "stream": stream,
      "input": message,
    }),
  }
}

fn build_headers(overrides: &[(String, String)]) -> Result<HeaderMap> {
  let mut h = HeaderMap::new();
  h.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static("application/json"),
  );
  h.insert(
    HeaderName::from_static("x-request-id"),
    HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).unwrap(),
  );
  for (k, v) in overrides {
    let name = HeaderName::from_bytes(k.to_ascii_lowercase().as_bytes())
      .map_err(|e| anyhow!("invalid header name `{k}`: {e}"))?;
    let value = HeaderValue::from_str(v).map_err(|e| anyhow!("invalid value for header `{k}`: {e}"))?;
    h.insert(name, value);
  }
  Ok(h)
}

fn load_body_file(path: &std::path::Path) -> Result<serde_json::Value> {
  use std::io::Read;
  let raw = if path == std::path::Path::new("-") {
    let mut buf = String::new();
    std::io::stdin()
      .read_to_string(&mut buf)
      .map_err(|e| anyhow!("read stdin: {e}"))?;
    buf
  } else {
    std::fs::read_to_string(path).map_err(|e| anyhow!("read {}: {e}", path.display()))?
  };
  let body_str = match raw.split_once("\nBody:\n") {
    Some((_, after)) => after.trim_start(),
    None => raw.trim_start(),
  };
  serde_json::from_str(body_str).map_err(|e| anyhow!("parse body file as JSON: {e}"))
}

fn pick_default_model(state: &AppState, provider_filter: Option<&str>) -> Result<String> {
  for acct in state.pool.all() {
    if let Some(p) = provider_filter {
      if acct.provider.info().id != p {
        continue;
      }
    }
    if let Some(m) = acct.provider.info().default_models.first() {
      return Ok(m.id.clone());
    }
  }
  match provider_filter {
    Some(p) => anyhow::bail!("no models available for provider '{}'; pass --model", p),
    None => anyhow::bail!("no models available; pass --model explicitly"),
  }
}

fn route_mode_name(mode: RouteMode) -> &'static str {
  match mode {
    RouteMode::Passthrough => "passthrough",
    RouteMode::Exact => "exact",
    RouteMode::Route => "route",
    RouteMode::Fuzzy => "fuzzy",
  }
}

fn print_text_response(status: reqwest::StatusCode, body: &[u8], stream: bool) -> Result<()> {
  println!("status: {}", status.as_u16());
  if stream {
    let text = String::from_utf8_lossy(body);
    println!("{text}");
    return Ok(());
  }

  let text = String::from_utf8_lossy(body);
  let json: serde_json::Value = match serde_json::from_slice(body) {
    Ok(v) => v,
    Err(_) => {
      println!("{text}");
      return Ok(());
    }
  };

  if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
    for (i, choice) in choices.iter().enumerate() {
      let content = choice
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("(no content)");
      println!("--- choice {} ---", i);
      println!("{content}");
    }
  } else if let Some(output) = json.get("output").and_then(|o| o.as_array()) {
    for item in output {
      if let Some(content) = item.get("content") {
        if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
          println!("{text}");
        } else if let Some(arr) = content.as_array() {
          for part in arr {
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
              println!("{text}");
            }
          }
        }
      }
    }
  } else if let Some(content) = json.get("content").and_then(|c| c.as_array()) {
    for block in content {
      if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
        println!("{text}");
      }
    }
  } else {
    println!("{text}");
  }

  Ok(())
}

fn print_json_response(status: reqwest::StatusCode, body: &[u8], stream: bool) -> Result<()> {
  if stream {
    let wrapper = serde_json::json!({
      "status": status.as_u16(),
      "stream": true,
      "body": String::from_utf8_lossy(body),
    });
    println!("{}", serde_json::to_string_pretty(&wrapper)?);
    return Ok(());
  }
  let json: serde_json::Value = serde_json::from_slice(body).unwrap_or_else(|_| {
    serde_json::json!({
      "status": status.as_u16(),
      "body": String::from_utf8_lossy(body),
    })
  });
  let output = serde_json::to_string_pretty(&json)?;
  println!("{output}");
  Ok(())
}

fn print_dry_run(dry: &llm_router::pipeline::DryRunOutput, format: OutputFormat) -> Result<()> {
  let headers = headers_json(&dry.headers);
  let body_json: serde_json::Value = serde_json::from_slice(dry.body.as_ref())
    .unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(dry.body.as_ref()).into_owned()));
  match format {
    OutputFormat::Json => {
      println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
          "dry_run": true,
          "account": dry.account_id,
          "provider": dry.provider_id,
          "model": dry.model,
          "endpoint": dry.endpoint.to_string(),
          "headers": headers,
          "body": body_json,
        }))?
      );
    }
    OutputFormat::Text => {
      println!("dry-run: true");
      println!("account: {}", dry.account_id);
      println!("provider: {}", dry.provider_id);
      println!("model: {}", dry.model);
      println!("endpoint: {}", dry.endpoint);
      println!("headers:");
      for (name, value) in headers.as_object().into_iter().flat_map(|m| m.iter()) {
        println!("  {name}: {}", value.as_str().unwrap_or(""));
      }
      println!("body:");
      println!("{}", serde_json::to_string_pretty(&body_json)?);
    }
  }
  Ok(())
}

fn headers_json(headers: &HeaderMap) -> serde_json::Value {
  let mut map = serde_json::Map::new();
  for (name, value) in headers {
    let value = value.to_str().unwrap_or("<non-utf8>");
    map.insert(
      name.as_str().to_string(),
      serde_json::Value::String(redact_header(name.as_str(), value)),
    );
  }
  serde_json::Value::Object(map)
}

fn redact_header(name: &str, value: &str) -> String {
  match name.to_ascii_lowercase().as_str() {
    "authorization" | "proxy-authorization" | "cookie" | "set-cookie" | "x-api-key" => "<redacted>".into(),
    _ => value.to_string(),
  }
}
