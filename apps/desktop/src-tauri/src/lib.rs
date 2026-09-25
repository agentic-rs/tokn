mod accounts;
mod commands;
mod config;
mod data;
mod gateway;
mod inspect;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

pub fn run() {
  let exiting = AtomicBool::new(false);
  tauri::Builder::default()
    .manage(gateway::Gateway::default())
    .manage(accounts::Logins::default())
    .invoke_handler(tauri::generate_handler![
      commands::gateway_status,
      commands::start_gateway,
      commands::stop_gateway,
      commands::reload_gateway,
      commands::read_routing,
      commands::save_routing,
      accounts::list_accounts,
      accounts::account_providers,
      accounts::edit_account,
      accounts::probe_account,
      accounts::import_account,
      accounts::begin_account_login,
      accounts::complete_account_login,
      accounts::cancel_account_login,
      commands::read_usage,
      inspect::inspect_query
    ])
    .setup(|app| {
      gateway::monitor(app.handle().clone());
      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("Failed to build Tokn Desktop")
    .run(move |app, event| {
      if let tauri::RunEvent::ExitRequested { api, .. } = event {
        if !exiting.swap(true, Ordering::SeqCst) {
          api.prevent_exit();
          let app = app.clone();
          tauri::async_runtime::spawn(async move {
            if let Err(error) = app.state::<gateway::Gateway>().stop().await {
              eprintln!("Gateway shutdown: {error:#}");
            }
            app.exit(0);
          });
        }
      }
    });
}
