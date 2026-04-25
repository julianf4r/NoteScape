use std::fs;
use std::path::PathBuf;

use tauri::AppHandle;

use crate::db;
use crate::models::{AppSettings, CanvasItem, DatabaseLoadResult, StickyNote, TagItem};
use crate::paths::{
    current_database_path, default_db_path, read_database_path, write_database_path,
};

#[tauri::command]
pub(crate) fn load_app_data(app: AppHandle) -> Result<DatabaseLoadResult, String> {
    let db_path = read_database_path(&app)?;
    let default_path = default_db_path(&app)?;
    if !db_path.exists() && db_path != default_path {
        return Err(format!(
            "上次使用的数据库文件不存在：{}。请重新选择数据库、创建新数据库，或回退到默认数据库。",
            db_path.to_string_lossy()
        ));
    }
    if !db_path.exists() {
        write_database_path(&app, &db_path)?;
    }
    let data = db::load_or_seed(&db_path)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: db_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub(crate) fn reset_database_to_default(app: AppHandle) -> Result<DatabaseLoadResult, String> {
    let db_path = default_db_path(&app)?;
    let data = db::load_or_seed(&db_path)?;
    write_database_path(&app, &db_path)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: db_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub(crate) fn save_app_data(app: AppHandle, data: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let mut conn = db::open_database(&db_path)?;
    let app_data = db::parse_app_data(&data)?;
    db::save_structured_data(&mut conn, &app_data)
}

#[tauri::command]
pub(crate) fn save_canvas(app: AppHandle, canvas: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let canvas: CanvasItem = serde_json::from_str(&canvas).map_err(|error| error.to_string())?;
    db::upsert_canvas(&conn, &canvas)
}

#[tauri::command]
pub(crate) fn delete_canvas(app: AppHandle, id: String, deleted_at: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_canvas(&conn, id, deleted_at)
}

#[tauri::command]
pub(crate) fn restore_canvas(app: AppHandle, id: String, updated_at: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::restore_canvas(&conn, id, updated_at)
}

#[tauri::command]
pub(crate) fn remove_canvas_forever(app: AppHandle, id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::remove_canvas_forever(&conn, id)
}

#[tauri::command]
pub(crate) fn save_note(app: AppHandle, note: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let mut conn = db::open_database(&db_path)?;
    let note: StickyNote = serde_json::from_str(&note).map_err(|error| error.to_string())?;
    db::upsert_note(&mut conn, &note)
}

#[tauri::command]
pub(crate) fn delete_note(app: AppHandle, id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_note(&conn, id)
}

#[tauri::command]
pub(crate) fn delete_notes_by_canvas(app: AppHandle, canvas_id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_notes_by_canvas(&conn, canvas_id)
}

#[tauri::command]
pub(crate) fn save_tag(app: AppHandle, tag: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let tag: TagItem = serde_json::from_str(&tag).map_err(|error| error.to_string())?;
    db::upsert_tag(&conn, &tag)
}

#[tauri::command]
pub(crate) fn delete_tag(app: AppHandle, id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_tag(&conn, id)
}

#[tauri::command]
pub(crate) fn save_app_settings(app: AppHandle, settings: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let settings: AppSettings =
        serde_json::from_str(&settings).map_err(|error| error.to_string())?;
    db::save_settings(&conn, &settings)
}

#[tauri::command]
pub(crate) fn backup_database(app: AppHandle, backup_path: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    fs::copy(db_path, backup_path).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub(crate) fn export_json_file(path: String, data: String) -> Result<(), String> {
    fs::write(path, data).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn import_json_file(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn set_database_path(
    app: AppHandle,
    db_path: String,
) -> Result<DatabaseLoadResult, String> {
    let path = PathBuf::from(db_path);
    let data = db::load_existing_database(&path)?;
    write_database_path(&app, &path)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub(crate) fn create_database(
    app: AppHandle,
    db_path: String,
) -> Result<DatabaseLoadResult, String> {
    let path = PathBuf::from(db_path);
    let data = db::create_default_database(&path)?;
    write_database_path(&app, &path)?;
    Ok(DatabaseLoadResult {
        data,
        db_path: path.to_string_lossy().to_string(),
    })
}
