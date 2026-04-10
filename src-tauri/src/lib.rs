//! Tauri command handlers and app entry point.
//!
//! Registers all invoke handlers, manages SessionManager state, and sets up tracing.

mod cmd;
mod config;
mod core;
mod error;
mod utils;

use std::sync::Arc;
use tauri::Manager;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::core::ssh::{PendingAuthManager, TunnelManager};
use crate::core::{RecordingManager, SessionManager};

fn init_tracing(log_dir: std::path::PathBuf) {
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("dragonfly")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .expect("failed to initialize rolling file appender");

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("dragonfly=info,warn"));

    let local_time = fmt::time::OffsetTime::local_rfc_3339().unwrap_or_else(|_| {
        fmt::time::OffsetTime::new(
            time::UtcOffset::UTC,
            time::format_description::well_known::Rfc3339,
        )
    });

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_target(true)
                .with_timer(local_time.clone()),
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stderr)
                .compact()
                .with_timer(local_time),
        )
        .init();

    tracing::info!("Dragonfly starting");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let session_manager = Arc::new(SessionManager::new());
    let tunnel_manager = Arc::new(TunnelManager::new());
    let recording_manager = Arc::new(RecordingManager::new());
    let pending_auth_manager = Arc::new(PendingAuthManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(session_manager.clone())
        .manage(tunnel_manager.clone())
        .manage(recording_manager.clone())
        .manage(pending_auth_manager.clone())
        .setup(move |app| {
            let home_dir = app
                .path()
                .home_dir()
                .map_err(|e: tauri::Error| e.to_string())?;

            let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;
            init_tracing(log_dir);

            session_manager.set_app_handle(app.handle().clone());

            let config_dir = home_dir.join(".dragonfly");
            let mgr = session_manager.clone();
            tauri::async_runtime::spawn(async move {
                mgr.init_history_store(config_dir).await;
            });

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Dragonfly")
                .on_tray_icon_event(|tray, event| match event {
                    tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } => {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    if let Ok(settings) = crate::config::load_app_settings(window.app_handle()) {
                        if settings.general.minimize_to_tray {
                            let _ = window.hide();
                            api.prevent_close();
                            return;
                        }
                    }
                    // Main window closing: close all child windows
                    for label in &["settings", "new-session", "quick-command"] {
                        if let Some(child) = window.app_handle().get_webview_window(label) {
                            let _ = child.close();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd::stats::get_system_fonts,
            cmd::session::create_ssh_session,
            cmd::session::create_local_session,
            cmd::session::write_to_session,
            cmd::session::resize_session,
            cmd::session::attach_session,
            cmd::session::close_session,
            cmd::session::list_sessions,
            cmd::session::add_command_history,
            cmd::session::get_command_history,
            cmd::session::fuzzy_search_history,
            cmd::session::fuzzy_search_commands,
            cmd::session::start_recording,
            cmd::session::stop_recording,
            cmd::session::is_recording,
            cmd::session::submit_otp_response,
            cmd::session::cancel_otp_request,
            cmd::sftp::get_home_dir,
            cmd::sftp::list_remote_dir,
            cmd::sftp::delete_remote_file,
            cmd::sftp::rename_remote_file,
            cmd::sftp::download_remote_file,
            cmd::sftp::upload_local_file,
            cmd::sftp::get_file_properties,
            cmd::sftp::create_remote_file,
            cmd::sftp::create_remote_dir,
            cmd::sftp::create_remote_symlink,
            cmd::sftp::chmod_remote_file,
            cmd::sftp::download_remote_directory,
            cmd::sftp::upload_local_directory,
            cmd::connection::get_saved_connections,
            cmd::connection::save_connection,
            cmd::connection::delete_connection,
            cmd::connection::reorder_items,
            cmd::connection::get_ssh_keys,
            cmd::connection::get_ssh_key_passphrase,
            cmd::connection::save_ssh_key,
            cmd::connection::delete_ssh_key,
            cmd::connection::get_groups,
            cmd::connection::save_group,
            cmd::connection::delete_group,
            cmd::connection::clear_all_connections,
            cmd::connection::get_quick_commands,
            cmd::connection::save_quick_commands,
            cmd::connection::get_saved_passwords,
            cmd::connection::get_saved_password_value,
            cmd::connection::save_password,
            cmd::connection::delete_password,
            cmd::settings::get_app_settings,
            cmd::settings::save_app_settings,
            cmd::settings::verify_lock_password,
            core::watcher::start_file_watch,
            core::watcher::stop_file_watch,
            core::translate::translate_text,
            core::importer::import_sessions,
            cmd::stats::get_remote_stats,
            cmd::stats::get_terminal_cwd,
            cmd::tunnel::get_tunnels,
            cmd::tunnel::save_tunnel,
            cmd::tunnel::delete_tunnel,
            cmd::tunnel::open_tunnel,
            cmd::tunnel::close_tunnel,
            cmd::proxy::get_proxies,
            cmd::proxy::save_proxy,
            cmd::proxy::delete_proxy,
            cmd::proxy::get_proxy_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
