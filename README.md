# Stillwater

[![CI](https://github.com/iepathos/stillwater/actions/workflows/ci.yml/badge.svg)](https://github.com/iepathos/stillwater/actions/workflows/ci.yml)
[![Coverage](https://github.com/iepathos/stillwater/actions/workflows/coverage.yml/badge.svg)](https://github.com/iepathos/stillwater/actions/workflows/coverage.yml)
[![Security](https://github.com/iepathos/stillwater/actions/workflows/security.yml/badge.svg)](https://github.com/iepathos/stillwater/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/stillwater)](https://crates.io/crates/stillwater)
[![License](https://img.shields.io/badge/license-MIT)](LICENSE)

> *"Still waters run pure"*

A Rust library for pragmatic effect composition and validation, emphasizing the **pure core, imperative shell** pattern.

## Philosophy

**Stillwater** embodies a simple idea:
- **Still** = Pure functions (unchanging, referentially transparent)
- **Water** = Effects (flowing, performing I/O)

Keep your business logic pure and calm like still water. Let effects flow at the boundaries.

## What Problems Does It Solve?

### 1. "I want ALL validation errors, not just the first one"

```rust
use stillwater::Validation;

// Standard Result: stops at first error ❌
let email = validate_email(input)?;  // Stops here
let age = validate_age(input)?;      // Never reached if email fails

// Stillwater: accumulates all errors ✓
let user = Validation::all((
    validate_email(input),
    validate_age(input),
    validate_name(input),
))?;
// Returns: Err(vec![EmailError, AgeError, NameError])
```

### 2. "How do I test code with database calls?"

```rust
use stillwater::Effect;

// Pure business logic (no DB, easy to test)
fn calculate_discount(customer: &Customer, total: Money) -> Money {
    match customer.tier {
        Tier::Gold => total * 0.15,
        _ => total * 0.05,
    }
}

// Effects at boundaries (mockable)
fn process_order(id: OrderId) -> Effect<Invoice, Error, AppEnv> {
    IO::query(|db| db.fetch_order(id))        // I/O
        .and_then(|order| {
            let total = calculate_total(&order);  // Pure!
            IO::query(|db| db.fetch_customer(order.customer_id))
                .map(move |customer| (order, customer, total))
        })
        .map(|(order, customer, total)| {
            let discount = calculate_discount(&customer, total);  // Pure!
            create_invoice(order.id, total - discount)            // Pure!
        })
        .and_then(|invoice| IO::execute(|db| db.save(invoice))) // I/O
}

// Test with mock environment
#[test]
fn test_with_mock_db() {
    let env = MockEnv::new();
    let result = process_order(id).run(&env)?;
    assert_eq!(result.total, expected);
}
```

### 3. "My errors lose context as they bubble up"

```rust
use stillwater::Effect;

fetch_user(id)
    .context("Loading user profile")
    .and_then(|user| process_data(user))
    .context("Processing user data")
    .run(&env)?;

// Error output:
// Error: UserNotFound(12345)
//   -> Loading user profile
//   -> Processing user data
```

## Core Features

- **`Validation<T, E>`** - Accumulate all errors instead of short-circuiting
- **`Effect<T, E, Env>`** - Separate pure logic from I/O effects
- **Context chaining** - Never lose error context
- **Zero-cost abstractions** - Compiles to same code as hand-written
- **Works with `?` operator** - Integrates with Rust idioms
- **No heavy macros** - Clear types, obvious behavior

## Quick Start

```rust
use stillwater::{Validation, Effect, IO};

// 1. Validation with error accumulation
fn validate_user(input: UserInput) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(&input.email),
        validate_age(input.age),
        validate_name(&input.name),
    ))
    .map(|(email, age, name)| User { email, age, name })
}

// 2. Effect composition
fn create_user(input: UserInput) -> Effect<User, AppError, AppEnv> {
    // Validate (pure, accumulates errors)
    Effect::from_validation(validate_user(input))
        // Check if exists (I/O)
        .and_then(|user| {
            IO::query(|db| db.find_by_email(&user.email))
                .and_then(|existing| {
                    if existing.is_some() {
                        Effect::fail(AppError::EmailExists)
                    } else {
                        Effect::pure(user)
                    }
                })
        })
        // Save user (I/O)
        .and_then(|user| {
            IO::execute(|db| db.insert_user(&user))
                .map(|_| user)
        })
        .context("Creating new user")
}

// 3. Run at application boundary
let env = AppEnv { db, cache, logger };
let result = create_user(input).run(&env)?;
```

## Why Stillwater?

### Compared to existing solutions:

**vs. frunk:**
- ✓ Focused on practical use cases, not type-level programming
- ✓ Better documentation and examples
- ✓ Effect composition, not just validation

**vs. monadic:**
- ✓ No awkward macro syntax (`rdrdo! { ... }`)
- ✓ Zero-cost (no boxing by default)
- ✓ Idiomatic Rust, not Haskell port

**vs. hand-rolling:**
- ✓ Validation accumulation built-in
- ✓ Error context handling
- ✓ Testability patterns established
- ✓ Composable, reusable

### What makes it "Rust-first":

- ❌ No attempt at full monad abstraction (impossible without HKTs)
- ✓ Works with `?` operator via `Try` trait
- ✓ Zero-cost via generics and monomorphization
- ✓ Integrates with async/await
- ✓ Borrows checker friendly
- ✓ Clear error messages

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
stillwater = "0.1"

# Optional: async support
stillwater = { version = "0.1", features = ["async"] }
```

## Examples

Stillwater provides comprehensive examples organized into two categories:

### Tutorial Examples

Progressive introduction to core concepts:

- **validation**: Validation type and error accumulation patterns
- **effects**: Effect type and composition patterns
- **io_patterns**: IO module helpers for reading/writing
- **pipeline**: Data transformation pipelines

### Real-World Examples

Complete use cases demonstrating practical applications:

- **form_validation**: Multi-field form validation with error accumulation
- **user_registration**: User registration workflow with multiple services
- **error_context**: Error trails for better debugging
- **data_pipeline**: ETL pipeline with Validation
- **testing_patterns**: Testing pure vs effectful code

Run any example with `cargo run --example <name>`.

See [examples/](examples/) directory for full code.

## Production Readiness

**Status: 0.1 - Production Ready for Early Adopters**

- ✅ 111 unit tests passing
- ✅ 58 documentation tests passing
- ✅ Zero clippy warnings
- ✅ Comprehensive examples
- ✅ Full async support
- ✅ CI/CD pipeline with security audits

This library is stable and ready for use. The 0.x version indicates the API may evolve based on community feedback.

## Design Philosophy

See [DESIGN.md](./DESIGN.md) for detailed design decisions, patterns, and architecture.

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
- 🐛 Bug reports and feature requests via [issues](https://github.com/iepathos/stillwater/issues)
- 📖 Documentation improvements
- 🧪 More examples and use cases
- 💡 API feedback and design discussions

Before submitting PRs, please open an issue to discuss the change.

## License

MIT © Glen Baker <iepathos@gmail.com>

---

*"Like a still pond with water flowing through it, stillwater keeps your pure business logic calm and testable while effects flow at the boundaries."*
