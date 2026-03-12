use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Holds the master PTY handle and child writer
pub struct PtyState {
    pub writer: Option<Arc<Mutex<Box<dyn Write + Send>>>>,
    pub master: Option<Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>>,
}

impl Default for PtyState {
    fn default() -> Self {
        Self {
            writer: None,
            master: None,
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct PtyOutputPayload {
    data: String,
}

/// Spawn a new PTY session with the given shell
#[tauri::command]
pub async fn spawn_pty(
    app: AppHandle,
    state: tauri::State<'_, Arc<Mutex<PtyState>>>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let pty_system = native_pty_system();

    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Failed to open PTY: {}", e))?;

    // Determine what shell to use
    let shell = if cfg!(windows) {
        "powershell.exe".to_string()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    };

    let mut cmd = CommandBuilder::new(&shell);
    if cfg!(windows) {
        cmd.arg("-NoLogo");
    }

    let _child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("Failed to spawn shell: {}", e))?;

    // Get writer for sending input to PTY
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

    // Store writer in state
    {
        let mut pty_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        pty_state.writer = Some(Arc::new(Mutex::new(writer)));
    }

    // Spawn a thread to read PTY output and emit to frontend
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("Failed to clone reader: {}", e))?;

    // Store master for resize
    {
        let mut pty_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        pty_state.master = Some(Arc::new(Mutex::new(pair.master)));
    }

    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Convert to string, handling invalid UTF-8 gracefully
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app.emit("pty-output", PtyOutputPayload { data });
                }
                Err(_) => break,
            }
        }
    });

    Ok(())
}

/// Write data to the PTY (user keyboard input)
#[tauri::command]
pub async fn write_to_pty(
    state: tauri::State<'_, Arc<Mutex<PtyState>>>,
    data: String,
) -> Result<(), String> {
    let pty_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref writer) = pty_state.writer {
        let mut w = writer.lock().map_err(|e| format!("Writer lock error: {}", e))?;
        w.write_all(data.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        w.flush().map_err(|e| format!("Flush error: {}", e))?;
    } else {
        return Err("PTY not spawned yet".into());
    }
    Ok(())
}

/// Resize the PTY
#[tauri::command]
pub async fn resize_pty(
    state: tauri::State<'_, Arc<Mutex<PtyState>>>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let pty_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref master) = pty_state.master {
        let m = master.lock().map_err(|e| format!("Master lock error: {}", e))?;
        m.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("Resize error: {}", e))?;
    }
    Ok(())
}
