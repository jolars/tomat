# TODO

This document tracks known issues and planned improvements for tomat.

## Low Priority - Features

### Additional Status Bar Integration

**Status:** Most status bars already supported via plain text format with
template customization.

**Status Bar Applications to Support:**

**Status Bars:**

- [x] **polybar** - Supported via `--output plain` with custom templates
- [x] **dwmblocks** - Supported via `--output plain`
- [x] **goblocks** - Supported via `--output plain`
- [ ] **py3status** - May benefit from dedicated JSON format
- [ ] **bumblebee-status** - May benefit from dedicated JSON format

**Already Implemented:**

- ✅ Template-based output formatting (`display.text_format` config, `--format`
  flag)
- ✅ Three output formats: waybar, i3status-rs, plain
- ✅ Custom text templates with placeholders: `{icon}`, `{time}`, `{state}`,
  `{phase}`, `{session}`

**Research Needed:**

- Investigate if py3status/bumblebee-status need specific JSON formats
- Consider if additional formats are worth the maintenance burden

### History and Statistics Tracking

**Issue:** No way to track completed Pomodoros or view productivity statistics.

**Proposed Features:**

- Store completed Pomodoros in `~/.local/share/tomat/history.json`
- Add `tomat stats` command showing:
  - Pomodoros completed today
  - Pomodoros completed this week/month
  - Average session length
  - Streak tracking
- Integration with waybar tooltip to show daily stats

**Dependencies:**

- Need to decide on data format
- Consider privacy implications (rotation, optional feature?)

## Testing

### Add Stress Tests

Test daemon stability under:

- Rapid command sequences
- Concurrent connections
- Long-running sessions (days/weeks)
- System suspend/resume cycles
