# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0] - 2025-11-27

### Added

#### Effect Ergonomics (Spec 022)

- **`Effect::run_standalone()`** - Convenience method for effects with unit environment
  - Eliminates boilerplate `run(&())` for effects that don't need dependencies
  - Available on `Effect<T, E, ()>` types only
  - Cleaner code for examples, tests, and standalone effects
  - Example: `Effect::pure(42).run_standalone().await` instead of `Effect::pure(42).run(&()).await`

#### Effect Tracing Integration (Spec 023)

- **`Effect::instrument(span)`** - Attach tracing spans to effect execution
  - Follows standard `tracing::Instrument` pattern for async code
  - Spans are entered on execution and exited on completion
  - Works with all `tracing` span types (`info_span!`, `debug_span!`, etc.)
  - Composes naturally with effect chains and parallel execution
  - Requires `tracing` feature flag
- **`examples/tracing_demo.rs`** - Production-grade tracing patterns
  - Semantic spans with business data (user_id, order_id)
  - Context chains for error narratives
  - Quiet happy path, verbose error patterns

### Changed

#### Tracing API Simplification

- **Refactored tracing to use standard `instrument()` pattern**
  - Replaced custom tracing implementation with standard `tracing::Instrument`
  - Single method instead of multiple: just use `effect.instrument(span)`
  - Better interoperability with existing tracing ecosystem

#### Documentation

- **Enhanced README retry/resilience section** - Added policy-as-data explanation for retry patterns
- **Updated box allocation cost documentation** - Transparency about current allocation overhead

## [0.9.0] - 2025-11-26

### Added

#### Retry and Resilience Patterns (Spec 001)

- **`retry` module** - Comprehensive retry and resilience support for Effect-based computations
  - `RetryPolicy` - Configurable retry policies with builder pattern
  - Multiple backoff strategies: `constant`, `linear`, `exponential`, `fibonacci`
  - `with_max_retries()` - Limit number of retry attempts
  - `with_max_delay()` - Cap maximum delay between retries
  - `with_jitter()` - Add randomness to prevent thundering herd (requires `jitter` feature)
- **Effect retry methods**:
  - `Effect::retry()` - Retry with a policy until success or exhaustion
  - `Effect::retry_if()` - Conditional retry based on error predicate
  - `Effect::retry_with_hooks()` - Retry with observability callbacks for logging/metrics
  - `Effect::with_timeout()` - Add timeout to any effect
- **Error types**:
  - `RetryExhausted<E>` - Contains final error, attempt count, and total elapsed time
  - `RetrySuccess<T>` - Contains value and retry metadata
  - `TimeoutError<E>` - Wraps timeout or inner error
  - `RetryEvent` - Passed to hooks with attempt info, error, and timing
- **New Cargo features**:
  - `jitter` - Enable jitter support for retry policies (adds `rand` dependency)
- **Documentation**:
  - Comprehensive example in `examples/retry_patterns.rs` (8 patterns demonstrated)
  - API documentation with usage examples
  - Guide in `docs/guide/15-retry.md`

### Changed

#### Code Quality Improvements

- **Eliminated all `#[allow(dead_code)]` annotations** - Improved test coverage instead of suppressing warnings
  - `tests/homogeneous_integration.rs` - Added test cases for `Null`, `Bool`, `String` variants
  - `src/retry/tests.rs` - Shared `RetryTestError` enum across retry_if tests
  - `src/effect.rs` - Added error-path assertions to `and_then_auto` tests
  - Removed unused test infrastructure (EmailService, Logger, Database::save)
  - Tests now exercise both success and error paths

## [0.8.0] - 2025-11-24

### Added

#### Homogeneous Validation (Spec 021)

- **`validation::homogeneous` module** - Type-safe validation for discriminated unions
  - `validate_homogeneous()` - Ensures all items have same discriminant before combining
  - `combine_homogeneous()` - Safe combination after homogeneity validation
  - Prevents runtime panics from combining incompatible enum variants
  - Follows "pure core, imperative shell" - validates at boundaries
  - Accumulates ALL type mismatches, not just first one
- **Real-world use cases**:
  - MapReduce aggregations with heterogeneous input
  - Event stream validation
  - Configuration validation with typed values
  - Database query result validation
- **Documentation**:
  - Comprehensive guide in `docs/guide/09-homogeneous-validation.md`
  - API comparison table showing validation vs panic approaches
  - Property-based tests verifying correctness
  - Integration tests with real-world scenarios
  - Runnable example in `examples/homogeneous_validation.rs`
