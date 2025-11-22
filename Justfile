# Stillwater - Justfile
# Quick development commands for Rust projects

# Default recipe - show available commands
default:
    @just --list

# Development commands
alias t := test
alias c := check
alias f := fmt
alias l := lint

# === BUILDING ===

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Build with all features enabled
build-all:
    cargo build --all-features

# Build with optimizations for native CPU
build-native:
    RUSTFLAGS="-C target-cpu=native" cargo build --release

# Clean build artifacts
clean:
    cargo clean

# === TESTING ===

# Run all tests
test:
    cargo build
    @echo "Running tests..."
    cargo test

# Run tests with output
test-verbose:
    cargo test --nocapture

# Run tests with specific pattern
test-pattern PATTERN:
    cargo test {{PATTERN}}

# Run tests and watch for changes
test-watch:
    cargo watch -x test

# Run tests with coverage using llvm-cov
coverage:
    #!/usr/bin/env bash
    # Ensure rustup's cargo is in PATH (needed for llvm-tools-preview)
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "Cleaning previous coverage data..."
    cargo llvm-cov clean
    echo "Generating code coverage report with llvm-cov..."
    cargo llvm-cov --features async --lib --html --output-dir target/coverage
    echo "Coverage report generated at target/coverage/html/index.html"

# Run tests with coverage (lcov format)
coverage-lcov:
    #!/usr/bin/env bash
    set -euo pipefail  # Exit on error, undefined variables, and pipe failures
    # Ensure rustup's cargo is in PATH (needed for llvm-tools-preview)
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "Cleaning previous coverage data..."
    cargo llvm-cov clean
    # Ensure target/coverage directory exists
    mkdir -p target/coverage
    echo "Generating code coverage report with llvm-cov (lcov format)..."
    cargo llvm-cov --features async --lib --lcov --output-path target/coverage/lcov.info
    echo "Coverage report generated at target/coverage/lcov.info"
    # Verify the file was actually created
    if [ ! -f target/coverage/lcov.info ]; then
        echo "ERROR: Coverage file was not generated at target/coverage/lcov.info"
        exit 1
    fi

# Run tests with coverage and check threshold
coverage-check:
    #!/usr/bin/env bash
    # Ensure rustup's cargo is in PATH (needed for llvm-tools-preview)
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "Checking code coverage threshold..."
    cargo llvm-cov clean
    mkdir -p target/coverage
    cargo llvm-cov --features async --lib --json --output-path target/coverage/coverage.json
    COVERAGE=$(cat target/coverage/coverage.json | jq -r '.data[0].totals.lines.percent')
    echo "Current coverage: ${COVERAGE}%"
    if (( $(echo "$COVERAGE < 80" | bc -l) )); then
        echo "⚠️  Coverage is below 80%: $COVERAGE%"
        exit 1
    else
        echo "✅ Coverage meets 80% threshold: $COVERAGE%"
    fi

# Open coverage report in browser
coverage-open: coverage
    open target/coverage/html/index.html

# Run integration tests only
test-integration:
    cargo test --test '*'

# Run benchmarks
bench:
    cargo bench

# Run all tests including doc tests
test-all:
    cargo test --all-features

# === CODE QUALITY ===

# Format code
fmt:
    cargo fmt

# Check formatting without making changes
fmt-check:
    cargo fmt --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run clippy with all targets
lint-all:
    cargo clippy --lib --tests --all-features -- -D warnings

# Quick check without building
check:
    cargo check

# Check all targets and features
check-all:
    cargo check --all-targets --all-features

# Fix automatically fixable lints
fix:
    cargo fix --allow-dirty

# === DOCUMENTATION ===

# Generate and open documentation
doc:
    cargo doc --open

# Generate documentation for all dependencies
doc-all:
    cargo doc --all --open

# Check documentation for errors
doc-check:
    cargo doc --no-deps

# === DEPENDENCIES ===

# Update dependencies
update:
    cargo update

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Add a new dependency
add CRATE:
    cargo add {{CRATE}}

# Add a development dependency
add-dev CRATE:
    cargo add --dev {{CRATE}}

# Remove a dependency
remove CRATE:
    cargo remove {{CRATE}}

# === UTILITY ===

# Show project tree structure
tree:
    tree -I 'target|node_modules'

# Show git status
status:
    git status

# Create a new module
new-module NAME:
    mkdir -p src/{{NAME}}
    echo "//! {{NAME}} module" > src/{{NAME}}/mod.rs
    echo "pub mod {{NAME}};" >> src/lib.rs

# Create a new integration test
new-test NAME:
    echo "//! Integration test for {{NAME}}" > tests/{{NAME}}.rs

# Create a new example
new-example NAME:
    echo "//! Example: {{NAME}}" > examples/{{NAME}}.rs

# === CI/CD SIMULATION ===

# Run all CI checks locally (matches GitHub Actions)
ci:
    @echo "Running CI checks (matching GitHub Actions)..."
    @echo "Setting environment variables..."
    @export CARGO_TERM_COLOR=always && \
     export CARGO_INCREMENTAL=0 && \
     export RUSTFLAGS="-Dwarnings" && \
     export RUST_BACKTRACE=1 && \
     echo "Running tests..." && \
     cargo test --features async && \
     echo "Running doctests..." && \
     cargo test --doc --features async && \
     echo "Running clippy..." && \
     cargo clippy --all-targets --features async -- -D warnings && \
     echo "Checking formatting..." && \
     cargo fmt --all -- --check && \
     echo "Checking documentation..." && \
     cargo doc --no-deps --document-private-items && \
     echo "Checking Cargo.lock is up to date..." && \
     cargo generate-lockfile && \
     git diff --exit-code Cargo.lock && \
     echo "All CI checks passed!"

# Full CI build pipeline
ci-build:
    @echo "Building stillwater..."
    @echo "Checking code formatting..."
    cargo fmt --all -- --check
    @echo "Running clippy..."
    cargo clippy --lib --tests --all-features -- -D warnings
    @echo "Building project..."
    cargo build --release
    @echo "Running tests..."
    cargo test --all
    @echo "Build successful!"

# Pre-commit hook simulation
pre-commit: fmt lint test
    @echo "Pre-commit checks passed!"

# Full development cycle check
full-check: clean build test lint doc audit
    @echo "Full development cycle completed successfully!"

# === INSTALLATION ===

# Install development tools
install-tools:
    rustup component add rustfmt clippy llvm-tools-preview
    cargo install cargo-watch cargo-llvm-cov cargo-audit cargo-outdated

# Install additional development tools
install-extras:
    cargo install cargo-expand cargo-machete cargo-deny cargo-udeps

# === RELEASE ===

# Prepare for release (dry run)
release-check:
    cargo publish --dry-run

# Create a new release (requires manual version bump)
release:
    cargo publish

# === ADVANCED ===

# Expand macros for debugging
expand:
    cargo expand

# Find unused dependencies
unused-deps:
    cargo machete

# Security-focused dependency check
security-check:
    cargo deny check

# Find duplicate dependencies
duplicate-deps:
    cargo tree --duplicates

# === HELP ===

# Show detailed help for cargo commands
help:
    @echo "Cargo commands reference:"
    @echo "  cargo run      - Run examples"
    @echo "  cargo test     - Run tests"
    @echo "  cargo build    - Build the project"
    @echo "  cargo fmt      - Format code"
    @echo "  cargo clippy   - Run linter"
    @echo "  cargo check    - Quick syntax check"
    @echo "  cargo doc      - Generate documentation"
    @echo ""
    @echo "Use 'just <command>' for convenience aliases!"
