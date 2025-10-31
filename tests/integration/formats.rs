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
