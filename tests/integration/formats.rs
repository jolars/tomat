use super::common::TestDaemon;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_status_default_format_returns_json() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Get status without specifying format (should default to waybar)
    let status = daemon.send_command(&["status"])?;

    // Should be a JSON object with expected fields
    assert!(
        status.is_object(),
        "Default format should return JSON object"
    );
    assert!(status.get("text").is_some(), "Should have text field");
    assert!(status.get("class").is_some(), "Should have class field");
    assert!(status.get("tooltip").is_some(), "Should have tooltip field");
    assert!(
        status.get("percentage").is_some(),
        "Should have percentage field"
    );

    Ok(())
}

#[test]
fn test_status_waybar_format_returns_json() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Get status with explicit waybar format
    let status = daemon.send_command(&["status", "--output", "waybar"])?;

    // Should be a JSON object with waybar-specific fields
    assert!(
        status.is_object(),
        "Waybar format should return JSON object"
    );
    assert!(status.get("text").is_some(), "Should have text field");
    assert!(status.get("class").is_some(), "Should have class field");
    assert!(status.get("tooltip").is_some(), "Should have tooltip field");
    assert!(
        status.get("percentage").is_some(),
        "Should have percentage field"
    );

    Ok(())
}

#[test]
fn test_status_plain_format_returns_text() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Get status with plain output
    let status = daemon.send_command(&["status", "--output", "plain"])?;

    // Should be a string, not a JSON object
    assert!(
        status.is_string(),
        "Plain format should return plain text string, got: {:?}",
        status
    );

    let text = status.as_str().unwrap();

    // Should contain expected symbols
    assert!(
        text.contains("🍅") || text.contains("☕") || text.contains("🏖️"),
        "Plain format should contain phase icon"
    );
    assert!(
        text.contains("▶") || text.contains("⏸"),
        "Plain format should contain state symbol"
    );
    assert!(
        text.contains(":"),
        "Plain format should contain time separator"
    );

    Ok(())
}

#[test]
fn test_status_plain_format_shows_pause_symbol() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer and wait for it to transition to paused break
    daemon.send_command(&["start", "--work", "0.05", "--break", "0.05"])?;
    daemon.wait_for_completion(10)?;
    thread::sleep(Duration::from_millis(200));

    // Get plain output status
    let status = daemon.send_command(&["status", "--output", "plain"])?;

    assert!(status.is_string(), "Should be string");
    let text = status.as_str().unwrap();

    // Should show break icon and pause symbol
    assert!(text.contains("☕"), "Should contain break icon");
    assert!(text.contains("⏸"), "Should contain pause symbol");

    Ok(())
}

#[test]
fn test_status_plain_format_shows_play_symbol() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start timer with auto-advance
    daemon.send_command(&["start", "--work", "0.1", "--auto-advance"])?;

    // Get plain output status
    let status = daemon.send_command(&["status", "--output", "plain"])?;

    assert!(status.is_string(), "Should be string");
    let text = status.as_str().unwrap();

    // Should show work icon and play symbol
    assert!(text.contains("🍅"), "Should contain work icon");
    assert!(text.contains("▶"), "Should contain play symbol");

    Ok(())
}

#[test]
fn test_status_invalid_format_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Try to get status with invalid output format
    let output = Command::new(TestDaemon::get_binary_path())
        .args(["status", "--output", "invalid"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    // Should fail or return error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("Error")
            || stderr.contains("Unknown format")
            || stdout.contains("Error")
            || stdout.contains("Unknown format"),
        "Invalid format should return error. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    Ok(())
}

#[test]
fn test_status_formats_show_same_content() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Get status in both output formats
    let waybar_status = daemon.send_command(&["status", "--output", "waybar"])?;
    let plain_status = daemon.send_command(&["status", "--output", "plain"])?;

    // Extract text from waybar format
    let waybar_text = waybar_status["text"].as_str().unwrap();
    let plain_text = plain_status.as_str().unwrap();

    // Both should show the same text content
    assert_eq!(
        waybar_text, plain_text,
        "Waybar and plain formats should show same text content"
    );

    Ok(())
}

