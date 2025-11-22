---
number: 008
title: Project Structure and Build Setup
category: infrastructure
priority: critical
status: draft
dependencies: []
created: 2025-11-21
---

# Specification 008: Project Structure and Build Setup

**Category**: infrastructure
**Priority**: critical
**Status**: draft
**Dependencies**: None (must be done first)

## Context

Before implementing any code, we need a properly structured Rust project with:
- Correct Cargo.toml configuration
- Logical directory structure
- Development dependencies (testing, benchmarking)
- CI/CD pipeline for quality gates
- Formatting and linting setup

This spec establishes the foundation for all other specifications to build upon.

## Objective

Initialize a complete Rust project structure for Stillwater with proper configuration, dependencies, directory layout, and CI/CD pipelines.

## Requirements

### Functional Requirements

- Initialize Cargo project with library crate
- Configure appropriate Rust edition (2021)
- Set up module structure (lib.rs, prelude, modules)
- Add development dependencies (tokio, proptest, criterion)
- Configure workspace for examples and benchmarks
- Set up rustfmt and clippy configurations
- Create CI pipeline (GitHub Actions)
- Configure docs.rs integration
- Set up test organization
- Create contributing guidelines

### Non-Functional Requirements

- Follow Rust best practices
- Support both stable and nightly Rust
- Fast CI pipeline (<5 minutes)
- Clear separation of concerns in modules
- Easy for contributors to set up locally
- Comprehensive documentation

## Acceptance Criteria

- [ ] Cargo.toml with correct metadata and dependencies
- [ ] src/lib.rs with module structure
- [ ] src/prelude.rs with convenience re-exports
- [ ] tests/ directory for integration tests
- [ ] examples/ directory structure
- [ ] benches/ directory for benchmarks
- [ ] .github/workflows/ci.yml for CI pipeline
- [ ] rustfmt.toml for code formatting
- [ ] clippy.toml for linting rules
- [ ] CONTRIBUTING.md with setup instructions
- [ ] LICENSE file (MIT or Apache-2.0)
- [ ] README.md links to documentation
- [ ] All commands in docs run successfully

## Technical Details

### Implementation Approach

#### Directory Structure

```
stillwater/
├── .github/
│   └── workflows/
│       ├── ci.yml              # Main CI pipeline
│       └── docs.yml            # Documentation deployment
├── benches/
│   ├── validation_bench.rs     # Validation benchmarks
│   └── effect_bench.rs         # Effect benchmarks
├── docs/
│   └── guide/
│       ├── 01-semigroup.md
│       ├── 02-validation.md
│       ├── 03-effects.md
│       └── 04-try-trait.md
├── examples/
│   ├── form_validation.rs
│   ├── user_registration.rs
│   ├── error_context.rs
│   ├── data_pipeline.rs
│   └── testing_patterns.rs
├── specs/
│   ├── 001-semigroup-trait.md
│   ├── 002-validation-type.md
│   └── ... (all specs)
├── src/
│   ├── lib.rs                  # Root module
│   ├── prelude.rs              # Re-exports
│   ├── semigroup.rs            # Spec 001
│   ├── validation.rs           # Spec 002
│   ├── effect.rs               # Spec 003
│   ├── context.rs              # Spec 004
│   └── io.rs                   # Spec 005
├── tests/
│   ├── validation_tests.rs     # Integration tests
│   ├── effect_tests.rs
│   └── integration_tests.rs
├── .gitignore
├── Cargo.toml
├── CONTRIBUTING.md
├── DESIGN.md
├── LICENSE
├── PHILOSOPHY.md
└── README.md
```

#### Cargo.toml

```toml
[package]
name = "stillwater"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV: Minimum Supported Rust Version
authors = ["Stillwater Contributors"]
license = "MIT OR Apache-2.0"
description = "Pragmatic functional effects for Rust: validation accumulation and effect composition"
repository = "https://github.com/yourusername/stillwater"
documentation = "https://docs.rs/stillwater"
keywords = ["functional", "validation", "effects", "monad", "error-handling"]
categories = ["rust-patterns", "data-structures"]
readme = "README.md"

[lib]
name = "stillwater"
path = "src/lib.rs"

[features]
default = []
try_trait = []  # Nightly-only Try trait support (Spec 007)
full = ["try_trait"]

[dependencies]
# No dependencies for core library (zero-cost abstractions)

[dev-dependencies]
# Async runtime for tests and examples
tokio = { version = "1.35", features = ["full"] }

# Property-based testing
proptest = "1.4"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[[example]]
name = "form_validation"
path = "examples/form_validation.rs"

[[example]]
name = "user_registration"
path = "examples/user_registration.rs"

[[example]]
name = "error_context"
path = "examples/error_context.rs"

[[example]]
name = "data_pipeline"
path = "examples/data_pipeline.rs"

[[example]]
name = "testing_patterns"
path = "examples/testing_patterns.rs"

[[bench]]
name = "validation_bench"
harness = false

[[bench]]
name = "effect_bench"
harness = false
```

