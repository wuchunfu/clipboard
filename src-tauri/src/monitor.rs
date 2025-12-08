use active_win_pos_rs::get_active_window;
use chrono::Local;
use clipboard_master::{CallbackResult, ClipboardHandler};
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::models::ClipboardItem;
use crate::state::AppState;
use crate::tray::update_tray_menu;

pub struct ClipboardMonitor {
    pub app_handle: tauri::AppHandle,
    pub last_text: String,
    pub last_image_hash: Vec<u8>,
}

impl ClipboardMonitor {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let mut last_text = String::new();
        if let Ok(text) = app_handle.clipboard().read_text() {
            last_text = text;
        }
        Self {
            app_handle,
            last_text,
            last_image_hash: Vec::new(),
        }
    }

    fn is_password_manager(&self, app_name: &str) -> bool {
        let state = self.app_handle.state::<AppState>();
        let config = state.config.lock().unwrap();
        config
            .sensitive_apps
            .iter()
            .any(|app| app_name.contains(app) || app_name.eq_ignore_ascii_case(app))
    }

    fn calculate_entropy(&self, s: &str) -> f64 {
        let mut counts = std::collections::HashMap::new();
        for c in s.chars() {
            *counts.entry(c).or_insert(0) += 1;
        }
        let len = s.chars().count() as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
        entropy
    }

    fn is_sensitive_content(&self, text: &str) -> bool {
        // Simple heuristic: high entropy and reasonable length for a password
        if text.len() > 8 && text.len() < 64 {
            let entropy = self.calculate_entropy(text);
            // Threshold is arbitrary, but > 3.5 usually indicates random-ish strings
            if entropy > 3.5 {
                return true;
            }
        }
        false
    }
}

impl ClipboardHandler for ClipboardMonitor {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        let state = self.app_handle.state::<AppState>();

        // Check if paused
        if let Ok(paused) = state.is_paused.lock() {
            if *paused {
                return CallbackResult::Next;
            }
        }

        // Check active application
        if let Ok(active_window) = get_active_window() {
            if self.is_password_manager(&active_window.app_name) {
                log::info!(
                    "Ignored clipboard change from sensitive app: {}",
                    active_window.app_name
                );
                return CallbackResult::Next;
            }
        }

        let mut updated = false;
        let max_size = state.config.lock().unwrap().max_history_size;

        // Check text
        if let Ok(text) = self.app_handle.clipboard().read_text() {
            if text != self.last_text && !text.is_empty() {
                self.last_text = text.clone();
                let is_sensitive = self.is_sensitive_content(&text);

                let item = ClipboardItem {
                    id: None,
                    content: text,
                    kind: "text".to_string(),
                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    is_sensitive,
                };

                match state.db.insert_item(&item, max_size) {
                    Ok(pruned_items) => {
                        // Delete pruned images
                        for pruned in pruned_items {
                            if pruned.kind == "image" {
                                let path = std::path::Path::new(&pruned.content);
                                if path.exists() {
                                    let _ = std::fs::remove_file(path);
                                }
                            }
                        }
                        updated = true;
                        if is_sensitive {
                            log::info!("New sensitive text captured");
                        } else {
                            log::info!("New text captured");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to insert text item: {}", e);
                    }
                }
            }
        }

        // Check image
        if let Ok(img) = self.app_handle.clipboard().read_image() {
            let rgba = img.rgba();
            if !rgba.is_empty()
                && (rgba.len() != self.last_image_hash.len()
                    || rgba != self.last_image_hash.as_slice())
            {
                self.last_image_hash = rgba.to_vec();

                let width = img.width();
                let height = img.height();
                if let Some(buffer) = image::RgbaImage::from_raw(width, height, rgba.to_vec()) {
                    let timestamp = Local::now().timestamp_nanos_opt().unwrap_or(0);
                    let filename = format!("{}.png", timestamp);
                    let app_data_dir = self.app_handle.path().app_data_dir().unwrap();
                    let image_path = app_data_dir.join("images").join(&filename);

                    if let Err(e) = buffer.save(&image_path) {
                        log::error!("Failed to save image to disk: {}", e);
                    } else {
                        let item = ClipboardItem {
                            id: None,
                            content: image_path.to_string_lossy().to_string(),
                            kind: "image".to_string(),
                            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                            is_sensitive: false,
                        };

                        match state.db.insert_item(&item, max_size) {
                            Ok(pruned_items) => {
                                // Delete pruned images
                                for pruned in pruned_items {
                                    if pruned.kind == "image" {
                                        let path = std::path::Path::new(&pruned.content);
                                        if path.exists() {
                                            let _ = std::fs::remove_file(path);
                                        }
                                    }
                                }
                                updated = true;
                                log::info!("New image captured and saved to {:?}", image_path);
                            }
                            Err(e) => {
                                log::error!("Failed to insert image item: {}", e);
                            }
                        }
                    }
                }
            }
        }

        if updated {
            let history = state.db.get_history(1, 20).unwrap_or_default();
            if let Err(e) = update_tray_menu(&self.app_handle, &history) {
                log::error!("Failed to update tray: {}", e);
            }

            if let Err(e) = self.app_handle.emit("clipboard-update", ()) {
                log::error!("Failed to emit clipboard-update event: {}", e);
            }
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: std::io::Error) -> CallbackResult {
        log::error!("Clipboard listener error: {}", error);
        CallbackResult::Next
    }
}
