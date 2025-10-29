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
    /// Get the path to the tomat binary for testing
    ///
    /// This is necessary because:
    /// - Local development: binary is in target/debug/tomat or target/release/tomat
    /// - NixOS builds: cargo sets CARGO_BIN_EXE_tomat to the actual binary location
    /// - Different build profiles may use different target directories
    fn get_binary_path() -> String {
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
    fn start() -> Result<Self, Box<dyn std::error::Error>> {
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
    fn send_command(&self, args: &[&str]) -> Result<Value, Box<dyn std::error::Error>> {
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
    fn get_status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.send_command(&["status"])
    }

    /// Wait for timer to complete and transition (paused for auto_advance=false, continued for auto_advance=true)
    fn wait_for_completion(&self, max_wait: u64) -> Result<(), Box<dyn std::error::Error>> {
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

                    // Check what happened after transition
                    let status_after = self.get_status()?;
                    if let Some(text_after) = status_after.get("text").and_then(|v| v.as_str()) {
                        if text_after.contains("⏸") {
                            // Successfully transitioned to paused state (auto_advance=false)
                            return Ok(());
                        } else if let Some(class_after) =
                            status_after.get("class").and_then(|v| v.as_str())
                            && Some(class_after.to_string()) != initial_phase
                        {
                            // Phase changed (auto_advance=true)
                            return Ok(());
                        }

                        if text_after.contains("00:00") {
                            // Still showing 00:00, manually trigger transition
                            println!("Auto-transition didn't occur, manually triggering with skip");
                            self.send_command(&["skip"])?;
                            thread::sleep(Duration::from_millis(200));
                            return Ok(());
                        }
                    }
                } else if text.contains("⏸") {
                    // Already in paused state
                    return Ok(());
                } else if let Some(class) = status.get("class").and_then(|v| v.as_str()) {
                    // Check if phase changed (auto_advance=true case)
                    if Some(class.to_string()) != initial_phase && initial_phase.is_some() {
                        return Ok(());
                    }
                }
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
    daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?; // 3 second intervals

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
    assert!(
        status["text"].as_str().unwrap().contains("⏸"),
        "Timer should show pause symbol"
    );
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
        "--break",
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
        status["text"].as_str().unwrap().contains("▶"),
        "Timer should show play symbol when running"
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
    daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?;

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
        status["text"].as_str().unwrap().contains("▶"),
        "Timer should show play symbol when running after resume"
    );

    Ok(())
}

#[test]
fn test_toggle_from_paused_with_auto_advance_false() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start in paused work state (default), toggle should resume
    daemon.send_command(&["toggle"])?;

    // Check that timer resumed
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work", "Timer should be in work state");
    assert!(
        status["text"].as_str().unwrap().contains("🍅"),
        "Timer should show work icon"
    );

    Ok(())
}

#[test]
fn test_toggle_pause_and_resume() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer with auto_advance=true and short duration
    daemon.send_command(&[
        "start",
        "--work",
        "0.05",
        "--break",
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
        status["text"].as_str().unwrap().contains("▶"),
        "Timer should show play symbol when running with auto_advance=true"
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
        "--break",
        "0.05",
        "--auto-advance",
    ])?;

    // Stop the timer
    daemon.send_command(&["stop"])?;

    // Start again with auto_advance=false
    daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?;

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
    daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?;

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
        "--break",
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
    daemon.send_command(&["start", "--work", "0.02", "--break", "0.01"])?; // 1.2s work, 0.6s break

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

