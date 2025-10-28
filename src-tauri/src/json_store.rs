use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use tauri::{Manager, AppHandle};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Store {
    pub items: Vec<String>,
}

fn store_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap().join("store.json")
}

pub fn load_store(app: &AppHandle) -> Store {
    let p = store_path(app);
    match fs::read(&p) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Store::default(),
    }
}

pub fn save_store(app: &AppHandle, store: &Store) -> Result<(), String> {
    let p = store_path(app);
    if let Some(dir) = p.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
    let json = serde_json::to_vec_pretty(store).map_err(|e| e.to_string())?;
    let mut f = fs::File::create(&p).map_err(|e| e.to_string())?;
    f.write_all(&json).map_err(|e| e.to_string())
}
