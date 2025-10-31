use serde_json::Value;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper struct to manage test daemon lifecycle
pub struct TestDaemon {
    pub _temp_dir: TempDir,
    pub daemon_process: Child,
}

impl TestDaemon {
    /// Get the path to the tomat binary for testing
    ///
    /// This is necessary because:
    /// - Local development: binary is in target/debug/tomat or target/release/tomat
    /// - NixOS builds: cargo sets CARGO_BIN_EXE_tomat to the actual binary location
    /// - Different build profiles may use different target directories
    pub fn get_binary_path() -> String {
        // Check if CARGO_BIN_EXE_tomat is set (preferred method for cargo test)
        if let Ok(binary_path) = std::env::var("CARGO_BIN_EXE_tomat") {
            return binary_path;
        }

        // Fallback: detect based on CARGO_MANIFEST_DIR and profile
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };

        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            return format!("{}/target/{}/tomat", manifest_dir, profile);
        }

        // Final fallback for local development
        format!("target/{}/tomat", profile)
    }

    /// Start a new test daemon with a temporary socket
    pub fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let binary_path = Self::get_binary_path();

        // Start daemon with custom socket path and testing flag to disable notifications
        let mut daemon_process = Command::new(&binary_path)
            .arg("daemon")
            .arg("run") // Use the internal run command for testing
            .env("XDG_RUNTIME_DIR", temp_dir.path())
            .env("TOMAT_TESTING", "1") // Disable notifications during testing
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                format!(
                    "Failed to start daemon with binary '{}': {}",
                    binary_path, e
                )
            })?;

        // Wait a bit for daemon to start
        thread::sleep(Duration::from_millis(100));

        // Check if daemon is still running
        if let Some(exit_status) = daemon_process.try_wait()? {
            return Err(format!("Daemon exited early with status: {}", exit_status).into());
        }

        Ok(TestDaemon {
            _temp_dir: temp_dir,
            daemon_process,
        })
    }

    /// Send a command to the test daemon
    pub fn send_command(&self, args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
        let binary_path = Self::get_binary_path();
        let output = Command::new(&binary_path)
            .args(args)
            .env("XDG_RUNTIME_DIR", self._temp_dir.path())
            .output()
            .map_err(|e| format!("Failed to run command with binary '{}': {}", binary_path, e))?;

        if !output.status.success() {
            return Err(format!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        if stdout.trim().is_empty() {
            return Ok(Value::Null);
        }

        // Try to parse as JSON for status commands
        match serde_json::from_str(&stdout) {
            Ok(json) => Ok(json),
            Err(_) => Ok(Value::String(stdout.trim().to_string())),
        }
    }

    /// Get current timer status as JSON
    pub fn get_status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.send_command(&["status"])
    }

    /// Wait for timer to complete and transition (paused for auto_advance=false, continued for auto_advance=true)
    pub fn wait_for_completion(&self, max_wait: u64) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let max_duration = Duration::from_secs(max_wait);

        // For auto_advance=false timers, we wait for paused state
        // For auto_advance=true timers, we wait for phase change
        let mut initial_phase: Option<String> = None;
        let mut timer_completed = false;

        loop {
            if start.elapsed() > max_duration {
                return Err("Timeout waiting for timer completion".into());
            }

            let status = self.get_status()?;

            // Record initial phase
            if initial_phase.is_none()
                && let Some(class) = status.get("class").and_then(|v| v.as_str())
            {
                initial_phase = Some(class.to_string());
            }

            // Check if timer shows 00:00 (completed but not yet transitioned)
            if let Some(text) = status.get("text").and_then(|v| v.as_str()) {
                if text.contains("00:00") && !timer_completed {
                    // Timer reached 00:00, wait a moment for automatic transition
                    thread::sleep(Duration::from_millis(1500));
                    timer_completed = true;
                    continue;
                }
            }

            // Check for state after completion
            if timer_completed {
                if let Some(class) = status.get("class").and_then(|v| v.as_str()) {
                    if let Some(ref initial) = initial_phase {
                        // Check for auto_advance=false: should be paused in new phase
                        if class.contains("paused") && !class.contains(initial) {
                            return Ok(()); // Successfully transitioned to paused state
                        }
                        // Check for auto_advance=true: should be running in new phase
                        if !class.contains("paused") && !class.contains(initial) {
                            return Ok(()); // Successfully transitioned to running state
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        // Try to gracefully shut down the daemon
        let _ = self.daemon_process.kill();
        let _ = self.daemon_process.wait();
    }
}
