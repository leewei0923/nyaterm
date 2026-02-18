//! Remote file listing via SSH exec (ls -la) commands.
//!
//! Opens temporary connections using existing session credentials; no SFTP subsystem.

use crate::error::{AppError, AppResult};
use crate::session::SessionManager;
use crate::ssh::{SshAuth, SshConfig, SshHandler};
use russh::client;
use russh::ChannelMsg;
use serde::Serialize;
use std::sync::Arc;

/// Parsed entry from `ls -la` output for the file explorer.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub permissions: String,
}

async fn open_tmp_connection(
    app: &tauri::AppHandle,
    manager: &SessionManager,
    session_id: &str,
) -> AppResult<client::Handle<SshHandler>> {
    let config = {
        let sessions = manager.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| AppError::SessionNotFound(format!("Session '{}' not found", session_id)))?;

        session
            .ssh_config
            .as_ref()
            .ok_or_else(|| AppError::Config("Not an SSH session".to_string()))?
            .clone()
            .downcast::<SshConfig>()
            .map_err(|_| AppError::Config("Failed to get SSH config".to_string()))?
    };

    let ssh_config = Arc::new(client::Config::default());
    let mut handle = client::connect(
        ssh_config,
        (config.host.as_str(), config.port),
        SshHandler::new(app.clone(), config.host.clone(), config.port),
    )
    .await
    .map_err(|e| AppError::Auth(format!("SSH connection failed: {}", e)))?;

    match &config.auth {
        SshAuth::Password { password } => {
            let ok = handle
                .authenticate_password(&config.username, password)
                .await
                .map_err(|e| AppError::Auth(format!("Auth failed: {}", e)))?;
            if !ok {
                return Err(AppError::Auth(
                    "Auth failed: invalid credentials".to_string(),
                ));
            }
        }
        SshAuth::Key {
            key_data,
            passphrase,
        } => {
            let key = russh_keys::decode_secret_key(key_data, passphrase.as_deref())?;
            let ok = handle
                .authenticate_publickey(&config.username, Arc::new(key))
                .await
                .map_err(|e| AppError::Auth(format!("Key auth failed: {}", e)))?;
            if !ok {
                return Err(AppError::Auth("Auth failed: key rejected".to_string()));
            }
        }
    }

    Ok(handle)
}

async fn exec_remote_command(
    handle: &client::Handle<SshHandler>,
    cmd: &str,
) -> AppResult<String> {
    let channel = handle
        .channel_open_session()
        .await
        .map_err(|e| AppError::Channel(format!("Failed to open channel: {}", e)))?;

    channel
        .exec(true, cmd.as_bytes())
        .await
        .map_err(|e| AppError::Channel(format!("Exec failed: {}", e)))?;

    let mut output = String::new();
    let mut channel = channel;
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { ref data } => {
                output.push_str(&String::from_utf8_lossy(data));
            }
            ChannelMsg::Eof => break,
            _ => {}
        }
    }

    Ok(output)
}

/// Resolves $HOME on the remote host for default file explorer path.
pub async fn get_home_dir(
    app: tauri::AppHandle,
    manager: Arc<SessionManager>,
    session_id: &str,
) -> AppResult<String> {
    let handle = open_tmp_connection(&app, &manager, session_id).await?;
    let output = exec_remote_command(&handle, "echo $HOME").await?;
    let _ = handle
        .disconnect(russh::Disconnect::ByApplication, "", "English")
        .await;

    let home = output.trim().to_string();
    if home.is_empty() {
        Err(AppError::Config(
            "Failed to determine home directory".to_string(),
        ))
    } else {
        Ok(home)
    }
}

/// Lists a remote directory via `ls -la`; parses output into `FileEntry`s.
pub async fn list_remote_dir(
    app: tauri::AppHandle,
    manager: Arc<SessionManager>,
    session_id: &str,
    path: &str,
) -> AppResult<Vec<FileEntry>> {
    let handle = open_tmp_connection(&app, &manager, session_id).await?;

    let escaped = build_path_for_shell(path);
    let cmd = format!(
        "ls -la --color=never {} 2>/dev/null || ls -la {} 2>/dev/null",
        escaped, escaped
    );
    let output = exec_remote_command(&handle, &cmd).await?;
    let _ = handle
        .disconnect(russh::Disconnect::ByApplication, "", "English")
        .await;

    let entries = parse_ls_output(&output);
    Ok(entries)
}

/// Build a shell-safe path string.
/// `~` and `~/...` are left unquoted so the remote shell expands them;
/// everything else is single-quote escaped.
fn build_path_for_shell(path: &str) -> String {
    if path == "~" {
        "~".to_string()
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("~/{}", shell_escape(rest))
    } else {
        shell_escape(path)
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn parse_ls_output(output: &str) -> Vec<FileEntry> {
    let mut entries = Vec::new();

    for line in output.lines() {
        if line.starts_with("total ") || line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let permissions = parts[0].to_string();
        let is_dir = permissions.starts_with('d');
        let size: u64 = parts[4].parse().unwrap_or(0);
        let name = parts[8..].join(" ");

        if name == "." || name == ".." {
            continue;
        }

        let name = name.trim_end_matches(|c| c == '@' || c == '/' || c == '*');

        entries.push(FileEntry {
            name: name.to_string(),
            is_dir,
            size,
            permissions,
        });
    }

    entries
}
