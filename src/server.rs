use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::ServerResponse;
use crate::timer::TimerState;

#[derive(Serialize, Deserialize)]
struct ClientMessage {
    command: String,
    args: serde_json::Value,
}

fn get_socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
        .join("tomat.sock")
}

fn get_pid_file_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
        .join("tomat.pid")
}

fn get_state_file_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{}", unsafe { libc::getuid() })))
        .join("tomat.state")
}

/// Save timer state to disk
fn save_state(state: &TimerState) {
    let state_path = get_state_file_path();
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&state_path, json) {
                eprintln!("Failed to save timer state: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize timer state: {}", e);
        }
    }
}

/// Load timer state from disk
fn load_state() -> Option<TimerState> {
    let state_path = get_state_file_path();

    if !state_path.exists() {
        return None;
    }

    match std::fs::read_to_string(&state_path) {
        Ok(contents) => match serde_json::from_str::<TimerState>(&contents) {
            Ok(state) => {
                println!("Restored timer state from {:?}", state_path);
                println!(
                    "  State: phase={:?}, paused={}, work={}min, break={}min, long_break={}min",
                    state.phase,
                    state.is_paused,
                    state.work_duration,
                    state.break_duration,
                    state.long_break_duration
                );
                Some(state)
            }
            Err(e) => {
                eprintln!(
                    "Failed to parse state file (corrupted?): {}. Starting with fresh state.",
                    e
                );
                // Remove corrupted state file
                let _ = std::fs::remove_file(&state_path);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to read state file: {}", e);
            None
        }
    }
}

/// Validate timer parameters
fn validate_timer_params(
    work: f32,
    break_time: f32,
    long_break: f32,
    sessions: u32,
) -> Result<(), String> {
    // Validate work duration
    if work <= 0.0 {
        return Err("Work duration must be greater than 0".to_string());
    }
    if work > 600.0 {
        return Err("Work duration must be 600 minutes (10 hours) or less".to_string());
    }

    // Validate break duration
    if break_time <= 0.0 {
        return Err("Break duration must be greater than 0".to_string());
    }
    if break_time > 600.0 {
        return Err("Break duration must be 600 minutes (10 hours) or less".to_string());
    }

    // Validate long break duration
    if long_break <= 0.0 {
        return Err("Long break duration must be greater than 0".to_string());
    }
    if long_break > 600.0 {
        return Err("Long break duration must be 600 minutes (10 hours) or less".to_string());
    }

    // Validate sessions
    if sessions == 0 {
        return Err("Sessions must be at least 1".to_string());
    }
    if sessions > 100 {
        return Err("Sessions must be 100 or less".to_string());
    }

    Ok(())
}

pub async fn send_command(
    command: &str,
    args: serde_json::Value,
) -> Result<ServerResponse, Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path).await?;

    let message = ClientMessage {
        command: command.to_string(),
        args,
    };

    let request = serde_json::to_string(&message)?;
    stream.write_all(request.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    Ok(serde_json::from_str(&response)?)
}

/// Execute a hook asynchronously (fire-and-forget)
fn execute_hook(hooks: &crate::config::HooksConfig, event: &str, state: &TimerState) {
    let hooks = hooks.clone();
    let phase_str = state.phase.to_string();
    let remaining = state.get_remaining_seconds();
    let session_count = state.current_session_count;
    let auto_advance = format!("{:?}", state.auto_advance).to_lowercase();
    let event = event.to_string();

    tokio::spawn(async move {
        hooks
            .execute_hook(&event, &phase_str, remaining, session_count, &auto_advance)
            .await;
    });
}

