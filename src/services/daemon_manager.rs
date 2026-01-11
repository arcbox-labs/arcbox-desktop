//! Daemon lifecycle management.
//!
//! Manages the arcbox daemon process lifecycle:
//! - Locates daemon binary in app bundle
//! - Spawns daemon process on startup
//! - Monitors health via ping endpoint
//! - Gracefully shuts down on app exit

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use gpui::*;

/// Data directory name (matches daemon default: ~/.arcbox)
const DATA_DIR_NAME: &str = ".arcbox";

/// Daemon binary name
const DAEMON_BINARY: &str = "arcbox";

/// Maximum startup wait time
const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Ping retry interval during startup
const STARTUP_PING_INTERVAL: Duration = Duration::from_millis(200);

/// Daemon manager state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonState {
    /// Not started yet
    Stopped,
    /// Starting up, waiting for health check
    Starting,
    /// Running and healthy
    Running,
    /// Failed to start or crashed
    Failed(String),
}

impl Default for DaemonState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Events emitted by DaemonManager
#[derive(Debug, Clone)]
pub enum DaemonManagerEvent {
    /// Daemon state changed
    StateChanged(DaemonState),
}

/// Manages the daemon process lifecycle
pub struct DaemonManager {
    state: DaemonState,
    /// Path to daemon binary
    daemon_path: Option<PathBuf>,
    /// Path to socket file
    socket_path: PathBuf,
    /// Path to data directory
    data_dir: PathBuf,
    /// Child process handle (kept to prevent drop)
    #[allow(dead_code)]
    child_handle: Option<std::process::Child>,
}

impl DaemonManager {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let (socket_path, data_dir) = Self::get_paths();
        let daemon_path = Self::find_daemon_binary();

