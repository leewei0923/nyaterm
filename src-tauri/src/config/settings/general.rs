use super::super::{default_false, default_true};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_true")]
    pub startup_restore: bool,
    #[serde(default = "default_shell")]
    pub default_local_shell: String,
    #[serde(default = "default_false")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub boss_key: Option<String>,
    #[serde(default = "default_true")]
    pub confirm_on_close: bool,
}

fn default_shell() -> String {
    if cfg!(windows) {
        "powershell.exe".to_string()
    } else {
        "bash".to_string()
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            startup_restore: false,
            default_local_shell: default_shell(),
            minimize_to_tray: false,
            boss_key: None,
            confirm_on_close: true,
        }
    }
}
