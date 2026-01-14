# Examples

This directory contains ready-to-use configuration examples for tomat and related tools.

## Quick Setup

### Tomat Configuration
```bash
# Copy and customize tomat configuration
mkdir -p ~/.config/tomat
cp examples/config.toml ~/.config/tomat/config.toml
# Edit ~/.config/tomat/config.toml as needed
```

### Waybar Integration
```bash
# Add tomat module to your waybar config
# Copy the "custom/tomat" section from examples/waybar-config.json
# to your ~/.config/waybar/config

# Add styling to your waybar CSS
# Copy styles from examples/waybar-style.css
# to your ~/.config/waybar/style.css
```

### Systemd Service
```bash
# Set up auto-start with systemd
mkdir -p ~/.config/systemd/user
cp examples/systemd.service ~/.config/systemd/user/tomat.service
systemctl --user enable tomat.service
systemctl --user start tomat.service
```

## Files

- **`config.toml`** - Complete tomat configuration with all options
- **`waybar-config.json`** - Waybar module configuration  
- **`waybar-style.css`** - CSS styling for waybar integration
- **`systemd.service`** - Systemd user service for auto-start

## Usage Tips

### Waybar Setup
1. Copy the `custom/tomat` module from `waybar-config.json` to your waybar config
2. Add the styling from `waybar-style.css` to your waybar CSS file  
3. Restart waybar: `killall waybar && waybar &`

### Customization
- **Colors**: Modify the CSS colors in `waybar-style.css` to match your theme
- **Timer settings**: Adjust durations and behavior in `config.toml`
- **Notifications**: Configure desktop notifications and icons in `config.toml`

### Testing
```bash
# Start daemon and test waybar integration
tomat daemon start
tomat start --work 0.1 --break 0.05  # Short test timer
# Check that waybar shows the timer status
tomat daemon stop
```