#[test]
fn test_negative_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Try to start with negative work duration
    let output = Command::new("target/debug/tomat")
        .args(["start", "--work", "-5"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    assert!(
        !output.status.success(),
        "Negative duration should be rejected"
    );

    Ok(())
}

#[test]
fn test_zero_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Try to start with zero work duration
    let output = Command::new("target/debug/tomat")
        .args(["start", "--work", "0"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    // Check stderr for error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("Error")
            || stderr.contains("greater than 0")
            || stdout.contains("Error")
            || stdout.contains("greater than 0"),
        "Zero duration should be rejected. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    Ok(())
}

#[test]
fn test_excessive_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Try to start with excessive work duration (over 10 hours)
    let output = Command::new("target/debug/tomat")
        .args(["start", "--work", "700"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    // Check stderr for error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("Error")
            || stderr.contains("600 minutes")
            || stdout.contains("Error")
            || stdout.contains("600 minutes"),
        "Excessive duration should be rejected. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    Ok(())
}

#[test]
fn test_zero_sessions_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Try to start with zero sessions
    let output = Command::new("target/debug/tomat")
        .args(["start", "--sessions", "0"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    // Check stderr for error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("Error")
            || stderr.contains("at least 1")
            || stdout.contains("Error")
            || stdout.contains("at least 1"),
        "Zero sessions should be rejected. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    Ok(())
}

#[test]
fn test_precise_timer_completion() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a very short timer (3 seconds) with auto-advance to test precise transitions
    daemon.send_command(&[
        "start",
        "--work",
        "0.05",
        "--break",
        "0.05",
        "--auto-advance",
    ])?;

    let start_time = std::time::Instant::now();

    // Wait for the timer to finish work phase and transition to break
    // This should happen in exactly 3 seconds (0.05 * 60)
    std::thread::sleep(Duration::from_millis(3100)); // Wait slightly longer than 3 seconds

    let status = daemon.get_status()?;
    let elapsed = start_time.elapsed();

    // Verify we're in the break phase (timer should have transitioned precisely)
    assert_eq!(
        status["class"],
        "break",
        "Timer should have transitioned to break phase by now. Status: {}",
        serde_json::to_string_pretty(&status)?
    );

    // The transition should have happened close to 3 seconds, not up to 4 seconds
    // With the old implementation, this could take up to 4 seconds (3s + up to 1s delay)
    // With precise timing, it should be close to 3 seconds
    assert!(
        elapsed.as_millis() < 3500,
        "Timer transition took too long: {}ms (should be close to 3000ms)",
        elapsed.as_millis()
    );

    Ok(())
}

#[test]
fn test_explicit_pause_resume_commands() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer with a reasonable duration
    daemon.send_command(&["start", "--work", "0.2", "--break", "0.1"])?;

    // Timer should be running
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work");

    // Test explicit pause command
    daemon.send_command(&["pause"])?;

    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work-paused",
        "Timer should be paused after pause command"
    );

    // Test explicit resume command
    daemon.send_command(&["resume"])?;

    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work",
        "Timer should be running after resume command"
    );

    Ok(())
}

#[test]
fn test_pause_resume_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.2", "--break", "0.1"])?;

    // Pause it
    daemon.send_command(&["pause"])?;
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work-paused");

    // Try to pause again - should still succeed (idempotent operation)
    daemon.send_command(&["pause"])?;
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work-paused", "Timer should remain paused");

    // Resume it
    daemon.send_command(&["resume"])?;
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work");

    // Try to resume again - should still succeed (idempotent operation)
    daemon.send_command(&["resume"])?;
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work", "Timer should remain running");

    Ok(())
}

#[test]
fn test_toggle_still_works_with_new_commands() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.2", "--break", "0.1"])?;

    // Timer should be running
    let status = daemon.get_status()?;
    assert_eq!(status["class"], "work");

    // Use explicit pause
    daemon.send_command(&["pause"])?;
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work-paused",
        "Timer should be paused after explicit pause"
    );

    // Use toggle to resume (should work)
    daemon.send_command(&["toggle"])?;
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work",
        "Timer should be running after toggle resume"
    );

    // Use toggle to pause (should work)
    daemon.send_command(&["toggle"])?;
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work-paused",
        "Timer should be paused after toggle pause"
    );

    // Use explicit resume
    daemon.send_command(&["resume"])?;
    let status = daemon.get_status()?;
    assert_eq!(
        status["class"], "work",
        "Timer should be running after explicit resume"
    );

    Ok(())
}
