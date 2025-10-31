use super::common::TestDaemon;
use std::process::Command;

#[test]
fn test_negative_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["start", "--work", "-5"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value") || !output.status.success(),
        "Negative duration should be rejected"
    );

    Ok(())
}

#[test]
fn test_zero_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["start", "--work", "0"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    if output.status.success() {
        // If command succeeded, check if daemon rejected it
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stderr.contains("Error") || stdout.contains("Error"),
            "Zero duration should be rejected"
        );
    }

    Ok(())
}

#[test]
fn test_excessive_duration_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["start", "--work", "10000"]) // 10000 minutes is excessive
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    if output.status.success() {
        // If command succeeded, check if daemon rejected it
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stderr.contains("Error") || stdout.contains("Error"),
            "Excessive duration should be rejected"
        );
    }

    Ok(())
}

#[test]
fn test_zero_sessions_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["start", "--work", "25", "--sessions", "0"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    if output.status.success() {
        // If command succeeded, check if daemon rejected it
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stderr.contains("Error") || stdout.contains("Error"),
            "Zero sessions should be rejected"
        );
    }

    Ok(())
}
