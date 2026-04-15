# Contributing to LazyJira

Thank you for your interest in contributing to LazyJira! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust (latest stable version recommended)
- Git

### Development Setup

1. **Clone the repository**

```bash
git clone <repository-url>
cd lazyjira
```

2. **Build the project**

```bash
make build
```

3. **Run tests**

```bash
# Test CLI
cd cli && cargo test

# Test TUI
cd tui && cargo test
```

4. **Install development version**

```bash
make link  # Creates symlinks to development builds
```

## Development Workflow

### 1. Choose an Issue

- Check existing [issues](../../issues) for tasks to work on
- Create a new issue if you have a feature request or bug report
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Ensure both CLI and TUI work correctly

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add your feature description

- What was changed
- Why it was changed
- Any breaking changes
"
```

Follow conventional commit format:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation
- `refactor:` for code refactoring
- `test:` for test additions

### 6. Submit a Pull Request

- Push your branch to your fork
- Create a Pull Request with a clear description
- Reference any related issues
- Wait for review and address feedback

## Project Structure

```
lazyjira/
├── cli/                   # Command-line interface
│   ├── src/
│   │   ├── main.rs        # CLI entry point and commands
│   │   └── ...            # CLI-specific modules
│   └── Cargo.toml
├── tui/                   # Terminal user interface
│   ├── src/
│   │   ├── main.rs        # TUI entry point
│   │   ├── jira.rs        # Jira API client
│   │   ├── config.rs      # Configuration handling
│   │   ├── cache.rs       # Caching logic
│   │   ├── data/          # Data models and loading
│   │   ├── ui/            # UI components
│   │   ├── layout/        # Layout management
│   │   └── model/         # Data models
│   └── Cargo.toml
├── Makefile               # Build and deployment scripts
├── README.md              # Project documentation
└── CONTRIBUTING.md        # This file
```

## Code Guidelines

### Rust Standards

- Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for code formatting
- Run `clippy` for linting
- Write comprehensive documentation for public APIs

### Error Handling

- Use `Result<T, E>` for operations that can fail
- Provide meaningful error messages
- Handle errors gracefully in user-facing code

### Configuration

- Both CLI and TUI share the same config file. The location depends on your operating system:
  - **macOS**: `~/Library/Application Support/LazyJira/env`
  - **Linux**: `~/.config/lazyjira/env`
  - **Windows**: `%APPDATA%\LazyJira\env`
- Use the `Config` struct for configuration management
- Validate configuration on load

## Making Changes

### Adding CLI Commands

1. Add the command to the `JiraCommand` enum in `cli/src/main.rs`
2. Implement the command handler function
3. Add appropriate help text and examples
4. Update the README with usage examples

### Adding TUI Features

1. Update the UI components in `tui/src/ui/`
2. Modify state management in `tui/src/state.rs`
3. Update event handling in `tui/src/main.rs`
4. Test keyboard navigation and interactions

### Configuration Changes

1. Update the `Config` struct in both `cli/src/config.rs` and `tui/src/config.rs`
2. Modify the `init_config()` function for new config values
3. Update documentation and help text

## Reporting Issues

### Bug Reports

- Use the issue template if available
- Include steps to reproduce
- Provide system information (OS, Rust version)
- Include error messages and logs
- Mention if it's CLI or TUI specific

### Feature Requests

- Clearly describe the proposed feature
- Explain the use case and benefits
- Consider if it fits the project scope
- Be open to discussion and alternatives

## Documentation

### Code Documentation

- Add doc comments to public functions and structs
- Use examples in documentation where helpful
- Keep comments up to date with code changes

### User Documentation

- Update README.md for new features
- Add examples and usage instructions
- Document breaking changes
- Update installation instructions if needed

## Communication

- Be respectful and constructive in discussions
- Use clear and concise language
- Provide context for your suggestions
- Be open to feedback and different perspectives

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project.

## Recognition

Contributors will be acknowledged in the project documentation. Thank you for helping make LazyJira better!
