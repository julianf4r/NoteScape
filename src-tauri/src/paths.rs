use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

const PREF_FILE: &str = "database-path.txt";

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map_err(|error| error.to_string())
}

pub(crate) fn default_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("notescape.sqlite3"))
}

fn pref_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join(PREF_FILE))
}

pub(crate) fn read_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let pref = pref_path(app)?;
    if let Ok(saved) = fs::read_to_string(pref) {
        let trimmed = saved.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    default_db_path(app)
}

pub(crate) fn write_database_path(app: &AppHandle, db_path: &Path) -> Result<(), String> {
    let pref = pref_path(app)?;
    if let Some(parent) = pref.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(pref, db_path.to_string_lossy().as_bytes()).map_err(|error| error.to_string())
}

pub(crate) fn current_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let db_path = read_database_path(app)?;
    let default_path = default_db_path(app)?;
    if !db_path.exists() && db_path != default_path {
        return Err(format!(
            "当前数据库文件不存在：{}。请重新选择数据库、创建新数据库，或回退到默认数据库。",
            db_path.to_string_lossy()
        ));
    }
    Ok(db_path)
}
