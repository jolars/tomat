use super::common::TestDaemon;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a hook script that writes to a file
fn create_hook_script(temp_dir: &Path, script_name: &str, marker_file: &str) -> std::path::PathBuf {
    let script_path = temp_dir.join(script_name);
    let marker_path = temp_dir.join(marker_file);

    // Create a simple shell script that writes to a marker file
    let script_content = format!(
        "#!/usr/bin/env bash\necho \"executed\" > {}",
        marker_path.display()
    );

    fs::write(&script_path, script_content).expect("Failed to write hook script");

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    script_path
}

/// Helper to check if hook was executed by checking marker file
fn hook_was_executed(temp_dir: &Path, marker_file: &str) -> bool {
    let marker_path = temp_dir.join(marker_file);
    marker_path.exists()
}

/// Helper to clear hook marker
fn clear_hook_marker(temp_dir: &Path, marker_file: &str) {
    let marker_path = temp_dir.join(marker_file);
    if marker_path.exists() {
        fs::remove_file(marker_path).ok();
    }
}

#[test]
fn test_hook_executes_on_resume_with_auto_advance_false() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook script
    let hook_script = create_hook_script(&temp_path, "work_hook.sh", "work_hook_marker");

    // Create config with hook
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = false

[hooks.on_work_start]
cmd = "{}"
"#,
        hook_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer with auto_advance = false
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Hook should execute immediately on initial start (timer is running)
    thread::sleep(Duration::from_millis(500));
    assert!(
        hook_was_executed(&temp_path, "work_hook_marker"),
        "on_work_start hook should execute on initial start"
    );
    clear_hook_marker(&temp_path, "work_hook_marker");

    // Let work session complete
    thread::sleep(Duration::from_secs(7));

    // Timer should have transitioned to Break (paused)
    let status = daemon.get_status().expect("Failed to get status");

    // Verify we're in a break phase and paused (class will be "break-paused")
    let class = status["class"].as_str().expect("Missing class field");
    assert!(
        class.contains("break") && class.contains("paused"),
        "Expected break-paused, got: {}",
        class
    );

    // Wait for break to complete
    thread::sleep(Duration::from_secs(4));

    // Skip to trigger transition to work (will be paused)
    daemon.send_command(&["skip"]).expect("Failed to skip");

    // Check status - should be in Work phase, paused
    let status = daemon.get_status().expect("Failed to get status");
    let class = status["class"].as_str().expect("Missing class field");
    assert_eq!(class, "work-paused", "Expected work-paused state");

    // Hook should NOT have been executed (timer is paused after transition)
    assert!(
        !hook_was_executed(&temp_path, "work_hook_marker"),
        "on_work_start hook should NOT execute when transitioning to paused work state"
    );

    // Now resume the timer - this should execute the on_work_start hook
    daemon.send_command(&["resume"]).expect("Failed to resume");

    // Wait a bit for hook to execute
    thread::sleep(Duration::from_millis(500));

    // Hook should NOW have been executed
    assert!(
        hook_was_executed(&temp_path, "work_hook_marker"),
        "on_work_start hook should execute when timer is resumed after phase transition"
    );
}

#[test]
fn test_hook_executes_immediately_with_auto_advance_true() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook script
    let hook_script = create_hook_script(&temp_path, "work_hook.sh", "work_hook_marker");

    // Create config with hook and auto_advance = true
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = true

[hooks.on_work_start]
cmd = "{}"
"#,
        hook_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Hook should execute immediately on start
    thread::sleep(Duration::from_millis(500));
    assert!(hook_was_executed(&temp_path, "work_hook_marker"));
    clear_hook_marker(&temp_path, "work_hook_marker");

    // Let work session complete
    thread::sleep(Duration::from_secs(7));

    // Timer should have transitioned to Break and be running
    let status = daemon.get_status().expect("Failed to get status");
    let class = status["class"].as_str().expect("Missing class field");
    assert!(
        class.contains("break") && !class.contains("paused"),
        "Expected break (running), got: {}",
        class
    );

    // Wait for break to complete - should auto-advance to Work
    thread::sleep(Duration::from_secs(4));

    // Wait a bit for hook to execute
    thread::sleep(Duration::from_millis(500));

    // Hook should have been executed immediately when transitioning to Work
    assert!(
        hook_was_executed(&temp_path, "work_hook_marker"),
        "on_work_start hook should execute immediately with auto_advance = true"
    );
}

#[test]
fn test_break_hook_executes_on_resume_with_auto_advance_false() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook script
    let hook_script = create_hook_script(&temp_path, "break_hook.sh", "break_hook_marker");

    // Create config with hook
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = false

[hooks.on_break_start]
cmd = "{}"
"#,
        hook_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Let work session complete
    thread::sleep(Duration::from_secs(7));

    // Timer should have transitioned to Break (paused)
    let status = daemon.get_status().expect("Failed to get status");
    let class = status["class"].as_str().expect("Missing class field");
    assert!(
        class.contains("break") && class.contains("paused"),
        "Expected break-paused, got: {}",
        class
    );

    // Hook should NOT have been executed yet
    assert!(!hook_was_executed(&temp_path, "break_hook_marker"));

    // Resume the timer - hook should execute now
    daemon.send_command(&["resume"]).expect("Failed to resume");

    // Wait for hook to execute
    thread::sleep(Duration::from_millis(500));

    // Hook should have been executed
    assert!(
        hook_was_executed(&temp_path, "break_hook_marker"),
        "on_break_start hook should execute when timer is resumed after phase transition"
    );
}

