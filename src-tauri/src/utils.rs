use crate::models::ClipboardItem;
use crate::state::AppState;
use base64::{engine::general_purpose, Engine as _};
use std::fs;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

pub fn write_to_clipboard(app: &tauri::AppHandle, item: &ClipboardItem) -> Result<(), String> {
    if item.kind == "text" {
        app.clipboard()
            .write_text(item.content.clone())
            .map_err(|e| e.to_string())?;
    } else if item.kind == "image" {
        let bytes = if item.content.starts_with('/') || item.content.chars().nth(1) == Some(':') {
            // It's a file path
            fs::read(&item.content).map_err(|e| e.to_string())?
        } else {
            // It's base64 (legacy support)
            general_purpose::STANDARD
                .decode(&item.content)
                .map_err(|e| e.to_string())?
        };

        let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let width = img.width();
        let height = img.height();
        let rgba_bytes = rgba.into_raw();

        // Update last_app_image_change
        let state = app.state::<AppState>();
        if let Ok(mut last_change) = state.last_app_image_change.lock() {
            *last_change = Some(rgba_bytes.clone());
        }

        let tauri_img = tauri::image::Image::new(&rgba_bytes, width, height);
        app.clipboard()
            .write_image(&tauri_img)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
