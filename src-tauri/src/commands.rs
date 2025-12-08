use chrono::Local;
use std::fs;
use tauri::Emitter;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::models::{AppConfig, ClipboardItem};
use crate::state::AppState;
use crate::tray::update_tray_menu;
use crate::utils::write_to_clipboard;

#[tauri::command]
pub fn get_history(
    state: tauri::State<AppState>,
    page: usize,
    page_size: usize,
) -> Vec<ClipboardItem> {
    state.db.get_history(page, page_size).unwrap_or_default()
}

#[tauri::command]
pub fn set_clipboard_item(
    app: tauri::AppHandle,
    content: String,
    kind: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let item = ClipboardItem {
        id: None,
        content: content.clone(),
        kind: kind.clone(),
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        is_sensitive: false, // Manually added items are assumed not sensitive
    };

    // Write to clipboard
    if let Err(e) = write_to_clipboard(&app, &item) {
        log::error!("Failed to write to clipboard: {}", e);
        return Err(e);
    }

    // Update DB
    let max_size = state.config.lock().unwrap().max_history_size;
    match state.db.insert_item(&item, max_size) {
        Ok(pruned_items) => {
            // Delete pruned images
            for pruned in pruned_items {
                if pruned.kind == "image" {
                    let path = std::path::Path::new(&pruned.content);
                    if path.exists() {
                        if let Err(e) = fs::remove_file(path) {
                            log::error!("Failed to delete pruned image file: {}", e);
                        } else {
                            log::info!("Deleted pruned image file: {:?}", path);
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to insert item into DB: {}", e);
            return Err(e.to_string());
        }
    }

    // Update Tray
    let history = state.db.get_history(1, 20).unwrap_or_default();
    if let Err(e) = update_tray_menu(&app, &history) {
        log::error!("Failed to update tray menu: {}", e);
    }

    log::info!("Clipboard item set successfully");
    Ok(())
}

#[tauri::command]
pub fn delete_item(
    app: tauri::AppHandle,
    index: usize,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    match state.db.delete_item(index) {
        Ok(Some(item)) => {
            if item.kind == "image" {
                let path = std::path::Path::new(&item.content);
                if path.exists() {
                    if let Err(e) = fs::remove_file(path) {
                        log::error!("Failed to delete image file: {}", e);
                    } else {
                        log::info!("Deleted image file: {:?}", path);
                    }
                }
            }
        }
        Ok(None) => {
            log::warn!("Item at index {} not found", index);
        }
        Err(e) => {
            log::error!("Failed to delete item from DB: {}", e);
            return Err(e.to_string());
        }
    }

    // Update Tray
    let history = state.db.get_history(1, 20).unwrap_or_default();
    if let Err(e) = update_tray_menu(&app, &history) {
        log::error!("Failed to update tray menu after delete: {}", e);
    }
    log::info!("Deleted item at index {}", index);
    Ok(())
}

#[tauri::command]
pub fn toggle_sensitive(state: tauri::State<AppState>, index: usize) -> Result<bool, String> {
    match state.db.toggle_sensitive(index) {
        Ok(new_state) => {
            log::info!(
                "Toggled sensitive state for item {} to {}",
                index,
                new_state
            );
            Ok(new_state)
        }
        Err(e) => {
            log::error!("Failed to toggle sensitive state: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn clear_history(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    match state.db.clear_history() {
        Ok(items) => {
            for item in items {
                if item.kind == "image" {
                    let path = std::path::Path::new(&item.content);
                    if path.exists() {
                        if let Err(e) = fs::remove_file(path) {
                            log::error!("Failed to delete image file: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to clear history: {}", e);
            return Err(e.to_string());
        }
    }

    // Update Tray
    let _ = update_tray_menu(&app, &[]);
    Ok(())
}

#[tauri::command]
pub fn get_config(state: tauri::State<AppState>) -> AppConfig {
    let config = state.config.lock().unwrap();
    config.clone()
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    shortcut: String,
    max_history_size: usize,
    language: String,
    theme: String,
    sensitive_apps: Vec<String>,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let old_shortcut = {
        let config = state.config.lock().unwrap();
        config.shortcut.clone()
    };

    let new_config = AppConfig {
        shortcut: shortcut.clone(),
        max_history_size,
        language: language.clone(),
        theme: theme.clone(),
        sensitive_apps,
    };

    // Save to file
    if let Ok(json) = serde_json::to_string_pretty(&new_config) {
        if let Err(e) = fs::write(&state.config_path, json) {
            log::error!("Failed to save config file: {}", e);
            return Err(e.to_string());
        }
    }

    // Update state
    {
        let mut config = state.config.lock().unwrap();
        *config = new_config;
    }

    // Update shortcut if changed
    if shortcut != old_shortcut {
        let shortcut_manager = app.global_shortcut();
        let _ = shortcut_manager.unregister(old_shortcut.as_str());
        if let Err(e) = shortcut_manager.register(shortcut.as_str()) {
            log::error!("Failed to register new shortcut: {}", e);
        }
    }

    // Emit event
    let _ = app.emit("config-updated", ());

    Ok(())
}

#[tauri::command]
pub fn set_paused(paused: bool, state: tauri::State<AppState>) {
    let mut is_paused = state.is_paused.lock().unwrap();
    *is_paused = paused;
}

#[tauri::command]
pub fn get_paused(state: tauri::State<AppState>) -> bool {
    let is_paused = state.is_paused.lock().unwrap();
    *is_paused
}

#[tauri::command]
pub fn get_item_content(state: tauri::State<AppState>, id: i64) -> Result<String, String> {
    state.db.get_item_content(id).map_err(|e| e.to_string())
}
