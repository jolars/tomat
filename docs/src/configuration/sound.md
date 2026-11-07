# Sound Settings

The `[sound]` section controls audio notifications when transitioning between
work and break phases, along with optional ticking during work sessions. By
default, tomat plays WAV files built into the application for transitions and
leaves ticking disabled. On Linux, this requires ALSA (Advanced Linux Sound
Architecture); on macOS, Tomat uses Core Audio without additional dependencies. If the audio
system is unavailable, the timer continues without audio. Tomat releases the
audio device when no sounds are playing.

```toml
[sound]
mode = "embedded"
volume = 0.5
```

## Options

`mode`
: Sound notification mode. Controls how phase transitions are announced.

  `"embedded"` (default)
  : Use built-in audio files

  `"system-beep"`
  : Play a short beep tone

  `"none"`
  : No sound notifications

`volume`
: Audio volume level for embedded and custom sounds (0.0-1.0). Default: `0.5`

`work_to_break`
: Path to custom sound file for work→break transitions. Overrides embedded sound. Optional.

`break_to_work`
: Path to custom sound file for break→work transitions. Overrides embedded sound. Optional.

`work_to_long_break`
: Path to custom sound file for work→long break transitions. Overrides embedded sound. Optional.


`"enabled"`
: Enable sound notifications.

  > [!WARNING]
  > 
  > Deprecated option. Use `mode = "embedded"` instead.

`"system_beep"`
: Use system beep.

  > [!WARNING]
  >
  > Deprecated option. Use `mode = "system-beep"` instead.

`"use_embedded"`
: Use embedded sounds.

  > [!WARNING]
  >
  > Deprecated option. Use `mode = "embedded"` instead.


## Ticking During Work

Enable a repeating clock sound while a work session is running:

```toml
[sound.ticking]
enabled = true
volume = 0.2
```

Ticking stops when you pause, stop, or enter a break or long break. It starts
again when you resume work. Resetting a running work session keeps the loop
playing. Restart the daemon after changing these settings.

The `[sound.ticking]` options are independent of transition alerts. You can use
`[sound] mode = "none"` or `"system-beep"` and still enable ticking.

`enabled`
: Play ticking during running work sessions. Default: `false`.

`volume`
: Ticking volume from 0.0 to 1.0. Default: `0.2`. Does not use `[sound] volume`.

`file`
: Optional path to a custom audio loop. WAV, OGG, FLAC, and MP3 are supported.
  The entire recording repeats, so choose a file with a smooth loop boundary.
  If the file cannot be opened or decoded, tomat uses the built-in clock sound.

For example, use a custom focus sound without transition alerts:

```toml
[sound]
mode = "none"

[sound.ticking]
enabled = true
volume = 0.15
file = "/home/user/sounds/focus-loop.ogg"
```

Ticking requires a build with the `audio` feature, which is enabled by default.

## Examples

To use your own sound files, keep `mode = "embedded"` and specify paths to your
audio files. Custom sounds override the built-in ones:

```toml
[sound]
mode = "embedded"
work_to_break = "/home/user/sounds/work-done.ogg"
break_to_work = "/home/user/sounds/break-over.ogg"
work_to_long_break = "/home/user/sounds/long-break.ogg"
volume = 0.7
```

To disable all audio:

```toml
[sound]
mode = "none"

[sound.ticking]
enabled = false
```

To use system beep only:

```toml
[sound]
mode = "system-beep"
```

## Embedded Sound Sources

The built-in sounds come from Freesound and OpenGameArt. All four recordings
are dedicated to the public domain under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).

  | Transition           | Recording                                                                         | Creator   |
  | -------------------- | --------------------------------------------------------------------------------- | --------- |
  | Work → Break         | [bell3.wav](https://freesound.org/people/creeeeak/sounds/531026/)                 | creeeeak  |
  | Break → Work         | [Street Sign Gong - 03](https://freesound.org/people/robin.kaf/sounds/711881/)    | robin.kaf |
  | Work → Long Break    | [Subway MTA Door Close Chime](https://freesound.org/people/cbrews/sounds/434085/) | cbrews    |
  | Work-session ticking | [Ticking clock.](https://opengameart.org/content/ticking-clock)                   | bart      |

Tomat uses copies converted to 22.05 kHz, mono, 16-bit PCM WAV to reduce the
embedded file sizes.

The clock recording preserves the original eight-second loop of eight ticks.
