use super::common::TestDaemon;
use std::process::Command;

#[test]
fn test_quiet_suppresses_success_messages() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    for args in [
        &["--quiet", "start"][..],
        &["pause", "--quiet"][..],
        &["-q", "resume"][..],
        &["toggle", "-q"][..],
        &["--quiet", "skip"][..],
        &["stop", "--quiet"][..],
    ] {
        let output = Command::new(TestDaemon::get_binary_path())
            .args(args)
            .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
            .output()?;

        assert!(output.status.success(), "command failed: {args:?}");
        assert!(output.stdout.is_empty(), "stdout was not quiet: {args:?}");
        assert!(output.stderr.is_empty(), "stderr was not quiet: {args:?}");
    }

    Ok(())
}

#[test]
fn test_quiet_preserves_status_output() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["status", "--quiet", "--output", "plain"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
    assert!(output.stderr.is_empty());

    Ok(())
}

#[test]
fn test_quiet_preserves_errors_and_failure_status() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    let output = Command::new(TestDaemon::get_binary_path())
        .args(["--quiet", "start", "--sessions", "0"])
        .env("XDG_RUNTIME_DIR", daemon._temp_dir.path())
        .output()?;

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());

    Ok(())
}

#[test]
fn test_quiet_daemon_management() -> Result<(), Box<dyn std::error::Error>> {
    let runtime_dir = tempfile::tempdir()?;
    let binary = TestDaemon::get_binary_path();

    let start = Command::new(&binary)
        .args(["daemon", "start", "--quiet"])
        .env("XDG_RUNTIME_DIR", runtime_dir.path())
        .env("TOMAT_TESTING", "1")
        .output()?;

    assert!(start.status.success());
    assert!(start.stdout.is_empty());
    assert!(start.stderr.is_empty());

    let status = Command::new(&binary)
        .args(["--quiet", "daemon", "status"])
        .env("XDG_RUNTIME_DIR", runtime_dir.path())
        .output()?;

    assert!(status.status.success());
    assert!(!status.stdout.is_empty());
    assert!(status.stderr.is_empty());

    let stop = Command::new(&binary)
        .args(["-q", "daemon", "stop"])
        .env("XDG_RUNTIME_DIR", runtime_dir.path())
        .output()?;

    assert!(stop.status.success());
    assert!(stop.stdout.is_empty());
    assert!(stop.stderr.is_empty());

    Ok(())
}

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
