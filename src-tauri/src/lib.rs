mod commands;
mod crypto;
mod db;
mod models;
mod monitor;
mod state;
mod tray;
mod utils;

use clipboard_master::Master;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

use crate::commands::*;
use crate::crypto::Crypto;
use crate::db::Database;
use crate::models::AppConfig;
use crate::monitor::ClipboardMonitor;
use crate::state::AppState;
use crate::tray::create_history_menu;
use crate::utils::write_to_clipboard;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load config first
    let app_data_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".clipboard-manager"))
        .unwrap_or_else(|_| PathBuf::from(".clipboard-manager"));

    if !app_data_dir.exists() {
        let _ = fs::create_dir_all(&app_data_dir);
    }

    let config_path = app_data_dir.join("config.json");
    let config = if let Ok(content) = fs::read_to_string(&config_path) {
        serde_json::from_str::<AppConfig>(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    };

    let db_path = app_data_dir.join("history.db");
    let key_path = app_data_dir.join("secret.key");
    let crypto = Arc::new(Crypto::new(&key_path));
    let db = Arc::new(Database::new(&db_path, crypto).expect("Failed to initialize database"));

    let shortcut_key = config.shortcut.clone();
    let config_arc = Arc::new(Mutex::new(config));

    let is_paused = Arc::new(Mutex::new(false));
    let is_paused_state = is_paused.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcut(shortcut_key.as_str())
                .expect("Failed to register shortcut")
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("popup") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                // Get mouse position
                                use mouse_position::mouse_position::Mouse;
                                let position = Mouse::get_mouse_position();
                                if let Mouse::Position { x, y } = position {
                                    let mut final_x = x;
                                    let mut final_y = y;
                                    log::info!("Mouse Position: ({}, {})", x, y);

                                    if let Ok(monitors) = window.available_monitors() {
                                        for m in monitors {
                                            let m_pos = m.position();
                                            let m_size = m.size();
                                            let scale = m.scale_factor();
                                            let x = x * scale as i32;
                                            let y = y * scale as i32;
                                            final_x = x;
                                            final_y = y;
                                            // Check if mouse is in this monitor
                                            if x >= m_pos.x
                                                && x < m_pos.x + m_size.width as i32
                                                && y >= m_pos.y
                                                && y < m_pos.y + m_size.height as i32
                                            {
                                                if let Ok(w_size) = window.outer_size() {
                                                    let w = w_size.width as i32;
                                                    let h = w_size.height as i32;

                                                    // If window goes off the right edge, shift to left of cursor
                                                    if x + w > m_pos.x + m_size.width as i32 {
                                                        final_x = x - w;
                                                    }

                                                    // If window goes off the bottom edge, shift to above cursor
                                                    if y + h > m_pos.y + m_size.height as i32 {
                                                        final_y = y - h;
                                                    }
                                                }
                                                break;
                                            }
                                        }
                                    }
                                    let _ = window.set_position(tauri::Position::Physical(
                                        tauri::PhysicalPosition {
                                            x: final_x,
                                            y: final_y,
                                        },
                                    ));
                                } else {
                                    // Fallback to center if mouse position fails
                                    let _ = window.center();
                                }

                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(move |app| {
            // Set activation policy to Accessory to hide from Dock
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let handle = app.handle().clone();

            // 初始化数据路径
            let app_data_dir = app.path().app_data_dir()?;
            if !app_data_dir.exists() {
                let _ = fs::create_dir_all(&app_data_dir);
            }
            let images_dir = app_data_dir.join("images");
            if !images_dir.exists() {
                let _ = fs::create_dir_all(&images_dir);
            }

            // 将状态交给 Tauri 管理
            app.manage(AppState {
                db: db.clone(),
                config_path: config_path.clone(),
                config: config_arc.clone(),
                is_paused: is_paused_state.clone(),
            });

            // 托盘设置
            let menu = {
                let history = db.get_history(1, 20).unwrap_or_default();
                create_history_menu(app.handle(), &history).unwrap()
            };

            let _tray = TrayIconBuilder::with_id("tray")
                .icon(
                    app.default_window_icon()
                        .expect("No default window icon found")
                        .clone(),
                )
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    id if id.starts_with("history_") => {
                        if let Ok(index) = id.replace("history_", "").parse::<usize>() {
                            let state = app.state::<AppState>();
                            if let Ok(history) = state.db.get_history(1, 20) {
                                if let Some(item) = history.get(index) {
                                    let _ = write_to_clipboard(app, item);
                                }
                            }
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // 剪切板监听线程
            let monitor_handle = handle.clone();
            thread::spawn(move || {
                // Delay starting the monitor to avoid race conditions with startup tray menu
                std::thread::sleep(std::time::Duration::from_secs(1));

                let monitor = ClipboardMonitor::new(monitor_handle);
                match Master::new(monitor) {
                    Ok(mut master) => {
                        if let Err(e) = master.run() {
                            log::error!("Failed to run clipboard listener: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create clipboard master: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            set_clipboard_item,
            delete_item,
            toggle_sensitive,
            clear_history,
            get_config,
            save_config,
            set_paused,
            get_paused,
            get_item_content
        ])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            tauri::WindowEvent::Focused(false) => {
                if window.label() == "popup" {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
