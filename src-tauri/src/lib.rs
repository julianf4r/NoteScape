mod commands;
mod db;
mod defaults;
mod models;
mod paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_app_data,
            commands::save_app_data,
            commands::reset_database_to_default,
            commands::set_database_path,
            commands::create_database,
            commands::save_canvas,
            commands::delete_canvas,
            commands::restore_canvas,
            commands::remove_canvas_forever,
            commands::save_note,
            commands::delete_note,
            commands::delete_notes_by_canvas,
            commands::save_tag,
            commands::delete_tag,
            commands::save_app_settings,
            commands::backup_database,
            commands::export_json_file,
            commands::import_json_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
