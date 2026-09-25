use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::VecDeque, net::SocketAddr, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tauri::{Emitter, Manager};
use tokio::{
  io::AsyncReadExt,
  process::{Child, Command},
  sync::Mutex,
};

#[derive(Default)]
pub struct Gateway {
  child: Mutex<Option<Child>>,
  last_error: Mutex<Option<String>>,
  stderr_tail: Arc<Mutex<VecDeque<u8>>>,
}

#[derive(Clone, Serialize)]
pub struct GatewayStatus {
  pub state: String,
  pub ownership: String,
  pub address: String,
  pub pid: Option<u32>,
  pub last_error: Option<String>,
}

fn client() -> Result<reqwest::Client> {
  Ok(
    reqwest::Client::builder()
      .no_proxy()
      .redirect(reqwest::redirect::Policy::none())
      .timeout(Duration::from_secs(2))
      .build()?,
  )
}

async fn healthy(address: SocketAddr) -> bool {
  let Ok(client) = client() else {
    return false;
  };
  match client.get(format!("http://{address}/healthz")).send().await {
    Ok(response) if response.status().is_success() => response.text().await.is_ok_and(|body| body == "ok"),
    _ => false,
  }
}

impl Gateway {
  async fn exit_error(&self, exit: std::process::ExitStatus) -> String {
    let tail: Vec<u8> = self.stderr_tail.lock().await.iter().copied().collect();
    format!("Gateway exited ({exit}). {}", String::from_utf8_lossy(&tail))
  }

  pub async fn status(&self, address: SocketAddr) -> Result<GatewayStatus> {
    let mut child = self.child.lock().await;
    if let Some(process) = child.as_mut() {
      if let Some(exit) = process.try_wait()? {
        *self.last_error.lock().await = Some(self.exit_error(exit).await);
        *child = None;
      }
    }
    let running = healthy(address).await;
    Ok(GatewayStatus {
      state: if running {
        "running"
      } else if child.is_some() {
        "starting"
      } else {
        "stopped"
      }
      .into(),
      ownership: if child.is_some() {
        "managed"
      } else if running {
        "external"
      } else {
        "none"
      }
      .into(),
      address: address.to_string(),
      pid: child.as_ref().and_then(Child::id),
      last_error: self.last_error.lock().await.clone(),
    })
  }

  pub async fn start(&self, executable: PathBuf, config_path: PathBuf, address: SocketAddr) -> Result<()> {
    let mut child = self.child.lock().await;
    if let Some(process) = child.as_mut() {
      if process.try_wait()?.is_none() {
        bail!("An app-managed gateway is already running");
      }
      *child = None;
    }
    // Refuse any occupied port, including an unrelated service.
    let guard = tokio::net::TcpListener::bind(address)
      .await
      .context("Listener is already in use; the desktop app will not replace it")?;
    drop(guard);
    let mut process = Command::new(executable)
      .arg("--config")
      .arg(config_path)
      .arg("serve")
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .kill_on_drop(true)
      .spawn()
      .context("Starting the bundled gateway")?;
    self.stderr_tail.lock().await.clear();
    if let Some(mut stderr) = process.stderr.take() {
      let tail = self.stderr_tail.clone();
      tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        while let Ok(count) = stderr.read(&mut buffer).await {
          if count == 0 {
            break;
          }
          let mut tail = tail.lock().await;
          tail.extend(&buffer[..count]);
          while tail.len() > 8192 {
            tail.pop_front();
          }
        }
      });
    }
    *self.last_error.lock().await = None;
    *child = Some(process);
    // A successful spawn alone is not a successful start. Surface early exit
    // and readiness failure to the action that initiated startup.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
      if let Some(exit) = child.as_mut().context("Missing gateway child")?.try_wait()? {
        *child = None;
        let message = self.exit_error(exit).await;
        *self.last_error.lock().await = Some(message.clone());
        bail!(message);
      }
      if healthy(address).await {
        return Ok(());
      }
      if tokio::time::Instant::now() >= deadline {
        // Keep ownership so Stop and app exit can still terminate a gateway
        // that is alive but never became healthy.
        let message = "Gateway is still starting after 20 seconds. Check its logs or stop it before retrying.";
        *self.last_error.lock().await = Some(message.into());
        bail!(message);
      }
      tokio::time::sleep(Duration::from_millis(250)).await;
    }
  }

  pub async fn stop(&self) -> Result<()> {
    let mut child = self.child.lock().await;
    if let Some(process) = child.as_mut() {
      if process.try_wait()?.is_none() {
        let pid = process.id().context("Gateway has no process ID")?;
        // Keep the Child unreaped and the lock held until termination, so an
        // unrelated process can never inherit this PID before the signal.
        nix::sys::signal::kill(
          nix::unistd::Pid::from_raw(pid as i32),
          nix::sys::signal::Signal::SIGTERM,
        )?;
        match tokio::time::timeout(Duration::from_secs(40), process.wait()).await {
          Ok(result) => {
            result?;
          }
          Err(_) => process.kill().await?,
        }
      }
      *child = None;
    }
    Ok(())
  }
}