- **Testing**:
  - Property-based tests with proptest
  - Integration tests demonstrating practical patterns
  - Edge case coverage (empty, single item, all same, mixed types)

#### Testing Utilities (Spec 018)

- **`MockEnv` builder** - Composable test environment builder
  - Chain multiple dependencies with `.with()` method
  - Signature: `MockEnv::new().with(|| dependency).build()`
  - Creates nested tuple structure: `(((), Dep1), Dep2)`
  - Enables clean dependency injection for tests
- **`assert_success!` macro** - Assert validation succeeds
  - Panics with error details if validation is a `Failure`
  - Usage: `assert_success!(validate_email("user@example.com"))`
- **`assert_failure!` macro** - Assert validation fails
  - Panics with value details if validation is a `Success`
  - Usage: `assert_failure!(validate_email("invalid"))`
- **`assert_validation_errors!` macro** - Assert specific errors
  - Verifies exact error messages in failures
  - Usage: `assert_validation_errors!(result, vec!["error1", "error2"])`
- **`TestEffect` wrapper** - Deterministic effect testing
  - Test effects without real I/O operations
  - Provides `new()`, `run()`, and `into_effect()` methods
  - Enables controlled testing of success and failure paths
- **Property-based testing support** - Optional `proptest` feature
  - `Arbitrary` implementation for `Validation<T, E>`
  - Generates random Success and Failure instances
  - Enables comprehensive property testing

#### Documentation

- **`docs/guide/14-testing.md`** - Comprehensive testing guide (433 lines)
  - MockEnv builder patterns for test environments
  - Assertion macro usage and examples
  - TestEffect patterns for deterministic testing
  - Property-based testing with proptest
  - Testing best practices and patterns
- **`tests/testing_utilities.rs`** - Integration test examples (369 lines)
  - Real-world testing patterns
  - Domain model testing examples
  - Error accumulation verification
  - Effect testing with mock environments

#### Module and Exports

- **New `src/testing.rs` module** - All testing utilities (494 lines)
  - Full documentation with examples
  - Comprehensive test coverage (13 unit tests)
- **Exported in prelude** - `MockEnv`, `TestEffect`, and assertion macros
  - Available via `use stillwater::prelude::*;`
  - Direct access via `stillwater::testing::{...}`
- **New Cargo feature** - `proptest` for property-based testing
  - Optional dependency on proptest crate
  - Enable with `features = ["proptest"]`

## [0.6.0] - 2025-11-24

### Added

#### Traverse and Sequence Utilities (Spec 016)

- **`traverse()`** - Apply a validation function over a collection
  - Validates each element and accumulates all errors
  - Signature: `traverse<T, U, E, F, I>(iter: I, f: F) -> Validation<Vec<U>, E>`
  - More efficient than `map(f).sequence()` for validation workflows
- **`sequence()`** - Convert collection of validations into validation of collection
  - Signature: `sequence<T, E, I>(iter: I) -> Validation<Vec<T>, E>`
  - Accumulates all errors using `Semigroup`
  - Useful when you already have `Vec<Validation<T, E>>`
- **`traverse_effect()`** - Apply an effect function over a collection
  - Processes each element with fail-fast semantics
  - Signature: `traverse_effect<T, U, E, Env, F, I>(iter: I, f: F) -> Effect<Vec<U>, E, Env>`
  - Stops at first error for efficiency
- **`sequence_effect()`** - Convert collection of effects into effect of collection
  - Signature: `sequence_effect<T, E, Env, I>(iter: I) -> Effect<Vec<T>, E, Env>`
  - Fail-fast execution for effect collections

#### Documentation

- **`docs/guide/12-traverse-patterns.md`** - Comprehensive guide to traverse operations (581 lines)
  - Collection validation patterns with error accumulation
  - Effect processing over collections
  - Batch operations and practical examples
  - Comparison of traverse vs sequence
  - Real-world use cases: batch user registration, config validation, file processing
- **Updated `docs/guide/02-validation.md`** - Added traverse examples (+102 lines)
  - Collection validation patterns
  - Batch user registration example
  - Demonstrates error accumulation with traverse
- **Updated `docs/guide/README.md`** - Added chapter 12 and updated quick reference
  - New "Advanced Patterns" section with traverse chapter
  - Added collection validation and batch processing to quick reference table
