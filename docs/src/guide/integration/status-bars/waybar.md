# Waybar

Tomat is designed specifically for Waybar integration with rich JSON output and
CSS styling support.

## Basic Configuration

Add this to your waybar config (`~/.config/waybar/config`):

```json
{
  "modules-right": ["custom/tomat"],
  "custom/tomat": {
    "exec": "tomat status",
    "interval": 1,
    "return-type": "json",
    "format": "{text}",
    "tooltip": true,
    "on-click": "tomat toggle",
    "on-click-right": "tomat skip"
  }
}
```

## Styling

Add CSS styling (`~/.config/waybar/style.css`):

```css
#custom-tomat {
  padding: 0 10px;
  margin: 0 5px;
  border-radius: 5px;
}

#custom-tomat.work {
  background-color: #ff6b6b;
  color: #ffffff;
}

#custom-tomat.work-paused {
  background-color: #ff9999;
  color: #ffffff;
}

#custom-tomat.break {
  background-color: #4ecdc4;
  color: #ffffff;
}

#custom-tomat.break-paused {
  background-color: #7dd3db;
  color: #ffffff;
}

#custom-tomat.long-break {
  background-color: #45b7d1;
  color: #ffffff;
}

#custom-tomat.long-break-paused {
  background-color: #74c0db;
  color: #ffffff;
}
```

## JSON Output Format

Tomat provides waybar-optimized JSON output:

```json
{
  "text": "🍅 24:30 ▶",
  "tooltip": "Work (1/4) - 25.0min",
  "class": "work",
  "percentage": 2.0
}
```

**Fields:**

- **text**: Display text with icon and status symbols
- **tooltip**: Detailed information for hover
- **class**: CSS class for styling
- **percentage**: Progress percentage (0-100)

**CSS Classes:**

- `work` / `work-paused` - Work session running/paused
- `break` / `break-paused` - Break session running/paused
- `long-break` / `long-break-paused` - Long break running/paused

**Visual Indicators:**

- **Icons**: 🍅 (work), ☕ (break), 🏖️ (long break)
- **State**: ▶ (running), ⏸ (paused)
