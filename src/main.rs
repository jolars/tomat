mod cli;
mod config;
mod server;
mod timer;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, Commands, DaemonAction};
use crate::config::Config;
use crate::server::{run_daemon, send_command};

#[derive(Serialize, Deserialize)]
struct ServerResponse {
    success: bool,
    data: serde_json::Value,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Daemon { action } => match action {
            DaemonAction::Start => {
                crate::server::start_daemon().await?;
            }
            DaemonAction::Stop => {
                crate::server::stop_daemon().await?;
            }
            DaemonAction::Status => {
                crate::server::daemon_status().await?;
            }
            DaemonAction::Run => {
                run_daemon().await?;
            }
        },

        Commands::Start { timer } => {
            let work = timer.get_work(config.timer.work);
            let break_time = timer.get_break_time(config.timer.break_time);
            let long_break = timer.get_long_break(config.timer.long_break);
            let sessions = timer.get_sessions(config.timer.sessions);
            let auto_advance = timer.get_auto_advance(config.timer.auto_advance);

            let args = serde_json::json!({
                "work": work,
                "break": break_time,
                "long_break": long_break,
                "sessions": sessions,
                "auto_advance": auto_advance
            });

            match send_command("start", args).await {
                Ok(response) => {
                    if response.success {
                        println!(
                            "Pomodoro started: {}min work, {}min break, {}min long break every {} sessions",
                            work, break_time, long_break, sessions
                        );
                    } else {
                        eprintln!("Error: {}", response.message);
                    }
                }
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }

        Commands::Stop => match send_command("stop", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("Timer stopped");
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Status => match send_command("status", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("{}", serde_json::to_string(&response.data)?);
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Skip => match send_command("skip", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("Skipped to next phase");
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Toggle { timer } => {
            let work = timer.get_work(config.timer.work);
            let break_time = timer.get_break_time(config.timer.break_time);
            let long_break = timer.get_long_break(config.timer.long_break);
            let sessions = timer.get_sessions(config.timer.sessions);
            let auto_advance = timer.get_auto_advance(config.timer.auto_advance);

            let args = serde_json::json!({
                "work": work,
                "break": break_time,
                "long_break": long_break,
                "sessions": sessions,
                "auto_advance": auto_advance
            });

            match send_command("toggle", args).await {
                Ok(response) => {
                    if response.success {
                        println!("{}", response.message);
                    } else {
                        eprintln!("Error: {}", response.message);
                    }
                }
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }
    }

    Ok(())
}
