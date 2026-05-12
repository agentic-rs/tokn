use crate::cli::config_cmd::RouteModeArg;
use crate::config::Config;
use crate::provider::{Endpoint, RequestCtx};
use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use llm_config::RouteMode;
use llm_core::event::EventBus;
use std::path::PathBuf;
use std::sync::Arc;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
  Text,
  Json,
}

#[derive(Args, Debug)]
pub struct SmokeArgs {
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

  /// Output format.
  #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
  pub format: OutputFormat,

  /// Message to send.
  pub message: String,
}

pub async fn run(cfg_path: Option<PathBuf>, args: SmokeArgs) -> Result<()> {
  let (mut cfg, _) = Config::load(cfg_path.as_deref())?;
  let route_mode = args
    .route
    .map(RouteMode::from)
    .unwrap_or(cfg.server.route_mode);
  cfg.server.route_mode = route_mode;

  let state = llm_router::api::build_state(&cfg, Arc::new(EventBus::noop()))?;

  let model = match &args.model {
    Some(m) => m.clone(),
    None => pick_default_model(&state, args.provider.as_deref())?,
  };

  let endpoint: Endpoint = args.endpoint.into();
  let route = state
    .route
    .resolve(&model, None)
    .map_err(|e| anyhow!("{e}"))?;

  if route_mode == RouteMode::Passthrough {
    anyhow::bail!("passthrough mode requires the proxy; use a different --route mode");
  }

  let (acct, upstream_endpoint) = match (&args.provider, &args.account) {
    (None, None) => acquire_from_pool(&state, &route, endpoint, route_mode)?,
    (Some(provider_id), None) => acquire_from_provider(&state, provider_id, &route, endpoint, route_mode)?,
    (Some(provider_id), Some(account_id)) => {
      acquire_specific(&state, provider_id, account_id, &route, endpoint, route_mode)?
    }
    (None, Some(_)) => unreachable!("clap requires= prevents this"),
  };

  let provider = &acct.provider;
  let account_cfg = acct.config.load();

  if args.format == OutputFormat::Text {
    println!("account:  {}", account_cfg.id);
    println!("provider: {}", provider.info().id);
    println!("model:    {} -> {}", route.requested_model, route.upstream_model);
    println!("endpoint: {} -> {}", endpoint, upstream_endpoint);
    println!("route:    {}", route_mode_name(route_mode));
    println!();
  }

  let mut body = serde_json::json!({
    "model": route.upstream_model,
    "stream": false,
    "messages": [{"role": "user", "content": args.message}],
  });

  if upstream_endpoint != endpoint {
    body = llm_router::convert::convert_request(endpoint, upstream_endpoint, &body)
      .map_err(|e| anyhow!("request conversion failed: {e}"))?;
  }

  if let Some(transformer) = provider.input_transformer() {
    let meta = llm_core::pipeline::RequestMeta {
      endpoint,
      upstream_endpoint,
      model: route.requested_model.clone(),
      upstream_model: route.upstream_model.clone(),
      stream: false,
      session_id: None,
      request_id: None,
      attempt: 0,
      project_id: None,
      initiator: "smoke".into(),
      header_initiator: None,
      behave_as: None,
      inbound_headers: Default::default(),
    };
    body = transformer.transform_input(&meta, body)?;
  }

  let ctx = RequestCtx {
    endpoint: upstream_endpoint,
    http: &state.http,
    body: &body,
    body_bytes: None,
    content_encoding: None,
    stream: false,
    initiator: "smoke",
    inbound_headers: &Default::default(),
    behave_as: None,
    outbound: None,
  };

  let resp = match upstream_endpoint {
    Endpoint::ChatCompletions => provider.chat(ctx).await?,
    Endpoint::Responses => provider.responses(ctx).await?,
    Endpoint::Messages => provider.messages(ctx).await?,
  };

  let status = resp.status();
  let body_bytes = resp.bytes().await?;

  if args.format == OutputFormat::Json {
    print_json_response(status, &body_bytes)?;
  } else {
    print_text_response(status, &body_bytes)?;
  }

  if status.is_success() {
    acct.mark_success();
  } else {
    acct.mark_failure(state.pool.cooldown_base());
    std::process::exit(1);
  }

  Ok(())
}

