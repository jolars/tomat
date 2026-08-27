mod audio;
mod cli;
mod config;
mod server;
mod service;
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
                service::install(force, quiet)?;
            }
            DaemonAction::Uninstall => {
                service::uninstall(quiet)?;
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
