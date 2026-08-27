use std::path::Path;

pub fn install(force: bool, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        systemd::install(force, quiet)
    }
    #[cfg(target_os = "macos")]
    {
        launchd::install(force, quiet)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (force, quiet);
        Err(UNSUPPORTED_PLATFORM.into())
    }
}

pub fn uninstall(quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        systemd::uninstall(quiet)
    }
    #[cfg(target_os = "macos")]
    {
        launchd::uninstall(quiet)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = quiet;
        Err(UNSUPPORTED_PLATFORM.into())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
const UNSUPPORTED_PLATFORM: &str = "Tomat only manages user services on Linux (systemd) and macOS \
     (launchd). Start the daemon with 'tomat daemon start' or supervise 'tomat daemon run' yourself";

#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn confirm_overwrite(
    path: &Path,
    force: bool,
    quiet: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    use std::io::{IsTerminal, Write};

    if !path.exists() || force {
        return Ok(true);
    }

    // Prompting needs someone to answer, so a quiet or scripted run has to fail
    // loudly rather than block on stdin or exit successfully without acting.
    if quiet || !std::io::stdin().is_terminal() {
        return Err(format!(
            "Service file already exists at: {}. Re-run with --force to overwrite it",
            path.display()
        )
        .into());
    }

    print!(
        "⚠ Service file already exists at: {}\nOverwrite? [y/N]: ",
        path.display()
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input)? == 0 {
        return Ok(false);
    }
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

/// Read an environment variable that the daemon needs, ignoring empty values.
#[cfg(any(target_os = "linux", target_os = "macos", test))]
fn runtime_env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

#[cfg(any(target_os = "linux", test))]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod systemd {
    use super::{confirm_overwrite, runtime_env};
    use std::path::{Path, PathBuf};

    pub fn install(force: bool, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
        let service_path = service_path()?;
        if !confirm_overwrite(&service_path, force, quiet)? {
            println!("Installation cancelled.");
            return Ok(());
        }

        let service_dir = service_path
            .parent()
            .ok_or("Systemd service path has no parent directory")?;
        std::fs::create_dir_all(service_dir)?;
        std::fs::write(&service_path, service_content(&std::env::current_exe()?))?;

        if !quiet {
            println!(
                "✓ Systemd service file installed to: {}",
                service_path.display()
            );
        }

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
                            println!(
                                "\nTo keep the service running outside a login session:\n  \
                                 loginctl enable-linger $USER"
                            );
                        }
                    }
                    Ok(_) => {
                        eprintln!("⚠ Warning: Failed to enable tomat.service");
                        eprintln!(
                            "You can enable it manually with: systemctl --user enable tomat.service"
                        );
                    }
                    Err(error) => {
                        eprintln!("⚠ Warning: Failed to run systemctl enable: {error}");
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
            Err(error) => {
                eprintln!("⚠ Warning: Failed to run systemctl daemon-reload: {error}");
                eprintln!("Systemctl might not be available or you might not be using systemd");
            }
        }

        Ok(())
    }

    pub fn uninstall(quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
        let service_path = service_path()?;
        if !service_path.exists() {
            if !quiet {
                println!("Tomat service is not installed (service file not found)");
            }
            return Ok(());
        }

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
            Err(error) => eprintln!("⚠ Warning: Failed to run systemctl stop: {error}"),
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
            Err(error) => eprintln!("⚠ Warning: Failed to run systemctl disable: {error}"),
        }

        std::fs::remove_file(&service_path)?;
        if !quiet {
            println!("✓ Service file removed: {}", service_path.display());
        }

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
            Err(error) => eprintln!("⚠ Warning: Failed to run systemctl daemon-reload: {error}"),
        }

        if !quiet {
            println!("\nTomat service uninstalled successfully!");
        }
        Ok(())
    }

    pub fn service_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir =
            dirs::config_dir().or_else(|| dirs::home_dir().map(|home| home.join(".config")));

        Ok(config_dir
            .ok_or("Could not determine the user configuration directory")?
            .join("systemd")
            .join("user")
            .join("tomat.service"))
    }

    fn escape_environment_value(value: &str) -> String {
        value.replace('\\', "\\\\").replace('"', "\\\"")
    }

    pub fn service_content(exe_path: &Path) -> String {
        // The user manager already provides a correct XDG_RUNTIME_DIR, so only
        // the tomat-specific override has to be carried into the unit.
        let runtime_dir = runtime_env("TOMAT_RUNTIME_DIR")
            .map(|value| {
                format!(
                    "Environment=\"TOMAT_RUNTIME_DIR={}\"\n",
                    escape_environment_value(&value)
                )
            })
            .unwrap_or_default();

        format!(
            r#"[Install]
WantedBy=graphical-session.target

[Service]
Environment="PATH=%h/.local/bin:%h/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
{runtime_dir}ExecStart="{}" daemon run
Restart=on-failure
RestartSec=5

[Unit]
After=graphical-session.target
Description=Tomat Pomodoro server
PartOf=graphical-session.target
"#,
            exe_path.display()
        )
    }
}

