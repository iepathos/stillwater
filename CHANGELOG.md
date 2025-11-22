# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-11-22

### Added

#### Core Types
- `Validation<T, E>` type for accumulating errors instead of short-circuiting
- `Effect<T, E, Env>` type for composable async computations with environment dependencies
- `IO` helper for creating effects from I/O operations
- `ContextError<E>` for preserving error context chains
- `Semigroup` trait for combining errors (Vec, String, tuples)

#### Validation Features
- `Validation::all()` for combining multiple validations and accumulating errors
- `Validation::all_vec()` for validating collections
- `and()`, `and_then()`, `map()`, `map_err()` combinators
- Conversion to/from `Result`
- Support for tuples up to 12 elements

#### Effect Features
- `Effect::pure()` for wrapping pure values
- `Effect::fail()` for creating failed effects
- `Effect::from_fn()` for sync operations with environment access
- `Effect::from_async()` for async operations
- `Effect::from_validation()` for converting validations to effects
- Composition via `map()`, `and_then()`, `or_else()`, `with()`
- Context chaining with `.context()` method
- Helper methods: `tap()` for side effects, `check()` for validation, `and_then_ref()` for borrowing

#### Async Support
- Full async/await integration
- Optional `tokio` dependency (feature flag: `async`)
- `IO::read_async()` and `IO::write_async()` for async I/O patterns

#### Documentation
- Comprehensive API documentation with examples
- 9 complete examples covering common use cases:
  - `validation.rs` - Form validation patterns
  - `effects.rs` - Effect composition
  - `pipeline.rs` - Data transformation pipelines
  - `user_registration.rs` - Real-world user registration
  - `form_validation.rs` - Complex form validation
  - `error_context.rs` - Error context preservation
  - `testing_patterns.rs` - Testing with mock environments
  - `data_pipeline.rs` - ETL-style data processing
  - `io_patterns.rs` - I/O separation patterns
- Design documentation (DESIGN.md, PHILOSOPHY.md)

#### Testing & Quality
- 111 unit tests
- 58 documentation tests
- Integration tests for combinators and try trait
- CI/CD pipeline with:
  - Multi-platform testing (Ubuntu, macOS)
  - Code coverage tracking
  - Security audits via cargo-deny
  - Clippy and rustfmt checks

#### Infrastructure
- GitHub Actions workflows for CI, coverage, and security
- cargo-deny configuration for dependency auditing
- Comprehensive .gitignore
- MIT license

### Design Decisions

- **Zero-cost abstractions**: Uses generics and monomorphization, no runtime overhead
- **Rust-first**: Works with `?` operator, integrates with existing error handling
- **Pure core, imperative shell**: Explicit separation of pure logic and I/O effects
- **Progressive adoption**: Can be adopted incrementally alongside `Result`
- **No heavy macros**: Clear types and obvious behavior
- **Async-first**: Built with async/await in mind from the start

### Known Limitations

- `try_trait` feature requires nightly Rust (optional, not needed for core functionality)
- API may evolve in 0.x versions based on community feedback
- No HKT-style monad abstractions (intentional - Rust doesn't support HKTs)

[Unreleased]: https://github.com/iepathos/stillwater/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/iepathos/stillwater/releases/tag/v0.1.0
