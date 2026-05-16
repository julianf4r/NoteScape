use std::path::Path;

use rusqlite::{params, Connection};

use super::{open_database, read::load_structured_data};
use crate::defaults::default_app_data;
use crate::models::{AppData, AppSettings, CanvasImage, CanvasItem, DrawingItem, StickyNote, TagItem};

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
    tx.execute("DELETE FROM drawings", [])
        .map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM images", [])
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
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('chinese_font_family', ?1)",
        params![app_data.settings.chinese_font_family],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('english_font_family', ?1)",
        params![app_data.settings.english_font_family],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('monospace_font_family', ?1)",
        params![app_data.settings.monospace_font_family],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO app_meta (key, value) VALUES ('image_library_path', ?1)",
        params![app_data.settings.image_library_path],
    )
    .map_err(|error| error.to_string())?;

    for canvas in &app_data.canvases {
        tx.execute(
            "INSERT INTO canvases (id, name, description, created_at, updated_at, sort_order, deleted_at, viewport_offset_x, viewport_offset_y, viewport_scale)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                canvas.id,
                canvas.name,
                canvas.description,
                canvas.created_at,
                canvas.updated_at,
                canvas.sort_order,
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
            "INSERT INTO tags (id, name, color, count, sort_order, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![tag.id, tag.name, tag.color, 0, tag.sort_order, tag.created_at],
        )
        .map_err(|error| error.to_string())?;
    }

    for drawing in &app_data.drawings {
        insert_drawing(&tx, drawing)?;
    }

    for image in &app_data.images {
        insert_image(&tx, image)?;
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

fn insert_drawing(conn: &Connection, drawing: &DrawingItem) -> Result<(), String> {
    conn.execute(
        "INSERT INTO drawings (id, canvas_id, data, z_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            drawing.id,
            drawing.canvas_id,
            serde_json::to_string(drawing).map_err(|error| error.to_string())?,
            drawing.z_index,
            drawing.created_at,
            drawing.updated_at
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn insert_image(conn: &Connection, image: &CanvasImage) -> Result<(), String> {
    conn.execute(
        "INSERT INTO images (id, canvas_id, data, z_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            image.id,
            image.canvas_id,
            serde_json::to_string(image).map_err(|error| error.to_string())?,
            image.z_index,
            image.created_at,
            image.updated_at
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn upsert_canvas(conn: &Connection, canvas: &CanvasItem) -> Result<(), String> {
    conn.execute(
        "INSERT INTO canvases (id, name, description, created_at, updated_at, sort_order, deleted_at, viewport_offset_x, viewport_offset_y, viewport_scale)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            updated_at = excluded.updated_at,
            sort_order = excluded.sort_order,
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
            canvas.sort_order,
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
        "INSERT INTO tags (id, name, color, count, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            color = excluded.color,
            sort_order = excluded.sort_order",
        params![tag.id, tag.name, tag.color, 0, tag.sort_order, tag.created_at],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn upsert_drawing(conn: &Connection, drawing: &DrawingItem) -> Result<(), String> {
    conn.execute(
        "INSERT INTO drawings (id, canvas_id, data, z_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            canvas_id = excluded.canvas_id,
            data = excluded.data,
            z_index = excluded.z_index,
            updated_at = excluded.updated_at",
        params![
            drawing.id,
            drawing.canvas_id,
            serde_json::to_string(drawing).map_err(|error| error.to_string())?,
            drawing.z_index,
            drawing.created_at,
            drawing.updated_at
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn upsert_image(conn: &Connection, image: &CanvasImage) -> Result<(), String> {
    conn.execute(
        "INSERT INTO images (id, canvas_id, data, z_index, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            canvas_id = excluded.canvas_id,
            data = excluded.data,
            z_index = excluded.z_index,
            updated_at = excluded.updated_at",
        params![
            image.id,
            image.canvas_id,
            serde_json::to_string(image).map_err(|error| error.to_string())?,
            image.z_index,
            image.created_at,
            image.updated_at
        ],
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
        ("chinese_font_family", settings.chinese_font_family.clone()),
        ("english_font_family", settings.english_font_family.clone()),
        (
            "monospace_font_family",
            settings.monospace_font_family.clone(),
        ),
        ("image_library_path", settings.image_library_path.clone()),
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

fn bool_to_text(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
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

pub(crate) fn delete_drawing(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM drawings WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_image(conn: &Connection, id: String) -> Result<(), String> {
    conn.execute("DELETE FROM images WHERE id = ?1", params![id])
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_drawings_by_canvas(
    conn: &Connection,
    canvas_id: String,
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM drawings WHERE canvas_id = ?1",
        params![canvas_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(crate) fn delete_images_by_canvas(conn: &Connection, canvas_id: String) -> Result<(), String> {
    conn.execute("DELETE FROM images WHERE canvas_id = ?1", params![canvas_id])
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
