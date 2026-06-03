use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct DatabaseLoadResult {
    pub(crate) data: String,
    pub(crate) db_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppData {
    pub(crate) version: i64,
    pub(crate) canvases: Vec<CanvasItem>,
    pub(crate) notes: Vec<StickyNote>,
    #[serde(default)]
    pub(crate) drawings: Vec<DrawingItem>,
    #[serde(default)]
    pub(crate) images: Vec<CanvasImage>,
    pub(crate) tags: Vec<TagItem>,
    pub(crate) settings: AppSettings,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanvasItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    #[serde(default)]
    pub(crate) sort_order: i64,
    pub(crate) deleted_at: Option<String>,
    pub(crate) viewport: Option<ViewportState>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ViewportState {
    pub(crate) offset_x: f64,
    pub(crate) offset_y: f64,
    pub(crate) scale: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StickyNote {
    pub(crate) id: String,
    pub(crate) canvas_id: String,
    pub(crate) title: Option<String>,
    pub(crate) content: String,
    pub(crate) content_json: Option<serde_json::Value>,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) color: String,
    pub(crate) rotation: f64,
    pub(crate) z_index: i64,
    #[serde(default)]
    pub(crate) pinned: bool,
    pub(crate) previous_z_index: Option<i64>,
    pub(crate) tags: Vec<String>,
    pub(crate) font_size: f64,
    pub(crate) font_weight: String,
    pub(crate) text_align: String,
    pub(crate) decoration: Option<String>,
    pub(crate) checked_items: Option<Vec<ChecklistItem>>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChecklistItem {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) checked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DrawingPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DrawingItem {
    pub(crate) id: String,
    pub(crate) canvas_id: String,
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) points: Option<Vec<DrawingPoint>>,
    pub(crate) start: Option<DrawingPoint>,
    pub(crate) end: Option<DrawingPoint>,
    pub(crate) x: Option<f64>,
    pub(crate) y: Option<f64>,
    pub(crate) width: Option<f64>,
    pub(crate) height: Option<f64>,
    pub(crate) text: Option<String>,
    pub(crate) font_size: Option<f64>,
    pub(crate) scale: Option<f64>,
    pub(crate) color: String,
    pub(crate) stroke_width: f64,
    pub(crate) z_index: i64,
    #[serde(default)]
    pub(crate) pinned: bool,
    pub(crate) previous_z_index: Option<i64>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanvasImage {
    pub(crate) id: String,
    pub(crate) canvas_id: String,
    pub(crate) file_name: String,
    pub(crate) original_name: Option<String>,
    pub(crate) content_hash: Option<String>,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) rotation: f64,
    pub(crate) z_index: i64,
    #[serde(default)]
    pub(crate) pinned: bool,
    pub(crate) previous_z_index: Option<i64>,
    #[serde(default = "default_true")]
    pub(crate) show_background: bool,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TagItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) color: String,
    pub(crate) count: i64,
    #[serde(default)]
    pub(crate) sort_order: i64,
    pub(crate) created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    pub(crate) theme: String,
    pub(crate) default_font_size: f64,
    pub(crate) show_grid: bool,
    pub(crate) random_rotation: bool,
    pub(crate) note_shadow: bool,
    pub(crate) auto_save: bool,
    #[serde(default = "default_chinese_font_family")]
    pub(crate) chinese_font_family: String,
    #[serde(default = "default_english_font_family")]
    pub(crate) english_font_family: String,
    #[serde(default = "default_monospace_font_family")]
    pub(crate) monospace_font_family: String,
    #[serde(default)]
    pub(crate) image_library_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportedImageFile {
    pub(crate) file_name: String,
    pub(crate) original_name: String,
    pub(crate) content_hash: String,
    pub(crate) path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClipboardImageData {
    pub(crate) bytes: Vec<u8>,
    pub(crate) original_name: String,
    pub(crate) mime_type: String,
}

fn default_chinese_font_family() -> String {
    "Xiaolai, Microsoft YaHei".to_string()
}

fn default_english_font_family() -> String {
    "Segoe Print, Comic Sans MS".to_string()
}

fn default_monospace_font_family() -> String {
    "Consolas, Cascadia Mono, monospace".to_string()
}

fn default_true() -> bool {
    true
}
