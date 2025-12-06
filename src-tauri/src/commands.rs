use chrono::Local;
use std::fs;
use tauri::Emitter;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::models::{AppConfig, ClipboardItem};
use crate::state::AppState;
use crate::tray::update_tray_menu;
use crate::utils::write_to_clipboard;

#[tauri::command]
pub fn get_history(state: tauri::State<AppState>) -> Vec<ClipboardItem> {
    let history = state.history.lock().unwrap();
    history.items.clone()
}

#[tauri::command]
pub fn set_clipboard_item(
    app: tauri::AppHandle,
    content: String,
    kind: String,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let item = ClipboardItem {
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

    // Update history
    let mut history = state.history.lock().unwrap();
    history.push(item);

    // Save
    let items_to_save: Vec<&ClipboardItem> =
        history.items.iter().filter(|i| !i.is_sensitive).collect();
    if let Err(e) =
        serde_json::to_string(&items_to_save).map(|json| fs::write(&state.data_path, json))
    {
        log::error!("Failed to save history: {}", e);
    }

    // Update Tray
    if let Err(e) = update_tray_menu(&app, &history.items) {
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
    let mut history = state.history.lock().unwrap();
    if index < history.items.len() {
        history.items.remove(index);
        // Save
        let items_to_save: Vec<&ClipboardItem> =
            history.items.iter().filter(|i| !i.is_sensitive).collect();
        if let Err(e) =
            serde_json::to_string(&items_to_save).map(|json| fs::write(&state.data_path, json))
        {
            log::error!("Failed to save history after delete: {}", e);
        }

        // Update Tray
        if let Err(e) = update_tray_menu(&app, &history.items) {
            log::error!("Failed to update tray menu after delete: {}", e);
        }
        log::info!("Deleted item at index {}", index);
    } else {
        log::warn!("Attempted to delete item at invalid index {}", index);
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_sensitive(state: tauri::State<AppState>, index: usize) -> Result<bool, String> {
    let mut history = state.history.lock().unwrap();
    let new_state = if let Some(item) = history.items.get_mut(index) {
        item.is_sensitive = !item.is_sensitive;
        item.is_sensitive
    } else {
        return Err(format!("Item at index {} not found", index));
    };

    // Save (filtering out sensitive items)
    let items_to_save: Vec<&ClipboardItem> =
        history.items.iter().filter(|i| !i.is_sensitive).collect();
    if let Err(e) =
        serde_json::to_string(&items_to_save).map(|json| fs::write(&state.data_path, json))
    {
        log::error!("Failed to save history after toggle: {}", e);
    }

    log::info!(
        "Toggled sensitive state for item {} to {}",
        index,
        new_state
    );
    Ok(new_state)
}

#[tauri::command]
pub fn clear_history(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), String> {
    let mut history = state.history.lock().unwrap();
    history.items.clear();
    // Save (empty list)
    let _ = fs::write(&state.data_path, "[]");

    // Update Tray
    let _ = update_tray_menu(&app, &history.items);
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
    };

    // Save to file
    let json = serde_json::to_string(&new_config).map_err(|e| e.to_string())?;
    fs::write(&state.config_path, json).map_err(|e| e.to_string())?;

    // Update in-memory config
    {
        let mut config = state.config.lock().unwrap();
        *config = new_config;
    }

    // Update history max size
    {
        let mut history = state.history.lock().unwrap();
        history.max_size = max_history_size;
        // Trim if necessary
        while history.items.len() > max_history_size {
            history.items.pop();
        }
    }

    // Update global shortcut if changed
    if shortcut != old_shortcut {
        log::info!(
            "Updating global shortcut from '{}' to '{}'",
            old_shortcut,
            shortcut
        );
        // Unregister old shortcut
        if let Err(e) = app.global_shortcut().unregister(old_shortcut.as_str()) {
            log::warn!("Failed to unregister old shortcut: {}", e);
        }

        // Register new shortcut
        if let Err(e) = app.global_shortcut().register(shortcut.as_str()) {
            log::error!("Failed to register new shortcut: {}", e);
            return Err(format!("Failed to register new shortcut: {}", e));
        }
    }

    // Notify frontend
    if let Err(e) = app.emit("config-updated", ()) {
        log::error!("Failed to emit config-updated event: {}", e);
    }

    log::info!("Config saved successfully");
    Ok(())
}

#[tauri::command]
pub fn set_paused(paused: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let mut is_paused = state.is_paused.lock().map_err(|e| e.to_string())?;
    *is_paused = paused;
    log::info!("Clipboard recording paused: {}", paused);
    Ok(())
}

#[tauri::command]
pub fn get_paused(state: tauri::State<AppState>) -> Result<bool, String> {
    let is_paused = state.is_paused.lock().map_err(|e| e.to_string())?;
    Ok(*is_paused)
}