async fn handle_client(
    stream: UnixStream,
    state: &mut TimerState,
    config: &crate::config::Config,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if reader.read_line(&mut line).await? == 0 {
        return Ok(false);
    }

    let message: ClientMessage = serde_json::from_str(&line)?;

    let response = match message.command.as_str() {
        "start" => {
            // Load config fresh for each start command
            let fresh_config = crate::config::Config::load();

            let work = message
                .args
                .get("work")
                .and_then(|v| v.as_f64())
                .unwrap_or(fresh_config.timer.work as f64) as f32;
            let break_time = message
                .args
                .get("break")
                .and_then(|v| v.as_f64())
                .unwrap_or(fresh_config.timer.break_time as f64)
                as f32;
            let long_break = message
                .args
                .get("long_break")
                .and_then(|v| v.as_f64())
                .unwrap_or(fresh_config.timer.long_break as f64)
                as f32;
            let sessions = message
                .args
                .get("sessions")
                .and_then(|v| v.as_u64())
                .unwrap_or(fresh_config.timer.sessions as u64) as u32;
            let auto_advance = message
                .args
                .get("auto_advance")
                .and_then(|v| {
                    // Try as string first (new format)
                    if let Some(s) = v.as_str() {
                        s.parse::<crate::config::AutoAdvanceMode>().ok()
                    } else {
                        v.as_bool().map(|b| {
                            if b {
                                crate::config::AutoAdvanceMode::All
                            } else {
                                crate::config::AutoAdvanceMode::None
                            }
                        })
                    }
                })
                .unwrap_or_else(|| fresh_config.timer.auto_advance.clone());

            // Parse sound_mode (ignore for now, not stored in state)
            let _sound_mode = message
                .args
                .get("sound_mode")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<crate::config::SoundMode>().ok())
                .unwrap_or(crate::config::SoundMode::Embedded);

            let _volume = message
                .args
                .get("volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32;

            // Validate parameters
            if let Err(err_msg) = validate_timer_params(work, break_time, long_break, sessions) {
                ServerResponse {
                    success: false,
                    data: serde_json::Value::Null,
                    message: err_msg,
                }
            } else {
                state.work_duration = work;
                state.break_duration = break_time;
                state.long_break_duration = long_break;
                state.sessions_until_long_break = sessions;
                state.auto_advance = auto_advance;
                state.current_session_count = 0;

                // Always start a fresh work session
                state.start_work();

                // Execute work_start hook
                execute_hook(&config.hooks, "work_start", state);

                // Save state after starting
                save_state(state);

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: format!(
                        "Pomodoro started: {:.1}min work, {:.1}min break, {:.1}min long break every {} sessions",
                        work, break_time, long_break, sessions
                    ),
                }
            }
        }
        "stop" => {
            state.stop();

            // Execute hook
            execute_hook(&config.hooks, "stop", state);

            // Save state after stopping
            save_state(state);

            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Timer stopped".to_string(),
            }
        }
        "status" => {
            let format_str = message
                .args
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("waybar");

            match format_str.parse::<crate::timer::Format>() {
                Ok(_format) => {
                    // Return raw timer status for client-side formatting
                    let timer_status = state.get_timer_status();
                    let data = serde_json::to_value(timer_status)?;

                    ServerResponse {
                        success: true,
                        data,
                        message: "Status retrieved".to_string(),
                    }
                }
                Err(e) => ServerResponse {
                    success: false,
                    data: serde_json::Value::Null,
                    message: e,
                },
            }
        }
        "skip" => {
            // Execute skip hook BEFORE phase transition
            execute_hook(&config.hooks, "skip", state);

            if let Err(e) = state.next_phase(&config.sound, &config.notification, &config.hooks) {
                eprintln!("Error during phase transition: {}", e);
            }

            // Save state after phase transition
            save_state(state);

            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Skipped to next phase".to_string(),
            }
        }
        "toggle" => {
            if state.is_paused {
                // Check if this is the first toggle on an uninitialized timer
                // (start_time == 0 means timer has never been started)
                if state.start_time == 0 {
                    // Load fresh config to get user's configured defaults
                    let fresh_config = crate::config::Config::load();

                    // Initialize timer state with config defaults if not already set via CLI
                    state.work_duration = fresh_config.timer.work;
                    state.break_duration = fresh_config.timer.break_time;
                    state.long_break_duration = fresh_config.timer.long_break;
                    state.sessions_until_long_break = fresh_config.timer.sessions;
                    state.auto_advance = fresh_config.timer.auto_advance;
                    state.duration_minutes = state.work_duration;
                }

                // Resume if paused
                let pending_hook = state.resume();

                // Execute resume hook
                execute_hook(&config.hooks, "resume", state);

                // Execute pending phase hook if any
                if let Some(hook_event) = pending_hook {
                    execute_hook(&config.hooks, &hook_event, state);
                }

                // Save state after resuming
                save_state(state);

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer resumed".to_string(),
                }
            } else {
                // Pause timer if running (preserves progress)
                state.pause();

                // Execute hook
                execute_hook(&config.hooks, "pause", state);

                // Save state after pausing
                save_state(state);

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer paused".to_string(),
                }
            }
        }
        "pause" => {
            if state.is_paused {
                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer is already paused".to_string(),
                }
            } else {
                state.pause();

                // Execute hook
                execute_hook(&config.hooks, "pause", state);

                // Save state after pausing
                save_state(state);

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer paused".to_string(),
                }
            }
        }
        "resume" => {
            if !state.is_paused {
                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer is already running".to_string(),
                }
            } else {
                let pending_hook = state.resume();

                // Execute resume hook
                execute_hook(&config.hooks, "resume", state);

                // Execute pending phase hook if any
                if let Some(hook_event) = pending_hook {
                    execute_hook(&config.hooks, &hook_event, state);
                }

                // Save state after resuming
                save_state(state);

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer resumed".to_string(),
                }
            }
        }
        "shutdown" => {
            save_state(state);
            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Daemon shutting down".to_string(),
            }
        }
        _ => ServerResponse {
            success: false,
            data: serde_json::Value::Null,
            message: "Unknown command".to_string(),
        },
    };

    let should_shutdown = message.command == "shutdown";

    let response_json = serde_json::to_string(&response)?;
    let mut writer = reader.into_inner();
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(should_shutdown)
}

