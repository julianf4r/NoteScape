use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::models::{AppData, AppSettings, CanvasItem, StickyNote};

fn now_iso() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub(crate) fn default_app_data() -> AppData {
    let created_at = now_iso();
    AppData {
        version: 1,
        canvases: vec![CanvasItem {
            id: "canvas-default".to_string(),
            name: "默认画布".to_string(),
            description: None,
            created_at: created_at.clone(),
            updated_at: created_at.clone(),
            deleted_at: None,
            viewport: None,
        }],
        notes: vec![StickyNote {
            id: "note-default".to_string(),
            canvas_id: "canvas-default".to_string(),
            title: None,
            content: String::new(),
            content_json: None,
            x: 360.0,
            y: 260.0,
            width: 260.0,
            height: 220.0,
            color: "yellow".to_string(),
            rotation: -1.5,
            z_index: 1,
            pinned: false,
            tags: Vec::new(),
            font_size: 18.0,
            font_weight: "normal".to_string(),
            text_align: "left".to_string(),
            decoration: Some("tape".to_string()),
            checked_items: None,
            created_at: created_at.clone(),
            updated_at: created_at,
        }],
        tags: Vec::new(),
        settings: AppSettings {
            theme: "light".to_string(),
            default_font_size: 18.0,
            show_grid: true,
            random_rotation: true,
            note_shadow: true,
            auto_save: true,
        },
    }
}

pub(crate) fn default_app_data_json() -> Result<String, String> {
    serde_json::to_string(&default_app_data()).map_err(|error| error.to_string())
}
