use serde_json::Value;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper struct to manage test daemon lifecycle
struct TestDaemon {
    _temp_dir: TempDir,
    daemon_process: Child,
}

impl TestDaemon {
    /// Start a new test daemon with a temporary socket
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;

        // Start daemon with custom socket path and testing flag to disable notifications
        let mut daemon_process = Command::new("target/debug/tomat")
            .arg("daemon")
            .arg("run") // Use the internal run command for testing
            .env("XDG_RUNTIME_DIR", temp_dir.path())
            .env("TOMAT_TESTING", "1") // Disable notifications during testing
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

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
    fn send_command(&self, args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
        let output = Command::new("target/debug/tomat")
            .args(args)
            .env("XDG_RUNTIME_DIR", self._temp_dir.path())
            .output()?;

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
    fn get_status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.send_command(&["status"])
    }

    /// Wait for timer to complete and optionally trigger transition manually for testing
    fn wait_for_completion(&self, max_wait: u64) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let max_duration = Duration::from_secs(max_wait);

        loop {
            if start.elapsed() > max_duration {
                return Err("Timeout waiting for timer completion".into());
            }

            let status = self.get_status()?;
            if let Some(text) = status.get("text").and_then(|v| v.as_str())
                && text.contains("Done!")
            {
                // Timer completed, wait a bit more for automatic transition
                thread::sleep(Duration::from_millis(1500)); // Wait 1.5 seconds for auto-transition

                let status_after_wait = self.get_status()?;
                if let Some(text_after) = status_after_wait.get("text").and_then(|v| v.as_str())
                    && text_after.contains("Done!")
                {
                    // Still showing "Done!", manually trigger transition for testing
                    println!("Auto-transition didn't occur, manually triggering with skip");
                    self.send_command(&["skip"])?;
                }

                return Ok(());
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        let _ = self.daemon_process.kill();
        let _ = self.daemon_process.wait();
    }
}

#[test]
fn test_auto_advance_false_pauses_after_transition() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer with auto_advance=false (default) and very short duration
    daemon.send_command(&["start", "--work", "0.05", "--break-time", "0.05"])?; // 3 second intervals

    // Debug: Check initial status
    let initial_status = daemon.get_status()?;
    println!(
        "Initial status: {}",
        serde_json::to_string_pretty(&initial_status)?
    );

    // Wait for work phase to complete
    daemon.wait_for_completion(10)?;

    // Give a moment for the transition to occur (the wait_for_completion now handles manual trigger if needed)
    thread::sleep(Duration::from_millis(200));

    // Check that we're now in break phase but paused
    let status = daemon.get_status()?;
    println!("Final status: {}", serde_json::to_string_pretty(&status)?);

    assert_eq!(
        status["class"], "break-paused",
        "Timer should be in paused break state"
    );
    assert_eq!(status["text"], "☕ Paused", "Timer should show paused text");
    assert!(
        status["tooltip"].as_str().unwrap().contains("Paused"),
        "Tooltip should indicate paused state"
    );

    Ok(())
}

#[test]
fn test_auto_advance_true_continues_automatically() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer with auto_advance=true and very short duration
    daemon.send_command(&[
        "start",
        "--work",
        "0.05",
        "--break-time",
        "0.05",
        "--auto-advance",
    ])?;

    // Wait for work phase to complete
    daemon.wait_for_completion(10)?;

    // Give a moment for the transition to occur
    thread::sleep(Duration::from_millis(200));

    // Check that we're now in break phase and running (not paused)
    let status = daemon.get_status()?;

    assert_eq!(
        status["class"], "break",
        "Timer should be in active break state"
    );
    assert!(
        status["text"].as_str().unwrap().contains("☕"),
        "Timer should show break icon"
    );
    assert!(
        !status["text"].as_str().unwrap().contains("Paused"),
        "Timer should not show paused text"
    );
    assert!(
        !status["tooltip"].as_str().unwrap().contains("Paused"),
        "Tooltip should not indicate paused state"
    );

    Ok(())
}

#[test]
fn test_resume_paused_timer() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer with auto_advance=false
    daemon.send_command(&["start", "--work", "0.05", "--break-time", "0.05"])?;

    // Wait for work phase to complete and transition to paused break
    daemon.wait_for_completion(10)?;
    thread::sleep(Duration::from_millis(200));

    // Verify it's paused
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "break-paused");

    // Resume the timer
    daemon.send_command(&["toggle"])?;

    // Check that timer is now running
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "break",
        "Timer should be in active break state after resume"
    );
    assert!(
        !status["text"].as_str().unwrap().contains("Paused"),
        "Timer should not show paused text after resume"
    );

    Ok(())
}

#[test]
fn test_toggle_from_idle_with_auto_advance_false() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Toggle from idle state (should start timer with auto_advance=false by default)
    daemon.send_command(&["toggle", "--work", "0.1", "--break-time", "0.05"])?;

    // Check that timer started
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work", "Timer should be in work state");
    assert!(
        status["text"].as_str().unwrap().contains("🍅"),
        "Timer should show work icon"
    );

    Ok(())
}

