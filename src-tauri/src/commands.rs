use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{AppSettings, AudioInputDevice, AudioOutputDevice, BindingInput, SoundBinding};
use crate::shortcuts_tauri::rebuild_shortcuts;
use crate::state::AppState;

#[tauri::command]
pub fn list_bindings(state: State<'_, AppState>) -> AppResult<Vec<SoundBinding>> {
    state.bindings()
}

#[tauri::command]
pub fn add_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    input: BindingInput,
) -> AppResult<Vec<SoundBinding>> {
    state.add_binding(Uuid::new_v4().to_string(), input)?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn update_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    input: BindingInput,
) -> AppResult<Vec<SoundBinding>> {
    state.update_binding(&id, input)?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn delete_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Vec<SoundBinding>> {
    state.delete_binding(&id)?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn duplicate_binding(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> AppResult<Vec<SoundBinding>> {
    state.duplicate_binding(&id, Uuid::new_v4().to_string())?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn set_binding_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> AppResult<Vec<SoundBinding>> {
    state.set_binding_enabled(&id, enabled)?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn set_all_enabled(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> AppResult<Vec<SoundBinding>> {
    state.set_all_enabled(enabled)?;
    rebuild_shortcuts(&app, &state)?;
    state.bindings()
}

#[tauri::command]
pub fn play_binding(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.play_binding(&id)
}

#[tauri::command]
pub fn stop_all(state: State<'_, AppState>) -> AppResult<()> {
    state.stop_all()
}

#[tauri::command]
pub fn list_output_devices(state: State<'_, AppState>) -> AppResult<Vec<AudioOutputDevice>> {
    state.output_devices()
}

#[tauri::command]
pub fn list_input_devices(state: State<'_, AppState>) -> AppResult<Vec<AudioInputDevice>> {
    state.input_devices()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<AppSettings> {
    state.settings()
}

#[tauri::command]
pub fn set_output_device(
    state: State<'_, AppState>,
    device_name: Option<String>,
) -> AppResult<AppSettings> {
    state.set_output_device(device_name)
}

#[tauri::command]
pub fn set_input_device(
    state: State<'_, AppState>,
    device_name: Option<String>,
) -> AppResult<AppSettings> {
    state.set_input_device(device_name)
}

#[tauri::command]
pub fn set_orchestrator_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> AppResult<AppSettings> {
    state.set_orchestrator_enabled(enabled)
}
