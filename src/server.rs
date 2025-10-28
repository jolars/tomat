use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::time::sleep;

use crate::ServerResponse;
use crate::timer::{Phase, TimerState};

#[derive(Serialize, Deserialize)]
struct ClientMessage {
    command: String,
    args: serde_json::Value,
}

fn get_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    PathBuf::from(runtime_dir).join("tomat.sock")
}

fn get_pid_file_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    PathBuf::from(runtime_dir).join("tomat.pid")
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

async fn handle_client(
    stream: UnixStream,
    state: &mut TimerState,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if reader.read_line(&mut line).await? == 0 {
        return Ok(());
    }

    let message: ClientMessage = serde_json::from_str(&line)?;

    let response = match message.command.as_str() {
        "start" => {
            let work = message
                .args
                .get("work")
                .and_then(|v| v.as_f64())
                .unwrap_or(25.0) as f32;
            let break_time = message
                .args
                .get("break")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;
            let long_break = message
                .args
                .get("long_break")
                .and_then(|v| v.as_f64())
                .unwrap_or(15.0) as f32;
            let sessions = message
                .args
                .get("sessions")
                .and_then(|v| v.as_u64())
                .unwrap_or(4) as u32;
            let auto_advance = message
                .args
                .get("auto_advance")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            state.work_duration = work;
            state.break_duration = break_time;
            state.long_break_duration = long_break;
            state.sessions_until_long_break = sessions;
            state.auto_advance = auto_advance;
            state.current_session_count = 0;

            if state.is_paused && !matches!(state.phase, Phase::Idle) {
                // Resume if paused
                state.resume();
            } else {
                // Start new session
                state.start_work();
            }

            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Timer started".to_string(),
            }
        }
        "stop" => {
            state.stop();
            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Timer stopped".to_string(),
            }
        }
        "status" => {
            let status = state.get_status_output();
            ServerResponse {
                success: true,
                data: serde_json::to_value(status)?,
                message: "Status retrieved".to_string(),
            }
        }
        "skip" => {
            if let Err(e) = state.next_phase() {
                eprintln!("Error during phase transition: {}", e);
            }
            ServerResponse {
                success: true,
                data: serde_json::Value::Null,
                message: "Skipped to next phase".to_string(),
            }
        }
        "toggle" => {
            if matches!(state.phase, Phase::Idle) {
                // Start timer if idle
                let work = message
                    .args
                    .get("work")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(25.0) as f32;
                let break_time = message
                    .args
                    .get("break")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(5.0) as f32;
                let long_break = message
                    .args
                    .get("long_break")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(15.0) as f32;
                let sessions = message
                    .args
                    .get("sessions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(4) as u32;
                let auto_advance = message
                    .args
                    .get("auto_advance")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                state.work_duration = work;
                state.break_duration = break_time;
                state.long_break_duration = long_break;
                state.sessions_until_long_break = sessions;
                state.auto_advance = auto_advance;
                state.current_session_count = 0;
                state.start_work();

                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: format!(
                        "Timer started: {}min work, {}min break, {}min long break every {} sessions",
                        work, break_time, long_break, sessions
                    ),
                }
            } else if state.is_paused {
                // Resume if paused
                state.resume();
                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer resumed".to_string(),
                }
            } else {
                // Stop timer if running
                state.stop();
                ServerResponse {
                    success: true,
                    data: serde_json::Value::Null,
                    message: "Timer stopped".to_string(),
                }
            }
        }
        _ => ServerResponse {
            success: false,
            data: serde_json::Value::Null,
            message: "Unknown command".to_string(),
        },
    };

    let response_json = serde_json::to_string(&response)?;
    let mut writer = reader.into_inner();
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

pub async fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();
    let pid_file_path = get_pid_file_path();

    // Remove existing socket
    let _ = std::fs::remove_file(&socket_path);

    // Write PID file
    std::fs::write(&pid_file_path, std::process::id().to_string())?;

    let listener = UnixListener::bind(&socket_path)?;
    let mut state = TimerState::new(25.0, 5.0, 15.0, 4);

    println!("Tomat daemon listening on {:?}", socket_path);

    // Clean up PID file on exit
    let cleanup_pid_file = || {
        let _ = std::fs::remove_file(&pid_file_path);
    };

    // Set up signal handler for graceful shutdown
    let result = tokio::select! {
        result = daemon_loop(listener, &mut state) => result,
        _ = tokio::signal::ctrl_c() => {
            println!("Received interrupt signal, shutting down...");
            Ok(())
        }
    };

    cleanup_pid_file();
    result
}

async fn daemon_loop(
    listener: UnixListener,
    state: &mut TimerState,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            // Handle incoming connections
            Ok((stream, _)) = listener.accept() => {
                if let Err(e) = handle_client(stream, state).await {
                    eprintln!("Error handling client: {}", e);
                }
            }

            // Check timer completion every second
            _ = sleep(Duration::from_secs(1)) => {
                if state.is_finished()
                    && let Err(e) = state.next_phase() {
                        eprintln!("Error during phase transition: {}", e);
                    }
            }
        }
    }
}

/// Start the daemon in the background
pub async fn start_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = get_pid_file_path();
    let socket_path = get_socket_path();

    // Check if daemon is already running
    if let Ok(pid_str) = std::fs::read_to_string(&pid_file_path)
        && let Ok(pid) = pid_str.trim().parse::<u32>()
    {
        if is_process_running(pid) {
            println!("Daemon is already running (PID: {})", pid);
            return Ok(());
        } else {
            // Stale PID file, remove it
            let _ = std::fs::remove_file(&pid_file_path);
            let _ = std::fs::remove_file(&socket_path);
        }
    }

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

    println!("Started daemon in background (PID: {})", child.id());

    // Wait a moment to ensure daemon starts
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify daemon is running
    if socket_path.exists() {
        println!("Daemon started successfully");
    } else {
        return Err("Failed to start daemon".into());
    }

    Ok(())
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

    // Try to kill the process
    #[cfg(unix)]
    {
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
    }

    #[cfg(not(unix))]
    {
        return Err("Daemon killing not supported on this platform".into());
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
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, we can't easily check if a PID is running
        // This is a fallback that assumes the process might be running
        true
    }
}