#[cfg(any(target_os = "macos", test))]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod launchd {
    use super::{confirm_overwrite, runtime_env};
    use std::path::{Path, PathBuf};

    pub const LABEL: &str = "io.github.jolars.tomat";

    pub fn install(force: bool, quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not determine the user home directory")?;
        let service_path = service_path(&home);
        if !confirm_overwrite(&service_path, force, quiet)? {
            println!("Installation cancelled.");
            return Ok(());
        }

        let exe_path = std::env::current_exe()?;
        let service_target = service_target();

        // Unload the running agent and stop a manually started daemon before
        // the plist is touched, so a failure here leaves the working setup
        // intact instead of replacing it with something that never loaded.
        bootout_service(&service_target).map_err(|details| {
            format!("Failed to unload the existing Tomat LaunchAgent: {details}")
        })?;
        stop_daemon_with_cli(&exe_path).map_err(|details| {
            format!(
                "Failed to stop the existing Tomat daemon before loading the LaunchAgent: {details}"
            )
        })?;

        let previous = match std::fs::read(&service_path) {
            Ok(contents) => Some(contents),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };

        let service_dir = service_path
            .parent()
            .ok_or("LaunchAgent service path has no parent directory")?;
        std::fs::create_dir_all(service_dir)?;
        std::fs::write(&service_path, service_content(&exe_path, &home))?;

        if let Err(error) = load_service(&service_target, &service_path) {
            restore_service_file(&service_path, previous.as_deref());
            return Err(error.into());
        }

        if !quiet {
            println!("✓ LaunchAgent installed to: {}", service_path.display());
            println!("✓ Tomat service loaded and started");
            println!("\nTo check status:");
            println!("  launchctl print {service_target}");
            println!("\nDaemon errors are appended to (and never rotated in):");
            println!("  {}", log_path(&home).display());
        }
        Ok(())
    }

    pub fn uninstall(quiet: bool) -> Result<(), Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not determine the user home directory")?;
        let service_path = service_path(&home);
        if !service_path.exists() {
            if !quiet {
                println!("Tomat service is not installed (LaunchAgent file not found)");
            }
            return Ok(());
        }

        let exe_path = std::env::current_exe()?;
        let service_target = service_target();
        bootout_service(&service_target).map_err(|details| {
            format!("Failed to unload the Tomat LaunchAgent: {details}. The plist was preserved")
        })?;

        // The agent is unloaded, so the plist has to go even if the leftover
        // daemon cannot be reaped; otherwise launchd revives it at next login.
        std::fs::remove_file(&service_path)?;

        if let Err(details) = stop_daemon_with_cli(&exe_path) {
            eprintln!(
                "⚠ Warning: Failed to clean up the Tomat daemon after unloading the LaunchAgent: {details}"
            );
            eprintln!("Stop it manually with: tomat daemon stop");
        }

        if !quiet {
            println!("✓ LaunchAgent removed: {}", service_path.display());
            println!("\nTomat service uninstalled successfully!");
        }
        Ok(())
    }

    pub fn service_path(home: &Path) -> PathBuf {
        home.join("Library")
            .join("LaunchAgents")
            .join(format!("{LABEL}.plist"))
    }

    fn log_path(home: &Path) -> PathBuf {
        home.join("Library").join("Logs").join("tomat.log")
    }

    fn xml_escape(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn environment_entry(key: &str, value: &str) -> String {
        format!(
            "\n        <key>{}</key>\n        <string>{}</string>",
            xml_escape(key),
            xml_escape(value)
        )
    }

    pub fn service_content(exe_path: &Path, home: &Path) -> String {
        let executable = xml_escape(&exe_path.to_string_lossy());
        let path = [
            home.join(".local/bin"),
            home.join(".cargo/bin"),
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
            PathBuf::from("/usr/sbin"),
            PathBuf::from("/sbin"),
        ]
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(":");
        let log_path = xml_escape(&log_path(home).to_string_lossy());

        // launchd starts agents with a minimal environment, so a socket
        // location the user exported has to be baked into the plist or the
        // daemon and its clients end up on different sockets.
        let environment = ["TOMAT_RUNTIME_DIR", "XDG_RUNTIME_DIR"]
            .into_iter()
            .filter_map(|key| runtime_env(key).map(|value| environment_entry(key, &value)))
            .fold(environment_entry("PATH", &path), |entries, entry| {
                entries + &entry
            });

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{executable}</string>
        <string>--quiet</string>
        <string>daemon</string>
        <string>run</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>{environment}
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>5</integer>
    <key>StandardOutPath</key>
    <string>/dev/null</string>
    <key>StandardErrorPath</key>
    <string>{log_path}</string>
</dict>
</plist>
"#
        )
    }

    fn domain() -> String {
        format!("gui/{}", unsafe { libc::getuid() })
    }

    pub fn service_target() -> String {
        format!("{}/{LABEL}", domain())
    }

    pub fn command_output_details(output: &std::process::Output) -> String {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let details = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };

        if details.is_empty() {
            output.status.to_string()
        } else {
            details.to_string()
        }
    }

    fn service_is_absent(output: &std::process::Output) -> bool {
        if output.status.success() {
            return false;
        }

        let details = command_output_details(output).to_ascii_lowercase();
        details.contains("could not find service") || details.contains("service not found")
    }

    pub fn validate_bootout(
        bootout: &std::process::Output,
        print: Option<&std::process::Output>,
    ) -> Result<(), String> {
        if bootout.status.success() {
            return Ok(());
        }

        if print.is_some_and(service_is_absent) {
            return Ok(());
        }

        let bootout_details = command_output_details(bootout);
        match print {
            Some(print) if print.status.success() => Err(format!(
                "{bootout_details}; launchctl reports that the service is still loaded"
            )),
            Some(print) => Err(format!(
                "{bootout_details}; service absence could not be verified: {}",
                command_output_details(print)
            )),
            None => Err(bootout_details),
        }
    }

    fn bootout_service(service_target: &str) -> Result<(), String> {
        let bootout = std::process::Command::new("launchctl")
            .args(["bootout", service_target])
            .output()
            .map_err(|error| error.to_string())?;
        let print = if bootout.status.success() {
            None
        } else {
            Some(
                std::process::Command::new("launchctl")
                    .args(["print", service_target])
                    .output()
                    .map_err(|error| error.to_string())?,
            )
        };

        validate_bootout(&bootout, print.as_ref())
    }

    fn load_service(service_target: &str, service_path: &Path) -> Result<(), String> {
        let enable = std::process::Command::new("launchctl")
            .args(["enable", service_target])
            .output()
            .map_err(|error| error.to_string())?;
        if !enable.status.success() {
            return Err(format!(
                "Failed to enable the Tomat LaunchAgent: {}",
                command_output_details(&enable)
            ));
        }

        let bootstrap = std::process::Command::new("launchctl")
            .arg("bootstrap")
            .arg(domain())
            .arg(service_path)
            .output()
            .map_err(|error| error.to_string())?;
        if !bootstrap.status.success() {
            return Err(format!(
                "Failed to load the Tomat LaunchAgent: {}",
                command_output_details(&bootstrap)
            ));
        }

        Ok(())
    }

    /// Put the plist back the way it was when loading the new one failed.
    fn restore_service_file(service_path: &Path, previous: Option<&[u8]>) {
        let result = match previous {
            Some(contents) => std::fs::write(service_path, contents),
            None => std::fs::remove_file(service_path),
        };

        if let Err(error) = result {
            eprintln!(
                "⚠ Warning: Failed to restore the previous LaunchAgent at {}: {error}",
                service_path.display()
            );
        }
    }

    fn stop_daemon_with_cli(exe_path: &Path) -> Result<(), String> {
        let output = std::process::Command::new(exe_path)
            .args(["--quiet", "daemon", "stop"])
            .output()
            .map_err(|error| error.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(command_output_details(&output))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemd_service_uses_the_exact_executable_path() {
        let content = systemd::service_content(Path::new("/home/test user/bin/tomat"));

        assert!(content.contains("ExecStart=\"/home/test user/bin/tomat\" daemon run"));
    }

    #[test]
    fn systemd_service_restarts_only_after_a_failure() {
        let content = systemd::service_content(Path::new("/usr/bin/tomat"));

        assert!(content.contains("Restart=on-failure"));
    }

    #[test]
    fn launchd_service_escapes_paths_and_runs_quietly() {
        let content = launchd::service_content(
            Path::new("/Users/A & B/bin/tomat"),
            Path::new("/Users/A & B"),
        );

        assert!(content.contains("/Users/A &amp; B/bin/tomat"));
        assert!(content.contains("<string>--quiet</string>"));
        assert!(content.contains("<string>daemon</string>"));
        assert!(content.contains("<string>run</string>"));
    }

    #[test]
    fn launchd_service_restarts_only_after_a_failure() {
        let content =
            launchd::service_content(Path::new("/usr/local/bin/tomat"), Path::new("/Users/test"));

        assert!(content.contains("<key>SuccessfulExit</key>\n        <false/>"));
    }

    #[test]
    fn launchd_service_lives_in_the_user_launch_agents_directory() {
        assert_eq!(
            launchd::service_path(Path::new("/Users/test")),
            Path::new("/Users/test/Library/LaunchAgents/io.github.jolars.tomat.plist")
        );
    }

    #[test]
    fn failed_launchd_bootout_is_benign_when_the_service_is_absent() {
        let bootout = failed_command_output("Boot-out failed");
        let print = failed_command_output(
            "Could not find service \"io.github.jolars.tomat\" in domain for user gui: 501",
        );

        assert!(launchd::validate_bootout(&bootout, Some(&print)).is_ok());
    }

    #[test]
    fn failed_launchd_bootout_is_an_error_when_the_service_remains_loaded() {
        let bootout = failed_command_output("Boot-out failed: Operation not permitted");
        let print = successful_command_output();

        assert!(launchd::validate_bootout(&bootout, Some(&print)).is_err());
    }

    #[test]
    fn failed_launchd_bootout_is_an_error_when_absence_cannot_be_verified() {
        let bootout = failed_command_output("Boot-out failed: Input/output error");
        let print = failed_command_output("Operation not permitted");

        assert!(launchd::validate_bootout(&bootout, Some(&print)).is_err());
    }

    #[test]
    fn launchctl_failures_reported_on_stdout_are_not_lost() {
        let output = std::process::Output {
            status: failed_command_output("").status,
            stdout: b"Bootstrap failed: 5: Input/output error".to_vec(),
            stderr: Vec::new(),
        };

        assert_eq!(
            launchd::command_output_details(&output),
            "Bootstrap failed: 5: Input/output error"
        );
    }

    #[test]
    fn overwriting_an_existing_service_file_without_a_terminal_is_an_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let service_path = temp_dir.path().join("tomat.service");
        std::fs::write(&service_path, "existing").unwrap();

        let error = confirm_overwrite(&service_path, false, true).unwrap_err();

        assert!(error.to_string().contains("--force"));
        assert_eq!(
            std::fs::read_to_string(&service_path).unwrap(),
            "existing",
            "a cancelled install must not touch the service file"
        );
    }

    #[test]
    fn forcing_an_overwrite_never_prompts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let service_path = temp_dir.path().join("tomat.service");
        std::fs::write(&service_path, "existing").unwrap();

        assert!(confirm_overwrite(&service_path, true, true).unwrap());
    }

    fn failed_command_output(stderr: &str) -> std::process::Output {
        use std::os::unix::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(1 << 8),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    fn successful_command_output() -> std::process::Output {
        use std::os::unix::process::ExitStatusExt;

        std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn launchd_service_is_a_valid_plist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let plist_path = temp_dir.path().join("tomat.plist");
        std::fs::write(
            &plist_path,
            launchd::service_content(Path::new("/usr/local/bin/tomat"), Path::new("/Users/test")),
        )
        .unwrap();

        let status = std::process::Command::new("plutil")
            .arg("-lint")
            .arg(plist_path)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
