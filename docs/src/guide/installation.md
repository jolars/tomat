# Installation Instructions

This section provides instructions on how to install Tomat, a customizable
Pomodoro timer.

## Install from Crates.io

The simplest way to install Tomat is via Cargo. If you don't have Rust and Cargo
installed, follow the instructions at <https://www.rust-lang.org/tools/install>
first to set up your Rust environment.

Then, you can install Tomat from crates.io:

```bash
cargo install tomat
```

## NixOS

If you are using [NixOS](https://nixos.org), Tomat is available in the official
packages:

```nix
{
  environment.systemPackages = [
    pkgs.tomat
  ];
}
```

You still need to set up the systemd service for automatic startup.

### Home Manager

But if you're using
[home manager](https://nix-community.github.io/home-manager/), you're in luck!
Tomat is supported as a module:

```nix
{
  services.tomat = {
    enable = true;

    settings = {
      timer = {
        work = 25;
        break = 5;
      };
    };
  };
}
```

## Building from Source

Anoter way to install Tomat is to build it from source. First, clone the
repository:

```bash
git clone https://github.com/jolars/tomat.git
cd tomat
```

### Prerequisites

On Linux systems, audio notifications require ALSA development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

### Build and Install

Then, build and install Tomat using Cargo:

```bash
cargo install --path .
```

> [!NOTE]
>
> Audio will be automatically disabled if ALSA is not available. The timer will
> still work normally, but with desktop notifications only.

## Systemd Service Setup

Most users will want to run the Tomat daemon as a systemd user service so that
it starts automatically on login. Tomat provides a convenience command to
install the service:

```bash
tomat daemon install
```

After that, you can enable and start the service with:

```bash
systemctl --user enable tomat.service --now
```

### Alternative Manual Setup

If you prefer to set up the systemd service manually, you can copy the service
file from the examples directory:

```bash
# Manual systemd setup (if you prefer)
mkdir -p ~/.config/systemd/user
curl -o ~/.config/systemd/user/tomat.service https://raw.githubusercontent.com/jolars/tomat/main/examples/systemd.service
systemctl --user daemon-reload
systemctl --user enable tomat.service --now
```