fn acquire_from_pool(
  state: &llm_router::api::AppState,
  route: &llm_router::api::routing::RouteResolution,
  endpoint: Endpoint,
  route_mode: RouteMode,
) -> Result<(Arc<llm_router::accounts::AccountHandle>, Endpoint)> {
  match state.pool.acquire_for_route(None, route, endpoint) {
    llm_router::accounts::EndpointAcquire::Account { acct, endpoint } => Ok((acct, endpoint)),
    llm_router::accounts::EndpointAcquire::SessionExpired => {
      Err(anyhow!("session expired (unexpected for smoke test)"))
    }
    llm_router::accounts::EndpointAcquire::None => Err(anyhow!(
      "no account supports model '{}' on endpoint {} with route mode {}",
      route.requested_model,
      endpoint,
      route_mode_name(route_mode),
    )),
  }
}

fn acquire_from_provider(
  state: &llm_router::api::AppState,
  provider_id: &str,
  route: &llm_router::api::routing::RouteResolution,
  endpoint: Endpoint,
  route_mode: RouteMode,
) -> Result<(Arc<llm_router::accounts::AccountHandle>, Endpoint)> {
  let upstream_model = &route.upstream_model;
  for acct in state.pool.all() {
    if acct.provider.info().id != provider_id {
      continue;
    }
    if let Some(ep) = matching_endpoint(&acct, upstream_model, endpoint) {
      return Ok((acct.clone(), ep));
    }
  }
  Err(anyhow!(
    "no account for provider '{}' supports model '{}' on endpoint {} with route mode {}",
    provider_id,
    route.requested_model,
    endpoint,
    route_mode_name(route_mode),
  ))
}

fn acquire_specific(
  state: &llm_router::api::AppState,
  provider_id: &str,
  account_id: &str,
  route: &llm_router::api::routing::RouteResolution,
  endpoint: Endpoint,
  route_mode: RouteMode,
) -> Result<(Arc<llm_router::accounts::AccountHandle>, Endpoint)> {
  let upstream_model = &route.upstream_model;
  for acct in state.pool.all() {
    if acct.config.load().id != account_id {
      continue;
    }
    if acct.provider.info().id != provider_id {
      anyhow::bail!(
        "account '{}' belongs to provider '{}', not '{}'",
        account_id,
        acct.provider.info().id,
        provider_id,
      );
    }
    if let Some(ep) = matching_endpoint(&acct, upstream_model, endpoint) {
      return Ok((acct.clone(), ep));
    }
    anyhow::bail!(
      "account '{}' does not support model '{}' on endpoint {}",
      account_id,
      route.requested_model,
      endpoint,
    );
  }
  Err(anyhow!(
    "no account with id '{}' (provider '{}') found for route mode {}",
    account_id,
    provider_id,
    route_mode_name(route_mode),
  ))
}

fn matching_endpoint(
  acct: &llm_router::accounts::AccountHandle,
  model: &str,
  requested: Endpoint,
) -> Option<Endpoint> {
  [requested, Endpoint::ChatCompletions, Endpoint::Responses, Endpoint::Messages]
    .into_iter()
    .find(|ep| acct.provider.model_info(model).is_some() && acct.provider.supports(model, *ep))
}

fn pick_default_model(state: &llm_router::api::AppState, provider_filter: Option<&str>) -> Result<String> {
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

fn print_text_response(status: reqwest::StatusCode, body: &[u8]) -> Result<()> {
  println!("status: {}", status.as_u16());
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

fn print_json_response(status: reqwest::StatusCode, body: &[u8]) -> Result<()> {
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
