use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::defaults::{default_app_data, default_app_data_json};
use crate::models::{
    AppData, AppSettings, CanvasItem, ChecklistItem, StickyNote, TagItem, ViewportState,
};

pub(crate) fn open_database(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let conn = Connection::open(path).map_err(|error| error.to_string())?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| error.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS canvases (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            viewport_offset_x REAL,
            viewport_offset_y REAL,
            viewport_scale REAL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notes (
            id TEXT PRIMARY KEY,
            canvas_id TEXT NOT NULL,
            title TEXT,
            content TEXT NOT NULL,
            content_json TEXT,
            x REAL NOT NULL,
            y REAL NOT NULL,
            width REAL NOT NULL,
            height REAL NOT NULL,
            color TEXT NOT NULL,
            rotation REAL NOT NULL,
            z_index INTEGER NOT NULL,
            pinned INTEGER NOT NULL DEFAULT 0,
            font_size REAL NOT NULL,
            font_weight TEXT NOT NULL,
            text_align TEXT NOT NULL,
            decoration TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(canvas_id) REFERENCES canvases(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS note_tags (
            note_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY(note_id, tag_id),
            FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS checklist_items (
            id TEXT PRIMARY KEY,
            note_id TEXT NOT NULL,
            text TEXT NOT NULL,
            checked INTEGER NOT NULL,
            sort_order INTEGER NOT NULL,
            FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_notes_canvas_id ON notes(canvas_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id);
        ",
    )
    .map_err(|error| error.to_string())?;
    add_column_if_missing(conn, "canvases", "viewport_offset_x", "REAL")?;
    add_column_if_missing(conn, "canvases", "viewport_offset_y", "REAL")?;
    add_column_if_missing(conn, "canvases", "viewport_scale", "REAL")?;
    add_column_if_missing(conn, "notes", "content_json", "TEXT")?;
    add_column_if_missing(conn, "notes", "decoration", "TEXT")?;
    add_column_if_missing(conn, "notes", "pinned", "INTEGER NOT NULL DEFAULT 0")?;
    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| error.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    if !columns.iter().any(|name| name == column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn parse_app_data(data: &str) -> Result<AppData, String> {
    serde_json::from_str(data).map_err(|error| error.to_string())
}

fn is_database_empty(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM canvases", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    Ok(count == 0)
}

pub(crate) fn save_structured_data(
    conn: &mut Connection,
    app_data: &AppData,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;

    tx.execute("DELETE FROM checklist_items", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM note_tags", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM notes", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM tags", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM canvases", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM app_meta", [])
        .map_err(|error| error.to_string())?;

    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('version', ?1)",
        params![app_data.version.to_string()],
    )
    .map_err(|error| error.to_string())?;

    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('theme', ?1)",
        params![app_data.settings.theme],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('default_font_size', ?1)",
        params![app_data.settings.default_font_size.to_string()],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('show_grid', ?1)",
        params![bool_to_text(app_data.settings.show_grid)],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('random_rotation', ?1)",
        params![bool_to_text(app_data.settings.random_rotation)],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('note_shadow', ?1)",
        params![bool_to_text(app_data.settings.note_shadow)],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('auto_save', ?1)",
        params![bool_to_text(app_data.settings.auto_save)],
    )
    .map_err(|error| error.to_string())?;

    for canvas in &app_data.canvases {
        tx.execute(
            "INSERT INTO canvases (id, name, description, created_at, updated_at, deleted_at, viewport_offset_x, viewport_offset_y, viewport_scale)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                canvas.id,
                canvas.name,
                canvas.description,
                canvas.created_at,
                canvas.updated_at,
                canvas.deleted_at,
                canvas.viewport.as_ref().map(|viewport| viewport.offset_x),
                canvas.viewport.as_ref().map(|viewport| viewport.offset_y),
                canvas.viewport.as_ref().map(|viewport| viewport.scale)
            ],
        )
        .map_err(|error| error.to_string())?;
    }

    for tag in &app_data.tags {
        tx.execute(
            "INSERT INTO tags (id, name, color, count, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tag.id, tag.name, tag.color, 0, tag.created_at],
        )
        .map_err(|error| error.to_string())?;
    }

    for note in &app_data.notes {
        tx.execute(
            "INSERT INTO notes (
                id, canvas_id, title, content, content_json, x, y, width, height, color, rotation, z_index, pinned,
                font_size, font_weight, text_align, decoration, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                note.id,
                note.canvas_id,
                note.title,
                note.content,
                note.content_json.as_ref().map(|value| value.to_string()),
                note.x,
                note.y,
                note.width,
                note.height,
                note.color,
                note.rotation,
                note.z_index,
                note.pinned as i64,
                note.font_size,
                note.font_weight,
                note.text_align,
                note.decoration,
                note.created_at,
                note.updated_at
            ],
        )
        .map_err(|error| error.to_string())?;

        for tag_id in &note.tags {
            tx.execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![note.id, tag_id],
            )
            .map_err(|error| error.to_string())?;
        }

        if let Some(items) = &note.checked_items {
            for (index, item) in items.iter().enumerate() {
                tx.execute(
                    "INSERT INTO checklist_items (id, note_id, text, checked, sort_order)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        item.id,
                        note.id,
                        item.text,
                        item.checked as i64,
                        index as i64
                    ],
                )
                .map_err(|error| error.to_string())?;
            }
        }
    }

    tx.commit().map_err(|error| error.to_string())
}

pub(crate) fn upsert_canvas(conn: &Connection, canvas: &CanvasItem) -> Result<(), String> {
    conn.execute(
        "INSERT INTO canvases (id, name, description, created_at, updated_at, deleted_at, viewport_offset_x, viewport_offset_y, viewport_scale)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            updated_at = excluded.updated_at,
            deleted_at = excluded.deleted_at,
            viewport_offset_x = excluded.viewport_offset_x,
            viewport_offset_y = excluded.viewport_offset_y,
            viewport_scale = excluded.viewport_scale",
        params![
            canvas.id,
            canvas.name,
            canvas.description,
            canvas.created_at,
            canvas.updated_at,
            canvas.deleted_at,
            canvas.viewport.as_ref().map(|viewport| viewport.offset_x),
            canvas.viewport.as_ref().map(|viewport| viewport.offset_y),
            canvas.viewport.as_ref().map(|viewport| viewport.scale)
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn upsert_tag(conn: &Connection, tag: &TagItem) -> Result<(), String> {
    conn.execute(
        "INSERT INTO tags (id, name, color, count, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            color = excluded.color",
        params![tag.id, tag.name, tag.color, 0, tag.created_at],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn upsert_note(conn: &mut Connection, note: &StickyNote) -> Result<(), String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO notes (
            id, canvas_id, title, content, content_json, x, y, width, height, color, rotation, z_index, pinned,
            font_size, font_weight, text_align, decoration, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
        ON CONFLICT(id) DO UPDATE SET
            canvas_id = excluded.canvas_id,
            title = excluded.title,
            content = excluded.content,
            content_json = excluded.content_json,
            x = excluded.x,
            y = excluded.y,
            width = excluded.width,
            height = excluded.height,
            color = excluded.color,
            rotation = excluded.rotation,
            z_index = excluded.z_index,
            pinned = excluded.pinned,
            font_size = excluded.font_size,
            font_weight = excluded.font_weight,
            text_align = excluded.text_align,
            decoration = excluded.decoration,
            updated_at = excluded.updated_at",
        params![
            note.id,
            note.canvas_id,
            note.title,
            note.content,
            note.content_json.as_ref().map(|value| value.to_string()),
            note.x,
            note.y,
            note.width,
            note.height,
            note.color,
            note.rotation,
            note.z_index,
            note.pinned as i64,
            note.font_size,
            note.font_weight,
            note.text_align,
            note.decoration,
            note.created_at,
            note.updated_at
        ],
    )
    .map_err(|error| error.to_string())?;

    tx.execute("DELETE FROM note_tags WHERE note_id = ?1", params![note.id])
        .map_err(|error| error.to_string())?;
    for tag_id in &note.tags {
        tx.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            params![note.id, tag_id],
        )
        .map_err(|error| error.to_string())?;
    }

    tx.execute(
        "DELETE FROM checklist_items WHERE note_id = ?1",
        params![note.id],
    )
    .map_err(|error| error.to_string())?;
    if let Some(items) = &note.checked_items {
        for (index, item) in items.iter().enumerate() {
            tx.execute(
                "INSERT INTO checklist_items (id, note_id, text, checked, sort_order)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    item.id,
                    note.id,
                    item.text,
                    item.checked as i64,
                    index as i64
                ],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    tx.commit().map_err(|error| error.to_string())
}

pub(crate) fn save_settings(conn: &Connection, settings: &AppSettings) -> Result<(), String> {
    let entries = [
        ("theme", settings.theme.clone()),
        ("default_font_size", settings.default_font_size.to_string()),
        ("show_grid", bool_to_text(settings.show_grid).to_string()),
        (
            "random_rotation",
            bool_to_text(settings.random_rotation).to_string(),
        ),
        (
            "note_shadow",
            bool_to_text(settings.note_shadow).to_string(),
        ),
        ("auto_save", bool_to_text(settings.auto_save).to_string()),
    ];
    for (key, value) in entries {
        conn.execute(
            "INSERT INTO app_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
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
    };

    let canvases = load_canvases(conn)?;
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
    let conn = Connection::open(path)
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

fn bool_to_text(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
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

pub(crate) fn delete_canvas(
    conn: &Connection,
    id: String,
    deleted_at: String,
) -> Result<(), String> {
    conn.execute(
        "UPDATE canvases SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![deleted_at, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn restore_canvas(
    conn: &Connection,
    id: String,
    updated_at: String,
) -> Result<(), String> {
    conn.execute(
        "UPDATE canvases SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![updated_at, id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn remove_canvas_forever(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM canvases WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_note(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_notes_by_canvas(conn: &Connection, canvas_id: String) -> Result<(), String> {
    conn.execute("DELETE FROM notes WHERE canvas_id = ?1", params![canvas_id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_tag(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn create_default_database(path: &Path) -> Result<String, String> {
    let mut conn = open_database(path)?;
    let app_data = default_app_data();
    save_structured_data(&mut conn, &app_data)?;
    serde_json::to_string(&load_structured_data(&conn)?).map_err(|error| error.to_string())
}
