use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const PREF_FILE: &str = "database-path.txt";

#[derive(Serialize)]
struct DatabaseLoadResult {
    data: String,
    db_path: String,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

fn app_config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_config_dir().map_err(|error| error.to_string())
}

fn default_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("notescape.sqlite3"))
}

fn pref_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_config_dir(app)?.join(PREF_FILE))
}

fn read_database_path(app: &AppHandle) -> Result<PathBuf, String> {
    let pref = pref_path(app)?;
    if let Ok(saved) = fs::read_to_string(pref) {
        let trimmed = saved.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    default_db_path(app)
}

fn write_database_path(app: &AppHandle, db_path: &Path) -> Result<(), String> {
    let pref = pref_path(app)?;
    if let Some(parent) = pref.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(pref, db_path.to_string_lossy().as_bytes()).map_err(|error| error.to_string())
}

fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            data TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )
    .map_err(|error| error.to_string())?;
    Ok(conn)
}

fn load_or_seed(path: &Path, fallback_data: &str) -> Result<String, String> {
    let conn = open_database(path)?;
    let mut statement = conn
        .prepare("SELECT data FROM app_state WHERE id = 1")
        .map_err(|error| error.to_string())?;
    let existing = statement.query_row([], |row| row.get::<_, String>(0)).ok();
    drop(statement);

    if let Some(data) = existing {
        return Ok(data);
    }

    conn.execute(
        "INSERT INTO app_state (id, data, updated_at) VALUES (1, ?1, CURRENT_TIMESTAMP)",
        params![fallback_data],
    )
    .map_err(|error| error.to_string())?;
    Ok(fallback_data.to_string())
}

#[tauri::command]
fn load_app_data(app: AppHandle, default_data: String) -> Result<DatabaseLoadResult, String> {
    let db_path = read_database_path(&app)?;
    if !db_path.exists() {
        write_database_path(&app, &db_path)?;
    }
    let data = load_or_seed(&db_path, &default_data)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: db_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn save_app_data(app: AppHandle, data: String) -> Result<(), String> {
    let db_path = read_database_path(&app)?;
    let conn = open_database(&db_path)?;
    conn.execute(
        "INSERT INTO app_state (id, data, updated_at) VALUES (1, ?1, CURRENT_TIMESTAMP)
         ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = CURRENT_TIMESTAMP",
        params![data],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_database_path(
    app: AppHandle,
    db_path: String,
    fallback_data: String,
) -> Result<DatabaseLoadResult, String> {
    let path = PathBuf::from(db_path);
    let data = load_or_seed(&path, &fallback_data)?;
    write_database_path(&app, &path)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: path.to_string_lossy().to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_app_data,
            save_app_data,
            set_database_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