#### src/lib.rs

```rust
//! Stillwater: Pragmatic functional effects for Rust
//!
//! Stillwater provides composable abstractions for validation and effect handling:
//!
//! - **Validation**: Accumulate all errors instead of failing on the first
//! - **Effect**: Separate pure business logic from I/O side effects
//! - **Context Errors**: Preserve error trails for better debugging
//!
//! # Quick Start
//!
//! ```rust
//! use stillwater::prelude::*;
//!
//! // Validation with error accumulation
//! fn validate_user(email: &str, age: u8) -> Validation<User, Vec<Error>> {
//!     Validation::all((
//!         validate_email(email),
//!         validate_age(age),
//!     ))
//!     .map(|(email, age)| User { email, age })
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `try_trait`: Enable `?` operator support (requires nightly Rust)
//!
//! # Philosophy
//!
//! Stillwater follows the "pure core, imperative shell" pattern:
//! - Keep business logic pure (no I/O, no side effects)
//! - Push effects to the boundaries
//! - Make testing effortless (pure functions need no mocks)
//!
//! See [PHILOSOPHY.md] for more details.

#![cfg_attr(feature = "try_trait", feature(try_trait_v2))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod semigroup;
pub mod validation;
pub mod effect;
pub mod context;
pub mod io;

pub mod prelude;

// Re-export main types at crate root
pub use semigroup::Semigroup;
pub use validation::Validation;
pub use effect::Effect;
pub use context::ContextError;
pub use io::IO;
```

#### src/prelude.rs

```rust
//! Convenience re-exports for common types and traits.
//!
//! Import this module to get the most commonly used items:
//!
//! ```rust
//! use stillwater::prelude::*;
//! ```

pub use crate::semigroup::Semigroup;
pub use crate::validation::Validation;
pub use crate::effect::Effect;
pub use crate::context::ContextError;
pub use crate::io::IO;
```

#### .github/workflows/ci.yml

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust ${{ matrix.rust }}
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all-features

      - name: Run tests (no features)
        run: cargo test --no-default-features

      - name: Run doc tests
        run: cargo test --doc --all-features

  test-nightly:
    name: Test with Try trait (nightly)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust nightly
        uses: dtolnay/rust-toolchain@nightly

      - name: Run tests with Try trait
        run: cargo test --features try_trait

  fmt:
    name: Formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run Clippy
        run: cargo clippy --all-features -- -D warnings

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --all-features --out Xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  examples:
    name: Examples
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build examples
        run: cargo build --examples --all-features

      - name: Run examples
        run: |
          cargo run --example form_validation
          cargo run --example user_registration
          cargo run --example error_context
          cargo run --example data_pipeline
          cargo run --example testing_patterns

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Check docs
        run: cargo doc --all-features --no-deps
        env:
          RUSTDOCFLAGS: -D warnings

  msrv:
    name: Minimum Supported Rust Version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust 1.75
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.75"

      - name: Check MSRV
        run: cargo check --all-features
```

#### rustfmt.toml

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
format_code_in_doc_comments = true
normalize_comments = true
wrap_comments = true
comment_width = 80
```

#### clippy.toml

```toml
# Clippy configuration for stillwater

# Lints to allow
allow = [
    "clippy::module_name_repetitions",  # Effect::Effect is fine
    "clippy::type_complexity",          # Complex types are intentional
]

# Lints to deny
deny = [
    "clippy::all",
    "clippy::pedantic",
    "clippy::cargo",
]