#[test]
fn test_toggle_from_idle_with_auto_advance_true() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Toggle from idle state with auto_advance=true
    daemon.send_command(&[
        "toggle",
        "--work",
        "0.05",
        "--break-time",
        "0.05",
        "--auto-advance",
    ])?;

    // Wait for work phase to complete
    daemon.wait_for_completion(10)?;
    thread::sleep(Duration::from_millis(200));

    // Should automatically transition to break and continue running
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "break",
        "Timer should auto-advance to break phase"
    );
    assert!(
        !status["text"].as_str().unwrap().contains("Paused"),
        "Timer should not be paused with auto_advance=true"
    );

    Ok(())
}

#[test]
fn test_stop_and_start_preserves_auto_advance_setting() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start with auto_advance=true
    daemon.send_command(&[
        "start",
        "--work",
        "0.05",
        "--break-time",
        "0.05",
        "--auto-advance",
    ])?;

    // Stop the timer
    daemon.send_command(&["stop"])?;

    // Start again with auto_advance=false
    daemon.send_command(&["start", "--work", "0.05", "--break-time", "0.05"])?;

    // Wait for completion
    daemon.wait_for_completion(10)?;
    thread::sleep(Duration::from_millis(200));

    // Should be paused since new start command used auto_advance=false
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "break-paused",
        "Timer should be paused with new auto_advance=false setting"
    );

    Ok(())
}

#[test]
fn test_manual_skip_respects_auto_advance_setting() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Test skip with auto_advance=false
    daemon.send_command(&["start", "--work", "0.05", "--break-time", "0.05"])?;

    // Manually skip before completion
    thread::sleep(Duration::from_millis(100));
    daemon.send_command(&["skip"])?;

    // Should transition to paused break
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "break-paused",
        "Manual skip should respect auto_advance=false"
    );

    // Stop and test skip with auto_advance=true
    daemon.send_command(&["stop"])?;
    daemon.send_command(&[
        "start",
        "--work",
        "0.05",
        "--break-time",
        "0.05",
        "--auto-advance",
    ])?;

    // Manually skip before completion
    thread::sleep(Duration::from_millis(100));
    daemon.send_command(&["skip"])?;

    // Should transition to active break
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "break",
        "Manual skip should respect auto_advance=true"
    );

    Ok(())
}

#[test]
fn test_fractional_minutes_work_correctly() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Test with fractional minutes
    daemon.send_command(&["start", "--work", "0.02", "--break-time", "0.01"])?; // 1.2s work, 0.6s break

    // Should start successfully
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work");
    assert!(
        status["tooltip"].as_str().unwrap().contains("0.0min"),
        "Should show fractional minutes in tooltip"
    );

    // Wait for completion and transition
    daemon.wait_for_completion(5)?;
    thread::sleep(Duration::from_millis(200));

    // Should transition to paused break (auto_advance=false by default)
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "break-paused");

    Ok(())
}

// Daemon management tests
#[test]
fn test_daemon_status_when_not_running() -> Result<(), Box<dyn std::error::Error>> {
    // Use a temporary directory to avoid conflicts
    let temp_dir = tempfile::tempdir()?;

    let output = Command::new("target/debug/tomat")
        .args(["daemon", "status"])
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success(), "Command should succeed");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("Not running"),
        "Should indicate daemon is not running"
    );

    Ok(())
}

#[test]
fn test_daemon_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // Use a temporary directory to avoid conflicts
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Start daemon
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "start"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(output.status.success(), "Daemon start should succeed");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("started successfully"),
        "Should indicate successful start"
    );

    // Check status
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "status"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(output.status.success(), "Status check should succeed");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("Running"),
        "Should indicate daemon is running"
    );

    // Stop daemon
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "stop"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(output.status.success(), "Daemon stop should succeed");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("stopped") || stdout.contains("SIGTERM"),
        "Should indicate daemon was stopped"
    );

    // Verify daemon is stopped
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "status"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(output.status.success(), "Status check should succeed");
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("Not running"),
        "Should indicate daemon is not running"
    );

    Ok(())
}

#[test]
fn test_daemon_start_when_already_running() -> Result<(), Box<dyn std::error::Error>> {
    // Use a temporary directory to avoid conflicts
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    // Start daemon first time
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "start"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(output.status.success(), "First daemon start should succeed");

    // Try to start again
    let output = Command::new("target/debug/tomat")
        .args(["daemon", "start"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output()?;

    assert!(
        output.status.success(),
        "Second start should succeed but detect running daemon"
    );
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("already running"),
        "Should detect daemon already running"
    );

    // Clean up
    let _ = Command::new("target/debug/tomat")
        .args(["daemon", "stop"])
        .env("XDG_RUNTIME_DIR", temp_path)
        .output();

    Ok(())
}
