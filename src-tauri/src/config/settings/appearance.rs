use super::super::{default_false, default_true};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default = "default_app_theme")]
    pub theme: String,
    #[serde(default = "default_font")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_false")]
    pub ligatures: bool,
    #[serde(default = "default_opacity")]
    pub background_opacity: f64,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_true")]
    pub cursor_blink: bool,
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: f64,
    #[serde(default)]
    pub terminal_theme: Option<String>,
}

fn default_app_theme() -> String {
    "github-dark".to_string()
}
fn default_font() -> String {
    "JetBrains Mono, 'Noto Sans SC Variable', Consolas, monospace".to_string()
}
fn default_font_size() -> f64 {
    16.0
}
fn default_opacity() -> f64 {
    1.0
}
fn default_cursor_style() -> String {
    "block".to_string()
}
fn default_ui_font_size() -> f64 {
    16.0
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: default_app_theme(),
            font_family: default_font(),
            font_size: default_font_size(),
            ligatures: false,
            background_opacity: default_opacity(),
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            ui_font_size: default_ui_font_size(),
            terminal_theme: None,
        }
    }
}
