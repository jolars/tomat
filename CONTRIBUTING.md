# Contributing Guide

Thank you for your interest in contributing to tomat! This guide will help you get started.

## Development Setup

### Prerequisites

- Rust stable toolchain
- Git
- Optional: [Task](https://taskfile.dev/) for development workflows

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/jolars/tomat.git
cd tomat

# Build the project
cargo build

# Run tests
cargo test

# Run linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt
```

### Development Workflow

```bash
# Quick development check
task dev  # or manually: cargo check && cargo test && cargo clippy

# Start daemon for testing
cargo build && ./target/debug/tomat daemon start

# Test with short durations
./target/debug/tomat start --work 0.1 --break-time 0.05

# Stop daemon
./target/debug/tomat daemon stop
```

## Project Structure

```
tomat/
├── src/
│   ├── main.rs               # CLI parsing and command dispatching
│   ├── server.rs             # Unix socket server and daemon logic
│   └── timer.rs              # Timer state management
├── tests/
│   ├── cli.rs                # Integration tests (11 tests)
│   └── README.md             # Test documentation
├── docs/                     # Documentation
└── ...
```

## Contributing Guidelines

### Code Style

- **Formatting**: Use `cargo fmt` (enforced by CI)
- **Linting**: Pass `cargo clippy --all-targets --all-features -- -D warnings`
- **Error handling**: Use `Box<dyn std::error::Error>` for simplicity
- **Comments**: Only add comments for complex logic or public APIs

### Testing

- **Integration tests**: All new functionality must have integration tests
- **Test isolation**: Use temporary directories and custom socket paths
- **Fast tests**: Use fractional minutes (e.g., 0.05 = 3 seconds) for speed
- **Comprehensive coverage**: Test both auto-advance modes

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat: add auto-advance functionality
fix: resolve daemon cleanup on exit
docs: update waybar integration guide
test: add daemon lifecycle tests
refactor: simplify timer state machine
```

### Pull Request Process

1. **Fork and branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes**:
   - Write tests first (TDD approach recommended)
   - Implement functionality
   - Update documentation if needed

3. **Validate**:

   ```bash
   cargo test                    # All tests pass
   cargo clippy --all-targets --all-features -- -D warnings  # No warnings
   cargo fmt -- --check         # Proper formatting
   ```

4. **Commit and push**:

   ```bash
   git add .
   git commit -m "feat: add your feature"
   git push origin feature/your-feature-name
   ```

5. **Create pull request**:
   - Clear description of changes
   - Link any related issues
   - Include test results

## Types of Contributions

### Bug Reports

When reporting bugs, please include:

- **Environment**: OS, Rust version, tomat version
- **Steps to reproduce**: Clear sequence of commands
- **Expected vs actual behavior**
- **Logs/output**: Relevant error messages

Template:

````markdown
**Environment:**

- OS: Linux (Ubuntu 22.04)
- Rust: 1.75.0
- tomat: 0.1.0

**Steps to reproduce:**

1. Start daemon: `tomat daemon start`
2. Run timer: `tomat start --work 25`
3. Check status: `tomat status`

**Expected:** Timer shows running state
**Actual:** Timer shows paused state

**Output:**

```json
{ "class": "work-paused", "text": "🍅 25:00 ⏸" }
```
````

````

### Feature Requests
For new features, please include:
- **Problem statement**: What problem does this solve?
- **Proposed solution**: How should it work?
- **Alternative solutions**: Other approaches considered
- **Additional context**: Use cases, examples

### Documentation Improvements
- Fix typos or unclear explanations
- Add missing examples
- Improve waybar integration guides
- Update API documentation

### Code Contributions

#### Areas for Contribution
- **New features**: Configuration files, additional status bar integrations
- **Performance**: Optimization opportunities
- **Platform support**: Windows/macOS compatibility
- **Error handling**: Better error messages and recovery
- **Testing**: Additional test coverage

#### Architecture Guidelines
- **Single responsibility**: Each module has a clear purpose
- **Minimal dependencies**: Only add dependencies if absolutely necessary
- **Unix philosophy**: Small, focused, composable tools
- **Backward compatibility**: Don't break existing waybar configurations

## Testing Guidelines

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test cli test_auto_advance    # Auto-advance tests
cargo test --test cli test_daemon         # Daemon management

# Run with output for debugging
cargo test --test cli -- --nocapture
````

### Writing Tests

Integration tests are preferred over unit tests for this project:

```rust
#[test]
fn test_new_feature() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = TestDaemon::start()?;

    // Test your feature
    daemon.send_command(&["your-command", "--option", "value"])?;

    let status = daemon.get_status()?;
    assert_eq!(status["expected_field"], "expected_value");

    Ok(())
}
```

### Test Requirements

- **Isolated**: Use `TestDaemon` helper for process isolation
- **Fast**: Use fractional minutes for quick execution
- **Comprehensive**: Test both success and error cases
- **Deterministic**: No flaky tests due to timing

## Documentation Guidelines

### User Documentation

- **Clear examples**: Show real command usage
- **Complete workflows**: End-to-end scenarios
- **Troubleshooting**: Common issues and solutions
- **Platform specific**: Linux-focused but note limitations

### Developer Documentation

- **API changes**: Update docs/API.md for protocol changes
- **Architecture decisions**: Document significant design choices
- **Testing approach**: Explain test strategies for complex features

### Documentation Structure

```
docs/
├── USAGE.md          # Comprehensive usage examples
├── WAYBAR.md         # Waybar integration guide
├── API.md            # Internal API documentation
└── CONTRIBUTING.md   # This file
```

## Release Process

Releases are automated using semantic-release:

1. **Commit with conventional format**
2. **Merge to main branch**
3. **Automated release** creates:
   - Version bump based on commit types
   - Changelog generation
   - GitHub release
   - Git tags

### Version Scheme

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Pull Request Comments**: Code-specific discussions

### Response Expectations

- **Bug reports**: Response within 48 hours
- **Feature requests**: Initial response within 1 week
- **Pull requests**: Review within 1 week

### Maintainer Guidelines

Maintainers should:

- Be welcoming to newcomers
- Provide constructive feedback
- Respond promptly to contributions
- Maintain high code quality standards
- Keep documentation up to date

## Recognition

Contributors will be:

- Listed in CHANGELOG.md for their contributions
- Mentioned in release notes for significant features
- Added to GitHub repository contributors

Thank you for contributing to tomat! 🍅