pub async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();
    let pid_file_path = get_pid_file_path();

    // Create and lock PID file to prevent multiple daemon instances
    let mut pid_file = File::create(&pid_file_path)?;
    pid_file.try_lock_exclusive().map_err(|_| {
        format!(
            "Another daemon instance is already running. PID file locked: {:?}",
            pid_file_path
        )
    })?;

    // Write current PID to the locked file
    let pid = std::process::id();
    write!(pid_file, "{}", pid)?;
    pid_file.flush()?;

    // Now that we have the exclusive lock, safely remove existing socket if present
    // This is safe because we're the only daemon instance that can run now
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Load configuration first
    let config = crate::config::Config::load_with_logging(true);

    // Try to load existing state, fallback to config defaults if not found
    let mut state = load_state().unwrap_or_else(|| {
        println!("No existing state found, starting with config defaults");
        println!(
            "  Using: work={}min, break={}min, long_break={}min, sessions={}",
            config.timer.work,
            config.timer.break_time,
            config.timer.long_break,
            config.timer.sessions
        );
        TimerState::new(
            config.timer.work,
            config.timer.break_time,
            config.timer.long_break,
            config.timer.sessions,
        )
    });

    println!("Tomat daemon listening on {:?}", socket_path);

    // Clean up socket and PID file on exit
    let cleanup = || {
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&pid_file_path);
    };

    // Set up signal handler for graceful shutdown
    let result = tokio::select! {
        result = daemon_loop(listener, &mut state, &config) => result,
        _ = tokio::signal::ctrl_c() => {
            println!("Received interrupt signal, shutting down...");
            Ok(())
        }
    };

    // Keep the PID file lock alive until here (by keeping _pid_file in scope)
    drop(pid_file);
    cleanup();
    result
}

async fn daemon_loop(
    listener: UnixListener,
    state: &mut TimerState,
    config: &crate::config::Config,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            // Handle incoming connections
            Ok((stream, _)) = listener.accept() => {
                match handle_client(stream, state, config).await {
                    Ok(should_shutdown) if should_shutdown => {
                        println!("Shutdown requested, exiting gracefully");
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("Error handling client: {}", e);
                    }
                    _ => {}
                }
            }

            // Check timer completion with precise timing
            _ = async {
                if let Some(finish_timestamp) = state.get_finish_time() {
                    // Timer is running, calculate exact sleep duration
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if finish_timestamp > current_time {
                        // Timer hasn't finished yet, sleep until it does
                        let sleep_duration = Duration::from_secs(finish_timestamp - current_time);
                        tokio::time::sleep(sleep_duration).await;
                    }
                    // If finish_timestamp <= current_time, timer is already finished, so don't sleep
                } else {
                    // Timer is paused, check again after 1 second
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            } => {
                if state.is_finished() {
                    if let Err(e) = state.next_phase(&config.sound, &config.notification, &config.hooks) {
                        eprintln!("Error during phase transition: {}", e);
                    }
                    // Save state after automatic phase transition
                    save_state(state);
                }
            }
        }
    }
}

