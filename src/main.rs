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

fn serialize_status_output(
    status_output: timer::StatusOutput,
) -> Result<String, Box<dyn std::error::Error>> {
    match status_output {
        timer::StatusOutput::Plain(text) => Ok(text),
        _ => Ok(serde_json::to_string(&status_output)?),
    }
}

fn format_disconnected_status(output_format: &str) -> Result<String, Box<dyn std::error::Error>> {
    let format = output_format.parse::<timer::Format>()?;
    serialize_status_output(timer::StatusOutput::disconnected(&format))
}

fn is_daemon_unavailable(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);

    while let Some(error) = current {
        if let Some(error) = error.downcast_ref::<std::io::Error>()
            && matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            )
        {
            return true;
        }

        current = error.source();
    }

    false
}

fn handle_response(
    response: ServerResponse,
    quiet: bool,
    success_message: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !response.success {
        return Err(response.message.into());
    }

    if !quiet {
        println!("{}", success_message.unwrap_or(&response.message));
    }

    Ok(())
}

fn connection_error(error: impl std::fmt::Display) -> Box<dyn std::error::Error> {
    format!("Failed to connect to daemon: {error}").into()
}

/// Fetch and format timer status from daemon
async fn fetch_and_format_status(
    output_format: &str,
    text_template: &str,
    text_template_idle: &str,
    icons: &config::DisplayIcons,
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

    // Choose template based on phase
    let template = if matches!(timer_status.phase, timer::Phase::Idle) {
        text_template_idle
    } else {
        text_template
    };

    // Format with client-side template
    let status_output =
        timer::TimerState::format_status(&timer_status, &format_enum, template, icons);

    serialize_status_output(status_output)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let quiet = cli.quiet;

    match cli.command {
        Commands::Daemon { action } => match action {
            DaemonAction::Start => {
                crate::server::start_daemon(quiet).await?;
            }
            DaemonAction::Stop => {
                crate::server::stop_daemon(quiet).await?;
            }
            DaemonAction::Status => {
                crate::server::daemon_status().await?;
            }
            DaemonAction::Install { force } => {
                install_systemd_service(force, quiet)?;
            }
            DaemonAction::Uninstall => {
                uninstall_systemd_service(quiet)?;
            }
            DaemonAction::Run => {
                run_daemon(quiet).await?;
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

            // Handle sound_mode with deprecated flag support
            let sound_mode = if let Some(ref mode) = timer.sound_mode {
                Some(mode.clone())
            } else if timer.beep {
                Some("system-beep".to_string())
            } else if timer.sound {
                Some("embedded".to_string())
            } else {
                None
            };
            if let Some(mode) = sound_mode {
                args["sound_mode"] = serde_json::json!(mode);
            }

            if let Some(volume) = timer.volume {
                args["volume"] = serde_json::json!(volume);
            }

            let response = send_command("start", args)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, None)?;
        }

        Commands::Stop => {
            let response = send_command("stop", serde_json::Value::Null)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, Some("Timer stopped"))?;
        }

        Commands::Status { output, format } => {
            // Load config for display format defaults
            let config = Config::load();
            let text_template = format.unwrap_or_else(|| config.display.text_format.clone());
            let text_template_idle = config
                .display
                .text_format_idle
                .unwrap_or_else(|| config.display.text_format.clone());

            match fetch_and_format_status(
                &output,
                &text_template,
                &text_template_idle,
                &config.display.icons,
            )
            .await
            {
                Ok(output) => println!("{}", output),
                Err(e) if is_daemon_unavailable(e.as_ref()) => {
                    println!("{}", format_disconnected_status(&output)?);
                }
                Err(e) => return Err(connection_error(e)),
            }
        }

        Commands::Watch {
            output,
            format,
            interval,
        } => {
            // Load config for display format defaults
            let config = Config::load();
            let text_template = format.unwrap_or_else(|| config.display.text_format.clone());
            let text_template_idle = config
                .display
                .text_format_idle
                .unwrap_or_else(|| config.display.text_format.clone());
            let interval_duration = std::time::Duration::from_secs_f64(interval);

            loop {
                match fetch_and_format_status(
                    &output,
                    &text_template,
                    &text_template_idle,
                    &config.display.icons,
                )
                .await
                {
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        return Err(connection_error(e));
                    }
                }

                tokio::time::sleep(interval_duration).await;
            }
        }

        Commands::Skip => {
            let response = send_command("skip", serde_json::Value::Null)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, Some("Skipped to next phase"))?;
        }

        Commands::Pause => {
            let response = send_command("pause", serde_json::Value::Null)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, None)?;
        }

        Commands::Resume => {
            let response = send_command("resume", serde_json::Value::Null)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, None)?;
        }

        Commands::Toggle => {
            let response = send_command("toggle", serde_json::Value::Null)
                .await
                .map_err(connection_error)?;
            handle_response(response, quiet, None)?;
        }
    }

    Ok(())
}