pub fn executable() -> Result<PathBuf> {
  let path = std::env::current_exe()?
    .parent()
    .context("Missing application directory")?
    .join("tokn-gateway");
  if !path.is_file() {
    bail!("Bundled gateway missing. Run pnpm prepare:gateway before building the desktop app.");
  }
  Ok(path)
}

pub async fn reload(address: SocketAddr) -> Result<String> {
  let response = client()?
    .post(format!("http://{address}/admin/config/reload"))
    .header("x-tokn-admin", "reload")
    .timeout(Duration::from_secs(30))
    .send()
    .await?;
  let status = response.status();
  let body = response.text().await?;
  if !status.is_success() {
    bail!("Saved on disk, but gateway reload returned {status}: {body}");
  }
  Ok("Gateway reloaded. Routing changes are now active.".into())
}

pub fn monitor(app: tauri::AppHandle) {
  tauri::async_runtime::spawn(async move {
    let mut previous = String::new();
    loop {
      // Config loading includes disk I/O and compilation; keep it off the
      // runtime worker that drives process supervision.
      if let Ok(Ok(address)) = tokio::task::spawn_blocking(crate::config::address).await {
        if let Ok(status) = app.state::<Gateway>().status(address).await {
          if let Ok(encoded) = serde_json::to_string(&status) {
            if encoded != previous {
              previous = encoded;
              let _ = app.emit("gateway-status", status);
            }
          }
        }
      }
      tokio::time::sleep(Duration::from_secs(2)).await;
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;
  #[tokio::test]
  #[ignore = "requires TOKN_DESKTOP_TEST_GATEWAY and loopback sockets"]
  async fn bundled_gateway_start_reload_stop() {
    let executable = std::env::var_os("TOKN_DESKTOP_TEST_GATEWAY").expect("gateway binary path");
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let source = format!(
      r#"schema_version = 2
[defaults]
[listeners.local]
kind = "llm_api"
bind = "{address}"
client_auth = "none"
[service.persistence]
enabled = false
usage_db_path = "{}/usage.db"
sessions_db_path = "{}/sessions.db"
requests_dir = "{}/requests"
[service.logging]
target = "stderr"
[service.models]
enabled = false
"#,
      dir.path().display(),
      dir.path().display(),
      dir.path().display()
    );
    std::fs::write(&config, source).unwrap();
    drop(listener);
    let gateway = Gateway::default();
    gateway.start(executable.into(), config.clone(), address).await.unwrap();
    assert_eq!(gateway.status(address).await.unwrap().ownership, "managed");
    let document = crate::config::read_at(&config).unwrap();
    crate::config::save_at(
      &config,
      &document.revision,
      "[defaults]\n[model_scores.\"gpt-*\"]\ncodex = 10\n",
    )
    .unwrap();
    let reload_result = reload(address).await;
    gateway.stop().await.unwrap();
    assert!(reload_result.is_ok(), "{reload_result:?}");
    assert!(!healthy(address).await);
    assert_eq!(gateway.status(address).await.unwrap().ownership, "none");
  }

  #[tokio::test]
  async fn refuses_an_external_listener_and_stop_does_not_close_it() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let gateway = Gateway::default();
    assert!(gateway
      .start("/nonexistent".into(), "/nonexistent".into(), address)
      .await
      .is_err());
    gateway.stop().await.unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_ok());
  }
  #[tokio::test]
  async fn stops_and_reaps_owned_process() {
    let process = Command::new("/bin/sleep").arg("60").kill_on_drop(true).spawn().unwrap();
    let gateway = Gateway {
      child: Mutex::new(Some(process)),
      ..Gateway::default()
    };
    gateway.stop().await.unwrap();
    assert!(gateway.child.lock().await.is_none());
  }
}
