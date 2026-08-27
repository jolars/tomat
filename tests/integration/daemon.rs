use super::common::TestDaemon;
use std::process::Command;

#[test]
fn test_daemon_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Test that daemon is responsive
    let status = daemon.get_status()?;
    assert!(status.is_object(), "Daemon should return status object");

    // Test basic commands work
    daemon.send_command(&["start", "--work", "0.1"])?;
    let status = daemon.get_status()?;

    assert!(
        status.get("text").is_some(),
        "Status should have text field"
    );
    assert!(
        status.get("class").is_some(),
        "Status should have class field"
    );

    Ok(())
}

#[test]
fn test_daemon_status_when_not_running() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = TestDaemon::get_binary_path();
    let temp_dir = tempfile::tempdir()?;

    let output = Command::new(&binary_path)
        .args(["status"])
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.is_empty(),
        "Status should not write to stderr: {stderr}"
    );

    let status: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(status["text"], "");
    assert_eq!(status["tooltip"], "Tomat daemon is not running");
    assert_eq!(status["class"], "disconnected");
    assert_eq!(status["percentage"], 0.0);

    Ok(())
}

#[test]
fn test_disconnected_status_respects_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = TestDaemon::get_binary_path();
    let temp_dir = tempfile::tempdir()?;

    let i3status_output = Command::new(&binary_path)
        .args(["status", "--output", "i3status-rs"])
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .output()?;
    assert!(i3status_output.status.success());
    assert!(i3status_output.stderr.is_empty());

    let status: serde_json::Value = serde_json::from_slice(&i3status_output.stdout)?;
    assert_eq!(status["text"], "Tomat daemon is not running");
    assert_eq!(status["short_text"], "Tomat disconnected");
    assert_eq!(status["state"], "Warning");

    let plain_output = Command::new(&binary_path)
        .args(["status", "--output", "plain"])
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .output()?;
    assert!(plain_output.status.success());
    assert!(plain_output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(plain_output.stdout)?.trim(),
        "Tomat daemon is not running"
    );

    Ok(())
}

#[test]
fn test_daemon_status_with_stale_socket() -> Result<(), Box<dyn std::error::Error>> {
    let binary_path = TestDaemon::get_binary_path();
    let temp_dir = tempfile::tempdir()?;
    let socket_path = temp_dir.path().join("tomat.sock");
    let listener = std::os::unix::net::UnixListener::bind(&socket_path)?;
    drop(listener);

    let output = Command::new(&binary_path)
        .args(["status"])
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .output()?;

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let status: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(status["class"], "disconnected");

    Ok(())
}

#[test]
fn test_daemon_start_when_already_running() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Try to start another daemon in the same runtime directory
    let binary_path = TestDaemon::get_binary_path();
    let output = Command::new(&binary_path)
        .args(["daemon", "start"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    // Should detect that daemon is already running
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stderr.contains("already running") || stdout.contains("already running"),
        "Should detect daemon already running. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    Ok(())
}
