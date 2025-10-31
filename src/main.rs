mod audio;
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
            DaemonAction::Install => {
                install_systemd_service()?;
            }
            DaemonAction::Uninstall => {
                uninstall_systemd_service()?;
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
            let sound_enabled = timer.get_sound(config.sound.enabled);
            let system_beep = timer.get_beep(config.sound.system_beep);
            let volume = timer.get_volume(config.sound.volume);

            let args = serde_json::json!({
                "work": work,
                "break": break_time,
                "long_break": long_break,
                "sessions": sessions,
                "auto_advance": auto_advance,
                "sound_enabled": sound_enabled,
                "system_beep": system_beep,
                "volume": volume
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

        Commands::Status { output } => {
            let args = serde_json::json!({
                "output": output
            });

            match send_command("status", args).await {
                Ok(response) => {
                    if response.success {
                        // Handle plain format specially to avoid double JSON encoding
                        if output == "plain" {
                            // For plain format, extract the string content without quotes
                            if let Some(text) = response.data.as_str() {
                                println!("{}", text);
                            } else {
                                println!("{}", serde_json::to_string(&response.data)?);
                            }
                        } else {
                            println!("{}", serde_json::to_string(&response.data)?);
                        }
                    } else {
                        eprintln!("Error: {}", response.message);
                    }
                }
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }

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

        Commands::Pause => match send_command("pause", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("{}", response.message);
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Resume => match send_command("resume", serde_json::Value::Null).await {
            Ok(response) => {
                if response.success {
                    println!("{}", response.message);
                } else {
                    eprintln!("Error: {}", response.message);
                }
            }
            Err(e) => eprintln!("Failed to connect to daemon: {}", e),
        },

        Commands::Toggle => match send_command("toggle", serde_json::Value::Null).await {
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

/// Install systemd user service for tomat daemon
fn install_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Get the current executable path
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // Create systemd user directory using XDG config directory
    let systemd_dir = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("systemd").join("user")
    } else {
        // Fallback to HOME/.config if XDG config dir is not available
        let home = std::env::var("HOME")?;
        std::path::PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user")
    };

    fs::create_dir_all(&systemd_dir)?;

    // Generate service file content
    let service_content = format!(
        r#"[Unit]
Description=Tomat Pomodoro Timer Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart={} daemon run
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
"#,
        exe_path_str
    );

    // Write service file
    let service_path = systemd_dir.join("tomat.service");
    fs::write(&service_path, service_content)?;

    println!(
        "✓ Systemd service file installed to: {}",
        service_path.display()
    );

    // Reload systemd and enable service
    let reload_result = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    match reload_result {
        Ok(status) if status.success() => {
            println!("✓ Systemd daemon reloaded");

            let enable_result = std::process::Command::new("systemctl")
                .args(["--user", "enable", "tomat.service"])
                .status();

            match enable_result {
                Ok(status) if status.success() => {
                    println!("✓ Tomat service enabled");
                    println!("\nService installed successfully!");
                    println!("\nTo start the daemon:");
                    println!("  systemctl --user start tomat.service");
                    println!("\nTo check status:");
                    println!("  systemctl --user status tomat.service");
                    println!("\nTo enable auto-start on login:");
                    println!("  loginctl enable-linger $USER");
                }
                Ok(_) => {
                    eprintln!("⚠ Warning: Failed to enable tomat.service");
                    eprintln!(
                        "You can enable it manually with: systemctl --user enable tomat.service"
                    );
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to run systemctl enable: {}", e);
                    eprintln!(
                        "You can enable it manually with: systemctl --user enable tomat.service"
                    );
                }
            }
        }
        Ok(_) => {
            eprintln!("⚠ Warning: Failed to reload systemd daemon");
            eprintln!("You can reload manually with: systemctl --user daemon-reload");
        }
        Err(e) => {
            eprintln!("⚠ Warning: Failed to run systemctl daemon-reload: {}", e);
            eprintln!("Systemctl might not be available or you might not be using systemd");
        }
    }

    Ok(())
}

/// Uninstall systemd user service for tomat daemon
fn uninstall_systemd_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Use XDG config directory consistently
    let service_path = if let Some(config_dir) = dirs::config_dir() {
        config_dir
            .join("systemd")
            .join("user")
            .join("tomat.service")
    } else {
        // Fallback to HOME/.config if XDG config dir is not available
        let home = std::env::var("HOME")?;
        std::path::PathBuf::from(home)
            .join(".config")
            .join("systemd")
            .join("user")
            .join("tomat.service")
    };

    // Check if service file exists
    if !service_path.exists() {
        println!("Tomat service is not installed (service file not found)");
        return Ok(());
    }

    // Try to stop and disable the service first
    let stop_result = std::process::Command::new("systemctl")
        .args(["--user", "stop", "tomat.service"])
        .status();

    match stop_result {
        Ok(status) if status.success() => println!("✓ Tomat service stopped"),
        Ok(_) => eprintln!("⚠ Warning: Failed to stop tomat.service (might not be running)"),
        Err(e) => eprintln!("⚠ Warning: Failed to run systemctl stop: {}", e),
    }

    let disable_result = std::process::Command::new("systemctl")
        .args(["--user", "disable", "tomat.service"])
        .status();

    match disable_result {
        Ok(status) if status.success() => println!("✓ Tomat service disabled"),
        Ok(_) => eprintln!("⚠ Warning: Failed to disable tomat.service"),
        Err(e) => eprintln!("⚠ Warning: Failed to run systemctl disable: {}", e),
    }

    // Remove service file
    match fs::remove_file(&service_path) {
        Ok(()) => {
            println!("✓ Service file removed: {}", service_path.display());

            // Reload systemd
            let reload_result = std::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();

            match reload_result {
                Ok(status) if status.success() => println!("✓ Systemd daemon reloaded"),
                Ok(_) => eprintln!("⚠ Warning: Failed to reload systemd daemon"),
                Err(e) => eprintln!("⚠ Warning: Failed to run systemctl daemon-reload: {}", e),
            }

            println!("\nTomat service uninstalled successfully!");
        }
        Err(e) => {
            eprintln!("Failed to remove service file: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
