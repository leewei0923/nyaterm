use super::{load_json_doc, save_json_doc, uuid_v4};
use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

fn default_protocol() -> String {
    "socks5".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    1080
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    #[serde(default = "uuid_v4")]
    pub id: String,
    pub name: String,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProxiesConfig {
    #[serde(default)]
    proxies: Vec<ProxyConfig>,
}

pub fn load_proxies(app: &AppHandle) -> AppResult<Vec<ProxyConfig>> {
    let _ = app;
    let cfg: ProxiesConfig = load_json_doc(crate::storage::JSON_PROXIES)?;
    Ok(cfg.proxies)
}

pub fn save_proxies(app: &AppHandle, proxies: &[ProxyConfig]) -> AppResult<()> {
    let _ = app;
    save_json_doc(
        crate::storage::JSON_PROXIES,
        &ProxiesConfig {
            proxies: proxies.to_vec(),
        },
    )
}

pub fn load_proxy_by_id(app: &AppHandle, id: &str) -> AppResult<Option<ProxyConfig>> {
    let proxies = load_proxies(app)?;
    Ok(proxies.into_iter().find(|p| p.id == id))
}
