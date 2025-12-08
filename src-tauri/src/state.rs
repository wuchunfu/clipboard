use crate::db::Database;
use crate::models::AppConfig;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub db: Arc<Database>,
    pub config_path: PathBuf,
    pub config: Arc<Mutex<AppConfig>>,
    pub is_paused: Arc<Mutex<bool>>,
    pub last_app_change: Arc<Mutex<Option<String>>>,
}
