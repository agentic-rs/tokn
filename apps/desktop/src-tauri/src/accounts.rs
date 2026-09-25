use crate::{commands::blocking, config};
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
  time::Duration,
};
use tauri::Emitter;
use tokio::sync::{watch, Mutex};
use tokn_accounts::{
  context::ConfigContext,
  management::{self, AccountEdit, AccountProbe, AccountSummary, ProviderOptions},
};
use tokn_auth::{CredentialFlavor, CredentialSource, DeviceCodeHandle};

#[derive(Default)]
pub struct Logins(Mutex<HashMap<String, Arc<PendingLogin>>>);
struct PendingLogin {
  provider: tokn_accounts::context::ResolvedProviderAuth,
  client: reqwest::Client,
  account_id: String,
  handle: Mutex<Option<DeviceCodeHandle>>,
  cancel: watch::Sender<bool>,
  expires: tokio::time::Instant,
}

async fn context() -> Result<ConfigContext, String> {
  blocking(|| ConfigContext::load(Some(&config::path()?))).await
}

#[tauri::command]
pub async fn account_providers() -> Result<Vec<ProviderOptions>, String> {
  let context = context().await?;
  management::providers(&context).map_err(|_| "Unable to resolve configured account providers".into())
}

#[tauri::command]
pub async fn list_accounts() -> Result<Vec<AccountSummary>, String> {
  blocking(|| management::list(None))
    .await
    .map_err(|_| "Unable to read the credential store. Check its format and permissions.".into())
}

#[tauri::command]
pub async fn edit_account(edit: AccountEdit) -> Result<(), String> {
  blocking(move || management::edit(None, edit)).await.map_err(|_| {
    "Account was not changed. The store may be busy or the account may no longer exist; refresh and retry.".into()
  })
}

