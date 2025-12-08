use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: Option<i64>,
    pub content: String, // 文字内容或图片的Base64
    pub kind: String,    // "text" or "image"
    pub timestamp: String,
    #[serde(default)]
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub shortcut: String,
    pub max_history_size: usize,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_sensitive_apps")]
    pub sensitive_apps: Vec<String>,
}

fn default_language() -> String {
    "auto".to_string()
}

fn default_theme() -> String {
    "auto".to_string()
}

fn default_sensitive_apps() -> Vec<String> {
    vec![
        "1Password".to_string(),
        "Keychain Access".to_string(),
        "Bitwarden".to_string(),
        "LastPass".to_string(),
        "KeePassXC".to_string(),
        "Enpass".to_string(),
        "Dashlane".to_string(),
    ]
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "CommandOrControl+Shift+V".to_string(),
            max_history_size: 20,
            language: "auto".to_string(),
            theme: "auto".to_string(),
            sensitive_apps: default_sensitive_apps(),
        }
    }
}
