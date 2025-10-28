#[macro_export]
macro_rules! mpv_command_list {
  () => {
    mpv_commands::mpv_play,
    mpv_commands::mpv_pause,
    mpv_commands::mpv_resume,
    mpv_commands::mpv_stop,
    mpv_commands::mpv_set_volume,
    mpv_commands::mpv_seek
  };
}
use tauri::{AppHandle, State, Manager};
use std::{sync::Mutex, path::PathBuf};

use crate::clients::mpv_client;

pub struct AppAudio {
    mpv: Mutex<mpv_client::MpvClient>,
}

#[tauri::command]
pub fn mpv_play(path: String, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.play(path)
}

#[tauri::command]
pub fn mpv_pause(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.pause()
}

#[tauri::command]
pub fn mpv_resume(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.resume()
}

#[tauri::command]
pub fn mpv_stop(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.stop()
}

#[tauri::command]
pub fn mpv_set_volume(vol: f64, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.set_volume(vol)
}

#[tauri::command]
pub fn mpv_seek(seconds: f64, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.seek(seconds)
}

fn ipc_socket_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("mpv.sock")
}

pub fn create_client(app: &AppHandle) -> Result<AppAudio, String> {
    let sock = ipc_socket_path(&app);
    if let Some(dir) = sock.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("mkdir app data dir: {e}"))?;
    }
    let client = mpv_client::MpvClient::new(sock);

    Ok(AppAudio {
        mpv: Mutex::new(client),
    })
}