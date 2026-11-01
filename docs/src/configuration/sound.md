# Sound Settings

The `[sound]` section controls audio notifications when transitioning between
work/break phases. By default, tomat plays high-quality WAV files built into the
application. On Linux, this requires ALSA (Advanced Linux Sound Architecture);
on macOS, Tomat uses Core Audio without additional dependencies. If the audio
system is unavailable, Tomat falls back to the system beep or disables audio.

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
  : Use system beep (terminal bell)

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
```

To use system beep only:

```toml
[sound]
mode = "system-beep"
```

## Embedded Sound Sources

The built-in sounds come from Freesound. All three recordings are dedicated to
the public domain under [CC0
1.0](https://creativecommons.org/publicdomain/zero/1.0/).

  | Transition        | Recording                                                                         | Creator   |
  | ----------------- | --------------------------------------------------------------------------------- | --------- |
  | Work → Break      | [bell3.wav](https://freesound.org/people/creeeeak/sounds/531026/)                 | creeeeak  |
  | Break → Work      | [Street Sign Gong - 03](https://freesound.org/people/robin.kaf/sounds/711881/)    | robin.kaf |
  | Work → Long Break | [Subway MTA Door Close Chime](https://freesound.org/people/cbrews/sounds/434085/) | cbrews    |

Tomat uses copies converted to 22.05 kHz, mono, 16-bit PCM WAV to reduce the
embedded file sizes.
