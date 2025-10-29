mod cli;
mod server;
mod timer;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::cli::{Cli, Commands, DaemonAction};
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

        Commands::Start { timer } => match send_command("start", timer.to_json()).await {
            Ok(response) => {
                if response.success {
                    println!(
                        "Pomodoro started: {}min work, {}min break, {}min long break every {} sessions",
                        timer.work, timer.break_time, timer.long_break, timer.sessions
                    );
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

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

        Commands::Toggle { timer } => match send_command("toggle", timer.to_json()).await {
            Ok(response) => {
                if response.success {
                    println!("{}", response.message);
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },
    }

    Ok(())
}
