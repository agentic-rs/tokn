use crate::config::{Account, AuthType};
use crate::provider::{github_copilot as gh, zai, ID_GITHUB_COPILOT, ID_ZAI_CODING_PLAN, ZAI_PROVIDERS};
use crate::util::secret::Secret;
use anyhow::{anyhow, Context, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialSource {
  Login,
  Gh,
  CopilotPlugin,
  RefreshToken { token: String },
  Env { env_var: String },
}

pub fn known_providers() -> Vec<&'static str> {
  // Single source of truth: the auth registry.
  crate::auth_registry::known_providers().to_vec()
}

pub fn validate_provider(provider: &str) -> Result<()> {
  if crate::auth_registry::provider_auth_for(provider).is_some() {
    return Ok(());
  }
  Err(anyhow!(
    "unknown provider '{provider}'. Try one of: {}, {}",
    ID_GITHUB_COPILOT,
    ZAI_PROVIDERS.join(" | ")
  ))
}

pub fn validate_provider_source(provider: &str, source: &CredentialSource) -> Result<()> {
  let is_zai = ZAI_PROVIDERS.contains(&provider);
  let is_copilot = provider == ID_GITHUB_COPILOT;
  match (source, is_copilot, is_zai) {
    (CredentialSource::Login, true, _) => Ok(()),
    (CredentialSource::Login, _, true) => Ok(()),
    (CredentialSource::Gh, true, _) => Ok(()),
    (CredentialSource::CopilotPlugin, true, _) => Ok(()),
    (CredentialSource::RefreshToken { .. }, true, _) => Ok(()),
    (CredentialSource::Env { .. }, _, true) => Ok(()),
    (CredentialSource::Env { .. }, true, _) => Err(anyhow!(
      "`from=env` is not supported for github-copilot (it needs a long-lived OAuth token, not an API key). Use `from=login|gh|copilot-plugin`."
    )),
    (CredentialSource::Gh, _, true)
    | (CredentialSource::CopilotPlugin, _, true)
    | (CredentialSource::RefreshToken { .. }, _, true) => Err(anyhow!(
      "provider '{}' is a static-API-key provider. Use `from=login` or `from=env`.",
      provider
    )),
    _ => Err(anyhow!("unsupported provider/source combination")),
  }
}

pub async fn resolve_account(
  client: &reqwest::Client,
  provider: &str,
  id_override: Option<String>,
  source: CredentialSource,
) -> Result<Account> {
  validate_provider(provider)?;
  validate_provider_source(provider, &source)?;

  match source {
    CredentialSource::Login => {
      if provider == ID_GITHUB_COPILOT {
        copilot_login(client, id_override).await
      } else {
        zai_login(client, provider, id_override).await
      }
    }
    CredentialSource::Gh => copilot_account(id_override.unwrap_or_else(|| "imported".into()), from_gh()?),
    CredentialSource::CopilotPlugin => {
      copilot_account(id_override.unwrap_or_else(|| "imported".into()), from_copilot_plugin()?)
    }
    CredentialSource::RefreshToken { token } => {
      copilot_account(id_override.unwrap_or_else(|| "imported".into()), token)
    }
    CredentialSource::Env { env_var } => zai_account(
      id_override.unwrap_or_else(|| {
        if provider == ID_ZAI_CODING_PLAN {
          "coding-plan".into()
        } else {
          provider.into()
        }
      }),
      provider,
      from_env(&env_var)?,
    ),
  }
}

fn copilot_account(id: String, token: String) -> Result<Account> {
  Ok(Account {
    id,
    provider: ID_GITHUB_COPILOT.into(),
    enabled: true,
    tier: llm_core::account::AccountTier::Active,
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: None,
    api_key_expires_at: None,
    access_token: None,
    access_token_expires_at: None,
    id_token: None,
    refresh_token: Some(Secret::new(token)),
    extra: Default::default(),
    refresh_url: Some(gh::TOKEN_EXCHANGE_URL.into()),
    last_refresh: None,
    settings: toml::Table::new(),
  })
}

fn zai_account(id: String, provider: &str, key: String) -> Result<Account> {
  Ok(Account {
    id,
    provider: provider.into(),
    enabled: true,
    tier: llm_core::account::AccountTier::Active,
    tags: Vec::new(),
    label: None,
    base_url: Some(zai::default_base_url(provider).into()),
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: Some(Secret::new(key)),
    api_key_expires_at: None,
    access_token: None,
    access_token_expires_at: None,
    id_token: None,
    refresh_token: None,
    extra: Default::default(),
    refresh_url: None,
    last_refresh: None,
    settings: toml::Table::new(),
  })
}

