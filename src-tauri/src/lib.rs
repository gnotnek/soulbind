mod audio;
mod bindings;
mod commands;
mod error;
mod models;
mod shortcuts;
mod shortcuts_tauri;
mod state;
mod storage;
mod tray;

use commands::{
    add_binding, delete_binding, duplicate_binding, list_bindings, play_binding, set_all_enabled,
    set_binding_enabled, stop_all, update_binding,
};
use shortcuts_tauri::rebuild_shortcuts;
use state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::load())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }

                    let state = app.state::<AppState>();
                    if let Some(binding_id) = state.shortcut_binding_id(shortcut.id()) {
                        let _ = state.play_binding(&binding_id);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tray::setup(app)?;
            let state = app.state::<AppState>();
            rebuild_shortcuts(app.handle(), &state)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_bindings,
            add_binding,
            update_binding,
            delete_binding,
            duplicate_binding,
            set_binding_enabled,
            set_all_enabled,
            play_binding,
            stop_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running SoulBind");
}
