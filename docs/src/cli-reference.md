# Command-Line Help for `tomat`

This document contains the help content for the `tomat` command-line program.

**Command Overview:**

* [`tomat`↴](#tomat)
* [`tomat daemon`↴](#tomat-daemon)
* [`tomat daemon start`↴](#tomat-daemon-start)
* [`tomat daemon stop`↴](#tomat-daemon-stop)
* [`tomat daemon status`↴](#tomat-daemon-status)
* [`tomat daemon install`↴](#tomat-daemon-install)
* [`tomat daemon uninstall`↴](#tomat-daemon-uninstall)
* [`tomat start`↴](#tomat-start)
* [`tomat stop`↴](#tomat-stop)
* [`tomat status`↴](#tomat-status)
* [`tomat watch`↴](#tomat-watch)
* [`tomat skip`↴](#tomat-skip)
* [`tomat pause`↴](#tomat-pause)
* [`tomat resume`↴](#tomat-resume)
* [`tomat toggle`↴](#tomat-toggle)

## `tomat`

Tomat is a Pomodoro timer with a daemon-based architecture, designed for seamless integration with waybar and other status bars. It uses a Unix socket for client-server communication, ensuring your timer state persists across waybar restarts and system suspend/resume.

**Usage:** `tomat <COMMAND>`

EXAMPLES:

    # Start daemon and begin a session
    tomat daemon start
    tomat start

    # Custom session durations
    tomat start --work 45 --break 15

    # Check status (outputs JSON for waybar)
    tomat status

    # Toggle pause/resume
    tomat toggle

For more information, visit: https://github.com/jolars/tomat

###### **Subcommands:**

* `daemon` — Manage the background daemon
* `start` — Start a new Pomodoro session
* `stop` — Stop the current session
* `status` — Get current timer status
* `watch` — Continuously output status updates
* `skip` — Skip to the next phase
* `pause` — Pause the current timer
* `resume` — Resume a paused timer
* `toggle` — Toggle timer pause/resume



## `tomat daemon`

Manage the tomat daemon, which runs in the background and maintains timer state. The daemon must be running for timer commands to work.

**Usage:** `tomat daemon <COMMAND>`

###### **Subcommands:**

* `start` — Start the daemon in the background
* `stop` — Stop the running daemon
* `status` — Check daemon status
* `install` — Install systemd user service
* `uninstall` — Uninstall systemd user service



## `tomat daemon start`

Start the tomat daemon as a background process. The daemon manages timer state and handles client requests via a Unix socket at $XDG_RUNTIME_DIR/tomat.sock. Only one daemon instance can run at a time.

**Usage:** `tomat daemon start`



## `tomat daemon stop`

Stop the running tomat daemon gracefully. This will terminate any active timer session. The daemon will clean up its socket and PID files.

**Usage:** `tomat daemon stop`



## `tomat daemon status`

Check if the tomat daemon is currently running and report its process ID.

**Usage:** `tomat daemon status`



## `tomat daemon install`

Install and enable the tomat systemd user service. This allows the daemon to start automatically on login and restart if it crashes. The service file is installed to ~/.config/systemd/user/tomat.service.

**Usage:** `tomat daemon install`

After installation, manage the service with systemctl:
    systemctl --user start tomat.service
    systemctl --user status tomat.service
    systemctl --user stop tomat.service



## `tomat daemon uninstall`

Stop and remove the tomat systemd user service. This removes the service file and disables automatic startup.

**Usage:** `tomat daemon uninstall`



## `tomat start`

Start a new Pomodoro timer session with the specified durations. If no options are provided, uses defaults from ~/.config/tomat/config.toml or built-in defaults (25min work, 5min break, 15min long break, 4 sessions). Custom durations only apply to the current session.

**Usage:** `tomat start [OPTIONS]`

EXAMPLES:

    # Start with defaults
    tomat start

    # Custom work/break durations
    tomat start --work 45 --break 15

    # Auto-advance between phases
    tomat start --auto-advance

###### **Options:**

* `-w`, `--work <WORK>` — Duration of work sessions in minutes. If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 25 minutes.
* `-b`, `--break <BREAK_TIME>` — Duration of short breaks in minutes. If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 5 minutes.
* `-l`, `--long-break <LONG_BREAK>` — Duration of long breaks in minutes. Long breaks occur after completing the configured number of work sessions. If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 15 minutes.
* `-s`, `--sessions <SESSIONS>` — Number of work/break cycles before taking a long break. If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 4 sessions.
* `-a`, `--auto-advance <AUTO_ADVANCE>` — Control automatic phase transitions:
     all      - Auto-advance through all phases
     none     - Never auto-advance (pause at transitions)
     to-break - Auto-advance from work to break only
     to-work  - Auto-advance from break to work only

   If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 'none'.
* `--sound-mode <SOUND_MODE>` — Control sound notifications:
     embedded    - Use built-in audio files (default)
     system-beep - Use system beep (terminal bell)
     none        - No sound notifications

   If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 'embedded'.
* `--volume <VOLUME>` — Set the audio volume for sound notifications, from 0.0 (silent) to 1.0 (maximum). Values outside this range will be clamped. If not specified, uses the value from ~/.config/tomat/config.toml or the built-in default of 0.5.



## `tomat stop`

Stop the current Pomodoro session and return the timer to idle state.

**Usage:** `tomat stop`



## `tomat status`

Display the current timer status. Output format can be customized for different status bars (waybar, i3status-rs) or plain text. Text appearance can be customized using format templates.

**Usage:** `tomat status [OPTIONS]`

OUTPUT FORMATS:

`waybar`
  : JSON output for waybar (default)

`i3status-rs`
  : JSON output for i3status-rs

`plain`
  : Plain text output

FORMAT PLACEHOLDERS:

`{icon}`
  : Phase icon (🍅 work, ☕ break, 🏖️ long break)

`{time}`
  : Remaining time (MM:SS)

`{state}`
  : Play/pause symbol (▶/⏸)

`{phase}`
  : Phase name (Work/Break/Long Break)

`{session}`
  : Session progress (e.g. 1/4)

EXAMPLES:

    tomat status
    tomat status --output plain
    tomat status --format "{time}"
    tomat status --format "{phase}: {time} {state}"

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output format: waybar, i3status-rs, or plain

  Default value: `waybar`

  Possible values: `waybar`, `i3status-rs`, `plain`

* `-f`, `--format <FORMAT>` — Customize the text display using placeholders:
   {icon}    - Phase icon
   {time}    - Remaining time (MM:SS)
   {state}   - Play/pause symbol
   {phase}   - Phase name
   {session} - Session progress



## `tomat watch`

Continuously watch and output timer status updates. This maintains a single connection to the daemon and updates at the specified interval. Automatically exits when the daemon stops. More efficient than polling with 'status' command.

**Usage:** `tomat watch [OPTIONS]`

EXAMPLES:

    # Watch with default interval (0.25 seconds)
    tomat watch

    # Watch with 5-second updates
    tomat watch --interval 5

    # Watch with plain text output
    tomat watch --output plain

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output format: waybar, i3status-rs, or plain

  Default value: `waybar`

  Possible values: `waybar`, `i3status-rs`, `plain`

* `-f`, `--format <FORMAT>` — Custom text format (e.g. "{icon} {time}")
* `-i`, `--interval <INTERVAL>` — Update interval in seconds

  Default value: `0.25`



## `tomat skip`

Skip the current phase and immediately transition to the next phase (work → break → work → ... → long break). The timer will start in the new phase if auto-advance is enabled, otherwise it will be paused.

**Usage:** `tomat skip`



## `tomat pause`

Pause the currently running timer. Use 'resume' or 'toggle' to continue.

**Usage:** `tomat pause`



## `tomat resume`

Resume a paused timer from where it left off.

**Usage:** `tomat resume`



## `tomat toggle`

Toggle the timer state: pause if running, resume if paused. This is useful for waybar click handlers.

**Usage:** `tomat toggle`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