async fn copilot_login(client: &reqwest::Client, id_override: Option<String>) -> Result<Account> {
  println!("Requesting device code from GitHub…");
  let dc = gh::oauth::request_device_code(client).await?;
  println!();
  println!("  Open: {}", dc.verification_uri);
  println!("  Code: {}", dc.user_code);
  println!();
  println!("Waiting for authorization (expires in {}s)…", dc.expires_in);

  let gh_token = gh::oauth::poll_for_token(client, &dc).await?;
  println!("Got GitHub token. Verifying Copilot access…");

  let headers = llm_provider_copilot::config::CopilotHeaders::default();
  let resp = gh::token::exchange(client, &gh_token, &headers).await?;
  let id = match id_override {
    Some(s) => s,
    None => fetch_username(client, &gh_token)
      .await
      .unwrap_or_else(|_| "default".into()),
  };

  Ok(Account {
    id,
    provider: ID_GITHUB_COPILOT.into(),
    enabled: true,
    tier: llm_core::account::AccountTier::Active,
    tags: Vec::new(),
    label: None,
    base_url: None,
    headers: Default::default(),
    auth_type: Some(AuthType::Bearer),
    username: None,
    api_key: None,
    api_key_expires_at: None,
    access_token: Some(Secret::new(resp.token)),
    access_token_expires_at: Some(resp.expires_at),
    id_token: None,
    refresh_token: Some(Secret::new(gh_token)),
    extra: Default::default(),
    refresh_url: Some(gh::TOKEN_EXCHANGE_URL.into()),
    last_refresh: Some(time::OffsetDateTime::now_utc().unix_timestamp()),
    settings: toml::Table::new(),
  })
}

async fn zai_login(client: &reqwest::Client, provider_alias: &str, id_override: Option<String>) -> Result<Account> {
  println!("Z.ai uses a static API key. Create one at https://z.ai/manage-apikey/apikey-list");
  println!("(China endpoint: https://open.bigmodel.cn/usercenter/apikeys)");
  let key = rpassword::prompt_password("API key: ")
    .context("reading API key from stdin")?
    .trim()
    .to_string();
  if key.is_empty() {
    return Err(anyhow!("empty API key"));
  }

  println!("Verifying key against {} …", zai::DEFAULT_BASE_URL);
  verify_zai_key(client, &key).await?;
  println!("Key OK.");

  let id = id_override.unwrap_or_else(|| {
    if provider_alias == ID_ZAI_CODING_PLAN {
      "coding-plan".into()
    } else {
      provider_alias.into()
    }
  });
  zai_account(id, provider_alias, key)
}

async fn verify_zai_key(client: &reqwest::Client, key: &str) -> Result<()> {
  let url = format!("{}/models", zai::DEFAULT_BASE_URL.trim_end_matches('/'));
  let resp = client
    .get(&url)
    .header("authorization", format!("Bearer {key}"))
    .header("accept", "application/json")
    .send()
    .await
    .context("contacting Z.ai")?;
  let status = resp.status();
  if status.is_success() {
    return Ok(());
  }
  let body = resp.text().await.unwrap_or_default();
  Err(anyhow!(
    "Z.ai rejected the key (HTTP {status}). Body: {}",
    body.chars().take(200).collect::<String>()
  ))
}

fn from_env(name: &str) -> Result<String> {
  let v = std::env::var(name).with_context(|| format!("environment variable `{name}` is not set"))?;
  let v = v.trim().to_string();
  if v.is_empty() {
    return Err(anyhow!("environment variable `{name}` is empty"));
  }
  Ok(v)
}

fn from_gh() -> Result<String> {
  let out = std::process::Command::new("gh")
    .args(["auth", "token"])
    .output()
    .context("running `gh auth token` (is the GitHub CLI installed?)")?;
  if !out.status.success() {
    return Err(anyhow!(
      "`gh auth token` failed: {}",
      String::from_utf8_lossy(&out.stderr)
    ));
  }
  let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
  if token.is_empty() {
    return Err(anyhow!("`gh auth token` returned an empty token"));
  }
  Ok(token)
}

fn from_copilot_plugin() -> Result<String> {
  let home = directories::BaseDirs::new()
    .ok_or_else(|| anyhow!("cannot resolve home dir"))?
    .home_dir()
    .to_path_buf();
  let candidates = [
    home.join(".config/github-copilot/apps.json"),
    home.join(".config/github-copilot/hosts.json"),
  ];
  for path in &candidates {
    if !path.exists() {
      continue;
    }
    let raw = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if let Some(t) = scan_token(&v) {
      return Ok(t);
    }
  }
  Err(anyhow!("no Copilot plugin token found in ~/.config/github-copilot/"))
}