        Self {
            state: DaemonState::Stopped,
            daemon_path,
            socket_path,
            data_dir,
            child_handle: None,
        }
    }

    /// Get socket and data directory paths
    /// Uses ~/.arcbox to match daemon's default data directory
    fn get_paths() -> (PathBuf, PathBuf) {
        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(DATA_DIR_NAME);

        // Use docker.sock for Docker API (ping uses HTTP which expects Docker API)
        let socket_path = base_dir.join("docker.sock");
        (socket_path, base_dir)
    }

    /// Find daemon binary in app bundle or PATH
    fn find_daemon_binary() -> Option<PathBuf> {
        // 1. Check if running from app bundle (macOS)
        #[cfg(target_os = "macos")]
        {
            if let Some(bundle_path) = Self::get_bundle_path() {
                let daemon_in_bundle = bundle_path
                    .join("Contents")
                    .join("MacOS")
                    .join(DAEMON_BINARY);
                if daemon_in_bundle.exists() {
                    tracing::info!("Found daemon in bundle: {}", daemon_in_bundle.display());
                    return Some(daemon_in_bundle);
                }
            }
        }

        // 2. Check alongside the executable (for development)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let daemon_alongside = exe_dir.join(DAEMON_BINARY);
                if daemon_alongside.exists() {
                    tracing::info!("Found daemon alongside exe: {}", daemon_alongside.display());
                    return Some(daemon_alongside);
                }
            }
        }

        // 3. Check in PATH
        if let Ok(output) = std::process::Command::new("which")
            .arg(DAEMON_BINARY)
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    tracing::info!("Found daemon in PATH: {}", path);
                    return Some(PathBuf::from(path));
                }
            }
        }

        tracing::warn!("Daemon binary not found");
        None
    }

    /// Get the app bundle path on macOS
    #[cfg(target_os = "macos")]
    fn get_bundle_path() -> Option<PathBuf> {
        // The executable is at: ArcBox.app/Contents/MacOS/arcbox-desktop
        // We want: ArcBox.app/
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent() // MacOS/
                .and_then(|p| p.parent()) // Contents/
                .and_then(|p| p.parent()) // ArcBox.app/
                .map(|p| p.to_path_buf())
        })
    }

    /// Get current state
    pub fn state(&self) -> &DaemonState {
        &self.state
    }

    /// Get Docker API socket path (for HTTP ping)
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Get gRPC socket path (for gRPC client connections)
    pub fn grpc_socket_path(&self) -> PathBuf {
        self.data_dir.join("arcbox.sock")
    }

    /// Check if daemon is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, DaemonState::Running)
    }

    /// Start the daemon process
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if matches!(self.state, DaemonState::Starting | DaemonState::Running) {
            return;
        }

        let Some(daemon_path) = self.daemon_path.clone() else {
            self.set_state(DaemonState::Failed("Daemon binary not found".into()), cx);
            return;
        };

        self.set_state(DaemonState::Starting, cx);

        let socket_path = self.socket_path.clone();
        let data_dir = self.data_dir.clone();

        // Spawn background thread for daemon startup
        // This avoids blocking the UI and works without tokio runtime
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            // Run startup logic in a background thread
            let result = cx
                .background_executor()
                .spawn(async move {
                    Self::start_daemon_sync(&daemon_path, &socket_path, &data_dir)
                })
                .await;

            // Update state based on result
            let _ = cx.update(|cx| {
                this.update(cx, |this, cx| match result {
                    Ok(child) => {
                        this.child_handle = Some(child);
                        this.set_state(DaemonState::Running, cx);
                    }
                    Err(e) if e == "ALREADY_RUNNING" => {
                        // Daemon is already running, treat as success
                        tracing::info!("Connected to existing daemon");
                        this.set_state(DaemonState::Running, cx);
                    }
                    Err(e) => {
                        this.set_state(DaemonState::Failed(e), cx);
                    }
                })
            });
        })
        .detach();
    }

    /// Synchronous daemon startup logic (runs in background thread)
    fn start_daemon_sync(
        daemon_path: &PathBuf,
        socket_path: &PathBuf,
        data_dir: &PathBuf,
    ) -> Result<std::process::Child, String> {
        // Ensure data directory exists
        if let Err(e) = std::fs::create_dir_all(data_dir) {
            return Err(format!("Failed to create data dir: {}", e));
        }

        // Check if daemon is already running (e.g., from previous session)
        if Self::ping_daemon_sync(socket_path) {
            tracing::info!("Daemon already running");
            // Return a placeholder - we don't have the actual child handle
            // This is fine since we just need to track that it's running
            return Err("ALREADY_RUNNING".to_string());
        }

        // Remove stale socket file
        let _ = std::fs::remove_file(socket_path);

        // Spawn daemon process
        tracing::info!(
            "Starting daemon: {} daemon --socket {}",
            daemon_path.display(),
            socket_path.display()
        );

        let mut child = Command::new(daemon_path)
            .arg("daemon")
            .arg("--socket")
            .arg(socket_path)
            .arg("--data-dir")
            .arg(data_dir)
            .arg("--foreground")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

        // Spawn thread to read daemon stdout
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        tracing::debug!("[daemon] {}", line);
                    }
                }
            });
        }

        // Spawn thread to read daemon stderr
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        tracing::warn!("[daemon:err] {}", line);
                    }
                }
            });
        }

        // Wait for daemon to become ready
        let start_time = std::time::Instant::now();
        loop {
            if Self::ping_daemon_sync(socket_path) {
                tracing::info!("Daemon is ready");
                return Ok(child);
            }

            // Check if process exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Err(format!("Daemon exited with: {}", status));
                }
                Ok(None) => {} // Still running
                Err(e) => {
                    return Err(format!("Failed to check daemon status: {}", e));
                }
            }

            // Check timeout
            if start_time.elapsed() > STARTUP_TIMEOUT {
                let _ = child.kill();
                return Err("Daemon startup timeout".to_string());
            }

            std::thread::sleep(STARTUP_PING_INTERVAL);
        }
    }

    /// Synchronous ping to check if daemon is running
    fn ping_daemon_sync(socket_path: &PathBuf) -> bool {
        let Ok(mut stream) = UnixStream::connect(socket_path) else {
            return false;
        };

        // Set read timeout
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));

        // Send HTTP GET /_ping request
        let request = "GET /_ping HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        if stream.write_all(request.as_bytes()).is_err() {
            return false;
        }

        // Read response
        let mut response = vec![0u8; 1024];
        match stream.read(&mut response) {
            Ok(n) if n > 0 => {
                let response_str = String::from_utf8_lossy(&response[..n]);
                response_str.contains("200 OK") || response_str.contains("OK")
            }
            _ => false,
        }
    }

    /// Stop the daemon
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        // Send shutdown signal via socket or just let the process exit with app
        self.set_state(DaemonState::Stopped, cx);

        // The daemon will be terminated when the app exits since it's a child process
        // For graceful shutdown, we could send a SIGTERM or call a shutdown endpoint
    }

    fn set_state(&mut self, state: DaemonState, cx: &mut Context<Self>) {
        if self.state != state {
            tracing::info!("Daemon state: {:?} -> {:?}", self.state, state);
            self.state = state.clone();
            cx.emit(DaemonManagerEvent::StateChanged(state));
            cx.notify();
        }
    }
}

impl EventEmitter<DaemonManagerEvent> for DaemonManager {}
