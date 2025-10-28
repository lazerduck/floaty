# Floaty — a tiny Linux floating toolbar (Tauri)

A minimal, always‑on‑top, frameless widget for Linux that gives you a floating taskbar with:

* Media controls via MPRIS (works with Spotify, VLC, mpv, etc.)
* A tiny clock
* A simple Pomodoro timer
* Drag anywhere to move; right‑click to show tray menu; ESC to hide

This uses **Tauri** (Rust backend, tiny footprint) + vanilla HTML/JS for the UI.

---

## 0) Prereqs

* Rust toolchain (`rustup`)
* Node 18+ (for building the web assets)
* Tauri bundler: `cargo install tauri-cli`
* Linux with DBus (standard on most distros)

---

## 1) Project structure

```
floaty/
  src/
    index.html
    style.css
    main.js
  src-tauri/
    Cargo.toml
    tauri.conf.json
    src/
      main.rs
  package.json
```

---

## 2) package.json

```json
{
  "name": "floaty",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build"
  },
  "devDependencies": {}
}
```

---

## 3) src/index.html

```html
<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Floaty</title>
  <link rel="stylesheet" href="./style.css" />
</head>
<body>
  <div id="bar" class="drag">
    <div class="section drag">🎧</div>

    <div class="section">
      <button id="prev" class="nodrag">⏮</button>
      <button id="play" class="nodrag">⏯</button>
      <button id="next" class="nodrag">⏭</button>
      <span id="track" title="track" class="truncate"></span>
    </div>

    <div class="section">
      <span id="clock"></span>
    </div>

    <div class="section">
      <button id="pomodoro" class="nodrag" title="Start/Stop 25:00">🍅 25:00</button>
      <span id="pomo-status" class="mono"></span>
    </div>
  </div>

  <script type="module" src="./main.js"></script>
</body>
</html>
```

---

## 4) src/style.css

```css
:root { font-family: system-ui, sans-serif; }
body { margin: 0; }
#bar {
  -webkit-app-region: drag;
  position: fixed; inset: 0 auto auto 0;
  height: 56px; width: 420px;
  background: rgba(30, 30, 34, 0.92);
  border: 1px solid #444; border-radius: 12px;
  display: flex; align-items: center; gap: 12px;
  padding: 8px 12px; color: #eaeaea; user-select: none;
}
.section { display: flex; align-items: center; gap: 8px; }
button {
  -webkit-app-region: no-drag;
  border: 1px solid #555; background: #222; color: #eee;
  padding: 6px 8px; border-radius: 8px; cursor: pointer;
}
button:hover { filter: brightness(1.2); }
.truncate { max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace; }
.drag { -webkit-app-region: drag; }
.nodrag { -webkit-app-region: no-drag; }
```

---

## 5) src/main.js

```js
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";

const trackEl = document.getElementById("track");
const clockEl = document.getElementById("clock");
const pomoBtn = document.getElementById("pomodoro");
const pomoStatus = document.getElementById("pomo-status");

// Media controls
document.getElementById("play").addEventListener("click", () => invoke("toggle_play_pause"));
document.getElementById("prev").addEventListener("click", () => invoke("previous_track"));
document.getElementById("next").addEventListener("click", () => invoke("next_track"));

async function refreshTrack() {
  try {
    const info = await invoke("current_track");
    if (info && info.title) {
      trackEl.textContent = `${info.title}${info.artist ? " — " + info.artist : ""}`;
    } else {
      trackEl.textContent = "No player";
    }
  } catch { trackEl.textContent = "No player"; }
}
setInterval(refreshTrack, 2000);
refreshTrack();

// Clock
function tickClock(){
  const d = new Date();
  clockEl.textContent = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}
setInterval(tickClock, 1000); tickClock();

// Pomodoro (25/5 simple)
let timer = null; let remaining = 25*60; let onBreak = false;
function fmt(n){ const m = Math.floor(n/60), s = (n%60).toString().padStart(2,'0'); return `${m}:${s}`; }
function updatePomo(){ pomoStatus.textContent = onBreak ? `Break ${fmt(remaining)}` : `Focus ${fmt(remaining)}`; }
function stopPomo(){ clearInterval(timer); timer = null; remaining = 25*60; onBreak = false; updatePomo(); }
function startPomo(){ if(timer) { stopPomo(); return; } timer = setInterval(()=>{
  remaining--; if(remaining<=0){
    onBreak = !onBreak; remaining = onBreak ? 5*60 : 25*60;
    invoke("notify", { title: "Floaty", body: onBreak ? "Break time!" : "Back to focus." });
  }
  updatePomo();
},1000); updatePomo(); }

pomoBtn.addEventListener("click", startPomo);
updatePomo();

// ESC hides the window
window.addEventListener('keydown', (e)=>{ if(e.key==='Escape'){ appWindow.hide(); }});
```

---

## 6) src-tauri/Cargo.toml

```toml
[package]
name = "floaty"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "1", features = ["window-all", "system-tray", "notification", "shell-open"] }
mpris = "2"
once_cell = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tauri-build = { version = "1", features = [] }
```