- **`examples/traverse.rs`** - 7 comprehensive examples (450+ lines)
  - Basic traverse with validation
  - Sequencing existing validations
  - Batch user validation with error accumulation
  - Effect traverse for processing
  - Batch file processing simulation
  - Sequence effect patterns
  - Config validation example
- Updated README.md with traverse features and example reference

#### Module and Exports

- **New `src/traverse.rs` module** - All traverse/sequence implementations (400 lines)
  - Full documentation with examples
  - Comprehensive test coverage (18 tests)
- **Exported in prelude** - `traverse`, `sequence`, `traverse_effect`, `sequence_effect`
  - Available via `use stillwater::prelude::*;`
  - Direct access via `stillwater::traverse::{...}`

## [0.5.0] - 2025-11-23

### Added

#### Extended Semigroup Implementations (Spec 014)

- **`Semigroup` for `HashMap<K, V: Semigroup>`** - Merge maps, combining values with same key
  - Enables configuration merging and error aggregation by type
  - Values are combined using their Semigroup instance
- **`Semigroup` for `HashSet<T>`** - Union operation (all unique elements)
  - Useful for feature flags and permission sets
- **`Semigroup` for `BTreeMap<K, V: Semigroup>`** - Same as HashMap but maintains sorted keys
- **`Semigroup` for `BTreeSet<T>`** - Same as HashSet but maintains sorted elements
- **`Semigroup` for `Option<T: Semigroup>`** - Lifts semigroup operation to Option
  - `Some(a).combine(Some(b))` = `Some(a.combine(b))`
  - `Some(a).combine(None)` = `Some(a)`
  - `None.combine(Some(b))` = `Some(b)`
  - `None.combine(None)` = `None`

#### Wrapper Types for Alternative Semantics (Spec 014)

- **`First<T>`** - Keep first (left) value, discard second
  - Useful for default configuration values
- **`Last<T>`** - Keep last (right) value, discard first
  - Useful for override semantics in layered configuration
- **`Intersection<Set>`** - Set intersection instead of union
  - Useful for finding common permissions or required features

#### Parallel Effect Execution (Spec 015)

- **`Effect::par_all()`** - Run multiple effects in parallel, collect all results and errors
  - All effects run concurrently
  - Success only if all effects succeed
  - Accumulates all errors on failure
- **`Effect::par_try_all()`** - Run effects in parallel, short-circuit on first error
  - More efficient when you don't need all errors
  - Fail-fast semantics for critical operations
- **`Effect::race()`** - Race multiple effects, return first to succeed
  - Useful for fallback data sources and timeout patterns
  - Returns all errors if all effects fail
- **`Effect::par_all_limit()`** - Run effects in parallel with concurrency limit
  - Prevents resource exhaustion (connection pools, rate limits)
  - Configurable maximum concurrent tasks

#### Documentation

- **`docs/guide/11-parallel-effects.md`** - Comprehensive guide to parallel execution (643 lines)
  - Detailed examples for all four parallel methods
  - Performance considerations and best practices
  - Common patterns: scatter-gather, timeouts, graceful degradation
  - Error handling strategies for parallel operations
- **Updated `docs/guide/01-semigroup.md`** - Extended with collection implementations (+350 lines)
  - HashMap, HashSet, BTreeMap, BTreeSet examples
  - Option lifting patterns
  - Wrapper types (First, Last, Intersection)
  - Real-world use cases: config merging, error aggregation
- **`examples/parallel_effects.rs`** - 8 comprehensive examples (450 lines)
  - par_all with error accumulation
  - par_try_all for fail-fast
  - race for fallback sources and timeouts
  - par_all_limit for bounded concurrency
  - Dashboard loading (scatter-gather pattern)
  - Graceful degradation patterns
- **`examples/extended_semigroup.rs`** - 14 comprehensive examples (600 lines)
  - HashMap and BTreeMap merging
  - HashSet and BTreeSet union
  - Option lifting for optional errors
  - First/Last for configuration layering
  - Intersection for permission checking
  - Error aggregation by type
- Updated README.md with new features and examples

### Technical Details

#### Spec 014 Technical Details
- Zero-cost abstractions via trait monomorphization
- HashMap/BTreeMap combine uses `entry().and_modify().or_insert()`
- Set union via `extend()` for optimal performance
- Option combine handles all four cases correctly
- Property-based tests verify associativity for all implementations
- Wrapper types are zero-sized newtypes