# Warn on these
warn = [
    "clippy::clone_on_ref_ptr",
    "clippy::dbg_macro",
    "clippy::decimal_literal_representation",
    "clippy::exit",
    "clippy::filetype_is_file",
    "clippy::float_cmp_const",
]
```

#### .gitignore

```
# Rust
/target/
Cargo.lock  # Include for binary crates, exclude for libraries
**/*.rs.bk

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Testing
tarpaulin-report.html
cobertura.xml

# Benchmarking
benches/target/
```

#### CONTRIBUTING.md

```markdown
# Contributing to Stillwater

Thank you for your interest in contributing! This document provides guidelines and setup instructions.

## Development Setup

### Prerequisites

- Rust 1.75 or later (stable)
- For Try trait support: Rust nightly

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/stillwater.git
   cd stillwater
   ```

2. Run tests to verify setup:
   ```bash
   cargo test --all-features
   ```

3. Install development tools:
   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-tarpaulin  # For coverage
   ```

## Development Workflow

### Before Committing

1. **Format code**:
   ```bash
   cargo fmt
   ```

2. **Run linter**:
   ```bash
   cargo clippy --all-features -- -D warnings
   ```

3. **Run all tests**:
   ```bash
   cargo test --all-features
   ```

4. **Check examples**:
   ```bash
   cargo build --examples
   ```

5. **Check documentation**:
   ```bash
   cargo doc --all-features --no-deps
   ```

### Commit Messages

Follow conventional commits format:
- `feat: Add new feature`
- `fix: Fix bug`
- `docs: Update documentation`
- `test: Add tests`
- `refactor: Refactor code`
- `chore: Maintenance tasks`

## Code Standards

### Functional Programming Principles

- **Pure functions over stateful methods**: Easier to test and reason about
- **Immutability by default**: Prevent unexpected mutations
- **Function composition**: Build complex behavior from simple units
- **Separation of concerns**: Keep I/O at boundaries, pure logic in core

### Function Guidelines

- Maximum function length: 20 lines (prefer 5-10)
- Maximum nesting depth: 2 levels
- Maximum parameters: 3-4 (use structs for more)
- Single responsibility: One clear purpose per function

### Documentation

- All public items must have rustdoc comments
- Include examples in documentation
- Explain "why" not just "what"
- Link to relevant specs in comments

## Testing

### Test Organization

- Unit tests: In same file as implementation (`#[cfg(test)] mod tests`)
- Integration tests: In `tests/` directory
- Doc tests: In rustdoc examples
- Property tests: Using proptest

### Test Coverage

- Aim for >95% coverage
- All public APIs must have tests
- Include edge cases and error paths

### Running Tests

```bash
# All tests
cargo test --all-features

# Specific test
cargo test test_name

# With coverage
cargo tarpaulin --all-features

# Doc tests only
cargo test --doc
```

## Benchmarking

Run benchmarks with:
```bash
cargo bench
```

View results in `target/criterion/report/index.html`

## Questions?

Open an issue or discussion on GitHub!
```

### Architecture Changes

- New Cargo project structure
- CI/CD pipeline
- Development workflow

### APIs and Interfaces

No new APIs - this is infrastructure setup.

## Dependencies

- **Prerequisites**: None (this must be done first)
- **Affected Components**: All future specs build on this
- **External Dependencies**:
  - tokio (dev-dependency)
  - proptest (dev-dependency)
  - criterion (dev-dependency)

## Testing Strategy

### Validation Tests

```bash
# Verify all commands in CONTRIBUTING.md work
cargo fmt
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo build --examples
cargo doc --all-features --no-deps
cargo bench
```

### CI Pipeline Tests

- Push to GitHub triggers CI
- All jobs must pass
- Coverage report generated
- Examples run successfully

## Documentation Requirements

### Code Documentation

- README.md with setup instructions
- CONTRIBUTING.md with development workflow
- Inline comments in configuration files

### User Documentation

- Clear setup instructions
- Development workflow documented
- CI pipeline explained

### Architecture Updates

- Document module organization in DESIGN.md
- Explain build system choices

## Implementation Notes

### Zero Dependencies

Core library has ZERO runtime dependencies:
- Validation, Effect, Semigroup use only std
- All dev dependencies are for testing/benchmarking
- This keeps library lightweight and fast to compile

### MSRV Policy

Minimum Supported Rust Version: 1.75
- Test in CI to ensure compatibility
- Only bump MSRV when necessary
- Document MSRV changes in CHANGELOG

### Feature Flags

- `default`: No features enabled
- `try_trait`: Nightly-only Try trait support
- `full`: All features enabled

Users can opt-in to experimental features without affecting stable users.

### Examples as Documentation

Examples serve dual purpose:
1. Test that API is ergonomic
2. Provide copy-paste starting points for users

All examples must compile and run successfully in CI.

## Migration and Compatibility

No migration - this is initial setup.

## Open Questions

1. Should we support `no_std`?
   - Decision: Defer to post-MVP, requires async runtime considerations

2. What license should we use?
   - Decision: Dual MIT/Apache-2.0 (standard for Rust ecosystem)

3. Should we use a workspace for examples?
   - Decision: No, single crate is simpler for MVP

4. Should we enforce 100% documentation coverage?
   - Decision: Yes for public API, not for internal implementation