fn scan_token(v: &serde_json::Value) -> Option<String> {
  match v {
    serde_json::Value::Object(m) => {
      for (k, val) in m {
        if (k == "oauth_token" || k == "token") && val.as_str().filter(|s| !s.is_empty()).is_some() {
          return val.as_str().map(|s| s.to_string());
        }
        if let Some(found) = scan_token(val) {
          return Some(found);
        }
      }
      None
    }
    serde_json::Value::Array(a) => a.iter().find_map(scan_token),
    _ => None,
  }
}

async fn fetch_username(client: &reqwest::Client, gh_token: &str) -> Result<String> {
  #[derive(serde::Deserialize)]
  struct Me {
    login: String,
  }
  let me: Me = client
    .get("https://api.github.com/user")
    .header("authorization", format!("token {gh_token}"))
    .header("accept", "application/json")
    .header("user-agent", "llm-router")
    .send()
    .await?
    .error_for_status()?
    .json()
    .await?;
  Ok(me.login)
}

// ---------------------------------------------------------------------------
// Interactive helpers (used by `account add` and `config init`).
// ---------------------------------------------------------------------------

/// One-shot interactive flow: pick provider → pick credential source →
/// pick id → resolve account. Caller is responsible for upserting +
/// saving the resulting `Account`.
pub(crate) async fn interactive_add_account(
  client: &reqwest::Client,
  provider_override: Option<String>,
  id_override: Option<String>,
) -> Result<Account> {
  let provider = match provider_override {
    Some(p) => p,
    None => pick_provider()?,
  };
  validate_provider(&provider)?;
  let source = pick_source_interactive(&provider)?;
  let id = match id_override {
    Some(s) => Some(s),
    None => pick_account_id(&provider, &source)?,
  };
  resolve_account(client, &provider, id, source).await
}

pub(crate) fn pick_provider() -> Result<String> {
  let options = known_providers();
  let selected = inquire::Select::new("Pick account provider:", options)
    .with_starting_cursor(0)
    .prompt()
    .context("provider selection cancelled")?;
  Ok(selected.to_string())
}

pub(crate) fn pick_source_interactive(provider: &str) -> Result<CredentialSource> {
  let options: Vec<&str> = if provider == ID_GITHUB_COPILOT {
    vec!["login", "gh", "copilot-plugin", "refresh-token"]
  } else {
    vec!["login", "env"]
  };
  let picked = inquire::Select::new("Credential source:", options)
    .with_starting_cursor(0)
    .prompt()
    .context("credential source selection cancelled")?;
  match picked {
    "login" => Ok(CredentialSource::Login),
    "gh" => Ok(CredentialSource::Gh),
    "copilot-plugin" => Ok(CredentialSource::CopilotPlugin),
    "refresh-token" => {
      let token = inquire::Text::new("GitHub Copilot refresh token (leave empty to use env var):")
        .prompt()
        .context("refresh token prompt cancelled")?;
      let trimmed = token.trim().to_string();
      let token = if trimmed.is_empty() {
        let env_var = inquire::Text::new("Refresh token env var:")
          .with_initial_value("GITHUB_COPILOT_REFRESH_TOKEN")
          .prompt()
          .context("refresh token env var prompt cancelled")?;
        let value = std::env::var(&env_var).map_err(|_| anyhow!("environment variable `{env_var}` is not set"))?;
        let v = value.trim().to_string();
        if v.is_empty() {
          return Err(anyhow!("environment variable `{env_var}` is empty"));
        }
        v
      } else {
        trimmed
      };
      Ok(CredentialSource::RefreshToken { token })
    }
    "env" => {
      let env_var = inquire::Text::new("Environment variable containing API key:")
        .with_initial_value("ZAI_API_KEY")
        .prompt()
        .context("env var prompt cancelled")?;
      Ok(CredentialSource::Env { env_var })
    }
    _ => Err(anyhow!("unsupported credential source")),
  }
}

pub(crate) fn pick_account_id(provider: &str, source: &CredentialSource) -> Result<Option<String>> {
  let default_id = if provider == ID_GITHUB_COPILOT {
    "imported"
  } else {
    provider
  };
  let prompt = match source {
    CredentialSource::Login => "Account id (leave empty for auto):",
    _ => "Account id:",
  };
  let text = inquire::Text::new(prompt)
    .with_initial_value(default_id)
    .prompt()
    .context("account id prompt cancelled")?;
  let trimmed = text.trim().to_string();
  if trimmed.is_empty() {
    return Ok(None);
  }
  Ok(Some(trimmed))
}