#### Spec 015 Technical Details
- Uses `futures` crate for async combinators
- `par_all` and `par_try_all` use `join_all` and `try_join_all`
- `race` uses `select_ok` for first success
- `par_all_limit` uses `buffer_unordered` for concurrency control
- Environment must be `Sync` for safe cross-thread sharing
- Actual concurrency verified by timing tests
- Works with tokio, async-std, and other async runtimes

## [0.4.0] - 2025-11-23

### Added

#### NonEmptyVec Type
- **`NonEmptyVec<T>`** - Type-safe non-empty vector guaranteed to contain at least one element
  - Prevents runtime errors in operations requiring non-empty collections
  - Head + tail internal structure for guaranteed non-emptiness
  - Safe construction via `new()`, `singleton()`, `from_vec()` (returns `Option`)
  - Unsafe construction via `from_vec_unchecked()` for performance-critical paths

#### NonEmptyVec Operations
- **Safe accessors** that never fail:
  - `head()` - Get first element (always succeeds, no `Option`)
  - `last()` - Get last element (always succeeds, no `Option`)
  - `tail()` - Get all elements except the first
  - `len()` - Get length (always >= 1)
  - `is_empty()` - Always returns `false` (satisfies clippy lint)
- **Mutation methods**:
  - `push()` - Add element to end
  - `pop()` - Remove from end (returns `Option`, preserves head)
- **Functional operations**:
  - `map()` - Transform all elements (preserves non-emptiness)
  - `filter()` - Filter elements (returns `Vec<T>` since may become empty)
  - `iter()` - Iterate over all elements
  - `into_iter()` - Consuming iterator
  - `into_vec()` - Convert to regular `Vec<T>`

#### Trait Implementations
- **`Semigroup`** for `NonEmptyVec<T>` - Concatenation via `combine()`
- **`IntoIterator`** - Enables use in for loops
- **`Index<usize>`** - Array-like indexing (panics on out of bounds)

#### Validation Integration
- **`Validation::fail(error: E)`** - Convenience method for `Validation<T, NonEmptyVec<E>>`
  - Creates validation failure with single error
  - Eliminates need to manually construct `NonEmptyVec`
  - Type-safe error accumulation guaranteed to have at least one error

#### Testing
- **18 new unit tests** for NonEmptyVec:
  - Construction methods (new, singleton, from_vec, from_vec_unchecked)
  - Safe operations (head, tail, last, len)
  - Mutation (push, pop boundary conditions)
  - Functional operations (map, filter, iter, into_iter)
  - Trait implementations (Semigroup, Index, IntoIterator)
  - Panic tests for unsafe operations
- **2 integration tests** for Validation with NonEmptyVec
- Test count increased from 163 to 181 unit tests

#### Documentation
- **`examples/nonempty.rs`** - Comprehensive NonEmptyVec examples (330 lines)
  - 8 examples covering all functionality
  - Basic creation patterns
  - Safe operations demonstration
  - Mutation examples
  - Functional programming patterns
  - Semigroup concatenation
  - Validation integration
  - Real-world batch processing scenario
  - Safe aggregation operations (min, max, avg)
- **Module documentation** with examples for all public methods
- Updated README.md with NonEmptyVec feature
- Integration examples with Validation type

### Changed
- Bumped version from 0.3.0 to 0.4.0
- Updated README installation instructions to 0.4
- Updated examples table with nonempty.rs

### Technical Details
- Zero-cost abstraction using head + tail structure
- `from_vec()` uses `vec.remove(0)` for simplicity (O(n) but acceptable)
- `filter()` returns `Vec<T>` not `Option<NonEmptyVec<T>>` - simpler API
- `is_empty()` always returns `false` - included for clippy compliance
- Integrates seamlessly with existing Validation error accumulation

### Use Cases
- **Validation failures** - Guarantee at least one error when validation fails
- **Aggregations** - Operations like `max()`, `min()` that require elements
- **Batch processing** - Ensure batches are never empty
- **Type safety** - Eliminate `Option` unwraps in operations needing elements

## [0.3.0] - 2025-11-23

### Added

#### Reader Pattern Helpers
- **`Effect::ask()`** - Access entire environment in Reader monad pattern
  - Returns current environment as Effect value
  - Enables dependency injection without globals
  - Zero-cost abstraction for environment access
- **`Effect::asks(f)`** - Query and transform environment
  - Extract specific fields or compute derived values
  - Compose environment queries functionally
  - Type-safe environment projection
