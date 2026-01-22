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

/// Fetch and format timer status from daemon
async fn fetch_and_format_status(
    output_format: &str,
    text_template: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let args = serde_json::json!({
        "output": output_format,
    });

    let response = send_command("status", args).await?;

    if !response.success {
        return Err(response.message.into());
    }

    // Parse TimerStatus from response
    let timer_status: timer::TimerStatus = serde_json::from_value(response.data)?;

    // Parse output format
    let format_enum = output_format
        .parse::<timer::Format>()
        .unwrap_or(timer::Format::Waybar);

    // Format with client-side template
    let status_output =
        timer::TimerState::format_status(&timer_status, &format_enum, text_template);

    // Convert to string based on format type
    let output = match status_output {
        timer::StatusOutput::Plain(text) => text,
        _ => serde_json::to_string(&status_output)?,
    };

    Ok(output)
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
            // Only send values that were explicitly provided
            // Daemon will use config defaults for missing values
            let mut args = serde_json::json!({});

            if let Some(work) = timer.work {
                args["work"] = serde_json::json!(work);
            }
            if let Some(break_time) = timer.break_time {
                args["break"] = serde_json::json!(break_time);
            }
            if let Some(long_break) = timer.long_break {
                args["long_break"] = serde_json::json!(long_break);
            }
            if let Some(sessions) = timer.sessions {
                args["sessions"] = serde_json::json!(sessions);
            }
            if let Some(auto_advance) = &timer.auto_advance {
                args["auto_advance"] = serde_json::json!(auto_advance);
            }
            if let Some(sound_mode) = &timer.sound_mode {
                args["sound_mode"] = serde_json::json!(sound_mode);
            }
            if let Some(volume) = timer.volume {
                args["volume"] = serde_json::json!(volume);
            }

            match send_command("start", args).await {
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

        Commands::Status { output, format } => {
            // Load config only for display format default
            let text_template = format.unwrap_or_else(|| Config::load().display.text_format);

            match fetch_and_format_status(&output, &text_template).await {
                Ok(output) => println!("{}", output),
                Err(e) => eprintln!("Failed to connect to daemon: {}", e),
            }
        }

        Commands::Watch {
            output,
            format,
            interval,
        } => {
            // Load config only for display format default
            let text_template = format.unwrap_or_else(|| Config::load().display.text_format);
            let interval_duration = std::time::Duration::from_secs_f64(interval);

            loop {
                match fetch_and_format_status(&output, &text_template).await {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        eprintln!("Failed to connect to daemon: {}", e);
                        // Exit on error (daemon might be stopped)
                        break;
                    }
                }

                tokio::time::sleep(interval_duration).await;
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
# Inherit user's PATH for hooks to find system commands (e.g., notify-send)
Environment="PATH=/run/current-system/sw/bin:/etc/profiles/per-user/%u/bin:%h/.nix-profile/bin:%h/.cargo/bin:/usr/local/bin:/usr/bin:/bin"

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
