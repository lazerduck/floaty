use tauri::{AppHandle, Manager, State};
mod clients;
mod commands;
use commands::mpv_commands;

mod json_store;
use std::sync::Mutex;

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
fn add_item(
    value: String,
    app: AppHandle,
    state: State<'_, Mutex<json_store::Store>>,
) -> Result<Vec<String>, String> {
    let mut s = match state.lock() {
        Ok(s) => s,
        Err(_) => return Err("Failed to lock state".into()),
    };
    s.items.push(value);
    json_store::save_store(&app, &s)?;
    Ok(s.items.clone())
}

#[tauri::command]
fn delete_item(
    index: usize,
    app: AppHandle,
    state: State<'_, Mutex<json_store::Store>>,
) -> Result<Vec<String>, String> {
    let mut s = state.lock().unwrap();
    if index < s.items.len() {
        s.items.remove(index);
        json_store::save_store(&app, &s)?;
        Ok(s.items.clone())
    } else {
        Err(format!("index {} out of range", index))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle();

            let store = json_store::load_store(&app_handle);

            app.manage(mpv_commands::create_client(&app.handle())?);
            app.manage(Mutex::new(store));
            
            let main = app
                .get_webview_window("main")
                .expect("main window not found");

            let app_handle = app.handle().clone();
            main.on_window_event(move |_| {
                if let Some(state) = app_handle.try_state::<mpv_commands::AppAudio>() {
                    if let Ok(mut client) = state.mpv.lock() {
                        let _ = client.shutdown(); // send quit to mpv, best-effort
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            run_neofetch,
            list_items,
            add_item,
            delete_item,
            mpv_commands::mpv_play,
            mpv_commands::mpv_pause,
            mpv_commands::mpv_resume,
            mpv_commands::mpv_stop,
            mpv_commands::mpv_set_volume,
            mpv_commands::mpv_seek
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
