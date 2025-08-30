use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{Read, Write};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
    pub pixel_width: u16,
    pub pixel_height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub session_id: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct PtySession {
    pub id: String,
    pub size: TerminalSize,
    pub shell: String,
    pub working_dir: String,
}

pub struct PtyProcess {
    pub session: PtySession,
    pub writer: Arc<tokio::sync::Mutex<Option<Box<dyn std::io::Write + Send>>>>,
    pub master: Arc<std::sync::Mutex<Option<Box<dyn MasterPty + Send>>>>,
}

pub struct PtyManager {
    processes: Arc<Mutex<HashMap<String, PtyProcess>>>,
    output_sender: mpsc::UnboundedSender<TerminalOutput>,
}

impl PtyManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<TerminalOutput>) {
        let (output_sender, output_receiver) = mpsc::unbounded_channel();
        let manager = PtyManager {
            processes: Arc::new(Mutex::new(HashMap::new())),
            output_sender,
        };
        (manager, output_receiver)
    }

    pub fn create_session(
        &self,
        size: TerminalSize,
        shell: Option<String>,
        working_dir: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let session_id = Uuid::new_v4().to_string();
        
        let shell = shell.unwrap_or_else(|| {
            if cfg!(windows) {
                std::env::var("SHELL").unwrap_or_else(|_| "powershell.exe".to_string())
            } else {
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
            }
        });

        let working_dir = working_dir.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        let session = PtySession {
            id: session_id.clone(),
            size: size.clone(),
            shell: shell.clone(),
            working_dir: working_dir.clone(),
        };

        // Start the shell process and get a handle to stdin
        let (writer_handle, master_handle) = self.start_shell_process(&session_id, &shell, &working_dir, size.clone())?;

        // Track the process so we can write to it later
        let process = PtyProcess {
            session,
            writer: writer_handle,
            master: master_handle,
        };

        self.processes.lock().unwrap().insert(session_id.clone(), process);

        Ok(session_id)
    }

    pub fn resize_session(
        &self,
        session_id: &str,
        size: TerminalSize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proc) = self.processes.lock().unwrap().get_mut(session_id) {
            proc.session.size = size.clone();
            let cols = size.cols;
            let rows = size.rows;
            let pxw = size.pixel_width;
            let pxh = size.pixel_height;
            let master = proc.master.clone();
            tauri::async_runtime::spawn_blocking(move || {
                if let Some(ref mut master_opt) = *master.lock().unwrap() {
                    let _ = master_opt.resize(PtySize {
                        rows,
                        cols,
                        pixel_width: pxw,
                        pixel_height: pxh,
                    });
                }
            });
            Ok(())
        } else {
            Err("Session not found".into())
        }
    }

    pub fn write_to_session(
        &self,
        session_id: &str,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let processes = self.processes.lock().unwrap();
        let proc = processes.get(session_id).ok_or("Session not found")?;
        let writer_arc = proc.writer.clone();
        let data = data.to_string();

        tauri::async_runtime::spawn(async move {
            let mut guard = writer_arc.lock().await;
            if let Some(writer) = guard.as_mut() {
                if let Err(e) = writer.write_all(data.as_bytes()) {
                    log::error!("Failed to write to PTY: {}", e);
                } else {
                    let _ = writer.flush();
                }
            }
        });

        Ok(())
    }

    pub fn close_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proc) = self.processes.lock().unwrap().remove(session_id) {
            // Drop writer to signal EOF and close master to send SIGHUP on Unix
            tauri::async_runtime::block_on(async {
                let mut w = proc.writer.lock().await;
                *w = None;
            });
            if let Ok(mut m) = proc.master.lock() { *m = None; }
        }
        Ok(())
    }

    fn start_shell_process(
        &self,
        session_id: &str,
        shell: &str,
        working_dir: &str,
        size: TerminalSize,
    ) -> Result<(
        Arc<tokio::sync::Mutex<Option<Box<dyn std::io::Write + Send>>>>,
        Arc<std::sync::Mutex<Option<Box<dyn MasterPty + Send>>>>
    ), Box<dyn std::error::Error>> {
        let output_sender = self.output_sender.clone();
        let session_id_str = session_id.to_string();

        // Create native PTY system
        let pty_system = native_pty_system();

        // Open pty pair with initial size
        let pair = pty_system.openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.pixel_width,
            pixel_height: size.pixel_height,
        })?;

        // Build shell command
        let shell_prog = if cfg!(windows) { "powershell.exe" } else { shell };
        let mut cmd = CommandBuilder::new(shell_prog);
        cmd.cwd(working_dir);
        if cfg!(not(windows)) {
            cmd.env("TERM", "xterm-256color");
        }

        // Spawn child attached to the slave end
        let _child = pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        // Writer and master handles
        let writer = pair.master.take_writer()?;
        let master: Box<dyn MasterPty + Send> = pair.master;

        let writer_arc: Arc<tokio::sync::Mutex<Option<Box<dyn std::io::Write + Send>>>> =
            Arc::new(tokio::sync::Mutex::new(Some(writer)));
        let master_arc: Arc<std::sync::Mutex<Option<Box<dyn MasterPty + Send>>>> =
            Arc::new(std::sync::Mutex::new(Some(master)));

        // Create a separate blocking thread for reading from the PTY
        let (read_master_arc, output_sender2, sid2) = (master_arc.clone(), output_sender.clone(), session_id_str.clone());
        std::thread::spawn(move || {
            // Lock master and create a reader
            // Note: portable-pty provides a try_clone_reader() API on MasterPty
            let maybe_reader = {
                let mut guard = read_master_arc.lock().unwrap();
                if let Some(ref mut master) = *guard {
                    master.try_clone_reader().ok()
                } else { None }
            };
            if let Some(mut reader) = maybe_reader {
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = String::from_utf8_lossy(&buf[..n]).to_string();
                            let _ = output_sender2.send(TerminalOutput { session_id: sid2.clone(), data });
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        // Initial message
        let welcome_msg = format!("Welcome to Warp Terminal\r\nWorking directory: {}\r\n", working_dir);
        let _ = output_sender.send(TerminalOutput { session_id: session_id_str, data: welcome_msg });

        Ok((writer_arc, master_arc))
    }
}