#[tauri::command]
pub async fn probe_account(id: String, force: bool) -> Result<AccountProbe, String> {
  let context = context().await?;
  // The auth service performs a locked transaction across token exchange and
  // disk writes. Keep that work on a worker rather than the UI/runtime thread.
  let runtime = tokio::runtime::Handle::current();
  blocking(move || runtime.block_on(management::probe(&context, None, &id, force)))
    .await
    .map_err(|_| "Unable to check this account. The store may be busy; retry shortly.".into())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportRequest {
  id: String,
  provider: String,
  source: String,
  value: String,
  flavor: String,
}

#[tauri::command]
pub async fn import_account(request: ImportRequest) -> Result<(), String> {
  let context = context().await?;
  let provider = context
    .resolve_provider(&request.provider)
    .map_err(|_| "Provider is not available in the current configuration")?;
  let generated = request.id.trim().is_empty();
  let id = request.id.trim().to_string();
  if !generated {
    blocking(move || management::validate_new_id(None, &id))
      .await
      .map_err(|_| "Account ID is invalid, already exists, or the credential store cannot be read")?;
  }
  let flavor = match request.flavor.as_str() {
    "api_key" => CredentialFlavor::ApiKey,
    "refresh_token" => CredentialFlavor::RefreshToken,
    _ => return Err("Unknown credential type".into()),
  };
  let source = match request.source.as_str() {
    "string" => CredentialSource::String {
      value: request.value,
      flavor,
    },
    "env" => CredentialSource::Env {
      env_var: request.value,
      flavor,
    },
    "file" => CredentialSource::File {
      path: request.value.into(),
      flavor,
    },
    other => {
      let key = provider
        .auth()
        .custom_credential_sources()
        .iter()
        .find(|&&key| key == other)
        .ok_or("Unsupported credential source")?;
      CredentialSource::Custom {
        key,
        value: (!request.value.trim().is_empty()).then_some(request.value),
      }
    }
  };
  let client = context
    .build_http_client(false)
    .map_err(|_| "Unable to build provider HTTP client")?;
  let runtime = tokio::runtime::Handle::current();
  blocking(move || runtime.block_on(async move {
    let account = tokio::time::timeout(Duration::from_secs(60), management::import_account(&client, &provider, request.id.trim().to_string(), source)).await??;
    if generated { management::insert_generated(None, account) } else { management::insert(None, account) }
  })).await.map_err(|_| "Import was not completed. Check the credential/source, connectivity, and account ID, then retry. Existing accounts were not replaced.".into())
}

#[derive(Serialize)]
pub struct LoginTicket {
  login_id: String,
  user_code: String,
  verification_uri: String,
  expires_in: u64,
}

#[tauri::command]
pub async fn begin_account_login(
  state: tauri::State<'_, Logins>,
  id: String,
  provider: String,
) -> Result<LoginTicket, String> {
  let id = id.trim().to_string();
  let account_id = id.clone();
  if !id.is_empty() {
    blocking(move || management::validate_new_id(None, &account_id))
      .await
      .map_err(|_| "Account ID is invalid, already exists, or the credential store cannot be read")?;
  }
  let context = context().await?;
  let provider = context
    .resolve_provider(&provider)
    .map_err(|_| "Provider is not available in the current configuration")?;
  if !provider.auth().supports_device_flow() {
    return Err("This provider does not support device login. Import a credential instead.".into());
  }
  let client = context
    .build_http_client(false)
    .map_err(|_| "Unable to build provider HTTP client")?;
  let handle = tokio::time::timeout(Duration::from_secs(20), provider.auth().request_device_code(&client))
    .await
    .map_err(|_| "Device code request timed out")?
    .map_err(|_| "Unable to request a device code from the provider")?;
  // Only provider-generated HTTPS verification links are exposed to the UI.
  if !handle.verification_uri.starts_with("https://") {
    return Err("Provider returned an invalid verification URL".into());
  }
  static NEXT: AtomicU64 = AtomicU64::new(1);
  let login_id = NEXT.fetch_add(1, Ordering::Relaxed).to_string();
  let ticket = LoginTicket {
    login_id: login_id.clone(),
    user_code: handle.user_code.clone(),
    verification_uri: handle.verification_uri.clone(),
    expires_in: handle.expires_in,
  };
  let (cancel, _) = watch::channel(false);
  let mut pending = state.0.lock().await;
  pending.retain(|_, login| login.expires > tokio::time::Instant::now());
  if pending.len() >= 4 {
    return Err("Cancel an existing login before starting another".into());
  }
  pending.insert(
    login_id,
    Arc::new(PendingLogin {
      provider,
      client,
      account_id: id,
      expires: tokio::time::Instant::now() + Duration::from_secs(handle.expires_in.min(900)),
      handle: Mutex::new(Some(handle)),
      cancel,
    }),
  );
  Ok(ticket)
}

#[derive(Clone, Serialize)]
struct LoginProgress {
  login_id: String,
  phase: &'static str,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn complete_account_login(
  app: tauri::AppHandle,
  state: tauri::State<'_, Logins>,
  login_id: String,
) -> Result<(), String> {
  let login = state
    .0
    .lock()
    .await
    .get(&login_id)
    .cloned()
    .ok_or("Login expired or was cancelled")?;
  let handle = login
    .handle
    .lock()
    .await
    .take()
    .ok_or("Login is already being completed")?;
  let _ = app.emit(
    "account-login-progress",
    LoginProgress {
      login_id: login_id.clone(),
      phase: "waiting",
    },
  );
  let result = async {
    let outcome = await_login(&login, handle).await?;
    let _ = app.emit(
      "account-login-progress",
      LoginProgress {
        login_id: login_id.clone(),
        phase: "saving",
      },
    );
    let account = management::device_account(&login.provider, login.account_id.clone(), outcome);
    let generated = login.account_id.is_empty();
    blocking(move || {
      if generated {
        management::insert_generated(None, account)
      } else {
        management::insert(None, account)
      }
    })
    .await
    .map_err(|_| "Sign-in succeeded but saving failed. Check that the account ID is unique and retry.".to_string())
  }
  .await;
  state.0.lock().await.remove(&login_id);
  let _ = app.emit(
    "account-login-progress",
    LoginProgress {
      login_id,
      phase: if result.is_ok() { "complete" } else { "ended" },
    },
  );
  result
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cancel_account_login(state: tauri::State<'_, Logins>, login_id: String) -> Result<(), String> {
  if let Some(login) = state.0.lock().await.remove(&login_id) {
    login.cancel.send_replace(true);
  }
  Ok(())
}

async fn await_login(login: &PendingLogin, handle: DeviceCodeHandle) -> Result<tokn_auth::DeviceFlowOutcome, String> {
  let mut cancelled = login.cancel.subscribe();
  if *cancelled.borrow() {
    return Err("Login cancelled".into());
  }
  if login.expires <= tokio::time::Instant::now() {
    return Err("Login expired; request a new code".into());
  }
  tokio::select! {
    _ = cancelled.changed() => Err("Login cancelled".into()),
    result = tokio::time::timeout_at(login.expires, login.provider.auth().poll_device_code(&login.client, handle)) => {
      result.map_err(|_| "Login expired; request a new code")?.map_err(|_| "Login failed or was declined; request a new code".into())
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn cancelled_or_expired_login_never_contacts_provider() {
    for cancelled in [false, true] {
      let (cancel, _) = watch::channel(cancelled);
      let login = PendingLogin {
        provider: tokn_accounts::context::ResolvedProviderAuth::legacy("github-copilot").unwrap(),
        client: reqwest::Client::new(),
        account_id: "test".into(),
        handle: Mutex::new(None),
        cancel,
        expires: tokio::time::Instant::now() - Duration::from_secs(1),
      };
      let handle = DeviceCodeHandle {
        device_code: "private-code".into(),
        user_code: "public-code".into(),
        verification_uri: "https://github.com/login/device".into(),
        expires_in: 1,
        interval: 1,
      };
      let error = await_login(&login, handle).await.unwrap_err();
      assert!(error.contains(if cancelled { "cancelled" } else { "expired" }));
      assert!(!error.contains("private-code"));
    }
  }
}
