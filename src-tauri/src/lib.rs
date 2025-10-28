use tauri::{AppHandle, State, Manager};

mod json_store;
mod mpv_client;

use mpv_client::MpvClient;
use std::{path::PathBuf, sync::Mutex};

struct AppAudio {
    mpv: Mutex<MpvClient>,
}

fn ipc_socket_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("mpv.sock")
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn run_neofetch() -> String {
    use std::process::Command;

    // Try to run `neofetch --stdout` to get machine info as text.
    // If neofetch is missing or errors, return a helpful message.
    match Command::new("neofetch").arg("--stdout").output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                // Include both stdout/stderr where available
                let mut s = String::new();
                if !output.stdout.is_empty() {
                    s.push_str(&String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    if !s.is_empty() {
                        s.push_str("\n");
                    }
                    s.push_str("STDERR:\n");
                    s.push_str(&String::from_utf8_lossy(&output.stderr));
                }
                s.push_str(&format!("\nExit status: {}", output.status));
                s
            }
        }
        Err(e) => format!("Failed to spawn `neofetch`: {}\nMake sure `neofetch` is installed and available on PATH.", e),
    }
}

#[tauri::command]
fn list_items(state: State<'_, Mutex<json_store::Store>>) -> Vec<String> {
    state.lock().unwrap().items.clone()
}

#[tauri::command]
fn add_item(value: String, app: AppHandle, state: State<'_, Mutex<json_store::Store>>) -> Result<Vec<String>, String> {
  let mut s = match state.lock() {
      Ok(s) => s,
      Err(_) => return Err("Failed to lock state".into()),
  };
  s.items.push(value);
  json_store::save_store(&app, &s)?;
  Ok(s.items.clone())
}

#[tauri::command]
fn delete_item(index: usize, app: AppHandle, state: State<'_, Mutex<json_store::Store>>) -> Result<Vec<String>, String> {
    let mut s = state.lock().unwrap();
    if index < s.items.len() {
        s.items.remove(index);
        json_store::save_store(&app, &s)?;
        Ok(s.items.clone())
    } else {
        Err(format!("index {} out of range", index))
    }
}

#[tauri::command]
fn mpv_play(path: String, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.play(path)
}

#[tauri::command]
fn mpv_pause(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.pause()
}

#[tauri::command]
fn mpv_resume(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.resume()
}

#[tauri::command]
fn mpv_stop(audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.stop()
}

#[tauri::command]
fn mpv_set_volume(vol: f64, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.set_volume(vol)
}

#[tauri::command]
fn mpv_seek(seconds: f64, audio: State<'_, AppAudio>) -> Result<(), String> {
  let mut mpv = audio.mpv.lock().unwrap();
  mpv.seek(seconds)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let store = json_store::load_store(&app.handle());
            let sock = ipc_socket_path(&app.handle());
            if let Some(dir) = sock.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("mkdir app data dir: {e}"))?;
            }
            let client = MpvClient::new(sock);
            app.manage(AppAudio {
                mpv: Mutex::new(client),
            });
            app.manage(Mutex::new(store));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            run_neofetch, 
            list_items, 
            add_item, 
            delete_item,
            mpv_play,
            mpv_pause,
            mpv_resume,
            mpv_stop,
            mpv_set_volume,
            mpv_seek,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