/// Start the daemon in the background
pub async fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = get_pid_file_path();
    let socket_path = get_socket_path();

    // Check if daemon is already running by trying to read and verify PID file
    if let Ok(pid_str) = std::fs::read_to_string(&pid_file_path)
        && let Ok(pid) = pid_str.trim().parse::<u32>()
    {
        if is_process_running(pid) {
            println!(
                "Daemon is already running (PID: {}). Use 'tomat daemon stop' to stop it first.",
                pid
            );
            return Ok(());
        } else {
            // Stale PID file found - try to clean it up
            println!(
                "Found stale PID file (PID {} no longer running), cleaning up...",
                pid
            );
            let _ = std::fs::remove_file(&pid_file_path);
            let _ = std::fs::remove_file(&socket_path);
        }
    }

    // Try to lock the PID file to prevent race conditions with concurrent start attempts
    // We keep this lock until the spawned daemon creates its own lock
    let lock_file = File::create(&pid_file_path)?;
    lock_file
        .try_lock_exclusive()
        .map_err(|_| "Another daemon is starting up right now. Please wait and try again.")?;

    // Get the current executable path
    let exe_path = std::env::current_exe()?;

    // Start daemon in background
    let child = Command::new(&exe_path)
        .arg("daemon")
        .arg("run") // Internal command to actually run the daemon
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let child_pid = child.id();
    println!("Started daemon in background (PID: {})", child_pid);

    // Release the lock so the daemon can acquire it
    // The small time window here is acceptable because the daemon is already running
    drop(lock_file);

    // Poll for daemon startup with timeout (max 2 seconds, check every 10ms)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(2);
    let poll_interval = Duration::from_millis(10);

    loop {
        // Check if socket and PID file exist
        if socket_path.exists() && pid_file_path.exists() {
            // Verify this is OUR daemon by checking the PID
            if let Ok(pid_str) = std::fs::read_to_string(&pid_file_path)
                && let Ok(pid) = pid_str.trim().parse::<u32>()
            {
                if pid == child_pid {
                    // Our daemon successfully wrote its PID - now verify it responds
                    match send_command("status", serde_json::Value::Null).await {
                        Ok(_) => {
                            println!("Daemon started successfully");
                            return Ok(());
                        }
                        Err(_) if start.elapsed() < timeout => {
                            // Daemon not ready yet, keep waiting
                        }
                        Err(_) => {
                            return Err(
                                "Failed to start daemon - socket exists but daemon not responding"
                                    .into(),
                            );
                        }
                    }
                } else if start.elapsed() >= timeout {
                    // Different PID in file and timeout reached - another daemon won
                    return Err(
                        format!("Another daemon instance (PID: {}) started first", pid).into(),
                    );
                }
                // else: wrong PID but still time left, keep waiting
            }
        }

        if start.elapsed() > timeout {
            return Err(
                "Failed to start daemon - socket or PID file not created within timeout".into(),
            );
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Stop the running daemon
pub async fn stop_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = get_pid_file_path();
    let socket_path = get_socket_path();

    // Read PID from file
    let pid_str = match std::fs::read_to_string(&pid_file_path) {
        Ok(content) => content,
        Err(_) => {
            println!("No daemon PID file found");
            return Ok(());
        }
    };

    let pid = match pid_str.trim().parse::<u32>() {
        Ok(pid) => pid,
        Err(_) => {
            println!("Invalid PID in file, cleaning up");
            let _ = std::fs::remove_file(&pid_file_path);
            let _ = std::fs::remove_file(&socket_path);
            return Ok(());
        }
    };

    // Check if process is running
    if !is_process_running(pid) {
        println!("Daemon is not running, cleaning up stale files");
        let _ = std::fs::remove_file(&pid_file_path);
        let _ = std::fs::remove_file(&socket_path);
        return Ok(());
    }

    // Try graceful shutdown via socket command first
    match send_command("shutdown", serde_json::Value::Null).await {
        Ok(_) => {
            println!("Sent shutdown command to daemon");

            // Wait up to 5 seconds for graceful shutdown
            for _ in 0..50 {
                if !is_process_running(pid) {
                    println!("Daemon stopped gracefully");
                    let _ = std::fs::remove_file(&pid_file_path);
                    let _ = std::fs::remove_file(&socket_path);
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            println!("Daemon did not respond to shutdown command, trying signal-based shutdown");
        }
        Err(_) => {
            println!(
                "Could not send shutdown command (daemon may be unresponsive), trying signal-based shutdown"
            );
        }
    }

    // Fallback to signal-based shutdown
    unsafe {
        if libc::kill(pid as i32, libc::SIGTERM) == 0 {
            println!("Sent SIGTERM to daemon (PID: {})", pid);

            // Wait up to 5 seconds for graceful shutdown
            for _ in 0..50 {
                if !is_process_running(pid) {
                    println!("Daemon stopped gracefully");
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            // If still running, force kill
            if is_process_running(pid) {
                if libc::kill(pid as i32, libc::SIGKILL) == 0 {
                    println!("Force killed daemon (PID: {})", pid);
                } else {
                    return Err(format!("Failed to kill daemon process {}", pid).into());
                }
            }
        } else {
            return Err(format!("Failed to send signal to daemon process {}", pid).into());
        }
    }

    // Clean up files
    let _ = std::fs::remove_file(&pid_file_path);
    let _ = std::fs::remove_file(&socket_path);

    Ok(())
}

/// Check daemon status
pub async fn daemon_status() -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = get_pid_file_path();
    let socket_path = get_socket_path();

    // Check if PID file exists
    let pid = match std::fs::read_to_string(&pid_file_path) {
        Ok(content) => match content.trim().parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => {
                println!("Status: Not running (invalid PID file)");
                return Ok(());
            }
        },
        Err(_) => {
            println!("Status: Not running (no PID file)");
            return Ok(());
        }
    };

    // Check if process is actually running
    if !is_process_running(pid) {
        println!("Status: Not running (stale PID file)");
        return Ok(());
    }

    // Check if socket exists and is responsive
    if socket_path.exists() {
        // Try to connect to the daemon
        match send_command("status", serde_json::Value::Null).await {
            Ok(_) => {
                println!("Status: Running (PID: {}, socket: {:?})", pid, socket_path);
            }
            Err(_) => {
                println!("Status: Running but unresponsive (PID: {})", pid);
            }
        }
    } else {
        println!("Status: Process running but no socket (PID: {})", pid);
    }

    Ok(())
}

fn is_process_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_socket_path_uses_xdg_runtime_dir() {
        let socket_path = get_socket_path();
        let path_str = socket_path.to_string_lossy();

        assert!(
            path_str.contains("tomat.sock"),
            "Socket path should end with tomat.sock"
        );
    }

    #[test]
    fn test_get_pid_file_path_uses_xdg_runtime_dir() {
        let pid_path = get_pid_file_path();
        let path_str = pid_path.to_string_lossy();

        assert!(
            path_str.contains("tomat.pid"),
            "PID file path should end with tomat.pid"
        );
    }

    #[test]
    fn test_socket_and_pid_paths_in_same_directory() {
        let socket_path = get_socket_path();
        let pid_path = get_pid_file_path();

        assert_eq!(
            socket_path.parent(),
            pid_path.parent(),
            "Socket and PID file should be in the same directory"
        );
    }

    #[test]
    fn test_client_message_serialization() {
        let message = ClientMessage {
            command: "start".to_string(),
            args: serde_json::json!({
                "work": 25.0,
                "break": 5.0
            }),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.command, "start");
        assert_eq!(deserialized.args["work"], 25.0);
        assert_eq!(deserialized.args["break"], 5.0);
    }

    #[test]
    fn test_server_response_serialization() {
        let response = ServerResponse {
            success: true,
            data: serde_json::json!({"text": "🍅 25:00 ⏸"}),
            message: "Status retrieved".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ServerResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.success);
        assert_eq!(deserialized.message, "Status retrieved");
        assert_eq!(deserialized.data["text"], "🍅 25:00 ⏸");
    }

    #[test]
    fn test_is_process_running_for_self() {
        let current_pid = std::process::id();

        assert!(
            is_process_running(current_pid),
            "Current process should be detected as running"
        );
    }

    #[test]
    fn test_is_process_running_for_nonexistent_pid() {
        // Use a very high PID that is very unlikely to exist
        // We try multiple PIDs to avoid flakiness
        let nonexistent_pids = [99999, 99998, 99997];

        // At least one of these should not exist
        let any_nonexistent = nonexistent_pids.iter().any(|&pid| !is_process_running(pid));

        assert!(
            any_nonexistent,
            "At least one high PID should not be running"
        );
    }

    #[test]
    fn test_client_message_with_all_args() {
        let message = ClientMessage {
            command: "start".to_string(),
            args: serde_json::json!({
                "work": 30.0,
                "break": 10.0,
                "long_break": 20.0,
                "sessions": 3,
                "auto_advance": true
            }),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.command, "start");
        assert_eq!(deserialized.args["work"], 30.0);
        assert_eq!(deserialized.args["break"], 10.0);
        assert_eq!(deserialized.args["long_break"], 20.0);
        assert_eq!(deserialized.args["sessions"], 3);
        assert_eq!(deserialized.args["auto_advance"], true);
    }

    #[test]
    fn test_client_message_with_null_args() {
        let message = ClientMessage {
            command: "status".to_string(),
            args: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.command, "status");
        assert!(deserialized.args.is_null());
    }

    #[test]
    fn test_server_response_error() {
        let response = ServerResponse {
            success: false,
            data: serde_json::Value::Null,
            message: "Unknown command".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ServerResponse = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.success);
        assert_eq!(deserialized.message, "Unknown command");
        assert!(deserialized.data.is_null());
    }

    #[test]
    fn test_paths_are_absolute() {
        let socket_path = get_socket_path();
        let pid_path = get_pid_file_path();

        assert!(socket_path.is_absolute(), "Socket path should be absolute");
        assert!(pid_path.is_absolute(), "PID file path should be absolute");
    }

    #[test]
    fn test_validate_timer_params_valid() {
        assert!(validate_timer_params(25.0, 5.0, 15.0, 4).is_ok());
        assert!(validate_timer_params(0.1, 0.1, 0.1, 1).is_ok());
        assert!(validate_timer_params(600.0, 600.0, 600.0, 100).is_ok());
    }

    #[test]
    fn test_validate_timer_params_zero_work() {
        let result = validate_timer_params(0.0, 5.0, 15.0, 4);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Work duration must be greater than 0")
        );
    }

    #[test]
    fn test_validate_timer_params_negative_work() {
        let result = validate_timer_params(-5.0, 5.0, 15.0, 4);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Work duration must be greater than 0")
        );
    }

    #[test]
    fn test_validate_timer_params_excessive_work() {
        let result = validate_timer_params(700.0, 5.0, 15.0, 4);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("600 minutes"));
    }

    #[test]
    fn test_validate_timer_params_zero_break() {
        let result = validate_timer_params(25.0, 0.0, 15.0, 4);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Break duration must be greater than 0")
        );
    }

    #[test]
    fn test_validate_timer_params_excessive_long_break() {
        let result = validate_timer_params(25.0, 5.0, 700.0, 4);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("600 minutes"));
    }

    #[test]
    fn test_validate_timer_params_zero_sessions() {
        let result = validate_timer_params(25.0, 5.0, 15.0, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Sessions must be at least 1"));
    }

    #[test]
    fn test_validate_timer_params_excessive_sessions() {
        let result = validate_timer_params(25.0, 5.0, 15.0, 150);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("100 or less"));
    }

    #[test]
    fn test_state_persistence_round_trip() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        // SAFETY: Setting environment variable during tests is safe as tests have isolated environments
        unsafe {
            std::env::set_var("XDG_RUNTIME_DIR", temp_dir.path());
        }

        // Create a timer state
        let mut state = TimerState::new(30.0, 10.0, 20.0, 3);
        state.start_work();
        state.current_session_count = 2;
        state.auto_advance = crate::config::AutoAdvanceMode::All;

        // Save the state
        save_state(&state);

        // Load the state
        let loaded_state = load_state().expect("Should load state");

        // Verify all fields match
        assert_eq!(loaded_state.work_duration, 30.0);
        assert_eq!(loaded_state.break_duration, 10.0);
        assert_eq!(loaded_state.long_break_duration, 20.0);
        assert_eq!(loaded_state.sessions_until_long_break, 3);
        assert_eq!(loaded_state.current_session_count, 2);
        assert_eq!(
            loaded_state.auto_advance,
            crate::config::AutoAdvanceMode::All
        );
        assert!(!loaded_state.is_paused);
    }

    #[test]
    fn test_state_file_path_uses_xdg_runtime_dir() {
        let state_path = get_state_file_path();
        let path_str = state_path.to_string_lossy();

        assert!(
            path_str.contains("tomat.state"),
            "State file path should end with tomat.state"
        );
    }
}