---

## 7) src-tauri/src/main.rs

```rust
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
use mpris::{PlayerFinder, Player};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};
use serde::Serialize;

static PLAYER: Lazy<Mutex<Option<Player>>> = Lazy::new(|| Mutex::new(None));

fn ensure_player() -> Option<Player> {
  // Try cached first
  if let Some(p) = PLAYER.lock().unwrap().as_ref() { return Some(p.clone()); }
  // Else find active or first available
  let finder = PlayerFinder::new().ok()?;
  if let Some(p) = finder.find_active().ok().flatten() { *PLAYER.lock().unwrap() = Some(p.clone()); return Some(p); }
  if let Ok(mut iter) = finder.iter() { if let Some(Ok(p)) = iter.next() { *PLAYER.lock().unwrap() = Some(p.clone()); return Some(p); } }
  None
}

#[derive(Serialize)]
struct TrackInfo { title: Option<String>, artist: Option<String> }

#[tauri::command]
fn toggle_play_pause() { if let Some(p) = ensure_player() { let _ = p.play_pause(); } }
#[tauri::command]
fn next_track() { if let Some(p) = ensure_player() { let _ = p.next(); } }
#[tauri::command]
fn previous_track() { if let Some(p) = ensure_player() { let _ = p.previous(); } }

#[tauri::command]
fn current_track() -> Option<TrackInfo> {
  let p = ensure_player()?;
  if let Ok(md) = p.get_metadata() {
    let title = md.title().map(|s| s.to_string());
    let artist = md.artists().and_then(|a| a.get(0).cloned());
    return Some(TrackInfo { title, artist });
  }
  None
}

#[tauri::command]
fn notify(title: &str, body: &str) {
  let _ = tauri::api::notification::Notification::new("floaty").title(title).body(body).show();
}

fn main() {
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");
  let show = CustomMenuItem::new("show".to_string(), "Show");
  let tray_menu = SystemTrayMenu::new().add_item(show).add_item(quit);
  let tray = SystemTray::new().with_menu(tray_menu);

  tauri::Builder::default()
    .system_tray(tray)
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => {
        match id.as_str() {
          "quit" => { std::process::exit(0); }
          "show" => {
            let w = app.get_window("main").unwrap();
            let _ = w.show(); let _ = w.set_focus();
          }
          _ => {}
        }
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![toggle_play_pause, next_track, previous_track, current_track, notify])
    .setup(|app| {
      let w = app.get_window("main").unwrap();
      let _ = w.set_always_on_top(true);
      let _ = w.set_decorations(false);
      let _ = w.set_skip_taskbar(true);
      #[cfg(target_os = "linux")]
      {
        use tauri::LogicalSize;
        let _ = w.set_size(LogicalSize::new(420.0, 56.0));
        let _ = w.set_position(tauri::LogicalPosition::new(10.0, 10.0));
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri app");
}
```

---

## 8) src-tauri/tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "build": { "beforeDevCommand": "", "beforeBuildCommand": "" },
  "package": { "productName": "Floaty", "version": "0.1.0" },
  "tauri": {
    "windows": [
      {
        "title": "Floaty",
        "label": "main",
        "fullscreen": false,
        "resizable": false,
        "decorations": false,
        "transparent": false,
        "alwaysOnTop": true,
        "width": 420,
        "height": 56,
        "skipTaskbar": true
      }
    ],
    "systemTray": { "iconPath": "icons/icon.png", "menuOnLeftClick": true },
    "allowlist": {
      "all": false,
      "window": { "all": true },
      "shell": { "open": true },
      "notification": { "all": true }
    }
  }
}
```

---

## 9) Setup & run

```bash
# from inside ./floaty
npm install
cargo add --manifest-path src-tauri/Cargo.toml mpris
npm run dev
# or build a bundle
npm run build
```

### Notes

* MPRIS: any player that exposes MPRIS (Spotify, VLC, mpv, Rhythmbox, etc.) will be controllable.
* If you want **click‑through** (so it’s a true overlay), we can toggle window mouse events on/off from Rust (advanced).
* For **auto‑start**, add a desktop entry to `~/.config/autostart/floaty.desktop`.

---

## 10) Nice next steps

* Track artwork (fetch `mpris:artUrl` and render in the bar)
* Global shortcuts (e.g., Ctrl+Alt+Space to show/hide)
* Volume slider + mic mute (PulseAudio/PipeWire via `libpulse-binding` / `pipewire` crates)
* Clipboard history (Rust side) exposed to UI
* Quick‑notes popup (markdown)
* Theming: dark/light, size presets

```
[theming]
  size = "compact|cozy|roomy"
  accent = "blue|green|purple|auto"
```

---

## 11) Troubleshooting

* **"No player"**: start a player first; Spotify/mpv should register on DBus automatically.
* **Wayland dragging**: Tauri’s drag region works; if you’re on Wayland + fractional scaling, try XWayland fallback for now.
* **Tray icon missing**: install a tray host in your DE if it’s disabled.