/// Install systemd user service for tomat daemon
fn install_systemd_service(force: bool, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
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
        r#"[Install]
WantedBy=graphical-session.target

[Service]
Environment="PATH=%h/.local/bin:%h/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
ExecStart={} daemon run
Restart=always
RestartSec=5

[Unit]
After=graphical-session.target
Description=Tomat Pomodoro server
PartOf=graphical-session.target
"#,
        exe_path_str
    );

    // Write service file
    let service_path = systemd_dir.join("tomat.service");

    // Check if service file already exists (unless --force is used)
    if service_path.exists() && !force {
        use std::io::{self, Write};

        print!(
            "⚠ Service file already exists at: {}\nOverwrite? [y/N]: ",
            service_path.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("Installation cancelled.");
            return Ok(());
        }
    }

    fs::write(&service_path, service_content)?;

    if !quiet {
        println!(
            "✓ Systemd service file installed to: {}",
            service_path.display()
        );
    }

    // Reload systemd and enable service
    let reload_result = std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status();

    match reload_result {
        Ok(status) if status.success() => {
            if !quiet {
                println!("✓ Systemd daemon reloaded");
            }

            let enable_result = std::process::Command::new("systemctl")
                .args(["--user", "enable", "tomat.service"])
                .status();

            match enable_result {
                Ok(status) if status.success() => {
                    if !quiet {
                        println!("✓ Tomat service enabled");
                        println!("\nService installed successfully!");
                        println!("\nTo start the daemon:");
                        println!("  systemctl --user start tomat.service");
                        println!("\nTo check status:");
                        println!("  systemctl --user status tomat.service");
                        println!("\nTo enable auto-start on login:");
                        println!("  loginctl enable-linger $USER");
                    }
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
fn uninstall_systemd_service(quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
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
        if !quiet {
            println!("Tomat service is not installed (service file not found)");
        }
        return Ok(());
    }

    // Try to stop and disable the service first
    let stop_result = std::process::Command::new("systemctl")
        .args(["--user", "stop", "tomat.service"])
        .status();

    match stop_result {
        Ok(status) if status.success() => {
            if !quiet {
                println!("✓ Tomat service stopped");
            }
        }
        Ok(_) => eprintln!("⚠ Warning: Failed to stop tomat.service (might not be running)"),
        Err(e) => eprintln!("⚠ Warning: Failed to run systemctl stop: {}", e),
    }

    let disable_result = std::process::Command::new("systemctl")
        .args(["--user", "disable", "tomat.service"])
        .status();

    match disable_result {
        Ok(status) if status.success() => {
            if !quiet {
                println!("✓ Tomat service disabled");
            }
        }
        Ok(_) => eprintln!("⚠ Warning: Failed to disable tomat.service"),
        Err(e) => eprintln!("⚠ Warning: Failed to run systemctl disable: {}", e),
    }

    // Remove service file
    match fs::remove_file(&service_path) {
        Ok(()) => {
            if !quiet {
                println!("✓ Service file removed: {}", service_path.display());
            }

            // Reload systemd
            let reload_result = std::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status();

            match reload_result {
                Ok(status) if status.success() => {
                    if !quiet {
                        println!("✓ Systemd daemon reloaded");
                    }
                }
                Ok(_) => eprintln!("⚠ Warning: Failed to reload systemd daemon"),
                Err(e) => eprintln!("⚠ Warning: Failed to run systemctl daemon-reload: {}", e),
            }

            if !quiet {
                println!("\nTomat service uninstalled successfully!");
            }
        }
        Err(e) => {
            eprintln!("Failed to remove service file: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
