// Integration tests for tomat CLI
//
// These tests require building the binary and spawning daemon processes.
// They can be excluded on NixOS or CI builds with: cargo test -- --skip integration
//
// Test modules:
// - common: Shared TestDaemon helper and utilities
// - daemon: Daemon lifecycle and management tests
// - timer: Timer functionality and state management tests
// - formats: Output format tests (waybar, plain, i3status-rs)
// - commands: Command validation and error handling tests

#[cfg(test)]
mod integration;
