use std::fs;
use std::path::{Path, PathBuf};

use arboard::Clipboard;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::db;
use crate::models::{
    AppSettings, CanvasImage, CanvasItem, ClipboardImageData, DatabaseLoadResult, DrawingItem,
    ImportedImageFile, StickyNote, TagItem,
};
use crate::paths::{
    current_database_path, default_db_path, default_image_library_path, read_database_path,
    write_database_path,
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
pub(crate) fn save_drawing(app: AppHandle, drawing: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let drawing: DrawingItem = serde_json::from_str(&drawing).map_err(|error| error.to_string())?;
    db::upsert_drawing(&conn, &drawing)
}

#[tauri::command]
pub(crate) fn save_image(app: AppHandle, image: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let image: CanvasImage = serde_json::from_str(&image).map_err(|error| error.to_string())?;
    let previous = image_by_id(&conn, &image.id)?;
    db::upsert_image(&conn, &image)?;
    match previous {
        Some(previous) if previous.file_name != image.file_name => {
            let library = saved_image_library_path(&app, &conn)?;
            release_image_asset(&conn, &library, &previous.file_name)?;
            retain_image_asset(&conn, &image.file_name)?;
        }
        None => retain_image_asset(&conn, &image.file_name)?,
        _ => {}
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn delete_image(app: AppHandle, id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let image = image_by_id(&conn, &id)?;
    db::delete_image(&conn, id)?;
    if let Some(image) = image {
        let library = saved_image_library_path(&app, &conn)?;
        release_image_asset(&conn, &library, &image.file_name)?;
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn delete_images_by_canvas(app: AppHandle, canvas_id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let images = images_by_canvas_id(&conn, &canvas_id)?;
    db::delete_images_by_canvas(&conn, canvas_id)?;
    let library = saved_image_library_path(&app, &conn)?;
    for image in images {
        release_image_asset(&conn, &library, &image.file_name)?;
    }
    Ok(())
}

#[tauri::command]
pub(crate) fn delete_drawing(app: AppHandle, id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_drawing(&conn, id)
}

#[tauri::command]
pub(crate) fn delete_drawings_by_canvas(app: AppHandle, canvas_id: String) -> Result<(), String> {
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    db::delete_drawings_by_canvas(&conn, canvas_id)
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
pub(crate) fn import_image_file(
    app: AppHandle,
    source_path: String,
    library_path: String,
) -> Result<ImportedImageFile, String> {
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err("图片文件不存在".to_string());
    }
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let content_hash = file_hash(&source)?;
    let library = image_library_path(&app, &library_path)?;
    fs::create_dir_all(&library).map_err(|error| error.to_string())?;
    let original_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image")
        .to_string();
    if let Some((file_name, stored_original_name)) = image_asset_by_hash(&conn, &content_hash)? {
        let target = library.join(&file_name);
        if !target.exists() {
            fs::copy(&source, &target).map_err(|error| error.to_string())?;
        }
        return Ok(ImportedImageFile {
            file_name,
            original_name: stored_original_name,
            content_hash,
            path: target.to_string_lossy().to_string(),
        });
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value.to_ascii_lowercase()))
        .unwrap_or_default();
    let file_name = format!("{}{}", unique_file_stem(), extension);
    let target = library.join(&file_name);
    fs::copy(&source, &target).map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO image_assets (content_hash, file_name, original_name, ref_count, created_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![content_hash, file_name, original_name, now_iso()],
    )
    .map_err(|error| error.to_string())?;
    Ok(ImportedImageFile {
        file_name,
        original_name,
        content_hash,
        path: target.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub(crate) fn import_image_bytes(
    app: AppHandle,
    bytes: Vec<u8>,
    original_name: String,
    mime_type: String,
    library_path: String,
) -> Result<ImportedImageFile, String> {
    if bytes.is_empty() {
        return Err("图片内容为空".to_string());
    }
    let db_path = current_database_path(&app)?;
    let conn = db::open_database(&db_path)?;
    let content_hash = bytes_hash(&bytes);
    let library = image_library_path(&app, &library_path)?;
    fs::create_dir_all(&library).map_err(|error| error.to_string())?;
    let clean_original_name = if original_name.trim().is_empty() {
        "clipboard-image".to_string()
    } else {
        original_name
    };
    if let Some((file_name, stored_original_name)) = image_asset_by_hash(&conn, &content_hash)? {
        let target = library.join(&file_name);
        if !target.exists() {
            fs::write(&target, &bytes).map_err(|error| error.to_string())?;
        }
        return Ok(ImportedImageFile {
            file_name,
            original_name: stored_original_name,
            content_hash,
            path: target.to_string_lossy().to_string(),
        });
    }
    let extension = image_extension(&mime_type, &clean_original_name);
    let file_name = format!("{}{}", unique_file_stem(), extension);
    let target = library.join(&file_name);
    fs::write(&target, &bytes).map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO image_assets (content_hash, file_name, original_name, ref_count, created_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![content_hash, file_name, clean_original_name, now_iso()],
    )
    .map_err(|error| error.to_string())?;
    Ok(ImportedImageFile {
        file_name,
        original_name: clean_original_name,
        content_hash,
        path: target.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub(crate) fn resolve_image_path(
    app: AppHandle,
    file_name: String,
    library_path: String,
) -> Result<Option<String>, String> {
    let library = image_library_path(&app, &library_path)?;
    let path = library.join(file_name);
    Ok(path.exists().then(|| path.to_string_lossy().to_string()))
}

#[tauri::command]
pub(crate) fn default_image_library(app: AppHandle) -> Result<String, String> {
    Ok(default_image_library_path(&app)?.to_string_lossy().to_string())
}

#[tauri::command]
pub(crate) fn read_clipboard_image() -> Result<ClipboardImageData, String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    if let Ok(image) = clipboard.get_image() {
        return clipboard_bitmap_to_png(image);
    }
    let files = clipboard
        .get()
        .file_list()
        .map_err(|_| "系统剪贴板中没有图片".to_string())?;
    let path = files
        .into_iter()
        .find(|path| is_supported_image_path(path))
        .ok_or_else(|| "系统剪贴板中没有图片".to_string())?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Err("图片内容为空".to_string());
    }
    Ok(ClipboardImageData {
        bytes,
        original_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("clipboard-image")
            .to_string(),
        mime_type: mime_type_for_image_path(&path),
    })
}

fn is_supported_image_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg"
            )
        })
        .unwrap_or(false)
}

fn mime_type_for_image_path(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("bmp") => "image/bmp".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        _ => "image/png".to_string(),
    }
}

fn clipboard_bitmap_to_png(image: arboard::ImageData<'_>) -> Result<ClipboardImageData, String> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(
            image.bytes.as_ref(),
            image.width as u32,
            image.height as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(|error| error.to_string())?;
    Ok(ClipboardImageData {
        bytes,
        original_name: "clipboard-image.png".to_string(),
        mime_type: "image/png".to_string(),
    })
}

fn image_library_path(app: &AppHandle, library_path: &str) -> Result<PathBuf, String> {
    if library_path.trim().is_empty() {
        return default_image_library_path(app);
    }
    Ok(PathBuf::from(library_path))
}

fn saved_image_library_path(app: &AppHandle, conn: &Connection) -> Result<PathBuf, String> {
    let library_path = conn
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'image_library_path'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    image_library_path(app, &library_path)
}

fn image_by_id(conn: &Connection, id: &str) -> Result<Option<CanvasImage>, String> {
    conn.query_row("SELECT data FROM images WHERE id = ?1", params![id], |row| {
        row.get::<_, String>(0)
    })
    .optional()
    .map_err(|error| error.to_string())?
    .map(|data| serde_json::from_str::<CanvasImage>(&data).map_err(|error| error.to_string()))
    .transpose()
}

fn images_by_canvas_id(conn: &Connection, canvas_id: &str) -> Result<Vec<CanvasImage>, String> {
    let mut statement = conn
        .prepare("SELECT data FROM images WHERE canvas_id = ?1")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![canvas_id], |row| {
            let data: String = row.get(0)?;
            serde_json::from_str::<CanvasImage>(&data).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn image_asset_by_hash(
    conn: &Connection,
    content_hash: &str,
) -> Result<Option<(String, String)>, String> {
    conn.query_row(
        "SELECT file_name, original_name FROM image_assets WHERE content_hash = ?1",
        params![content_hash],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn retain_image_asset(conn: &Connection, file_name: &str) -> Result<(), String> {
    let actual_references = count_image_references(conn, file_name)?;
    conn.execute(
        "UPDATE image_assets SET ref_count = ?1 WHERE file_name = ?2",
        params![actual_references, file_name],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn release_image_asset(conn: &Connection, library: &Path, file_name: &str) -> Result<(), String> {
    let actual_references = count_image_references(conn, file_name)?;
    if actual_references > 0 {
        conn.execute(
            "UPDATE image_assets SET ref_count = ?1 WHERE file_name = ?2",
            params![actual_references, file_name],
        )
        .map_err(|error| error.to_string())?;
        return Ok(());
    }
    let asset = conn
        .query_row(
            "SELECT content_hash FROM image_assets WHERE file_name = ?1",
            params![file_name],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(content_hash) = asset else {
        let target = library.join(file_name);
        if target.exists() {
            fs::remove_file(&target).map_err(|error| error.to_string())?;
        }
        return Ok(());
    };
    let target = library.join(file_name);
    if target.exists() {
        fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    conn.execute(
        "DELETE FROM image_assets WHERE content_hash = ?1",
        params![content_hash],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn count_image_references(conn: &Connection, file_name: &str) -> Result<i64, String> {
    let mut statement = conn
        .prepare("SELECT data FROM images")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    let mut count = 0;
    for row in rows {
        let data = row.map_err(|error| error.to_string())?;
        let image: CanvasImage = serde_json::from_str(&data).map_err(|error| error.to_string())?;
        if image.file_name == file_name {
            count += 1;
        }
    }
    Ok(count)
}

fn file_hash(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    Ok(bytes_hash(&bytes))
}

fn bytes_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn image_extension(mime_type: &str, original_name: &str) -> String {
    let from_name = Path::new(original_name)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value.to_ascii_lowercase()));
    if let Some(extension) = from_name {
        return extension;
    }
    match mime_type {
        "image/jpeg" => ".jpg".to_string(),
        "image/png" => ".png".to_string(),
        "image/webp" => ".webp".to_string(),
        "image/gif" => ".gif".to_string(),
        "image/bmp" => ".bmp".to_string(),
        "image/svg+xml" => ".svg".to_string(),
        _ => ".png".to_string(),
    }
}

fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| unique_file_stem())
}

fn unique_file_stem() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("img-{nanos}")
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
