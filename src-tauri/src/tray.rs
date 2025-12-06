use crate::models::ClipboardItem;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::Wry;

pub fn create_history_menu(
    app: &tauri::AppHandle,
    history: &[ClipboardItem],
) -> Result<Menu<Wry>, String> {
    let menu = Menu::new(app).map_err(|e| e.to_string())?;

    // Add "Show History" item
    let show_item = MenuItem::with_id(app, "show", "Show History", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&show_item).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    for (i, item) in history.iter().take(10).enumerate() {
        let mut title = if item.kind == "text" {
            item.content.chars().take(20).collect::<String>()
        } else {
            format!("Image {}", item.timestamp)
        };
        if item.kind == "text" && item.content.chars().count() > 20 {
            title.push_str("...");
        }

        let menu_item =
            MenuItem::with_id(app, format!("history_{}", i), &title, true, None::<&str>)
                .map_err(|e| e.to_string())?;
        menu.append(&menu_item).map_err(|e| e.to_string())?;
    }

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    menu.append(
        &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(menu)
}

pub fn update_tray_menu(app: &tauri::AppHandle, history: &[ClipboardItem]) -> Result<(), String> {
    let tray = if let Some(tray) = app.tray_by_id("tray") {
        tray
    } else {
        return Ok(()); // Tray might not be ready yet
    };

    let menu = create_history_menu(app, history)?;
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    Ok(())
}
