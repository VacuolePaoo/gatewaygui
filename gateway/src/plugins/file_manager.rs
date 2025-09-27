use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
}

pub struct FileManagerState {
    pub selected_files: Mutex<Vec<FileInfo>>,
}

#[tauri::command]
pub fn set_selected_files(
    files: Vec<String>,
    state: State<FileManagerState>,
) -> Result<(), String> {
    let mut selected_files = state.selected_files.lock().unwrap();
    *selected_files = files.into_iter().map(|path| FileInfo { path }).collect();
    Ok(())
}

#[tauri::command]
pub fn get_selected_files(
    state: State<FileManagerState>,
) -> Result<Vec<String>, String> {
    let selected_files = state.selected_files.lock().unwrap();
    Ok(selected_files.iter().map(|file| file.path.clone()).collect())
}

#[tauri::command]
pub fn clear_selected_files(
    state: State<FileManagerState>,
) -> Result<(), String> {
    let mut selected_files = state.selected_files.lock().unwrap();
    selected_files.clear();
    Ok(())
}