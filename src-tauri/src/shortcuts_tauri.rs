use std::collections::HashMap;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::error::AppResult;
use crate::shortcuts::{build_shortcut_plan, index_insert, ShortcutPlanStatus};
use crate::state::AppState;

pub fn rebuild_shortcuts(app: &AppHandle, state: &AppState) -> AppResult<()> {
    let _ = app.global_shortcut().unregister_all();

    let mut index = HashMap::new();

    state.with_bindings_mut(|bindings| {
        for item in build_shortcut_plan(bindings.as_slice()) {
            match item.status {
                ShortcutPlanStatus::Disabled => {
                    bindings.update_status(&item.binding_id, "disabled");
                }
                ShortcutPlanStatus::Duplicate => {
                    bindings.update_status(&item.binding_id, "duplicate");
                }
                ShortcutPlanStatus::Invalid(error) => {
                    bindings.update_status(&item.binding_id, format!("invalid shortcut: {error}"));
                }
                ShortcutPlanStatus::Ready => match item.shortcut.parse::<Shortcut>() {
                    Ok(shortcut) => match app.global_shortcut().register(shortcut) {
                        Ok(()) => {
                            index_insert(&mut index, shortcut.id(), &item.binding_id);
                            bindings.update_status(&item.binding_id, "registered");
                        }
                        Err(error) => bindings.update_status(
                            &item.binding_id,
                            format!("registration failed: {error}"),
                        ),
                    },
                    Err(error) => bindings
                        .update_status(&item.binding_id, format!("invalid shortcut: {error}")),
                },
            }
        }
    })?;

    state.replace_shortcut_index(index)?;
    state.save()
}
