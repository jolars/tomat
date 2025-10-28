use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
                .and_then(|v| v.as_u64())
                .unwrap_or(25) as u32;
            let break_time = message
                .args
                .get("break")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as u32;
            let long_break = message
                .args
                .get("long_break")
                .and_then(|v| v.as_u64())
                .unwrap_or(15) as u32;
            let sessions = message
                .args
                .get("sessions")
                .and_then(|v| v.as_u64())
                .unwrap_or(4) as u32;

            state.work_duration = work;
            state.break_duration = break_time;
            state.long_break_duration = long_break;
            state.sessions_until_long_break = sessions;
            state.current_session_count = 0;
            state.start_work();

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
                    .and_then(|v| v.as_u64())
                    .unwrap_or(25) as u32;
                let break_time = message
                    .args
                    .get("break")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5) as u32;
                let long_break = message
                    .args
                    .get("long_break")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(15) as u32;
                let sessions = message
                    .args
                    .get("sessions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(4) as u32;

                state.work_duration = work;
                state.break_duration = break_time;
                state.long_break_duration = long_break;
                state.sessions_until_long_break = sessions;
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

    // Remove existing socket
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;
    let mut state = TimerState::new(25, 5, 15, 4);

    println!("Tomat daemon listening on {:?}", socket_path);

    loop {
        tokio::select! {
            // Handle incoming connections
            Ok((stream, _)) = listener.accept() => {
                if let Err(e) = handle_client(stream, &mut state).await {
                    eprintln!("Error handling client: {}", e);
                }
            }

            // Auto-advance timer every second
            _ = sleep(Duration::from_secs(1)) => {
                if state.is_finished()
                    && let Err(e) = state.next_phase() {
                        eprintln!("Error during automatic phase transition: {}", e);
                    }
            }
        }
    }
}
