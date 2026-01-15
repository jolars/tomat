# Timer Settings

The `[timer]` section controls timer durations and phase transition behavior.

```toml
[timer]
work = 25.0
break = 5.0
long_break = 15.0
sessions = 4
auto_advance = "none"
```

## Options

`work`
  : Duration of work phase in minutes (default: `25.0`)

`break`
  : Duration of short break phase in minutes (default: `5.0`)

`long_break`
  : Duration of long break phase in minutes (default: `15.0`)

`sessions`
  : Number of work sessions before a long break (default: `4`)

`auto_advance`
  : Controls how the timer transitions between phases.

    `"none"` (default)
    : Pause after every phase transition (requires manual resume)

    `"all"`
    : Automatically continue through all phases without pausing

    `"to-break"`
    : Auto-advance only from work to break/long-break

    `"to-work"`
    : Auto-advance only from break/long-break to work

    > [!NOTE]
    > 
    > Boolean values `true` and `false` are deprecated and will be
    > automatically converted to `"all"` and `"none"` respectively.


## Examples

To set a 30-minute work session, 10-minute break, 20-minute long break, with 3
sessions before a long break, and auto-advance from work to break, use the
following configuration.

```toml
[timer]
work = 30.0
break = 10.0
long_break = 20.0
sessions = 3
auto_advance = "to-break"
```

