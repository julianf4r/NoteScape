mod connection;
mod read;
mod schema;
mod write;

use crate::models::AppData;

pub(crate) use connection::open_database;
pub(crate) use read::{load_existing_database, load_or_seed};
pub(crate) use write::{
    create_default_database, delete_canvas, delete_note, delete_notes_by_canvas, delete_tag,
    remove_canvas_forever, restore_canvas, save_settings, save_structured_data, upsert_canvas,
    upsert_note, upsert_tag,
};

pub(crate) fn parse_app_data(data: &str) -> Result<AppData, String> {
    serde_json::from_str(data).map_err(|error| error.to_string())
}
