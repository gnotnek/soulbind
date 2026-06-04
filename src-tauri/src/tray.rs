use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Manager,
};

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show SoulBind", true, None::<&str>)?;
    let stop_all = MenuItem::with_id(app, "stop_all", "Stop All Sounds", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &stop_all, &quit])?;

    TrayIconBuilder::new()
        .tooltip("SoulBind")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "stop_all" => {
                let state = app.state::<crate::state::AppState>();
                let _ = state.stop_all();
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