#[test]
fn test_status_i3status_rs_format() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Get status with i3status-rs format
    let status = daemon.send_command(&["status", "--output", "i3status-rs"])?;

    // Should be a JSON object with i3status-rs specific fields
    assert!(
        status.is_object(),
        "i3status-rs format should return JSON object"
    );

    // Check required i3status-rs fields
    assert!(status.get("text").is_some(), "Should have text field");
    assert!(
        status.get("short_text").is_some(),
        "Should have short_text field"
    );
    assert!(status.get("state").is_some(), "Should have state field");

    // Verify content
    let text = status.get("text").and_then(|v| v.as_str()).unwrap();
    let state = status.get("state").and_then(|v| v.as_str()).unwrap();

    assert!(text.contains("🍅"), "Should contain work icon");
    assert!(text.contains("▶"), "Should contain play symbol");
    assert_eq!(state, "Critical", "Work phase should have Critical state");

    Ok(())
}

#[test]
fn test_watch_command_outputs_continuously() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer with very short durations for quick testing
    daemon.send_command(&["start", "--work", "0.1"])?;

    // Spawn watch command with short interval
    let mut watch_process = Command::new(TestDaemon::get_binary_path())
        .args(["watch", "--output", "plain", "--interval", "1"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Collect output for a few seconds
    thread::sleep(Duration::from_secs(3));

    // Kill the watch process
    watch_process.kill()?;
    let output = watch_process.wait_with_output()?;

    // Convert stdout to string
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Should have multiple status updates (at least 2 in 3 seconds with 1-second interval)
    assert!(
        lines.len() >= 2,
        "Watch command should output multiple status updates, got {} lines",
        lines.len()
    );

    // Each line should contain expected status symbols
    for line in &lines {
        assert!(
            line.contains("🍅") || line.contains("☕") || line.contains("🏖️"),
            "Watch output should contain phase icon: {}",
            line
        );
    }

    Ok(())
}

#[test]
fn test_watch_command_respects_interval() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.2"])?;

    // Spawn watch command with 2-second interval
    let mut watch_process = Command::new(TestDaemon::get_binary_path())
        .args(["watch", "--output", "plain", "--interval", "2"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    // Wait for 5 seconds
    thread::sleep(Duration::from_secs(5));

    // Kill the watch process
    watch_process.kill()?;
    let output = watch_process.wait_with_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // With 2-second interval and 5 seconds of running, should get ~3 updates (at 0s, 2s, 4s)
    // Allow some tolerance (2-4 lines)
    assert!(
        lines.len() >= 2 && lines.len() <= 4,
        "Watch with 2s interval should output 2-4 updates in 5 seconds, got {}",
        lines.len()
    );

    Ok(())
}

#[test]
fn test_watch_command_exits_when_daemon_stops() -> Result<(), Box<dyn std::error::Error>> {
    let mut daemon = TestDaemon::start()?;

    // Start a timer
    daemon.send_command(&["start", "--work", "0.2"])?;

    // Spawn watch command
    let mut watch_process = Command::new(TestDaemon::get_binary_path())
        .args(["watch", "--output", "plain"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Wait a bit for watch to start
    thread::sleep(Duration::from_millis(500));

    // Stop the daemon
    daemon.daemon_process.kill()?;
    daemon.daemon_process.wait()?;

    // Wait for watch to detect the daemon is gone
    thread::sleep(Duration::from_secs(2));

    // Check if watch process has exited
    let result = watch_process.try_wait()?;
    assert!(
        result.is_some(),
        "Watch process should exit when daemon stops"
    );

    // Get the error message
    let output = watch_process.wait_with_output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain error message about connection failure
    assert!(
        stderr.contains("Failed to connect"),
        "Watch should report connection error when daemon stops: {}",
        stderr
    );

    Ok(())
}
