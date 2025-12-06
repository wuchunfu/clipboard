use crate::models::{AppConfig, ClipboardHistory};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub history: Arc<Mutex<ClipboardHistory>>,
    pub data_path: PathBuf,
    pub config_path: PathBuf,
    pub config: Arc<Mutex<AppConfig>>,
    pub is_paused: Arc<Mutex<bool>>,
}
