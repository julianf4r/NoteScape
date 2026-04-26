use rusqlite::Connection;

pub(crate) fn init_schema(conn: &Connection) -> Result<(), String> {
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
            sort_order INTEGER NOT NULL DEFAULT 0,
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

        CREATE TABLE IF NOT EXISTS drawings (
            id TEXT PRIMARY KEY,
            canvas_id TEXT NOT NULL,
            data TEXT NOT NULL,
            z_index INTEGER NOT NULL,
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
        CREATE INDEX IF NOT EXISTS idx_drawings_canvas_id ON drawings(canvas_id);
        CREATE INDEX IF NOT EXISTS idx_note_tags_tag_id ON note_tags(tag_id);
        ",
    )
    .map_err(|error| error.to_string())?;
    add_column_if_missing(conn, "canvases", "viewport_offset_x", "REAL")?;
    add_column_if_missing(conn, "canvases", "viewport_offset_y", "REAL")?;
    add_column_if_missing(conn, "canvases", "viewport_scale", "REAL")?;
    add_column_if_missing(conn, "canvases", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
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