- **`Effect::local(f, effect)`** - Temporarily modify environment
  - Run effect with modified environment
  - Original environment preserved after execution
  - Enables scoped configuration overrides

#### Testing
- **8 new unit tests** for Reader pattern helpers:
  - Identity law verification for ask()
  - Composition tests for asks()
  - Nested environment modification with local()
  - Integration with existing Effect combinators
- Test count increased from 152 to 163 unit tests
- Documentation test count increased from 68 to 72

#### Documentation
- **`docs/guide/09-reader-pattern.md`** - Comprehensive Reader pattern guide (619 lines)
  - Reader monad concepts and motivation
  - Usage patterns for all three helpers
  - Best practices and anti-patterns
  - Real-world dependency injection examples
  - Integration with Effect combinators
- Updated `docs/guide/03-effects.md` with Reader pattern section (+124 lines)
- Updated README.md with Reader pattern example (+35 lines)
- Updated DESIGN.md with Reader pattern design rationale (+56 lines)

### Changed
- Bumped version from 0.2.0 to 0.3.0
- Enhanced Effect type with Reader monad capabilities
- Improved doctest examples with explicit type annotations

### Technical Details
- Reader pattern implementation follows monad laws
- ask() provides monadic unit for environment
- asks() and local() enable functional environment manipulation
- All helpers work seamlessly with async effects
- No runtime overhead - pure compile-time abstractions

## [0.2.0] - 2025-11-23

### Added

#### Core Traits
- **`Monoid` trait** - Extends Semigroup with identity elements (`empty()` method)
  - Enables folding without explicit initial values
  - Provides foundation for powerful composition patterns
  - Supports parallel reduction strategies

#### Monoid Implementations
- **Standard types**: `Vec<T>`, `String`, `Option<T>` (where `T: Semigroup`)
- **Tuples**: 2-element through 12-element tuples (where each element is `Monoid`)
- **Numeric wrappers**:
  - `Sum<T>` - Addition monoid with 0 as identity
  - `Product<T>` - Multiplication monoid with 1 as identity
  - `Max<T>` - Maximum semigroup (Monoid via `Option<Max<T>>`)
  - `Min<T>` - Minimum semigroup (Monoid via `Option<Min<T>>`)

#### Helper Traits and Functions
- **`One` trait** - Provides multiplicative identity for numeric types
  - Implementations for all standard integer and float types
- **`fold_all<M, I>(iter: I) -> M`** - Fold iterator using monoid identity
- **`reduce<M, I>(iter: I) -> M`** - Alias for `fold_all`

#### Testing
- **Property-based testing** with `proptest` dependency
  - Tests for monoid laws (left identity, right identity, associativity)
  - Tests for Vec, String, Option, Sum, Product monoids
  - Regression test suite for discovered edge cases
- **41 new unit tests** covering:
  - Identity laws for all monoid implementations
  - Associativity verification
  - fold_all and reduce operations
  - Edge cases (empty collections, None values)

#### Documentation
- **`docs/guide/08-monoid.md`** - Comprehensive monoid guide (353 lines)
  - Monoid laws and mathematical properties
  - Examples for all provided implementations
  - Numeric monoid patterns (Sum, Product, Max, Min)
  - fold_all and reduce usage
  - Real-world use cases
- **`examples/monoid.rs`** - Complete working examples (370 lines)
  - 10 examples covering all monoid types
  - Real-world scenarios: log aggregation, statistics, config merging
  - Progressive complexity from basics to advanced patterns
- Updated README.md with monoid features and examples
- Updated docs/guide/README.md with Chapter 8

### Changed
- Bumped version from 0.1.0 to 0.2.0
- Test count increased from 111 to 152 unit tests
- Documentation test count increased from 58 to 68

### Technical Details
- Uses macro-based code generation for tuple implementations
- Zero-cost abstractions via trait monomorphization
- Property-based tests ensure mathematical correctness
- No new runtime dependencies (proptest is dev-only)

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

[Unreleased]: https://github.com/iepathos/stillwater/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/iepathos/stillwater/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/iepathos/stillwater/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/iepathos/stillwater/compare/v0.6.0...v0.8.0
[0.6.0]: https://github.com/iepathos/stillwater/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/iepathos/stillwater/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/iepathos/stillwater/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/iepathos/stillwater/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/iepathos/stillwater/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/iepathos/stillwater/releases/tag/v0.1.0
