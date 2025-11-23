# Contributing to stillwater

Thank you for your interest in contributing to stillwater! This document provides guidelines and best practices for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Error Handling Guidelines](#error-handling-guidelines)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/stillwater.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Commit your changes with clear messages
7. Push to your fork and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git
- Claude CLI (for testing workflows)

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Error Handling Guidelines

Proper error handling is critical for production reliability. This project follows strict error handling practices to prevent panics and ensure graceful failure recovery.

### Core Principles

1. **No panic!() in production code** - All panic!() calls must be eliminated from production code paths
2. **No unwrap() in critical modules** - Replace with proper error handling using `?` operator or `ok_or_else()`
3. **Use Result types** - All fallible operations should return `Result<T, E>`
4. **Provide context** - Use `anyhow::Context` to add helpful error messages
5. **Graceful degradation** - Systems should fail gracefully with clear error messages

### Error Handling Patterns

#### Use the `?` Operator

```rust
// Good - propagates errors properly
pub async fn read_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .await
        .context("Failed to read file")?;
    Ok(content)
}

// Bad - will panic if file doesn't exist
pub async fn read_file(path: &Path) -> String {
    fs::read_to_string(path).await.unwrap()
}
```

#### Replace unwrap() with ok_or_else()

```rust
// Good - provides meaningful error
let value = map.get("key")
    .ok_or_else(|| anyhow!("Key 'key' not found in configuration"))?;

// Bad - will panic if key doesn't exist
let value = map.get("key").unwrap();
```

#### Use Pattern Matching for Options

```rust
// Good - handles both cases explicitly
match optional_value {
    Some(val) => process_value(val),
    None => {
        log::warn!("Value not present, using default");
        use_default()
    }
}

// Bad - assumes value exists
if optional_value.is_some() {
    process_value(optional_value.unwrap())
}
```

#### Platform-Specific Code

```rust
// Good - compile-time error for unsupported platforms
#[cfg(not(unix))]
compile_error!("Only Unix-like systems are supported");

// Bad - runtime panic
#[cfg(not(unix))]
panic!("Only Unix-like systems are supported");
```

### Critical Modules

The following modules are considered critical and must have zero unwrap() or panic!() calls:

1. **Analytics Engine** (`src/analytics/`)
   - Session tracking
   - Cost calculation
   - Performance metrics

2. **Cook Orchestrator** (`src/cook/orchestrator.rs`)
   - Workflow execution
   - Command coordination
   - Error recovery

3. **Git Operations** (`src/git/`)
   - Repository management
   - Worktree operations
   - Commit tracking

4. **Session Tracking** (`src/cook/session/`)
   - State management
   - Progress tracking
   - Checkpointing

5. **Resume Logic** (`src/resume_logic.rs`)
   - Workflow resumption
   - State recovery
   - Checkpoint validation

### Testing Error Paths

All error handling code must be tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_on_missing_file() {
        let result = read_file(Path::new("/nonexistent"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read"));
    }

    #[test]
    fn test_graceful_recovery() {
        let mut state = WorkflowState::new();
        let result = state.recover_from_error(test_error());
        assert!(result.is_ok());
        assert_eq!(state.status(), Status::Recovered);
    }
}
```

### Documentation Requirements

All functions that return `Result` must document:

```rust
/// Reads the configuration file from the specified path.
///
/// # Arguments
/// * `path` - Path to the configuration file
///
/// # Returns
/// * `Ok(Config)` - Successfully parsed configuration
/// * `Err` - If file doesn't exist or parsing fails
///
/// # Errors
/// This function will return an error if:
/// * The file cannot be read
/// * The file contents are not valid YAML
/// * Required configuration fields are missing
pub fn read_config(path: &Path) -> Result<Config> {
    // implementation
}
```

## Testing

### Test Categories

- **Unit Tests**: Test individual functions and modules
- **Integration Tests**: Test component interactions
- **Error Path Tests**: Specifically test error handling
- **Scenario Tests**: Test complete workflows

### Writing Tests

1. Test both success and failure paths
2. Use descriptive test names
3. Include edge cases
4. Test error messages and context

## Pull Request Process

1. Ensure all tests pass: `cargo test`
2. Run linter: `cargo clippy`
3. Format code: `cargo fmt`
4. Update documentation if needed
5. Add tests for new functionality
6. Ensure no new unwrap() or panic!() calls in production code
7. Update CHANGELOG.md with your changes
8. Submit PR with clear description

## Code Style

### Rust Style Guide

- Follow standard Rust naming conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer explicit error handling over panics
- Document public APIs
- Keep functions focused and small
- Use meaningful variable names

### Commit Messages

- Use present tense ("Add feature" not "Added feature")
- Keep first line under 50 characters
- Reference issues and pull requests
- Include context for why changes were made

Example:
```
fix: replace panic!() with proper error handling

- Eliminated all panic!() calls from production code
- Replaced unwrap() with ? operator in critical modules
- Added comprehensive error context using anyhow
- Fixes #123
```

## Questions?

If you have questions about contributing, please open an issue for discussion.

Thank you for contributing to stillwater!
