# Stillwater

[![Crates.io](https://img.shields.io/crates/v/stillwater)](https://crates.io/crates/stillwater)
[![Downloads](https://img.shields.io/crates/d/stillwater)](https://crates.io/crates/stillwater)
[![CI](https://github.com/iepathos/stillwater/actions/workflows/ci.yml/badge.svg)](https://github.com/iepathos/stillwater/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT)](LICENSE)

A Rust library for pragmatic effect composition and validation, emphasizing the **pure core, imperative shell** pattern.

## Philosophy

**Stillwater** embodies a simple idea:
- Pure functions (unchanging, referentially transparent)
- Effects (flowing, performing I/O)

Keep your business logic pure and calm like still water. Let effects flow at the boundaries.

## What Problems Does It Solve?

### 1. "I want ALL validation errors, not just the first one"

```rust
use stillwater::Validation;

// Standard Result: stops at first error
let email = validate_email(input)?;  // Stops here
let age = validate_age(input)?;      // Never reached if email fails

// Stillwater: accumulates all errors
let user = Validation::all((
    validate_email(input),
    validate_age(input),
    validate_name(input),
))?;
// Returns: Err(vec![EmailError, AgeError, NameError])
```

### 2. "How do I validate that all items have the same type before combining?"

```rust
use stillwater::validation::homogeneous::validate_homogeneous;
use std::mem::discriminant;

#[derive(Clone, Debug, PartialEq)]
enum Aggregate {
    Sum(f64),     // Can combine Sum + Sum
    Count(usize), // Can combine Count + Count
    // But Sum + Count is a type error!
}

// Without validation: runtime panic
let mixed = vec![Aggregate::Count(5), Aggregate::Sum(10.0)];
// items.into_iter().reduce(|a, b| a.combine(b))  // PANIC!

// With validation: type-safe error accumulation
let result = validate_homogeneous(
    mixed,
    |a| discriminant(a),
    |idx, _, _| format!("Type mismatch at index {}", idx),
);

match result {
    Validation::Success(items) => {
        // Safe to combine - all same type!
        let total = items.into_iter().reduce(|a, b| a.combine(b));
    }
    Validation::Failure(errors) => {
        // All mismatches reported: ["Type mismatch at index 1"]
    }
}
```

### 3. "How do I test code with database calls?"

```rust
use stillwater::prelude::*;

// Pure business logic (no DB, easy to test)
fn calculate_discount(customer: &Customer, total: Money) -> Money {
    match customer.tier {
        Tier::Gold => total * 0.15,
        _ => total * 0.05,
    }
}

// Effects at boundaries (mockable) - zero-cost by default
fn process_order(id: OrderId) -> impl Effect<Output = Invoice, Error = AppError, Env = AppEnv> {
    from_fn(move |env: &AppEnv| env.db.fetch_order(id))  // I/O
        .and_then(|order| {
            let total = calculate_total(&order);  // Pure!
            from_fn(move |env: &AppEnv| env.db.fetch_customer(order.customer_id))
                .map(move |customer| (order, customer, total))
        })
        .map(|(order, customer, total)| {
            let discount = calculate_discount(&customer, total);  // Pure!
            create_invoice(order.id, total - discount)            // Pure!
        })
        .and_then(|invoice| from_fn(move |env: &AppEnv| env.db.save(invoice))) // I/O
}

// Test with mock environment
#[tokio::test]
async fn test_with_mock_db() {
    let env = MockEnv::new();
    let result = process_order(id).run(&env).await?;
    assert_eq!(result.total, expected);
}
```

### 4. "I need to fetch multiple independent resources"

```rust
use stillwater::prelude::*;

// Combine independent effects - neither depends on the other
fn load_user_profile(id: UserId) -> impl Effect<Output = UserProfile, Error = AppError, Env = AppEnv> {
    fetch_user(id)
        .zip(fetch_settings(id))
        .zip(fetch_preferences(id))
        .map(|((user, settings), prefs)| UserProfile { user, settings, prefs })
}

// Or use zip3 for cleaner flat tuples
fn load_user_profile_v2(id: UserId) -> impl Effect<Output = UserProfile, Error = AppError, Env = AppEnv> {
    zip3(
        fetch_user(id),
        fetch_settings(id),
        fetch_preferences(id),
    )
    .map(|(user, settings, prefs)| UserProfile { user, settings, prefs })
}

// Combine results with a function directly using zip_with
let effect = fetch_price(item_id)
    .zip_with(fetch_quantity(item_id), |price, qty| price * qty);
```

### 5. "My errors lose context as they bubble up"

```rust
use stillwater::prelude::*;

fetch_user(id)
    .context("Loading user profile")
    .and_then(|user| process_data(user))
    .context("Processing user data")
    .run(&env).await?;

// Error output:
// Error: UserNotFound(12345)
//   -> Loading user profile
//   -> Processing user data
```

### 6. "I need clean dependency injection without passing parameters everywhere"

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Config {
    timeout: u64,
    retries: u32,
}

// Functions don't need explicit config parameters
fn fetch_data() -> impl Effect<Output = String, Error = String, Env = Config> {
    // Ask for config when needed
    asks(|cfg: &Config| format!("Fetching with timeout={}", cfg.timeout))
}

fn fetch_with_extended_timeout() -> impl Effect<Output = String, Error = String, Env = Config> {
    // Temporarily modify environment for specific operations
    local(
        |cfg: &Config| Config { timeout: cfg.timeout * 2, ..*cfg },
        fetch_data()
    )
}

let config = Config { timeout: 30, retries: 3 };
let result = fetch_with_extended_timeout().run(&config).await?;
// Uses timeout=60 without changing the original config
```

### 7. "I need guaranteed cleanup even when errors occur"

```rust
use stillwater::effect::bracket::{bracket, bracket2, acquiring, BracketError};
use stillwater::prelude::*;

// Single resource with guaranteed cleanup
let result = bracket(
    open_connection(),                           // Acquire
    |conn| async move { conn.close().await },    // Release (always runs)
    |conn| fetch_user(conn, user_id),            // Use
).run(&env).await;

// Multiple resources with LIFO cleanup order
let result = bracket2(
    open_database(),
    open_file(path),
    |db| async move { db.close().await },        // Released second
    |file| async move { file.close().await },    // Released first (LIFO)
    |db, file| process(db, file),
).run(&env).await;

// Fluent builder for ergonomic multi-resource management
let result = acquiring(open_conn(), |c| async move { c.close().await })
    .and(open_file(), |f| async move { f.close().await })
    .and(acquire_lock(), |l| async move { l.release().await })
    .with_flat3(|conn, file, lock| do_work(conn, file, lock))
    .run(&env)
    .await;

// Explicit error handling with BracketError
let result = bracket_full(acquire, release, use_fn).run(&env).await;
match result {
    Ok(value) => println!("Success"),
    Err(BracketError::AcquireError(e)) => println!("Acquire failed"),
    Err(BracketError::UseError(e)) => println!("Use failed, cleanup succeeded"),
    Err(BracketError::CleanupError(e)) => println!("Use succeeded, cleanup failed"),
    Err(BracketError::Both { use_error, cleanup_error }) => println!("Both failed"),
}
```

### 8. "Retry logic is scattered and hard to test"

```rust
use stillwater::{Effect, RetryPolicy};
use std::time::Duration;

// Stillwater: Policy as Data
// Define retry policies as pure, testable values
let api_policy = RetryPolicy::exponential(Duration::from_millis(100))
    .with_max_retries(5)
    .with_max_delay(Duration::from_secs(2))
    .with_jitter(0.25);

// Test the policy without any I/O
assert_eq!(api_policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
assert_eq!(api_policy.delay_for_attempt(1), Some(Duration::from_millis(200)));
assert_eq!(api_policy.delay_for_attempt(2), Some(Duration::from_millis(400)));

// Reuse the same policy across different effects
Effect::retry(|| fetch_user(id), api_policy.clone());
Effect::retry(|| save_order(order), api_policy.clone());

// Conditional retry: only retry transient failures
Effect::retry_if(
    || api_call(),
    api_policy,
    |err| matches!(err, ApiError::Timeout | ApiError::ServerError(_))
);

// Observability: hook into retry events for logging/metrics
Effect::retry_with_hooks(
    || api_call(),
    policy,
    |event| log::warn!(
        "Attempt {} failed: {}, retrying in {:?}",
        event.attempt, event.error, event.next_delay
    )
);
```

### 9. "I need to accumulate logs/metrics without threading state everywhere"

```rust
use stillwater::effect::writer::prelude::*;
use stillwater::effect::prelude::*;

// Without Writer: manually threading state
fn process(x: i32, logs: &mut Vec<String>) -> Result<i32, Error> {
    logs.push("Starting".into());
    let y = step1(x, logs)?;
    logs.push(format!("Step 1: {}", y));
    Ok(y)
}

// With Writer Effect: automatic accumulation
fn process_with_writer(x: i32) -> impl WriterEffect<
    Output = i32, Error = String, Env = (), Writes = Vec<String>
> {
    tell_one::<_, String, ()>("Starting".to_string())
        .and_then(move |_| into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(x * 2)))
        .tap_tell(|y| vec![format!("Step 1: {}", y)])
}

// Run and get both result and accumulated logs
let (result, logs) = process_with_writer(21).run_writer(&()).await;
assert_eq!(result, Ok(42));
assert_eq!(logs, vec!["Starting", "Step 1: 42"]);

// Use any Monoid for accumulation - not just Vec!
use stillwater::monoid::Sum;

// Count operations
let effect = tell::<Sum<u32>, String, ()>(Sum(1))
    .and_then(|_| tell(Sum(1)))
    .and_then(|_| tell(Sum(1)));

let (_, Sum(count)) = effect.run_writer(&()).await;
assert_eq!(count, 3);
```

### 10. "I want the type system to prevent resource leaks"

```rust
use stillwater::effect::resource::*;

// Mark effects with resource acquisition at the TYPE level
fn open_file(path: &str) -> impl ResourceEffect<Acquires = Has<FileRes>> {
    pure(FileHandle::new(path)).acquires::<FileRes>()
}

fn close_file(handle: FileHandle) -> impl ResourceEffect<Releases = Has<FileRes>> {
    pure(()).releases::<FileRes>()
}

// The bracket pattern guarantees resource neutrality
// Use the builder for ergonomic syntax (single type parameter)
fn read_file_safe(path: &str) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
    bracket::<FileRes>()
        .acquire(open_file(path))
        .release(|h| async move { close_file(h).run(&()).await })
        .use_fn(|h| read_contents(h))
}

// Transaction protocols enforced at compile time
fn begin_tx() -> impl ResourceEffect<Acquires = Has<TxRes>> { /* ... */ }
fn commit(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { /* ... */ }

// This function MUST be resource-neutral or it won't compile
fn transfer_funds() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
    bracket::<TxRes>()
        .acquire(begin_tx())
        .release(|tx| async move { commit(tx).run(&()).await })
        .use_fn(|tx| execute_queries(tx))
}
// Zero runtime overhead - all tracking is compile-time only!
```

## Core Features

- **`Validation<T, E>`** - Accumulate all errors instead of short-circuiting
- **Predicate combinators** - Composable validation logic with `and`, `or`, `not`, `all_of`, `any_of`
  - String predicates: `len_between`, `contains`, `starts_with`, `all_chars`, etc.
  - Number predicates: `between`, `gt`, `lt`, `positive`, `negative`, etc.
  - Collection predicates: `all`, `any`, `has_len`, `is_empty`, etc.
  - Seamless integration with `Validation` via `ensure()` and `validate()`
- **Validation combinators** - Declarative validation with `ensure` family (replaces verbose `and_then` boilerplate)
  - `Effect`: `.ensure()`, `.ensure_with()`, `.ensure_pred()`, `.unless()`, `.filter_or()`
  - `Validation`: `.ensure()`, `.ensure_fn()`, `.ensure_with()`, `.ensure_fn_with()`, `.unless()`, `.filter_or()`
  - Zero-cost: compiles to concrete types with no heap allocation
  - Reduces 12-line validation blocks to single-line predicates
- **Refined types** - "Parse, don't validate" pattern for type-level invariants
  - `Refined<T, P>` wrapper guarantees value satisfies predicate P at compile time
  - Numeric predicates: `Positive`, `NonNegative`, `Negative`, `NonZero`, `InRange<MIN, MAX>`
  - String predicates: `NonEmpty`, `Trimmed`, `MaxLength<N>`, `MinLength<N>`
  - Collection predicates: `NonEmpty`, `MaxSize<N>`, `MinSize<N>` for `Vec<T>`
  - Combinators: `And`, `Or`, `Not` for composing complex predicates
  - Type aliases: `NonEmptyString`, `PositiveI32`, `Port`, `Percentage`, etc.
  - Validation integration: `validate()`, `validate_vec()`, `with_field()` for error accumulation
  - Zero-cost: same memory layout as inner type, predicate is compile-time only
- **`NonEmptyVec<T>`** - Type-safe non-empty collections with guaranteed head element
- **`Effect` trait** - Zero-cost effect composition following the `futures` crate pattern
  - Zero heap allocations by default
  - Explicit `.boxed()` when type erasure is needed
  - Returns `impl Effect` for optimal performance
- **Zip combinators** - Combine independent effects into tuples
  - `zip()`, `zip_with()` methods for pairwise combination
  - `zip3()` through `zip8()` for flat tuple results
  - Zero-cost: all combinators return concrete types
- **Parallel effect execution** - Run independent effects concurrently
  - Zero-cost: `par2()`, `par3()`, `par4()` for heterogeneous effects
  - Boxed: `par_all()`, `par_try_all()`, `race()`, `par_all_limit()` for homogeneous collections
- **Retry and resilience** - Policy-as-data approach with exponential, linear, constant, and Fibonacci backoff. Includes jitter, conditional retry, retry hooks, and timeout support
- **Error recovery** - Selective error handling with predicate-based recovery
  - `recover()`, `recover_with()`, `recover_some()` for conditional error recovery
  - `fallback()`, `fallback_to()` for default values and alternative effects
  - Predicate composition for sophisticated recovery strategies
  - Real-world patterns: multi-tier caching, graceful degradation, API fallback
- **Resource management** - Comprehensive bracket pattern for safe acquire/use/release
  - `bracket()`, `bracket2()`, `bracket3()` for single and multiple resources with LIFO cleanup
  - `bracket_full()` returns `BracketError` with explicit error handling for all failure modes
  - `acquiring()` builder for fluent multi-resource management with `with_flat2/3/4`
  - Guaranteed cleanup even on errors, partial acquisition rollback
- **Compile-time resource tracking** - Type-level resource safety with zero runtime overhead
  - Resource markers: `FileRes`, `DbRes`, `LockRes`, `TxRes`, `SocketRes` (or define custom)
  - `ResourceEffect` trait with `Acquires`/`Releases` associated types
  - Extension methods: `.acquires::<R>()`, `.releases::<R>()`, `.neutral()`
  - `bracket::<R>()` builder for ergonomic resource brackets (single type parameter)
  - `resource_bracket` function for guaranteed resource-neutral operations
  - `assert_resource_neutral` for compile-time leak detection
- **Traverse and sequence** - Transform collections with `traverse()` and `sequence()` for both validations and effects
- **Reader pattern helpers** - Clean dependency injection with `ask()`, `asks()`, and `local()`
- **Writer Effect** - Accumulate logs, metrics, or audit trails alongside computation
  - `tell()`, `tell_one()` for emitting values to accumulator
  - `tap_tell()` for logging derived values after success
  - `censor()` for filtering/transforming accumulated writes
  - `listen()`, `pass()` for introspecting and controlling writes
  - `traverse_writer()`, `fold_writer()` for collection operations
  - Works with any `Monoid`: `Vec`, `Sum`, `Product`, custom types
- **`Semigroup` trait** - Associative combination of values
  - Extended implementations for `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Option`
  - Wrapper types: `First`, `Last`, `Intersection` for alternative semantics
- **`Monoid` trait** - Identity elements for powerful composition patterns
- **Testing utilities** - Ergonomic test helpers
  - `MockEnv` builder for composing test environments
  - Assertion macros: `assert_success!`, `assert_failure!`, `assert_validation_errors!`
  - `TestEffect` wrapper for deterministic effect testing
  - Optional `proptest` feature for property-based testing
- **Context chaining** - Never lose error context
- **Tracing integration** - Instrument effects with semantic spans using the standard `tracing` crate
- **Zero-cost abstractions** - Follows `futures` crate pattern: concrete types, no allocation by default
- **Works with `?` operator** - Integrates with Rust idioms
- **No heavy macros** - Clear types, obvious behavior

## Quick Start

```rust
use stillwater::prelude::*;

// 1. Validation with error accumulation
fn validate_user(input: UserInput) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(&input.email),
        validate_age(input.age),
        validate_name(&input.name),
    ))
    .map(|(email, age, name)| User { email, age, name })
}

// 2. Effect composition (zero-cost by default)
fn create_user(input: UserInput) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    // Validate (pure, accumulates errors)
    from_validation(validate_user(input).map_err(AppError::Validation))
        // Check if exists (I/O)
        .and_then(|user| {
            from_fn(move |env: &AppEnv| env.db.find_by_email(&user.email))
                .and_then(move |existing| {
                    if existing.is_some() {
                        fail(AppError::EmailExists)
                    } else {
                        pure(user)
                    }
                })
        })
        // Save user (I/O)
        .and_then(|user| {
            from_fn(move |env: &AppEnv| env.db.insert_user(&user))
                .map(move |_| user)
        })
        .context("Creating new user")
}

// 3. Run at application boundary
let env = AppEnv { db, cache, logger };
let result = create_user(input).run(&env).await?;
```

## Zero-Cost Effect System

Version 0.11.0 introduces a zero-cost effect system following the `futures` crate pattern:

```rust
// Free-standing constructors (not methods)
let effect = pure(42);                    // Not Effect::pure(42)
let effect = fail("error");               // Not Effect::fail("error")
let effect = from_fn(|env| Ok(env.value)); // Not Effect::from_fn(...)

// Chain combinators - each returns a concrete type, zero allocations
let result = pure(1)
    .map(|x| x + 1)
    .and_then(|x| pure(x * 2))
    .map(|x| x.to_string());

// Use .boxed() when you need type erasure
fn dynamic_effect(flag: bool) -> BoxedEffect<i32, String, ()> {
    if flag {
        pure(1).boxed()
    } else {
        pure(2).boxed()
    }
}

// Collections of effects require boxing
let effects: Vec<BoxedEffect<i32, String, Env>> = vec![
    effect1.boxed(),
    effect2.boxed(),
];
let results = par_all(effects, &env).await;
```

**When to use `.boxed()`:**
- Storing effects in collections (`Vec<BoxedEffect<...>>`)
- Returning different effect types from branches
- Recursive effect definitions
- Dynamic dispatch scenarios

**When NOT to use `.boxed()`:**
- Simple linear chains (use `impl Effect`)
- Fixed combinator sequences
- Performance-critical paths

## Why Stillwater?

### Compared to existing solutions:

**vs. frunk:**
- Focused on practical use cases, not type-level programming
- Better documentation and examples
- Effect composition, not just validation

**vs. monadic:**
- No awkward macro syntax (`rdrdo! { ... }`)
- Zero-cost by default (follows `futures` crate pattern)
- Idiomatic Rust, not Haskell port

**vs. hand-rolling:**
- Validation accumulation built-in
- Error context handling
- Testability patterns established
- Composable, reusable

### What makes it "Rust-first":

- No attempt at full monad abstraction (impossible without HKTs)
- Works with `?` operator via `Try` trait
- Zero-cost via concrete types and monomorphization (like `futures`)
- Integrates with async/await
- Borrows checker friendly
- Clear error messages

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stillwater = "1.0"

# Optional: async support
stillwater = { version = "0.11", features = ["async"] }

# Optional: tracing integration
stillwater = { version = "0.11", features = ["tracing"] }

# Optional: jitter for retry policies
stillwater = { version = "0.11", features = ["jitter"] }

# Optional: property-based testing
stillwater = { version = "0.11", features = ["proptest"] }

# Multiple features
stillwater = { version = "0.11", features = ["async", "tracing", "jitter"] }
```

## Examples

Run any example with `cargo run --example <name>`:

| Example | Demonstrates |
|---------|--------------|
| [predicates](examples/predicates.rs) | Composable predicate combinators for validation logic |
| [form_validation](examples/form_validation.rs) | Validation error accumulation |
| [homogeneous_validation](examples/homogeneous_validation.rs) | Type-safe validation for discriminated unions before combining |
| [nonempty](examples/nonempty.rs) | NonEmptyVec type for guaranteed non-empty collections |
| [user_registration](examples/user_registration.rs) | Effect composition and I/O separation |
| [error_context](examples/error_context.rs) | Error trails for debugging |
| [data_pipeline](examples/data_pipeline.rs) | Real-world ETL pipeline |
| [testing_patterns](examples/testing_patterns.rs) | Testing pure vs effectful code |
| [reader_pattern](examples/reader_pattern.rs) | Reader pattern with ask(), asks(), and local() |
| [writer_logging](examples/writer_logging.rs) | Writer Effect for accumulating logs, metrics, and audit trails |
| [validation](examples/validation.rs) | Validation type and error accumulation patterns |
| [effects](examples/effects.rs) | Effect type and composition patterns |
| [parallel_effects](examples/parallel_effects.rs) | Parallel execution with par_all, race, and par_all_limit |
| [recover_patterns](examples/recover_patterns.rs) | Error recovery with recover, recover_with, recover_some, fallback patterns |
| [retry_patterns](examples/retry_patterns.rs) | Retry policies, backoff strategies, timeouts, and resilience patterns |
| [io_patterns](examples/io_patterns.rs) | IO module helpers for reading/writing |
| [pipeline](examples/pipeline.rs) | Data transformation pipelines |
| [traverse](examples/traverse.rs) | Traverse and sequence for collections of validations and effects |
| [monoid](examples/monoid.rs) | Monoid and Semigroup traits for composition |
| [extended_semigroup](examples/extended_semigroup.rs) | Semigroup for HashMap, HashSet, Option, and wrapper types |
| [tracing_demo](examples/tracing_demo.rs) | Tracing integration with semantic spans and context |
| [boxing_decisions](examples/boxing_decisions.rs) | When to use `.boxed()` vs zero-cost effects |
| [resource_scopes](examples/resource_scopes.rs) | Bracket pattern for safe resource management with guaranteed cleanup |
| [resource_tracking](examples/resource_tracking.rs) | Compile-time resource tracking with type-level safety |
| [refined](examples/refined.rs) | Refined types for "parse, don't validate" pattern with type-level invariants |

See [examples/](examples/) directory for full code.

## Production Readiness

**Status: 0.11.0 - Production Ready**

- 355 unit tests passing
- 113 documentation tests passing
- 21 runnable examples
- Zero clippy warnings
- Full async support
- CI/CD pipeline with security audits

This library is stable and ready for use. The 0.x version indicates the API may evolve based on community feedback.

## Migration from 0.10.x

Version 0.11.0 introduces breaking changes with the zero-cost effect system. See [MIGRATION.md](docs/MIGRATION.md) for detailed upgrade instructions.

**Key changes:**
```rust
// Before (0.10.x)
Effect::pure(x)
Effect::fail(e)
Effect::from_fn(f)

// After (0.11.0)
pure(x)
fail(e)
from_fn(f)

// Return types changed
fn old() -> Effect<T, E, Env> { ... }      // Boxed by default
fn new() -> impl Effect<...> { ... }        // Zero-cost by default
fn boxed() -> BoxedEffect<T, E, Env> { ... } // Explicit boxing
```

## Documentation

- [User Guide](docs/guide/README.md) - Comprehensive tutorials
- [API Docs](https://docs.rs/stillwater) - Full API reference
- [FAQ](docs/FAQ.md) - Common questions
- [Design](DESIGN.md) - Architecture and decisions
- [Philosophy](PHILOSOPHY.md) - Core principles
- [Patterns](docs/PATTERNS.md) - Common patterns and recipes
- [Comparison](docs/COMPARISON.md) - vs other libraries
- [Migration Guide](docs/MIGRATION.md) - Upgrading from 0.10.x to 0.11.0

## Migrating from Result

Already using `Result` everywhere? No problem! Stillwater integrates seamlessly:

```rust
// Your existing code works as-is
fn validate_email(email: &str) -> Result<Email, Error> {
    // ...
}

// Upgrade to accumulation when you need it
fn validate_form(input: FormInput) -> Validation<Form, Vec<Error>> {
    Validation::all((
        Validation::from_result(validate_email(&input.email)),
        Validation::from_result(validate_age(input.age)),
    ))
}

// Convert back to Result when needed
let result: Result<Form, Vec<Error>> = validation.into_result();
```

Start small, adopt progressively. Use `Validation` only where you need error accumulation.

## Contributing

Contributions welcome! This is a young library with room to grow:
- Bug reports and feature requests via [issues](https://github.com/iepathos/stillwater/issues)
- Documentation improvements
- More examples and use cases
- API feedback and design discussions

Before submitting PRs, please open an issue to discuss the change.

## Ecosystem

Stillwater is part of a family of libraries that share the same functional programming philosophy:

| Library | Description |
|---------|-------------|
| [premortem](https://github.com/iepathos/premortem) | Configuration validation that finds all errors before your app runs. Multi-source loading with error accumulation and value origin tracing. |
| [postmortem](https://github.com/iepathos/postmortem) | Validation library that accumulates all errors with precise JSON path tracking. Composable schemas, cross-field validation, and effect integration. |
| [mindset](https://github.com/iepathos/mindset) | Zero-cost, effect-based state machines. Pure guards for validation, explicit actions for side effects, environment pattern for testability. |

All libraries emphasize:
- Error accumulation over short-circuiting
- Pure core, effects at the boundaries
- Zero-cost abstractions
- Testability through dependency injection

## License

MIT © Glen Baker <iepathos@gmail.com>

---

*"Like a still pond with water flowing through it, stillwater keeps your pure business logic calm and testable while effects flow at the boundaries."*
