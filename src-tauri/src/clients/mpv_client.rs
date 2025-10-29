use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

pub struct MpvClient {
    socket_path: PathBuf,
    child: Option<Child>,
}

impl MpvClient {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            child: None,
        }
    }

    pub fn ensure_running(&mut self) -> Result<(), String> {
        println!(
            "Ensuring mpv is running with socket at {}",
            self.socket_path.display()
        );
        #[cfg(unix)]
        if self.socket_path.exists() {
            match std::os::unix::net::UnixStream::connect(&self.socket_path) {
                Ok(_) => {
                    println!("mpv socket exists and is connectable");
                    return Ok(());
                }
                Err(e) => {
                    println!("mpv socket exists but not connectable: {}", e);
                    // Remove stale socket and respawn
                    let _ = fs::remove_file(&self.socket_path);
                }
            }
        }

        println!("Stopping existing mpv instance...");
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }

        let _ = fs::remove_file(&self.socket_path);
        println!("Starting mpv...");
        let mpv = Command::new("mpv")
            .arg("--no-video")
            .arg("--idle=yes")
            .arg("--force-window=no")
            .arg("--really-quiet")
            .arg(format!("--input-ipc-server={}", self.socket_path.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn mpv: {e}"))?;

        self.child = Some(mpv);
        println!("Waiting for mpv IPC socket to become connectable...");
        #[cfg(unix)]
        {
            for _ in 0..40 {
                if self.socket_path.exists() {
                    match std::os::unix::net::UnixStream::connect(&self.socket_path) {
                        Ok(_) => {
                            println!("mpv socket became connectable");
                            return Ok(());
                        }
                        Err(_) => {}
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
            println!(
                "mpv IPC socket did not become connectable: {}",
                self.socket_path.display()
            );
            return Err("mpv IPC socket did not become connectable in time".into());
        }

        #[cfg(not(unix))]
        {
            for _ in 0..40 {
                if self.socket_path.exists() {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(50));
            }
            println!(
                "mpv IPC socket did not appear: {}",
                self.socket_path.display()
            );
            Err("mpv IPC socket did not appear in time".into())
        }
    }

    fn send_cmd(&mut self, payload: Value) -> Result<Value, String> {
        let t = self.ensure_running();
        if let Err(e) = t {
            println!("Failed to ensure mpv is running: {e}");
            return Err(e);
        }
        #[cfg(unix)]
        {
            let mut stream = UnixStream::connect(&self.socket_path)
                .map_err(|e| format!("mpv connect failed: {e}"))?;

            let mut data = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            data.push(b'\n');
            stream.write_all(&data).map_err(|e| e.to_string())?;

            let mut buf = Vec::with_capacity(1024);
            let mut byte = [0u8; 1];
            while let Ok(n) = stream.read(&mut byte) {
                if n == 0 {
                    break;
                }
                if byte[0] == b'\n' {
                    break;
                }
                buf.push(byte[0]);
            }

            if buf.is_empty() {
                return Ok(json!({"status":"ok"}));
            }
            let v: Value = serde_json::from_slice(&buf).map_err(|e| e.to_string())?;
            Ok(v)
        }

        #[cfg(not(unix))]
        {
            Err("mpv IPC over Unix socket is only implemented for Unix. On Windows, use \\\\.\\pipe\\NAME with named pipes.".into())
        }
    }

    pub fn play<P: AsRef<Path>>(&mut self, file: P) -> Result<(), String> {
        let p = file.as_ref().to_string_lossy().to_string();
        let payload = json!({ "command": ["loadfile", p, "replace"] });
        let _ = self.send_cmd(payload)?;
        let _ = self.resume()?;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), String> {
        let _ = self.send_cmd(json!({ "command": ["set_property", "pause", true]}))?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        let _ = self.send_cmd(json!({ "command": ["set_property", "pause", false] }))?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        let _ = self.send_cmd(json!({ "command": ["stop"] }))?;
        Ok(())
    }

    pub fn set_volume(&mut self, vol_0_100: f64) -> Result<(), String> {
        let _ = self.send_cmd(json!({ "command": ["set_property", "volume", vol_0_100] }))?;
        Ok(())
    }

    /// Optional: small seek helper (seconds can be negative)
    pub fn seek(&mut self, seconds: f64) -> Result<(), String> {
        let _ = self.send_cmd(json!({ "command": ["seek", seconds, "relative"] }))?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        println!("Shutting down mpv...");
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let _ = fs::remove_file(&self.socket_path);
        Ok(())
    }
}


impl Drop for MpvClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