#[test]
fn test_work_end_hook_executes_before_transition() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook scripts for both work_end and break_start
    let work_end_script = create_hook_script(&temp_path, "work_end_hook.sh", "work_end_marker");
    let break_start_script =
        create_hook_script(&temp_path, "break_start_hook.sh", "break_start_marker");

    // Create config with both hooks
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = true

[hooks.on_work_end]
cmd = "{}"

[hooks.on_break_start]
cmd = "{}"
"#,
        work_end_script.display(),
        break_start_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Let work session complete
    thread::sleep(Duration::from_secs(7));

    // Both hooks should have been executed
    thread::sleep(Duration::from_millis(500));
    assert!(
        hook_was_executed(&temp_path, "work_end_marker"),
        "on_work_end hook should execute when work session completes"
    );
    assert!(
        hook_was_executed(&temp_path, "break_start_marker"),
        "on_break_start hook should execute after work_end"
    );
}

#[test]
fn test_break_end_hook_executes_before_transition() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook scripts
    let break_end_script = create_hook_script(&temp_path, "break_end_hook.sh", "break_end_marker");
    let work_start_script =
        create_hook_script(&temp_path, "work_start_hook.sh", "work_start_marker");

    // Create config with both hooks and auto_advance = true
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = true

[hooks.on_break_end]
cmd = "{}"

[hooks.on_work_start]
cmd = "{}"
"#,
        break_end_script.display(),
        work_start_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Let work complete and transition to break
    thread::sleep(Duration::from_secs(7));

    // Let break complete
    thread::sleep(Duration::from_secs(4));

    // Both hooks should have been executed
    thread::sleep(Duration::from_millis(500));
    assert!(
        hook_was_executed(&temp_path, "break_end_marker"),
        "on_break_end hook should execute when break completes"
    );
    assert!(
        hook_was_executed(&temp_path, "work_start_marker"),
        "on_work_start hook should execute after break_end"
    );
}

#[test]
fn test_end_hooks_execute_even_with_auto_advance_false() {
    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook script for work_end
    let work_end_script = create_hook_script(&temp_path, "work_end_hook.sh", "work_end_marker");

    // Create config with auto_advance = false
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = false

[hooks.on_work_end]
cmd = "{}"
"#,
        work_end_script.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Let work session complete
    thread::sleep(Duration::from_secs(7));

    // End hook should have executed even though timer is now paused
    thread::sleep(Duration::from_millis(500));
    assert!(
        hook_was_executed(&temp_path, "work_end_marker"),
        "on_work_end hook should execute regardless of auto_advance setting"
    );
}

#[test]
fn test_skip_hook_fires_before_phase_transition() {
    use std::fs;

    // Create temp dir for hooks and config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Create hook scripts that write the phase they see
    let skip_hook = temp_path.join("skip_hook.sh");
    let skip_marker = temp_path.join("skip_marker");
    let work_end_hook = temp_path.join("work_end_hook.sh");
    let work_end_marker = temp_path.join("work_end_marker");

    // Skip hook writes current phase
    let skip_content = format!(
        "#!/usr/bin/env bash\necho \"$TOMAT_PHASE\" > {}",
        skip_marker.display()
    );
    fs::write(&skip_hook, skip_content).expect("Failed to write skip hook");

    // Work end hook writes a marker
    let work_end_content = format!(
        "#!/usr/bin/env bash\necho \"work_ended\" > {}",
        work_end_marker.display()
    );
    fs::write(&work_end_hook, work_end_content).expect("Failed to write work end hook");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&skip_hook).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&skip_hook, perms).unwrap();

        let mut perms = fs::metadata(&work_end_hook).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&work_end_hook, perms).unwrap();
    }

    // Create config
    let config_path = temp_path.join("config.toml");
    let config_content = format!(
        r#"
[timer]
work = 0.1
break = 0.05
auto_advance = false

[hooks.on_skip]
cmd = "{}"

[hooks.on_work_end]
cmd = "{}"
"#,
        skip_hook.display(),
        work_end_hook.display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    // Start daemon with config
    let daemon = TestDaemon::start_with_config(Some(&config_path)).expect("Failed to start daemon");

    // Start timer
    daemon
        .send_command(&["start"])
        .expect("Failed to start timer");

    // Skip immediately
    thread::sleep(Duration::from_millis(100));
    daemon.send_command(&["skip"]).expect("Failed to skip");

    // Wait for hooks to execute
    thread::sleep(Duration::from_millis(500));

    // Verify skip hook executed and saw "work" phase (before transition)
    assert!(skip_marker.exists(), "skip hook should have executed");
    let phase_seen = fs::read_to_string(&skip_marker)
        .expect("Failed to read skip marker")
        .trim()
        .to_string();
    assert_eq!(
        phase_seen, "work",
        "skip hook should see 'work' phase (before transition to break)"
    );

    // Verify work_end hook also executed
    assert!(
        work_end_marker.exists(),
        "work_end hook should have executed after skip"
    );
}
