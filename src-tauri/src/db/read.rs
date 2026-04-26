use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use super::{open_database, parse_app_data, write::save_structured_data};
use crate::defaults::default_app_data_json;
use crate::models::{
    AppData, AppSettings, CanvasItem, ChecklistItem, DrawingItem, StickyNote, TagItem,
    ViewportState,
};

fn is_database_empty(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM canvases", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    Ok(count == 0)
}

pub(crate) fn load_structured_data(conn: &Connection) -> Result<AppData, String> {
    let version = meta_value(conn, "version")?
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1);

    let settings = AppSettings {
        theme: meta_value(conn, "theme")?.unwrap_or_else(|| "light".to_string()),
        default_font_size: meta_value(conn, "default_font_size")?
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(18.0),
        show_grid: meta_bool(conn, "show_grid", true)?,
        random_rotation: meta_bool(conn, "random_rotation", true)?,
        note_shadow: meta_bool(conn, "note_shadow", true)?,
        auto_save: meta_bool(conn, "auto_save", true)?,
        chinese_font_family: meta_value(conn, "chinese_font_family")?
            .unwrap_or_else(|| "Xiaolai, Microsoft YaHei".to_string()),
        english_font_family: meta_value(conn, "english_font_family")?
            .unwrap_or_else(|| "Segoe Print, Comic Sans MS".to_string()),
        monospace_font_family: meta_value(conn, "monospace_font_family")?
            .unwrap_or_else(|| "Consolas, Cascadia Mono, monospace".to_string()),
    };

    let canvases = load_canvases(conn)?;
    let drawings = load_drawings(conn)?;
    let tags = load_tags(conn)?;
    let mut notes = load_notes(conn)?;

    for note in &mut notes {
        note.tags = load_note_tags(conn, &note.id)?;
        note.checked_items = load_checklist_items(conn, &note.id)?;
    }

    Ok(AppData {
        version,
        canvases,
        notes,
        drawings,
        tags,
        settings,
    })
}

pub(crate) fn load_or_seed(path: &Path) -> Result<String, String> {
    let mut conn = open_database(path)?;
    if is_database_empty(&conn)? {
        let seed_data = match legacy_app_state_data(&conn)? {
            Some(data) => data,
            None => default_app_data_json()?,
        };
        let seed = parse_app_data(&seed_data)?;
        save_structured_data(&mut conn, &seed)?;
    }
    serde_json::to_string(&load_structured_data(&conn)?).map_err(|error| error.to_string())
}

fn required_table_exists(conn: &Connection, name: &str) -> Result<bool, String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    Ok(exists.is_some())
}

fn validate_existing_database(conn: &Connection) -> Result<(), String> {
    let required_tables = [
        "app_meta",
        "canvases",
        "notes",
        "tags",
        "note_tags",
        "checklist_items",
    ];
    for table in required_tables {
        if !required_table_exists(conn, table)? {
            return Err("所选文件不是有效的贴境数据库，缺少必要的数据表".to_string());
        }
    }
    let canvas_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM canvases", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if canvas_count < 1 {
        return Err("所选数据库没有画布数据，不能切换".to_string());
    }
    Ok(())
}

pub(crate) fn load_existing_database(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err("所选数据库文件不存在".to_string());
    }
    let conn = open_database(path)
        .map_err(|_| "无法打开所选文件，它不是有效的 SQLite 数据库".to_string())?;
    validate_existing_database(&conn)?;
    serde_json::to_string(&load_structured_data(&conn)?).map_err(|error| error.to_string())
}

fn legacy_app_state_data(conn: &Connection) -> Result<Option<String>, String> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'app_state'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    if exists.is_none() {
        return Ok(None);
    }

    conn.query_row("SELECT data FROM app_state WHERE id = 1", [], |row| {
        row.get(0)
    })
    .optional()
    .map_err(|error| error.to_string())
}

fn meta_value(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT value FROM app_meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn meta_bool(conn: &Connection, key: &str, fallback: bool) -> Result<bool, String> {
    Ok(meta_value(conn, key)?
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(fallback))
}

fn load_canvases(conn: &Connection) -> Result<Vec<CanvasItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, name, description, created_at, updated_at, deleted_at, viewport_offset_x, viewport_offset_y, viewport_scale
             FROM canvases
             ORDER BY created_at ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(CanvasItem {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                deleted_at: row.get(5)?,
                viewport: match (
                    row.get::<_, Option<f64>>(6)?,
                    row.get::<_, Option<f64>>(7)?,
                    row.get::<_, Option<f64>>(8)?,
                ) {
                    (Some(offset_x), Some(offset_y), Some(scale)) => Some(ViewportState {
                        offset_x,
                        offset_y,
                        scale,
                    }),
                    _ => None,
                },
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_tags(conn: &Connection) -> Result<Vec<TagItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT tags.id, tags.name, tags.color, COUNT(note_tags.note_id), tags.created_at
             FROM tags
             LEFT JOIN note_tags ON note_tags.tag_id = tags.id
             GROUP BY tags.id, tags.name, tags.color, tags.created_at
             ORDER BY tags.created_at ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(TagItem {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                count: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_drawings(conn: &Connection) -> Result<Vec<DrawingItem>, String> {
    let mut statement = conn
        .prepare(
            "SELECT data
             FROM drawings
             ORDER BY z_index ASC, created_at ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            let data: String = row.get(0)?;
            serde_json::from_str::<DrawingItem>(&data).map_err(|error| {
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

fn load_notes(conn: &Connection) -> Result<Vec<StickyNote>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, canvas_id, title, content, content_json, x, y, width, height, color, rotation, z_index, pinned,
                    font_size, font_weight, text_align, decoration, created_at, updated_at
             FROM notes
             ORDER BY pinned ASC, z_index ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(StickyNote {
                id: row.get(0)?,
                canvas_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                content_json: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|value| serde_json::from_str(&value).ok()),
                x: row.get(5)?,
                y: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                color: row.get(9)?,
                rotation: row.get(10)?,
                z_index: row.get(11)?,
                pinned: row.get::<_, i64>(12)? != 0,
                tags: Vec::new(),
                font_size: row.get(13)?,
                font_weight: row.get(14)?,
                text_align: row.get(15)?,
                decoration: row.get(16)?,
                checked_items: None,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_note_tags(conn: &Connection, note_id: &str) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare("SELECT tag_id FROM note_tags WHERE note_id = ?1 ORDER BY tag_id ASC")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![note_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_checklist_items(
    conn: &Connection,
    note_id: &str,
) -> Result<Option<Vec<ChecklistItem>>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, text, checked FROM checklist_items
             WHERE note_id = ?1
             ORDER BY sort_order ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![note_id], |row| {
            let checked: i64 = row.get(2)?;
            Ok(ChecklistItem {
                id: row.get(0)?,
                text: row.get(1)?,
                checked: checked != 0,
            })
        })
        .map_err(|error| error.to_string())?;
    let items = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok((!items.is_empty()).then_some(items))
}
