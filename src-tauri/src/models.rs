use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
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
}

fn default_language() -> String {
    "auto".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            shortcut: "CommandOrControl+Shift+V".to_string(),
            max_history_size: 20,
            language: "auto".to_string(),
        }
    }
}

#[derive(Default)]
pub struct ClipboardHistory {
    pub items: Vec<ClipboardItem>,
    pub max_size: usize,
}

impl ClipboardHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, item: ClipboardItem) {
        // 如果内容已存在，先移除旧的
        if let Some(index) = self
            .items
            .iter()
            .position(|x| x.content == item.content && x.kind == item.kind)
        {
            self.items.remove(index);
        }

        self.items.insert(0, item);
        if self.items.len() > self.max_size {
            self.items.pop();
        }
    }
}
