---
number: 010
title: Documentation Structure and Guides
category: documentation
priority: high
status: draft
dependencies: [001, 002, 003, 004, 005, 006, 008, 009]
created: 2025-11-21
---

# Specification 010: Documentation Structure and Guides

**Category**: documentation
**Priority**: high
**Status**: draft
**Dependencies**: All core specs (001-006), Spec 008 (project structure), Spec 009 (examples)

## Context

Great libraries need great documentation. Users discover libraries through documentation, learn patterns through guides, and reference APIs through rustdoc.

Stillwater needs comprehensive documentation covering:
1. **Getting started** - Quick examples to hook users
2. **Core concepts** - Philosophy and mental models
3. **API reference** - Generated from rustdoc
4. **Guides** - Deep dives into each feature
5. **Architecture** - Design decisions and internals

This spec defines the complete documentation structure and content for MVP.

## Objective

Create comprehensive, well-organized documentation that enables users to understand, adopt, and master Stillwater quickly and confidently.

## Requirements

### Functional Requirements

- README.md with quick start and examples
- User guides for each core feature
- Rustdoc comments for all public APIs
- Architecture documentation (DESIGN.md, PHILOSOPHY.md)
- Contributing guide (CONTRIBUTING.md)
- Changelog (CHANGELOG.md)
- FAQ document
- Migration guide (for future versions)
- Documentation tests (all code examples compile)

### Non-Functional Requirements

- Clear, beginner-friendly language
- Progressive disclosure (simple → advanced)
- Runnable code examples
- Consistent formatting and style
- Searchable (via docs.rs)
- Fast to navigate
- Mobile-friendly

## Acceptance Criteria

- [ ] README.md with quick start, examples, and links
- [ ] docs/guide/01-semigroup.md comprehensive guide
- [ ] docs/guide/02-validation.md comprehensive guide
- [ ] docs/guide/03-effects.md comprehensive guide
- [ ] docs/guide/04-error-context.md comprehensive guide
- [ ] docs/guide/05-io-module.md comprehensive guide
- [ ] docs/guide/06-helper-combinators.md comprehensive guide
- [ ] docs/guide/07-try-trait.md comprehensive guide
- [ ] docs/FAQ.md with common questions
- [ ] All rustdoc examples compile and run
- [ ] Documentation builds without warnings
- [ ] Links between docs work correctly
- [ ] Code examples follow consistent style

## Technical Details

### Documentation Structure

```
stillwater/
├── README.md                    # Project overview, quick start
├── PHILOSOPHY.md                # Core philosophy and mental models
├── DESIGN.md                    # Architecture and design decisions
├── CONTRIBUTING.md              # How to contribute (from Spec 008)
├── CHANGELOG.md                 # Version history
├── docs/
│   ├── guide/
│   │   ├── README.md            # Guide index
│   │   ├── 01-semigroup.md      # Semigroup trait guide
│   │   ├── 02-validation.md     # Validation guide
│   │   ├── 03-effects.md        # Effect guide
│   │   ├── 04-error-context.md  # Context errors guide
│   │   ├── 05-io-module.md      # IO helpers guide
│   │   ├── 06-helper-combinators.md  # Combinators guide
│   │   └── 07-try-trait.md      # Try trait guide
│   ├── FAQ.md                   # Frequently asked questions
│   ├── PATTERNS.md              # Common patterns and recipes
│   └── COMPARISON.md            # vs other libraries
└── src/
    └── lib.rs                   # Rustdoc with examples
```

### README.md Template

```markdown
# Stillwater

> Pragmatic functional effects for Rust: validation accumulation and effect composition

[![Crates.io](https://img.shields.io/crates/v/stillwater.svg)](https://crates.io/crates/stillwater)
[![Documentation](https://docs.rs/stillwater/badge.svg)](https://docs.rs/stillwater)
[![CI](https://github.com/yourusername/stillwater/workflows/CI/badge.svg)](https://github.com/yourusername/stillwater/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## What is Stillwater?

Stillwater provides composable abstractions for common functional programming patterns in Rust:

- **Validation**: Accumulate ALL errors instead of failing on the first
- **Effect**: Separate pure business logic from I/O side effects
- **Context**: Preserve error trails for better debugging

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
stillwater = "0.1"
tokio = { version = "1", features = ["full"] }  # For async examples
```

### Validation Example

```rust
use stillwater::prelude::*;

// Validate all fields and collect ALL errors
fn validate_user(email: &str, age: u8) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(email),
        validate_age(age),
    ))
    .map(|(email, age)| User { email, age })
}
```

**Key benefit**: Get all validation errors at once, not just the first.

### Effect Example

```rust
use stillwater::prelude::*;

// Separate pure logic from I/O
fn register_user(email: String) -> Effect<User, Error, AppEnv> {
    IO::read(|db: &Database| db.find_by_email(&email))
        .and_then(|existing| {
            if existing.is_some() {
                Effect::fail(Error::EmailExists)
            } else {
                create_and_save_user(email)
            }
        })
}
```

**Key benefit**: Pure business logic is trivial to test (no mocks needed).

## Core Concepts

### The Problem

**Standard validation fails fast**:
```rust
// ❌ Only reports first error
fn validate(data: Data) -> Result<Valid, Error> {
    let email = validate_email(data.email)?;  // Stops here on failure
    let age = validate_age(data.age)?;        // Never reached
    let pwd = validate_password(data.pwd)?;   // Never reached
    Ok(Valid { email, age, pwd })
}
```

**Stillwater accumulates all errors**:
```rust
// ✓ Reports all 3 errors at once
fn validate(data: Data) -> Validation<Valid, Vec<Error>> {
    Validation::all((
        validate_email(data.email),
        validate_age(data.age),
        validate_password(data.pwd),
    ))
    .map(|(email, age, pwd)| Valid { email, age, pwd })
}
```

### Philosophy: Pure Core, Imperative Shell

Stillwater helps you structure applications as:

- **Pure core**: Business logic with no side effects (easy to test)
- **Imperative shell**: I/O operations at the boundaries (controlled)

See [PHILOSOPHY.md](PHILOSOPHY.md) for details.

## Features

- ✅ **Validation with error accumulation** via Semigroup
- ✅ **Effect composition** for I/O separation
- ✅ **Error context trails** for better debugging
- ✅ **IO helpers** for ergonomic effect creation
- ✅ **Helper combinators** for common patterns
- ✅ **Try trait support** (nightly, optional)
- ✅ **Zero dependencies** (core library)
- ✅ **Fully documented** with examples

## Examples

Run any example with `cargo run --example <name>`:

| Example | Demonstrates |
|---------|--------------|
| [form_validation](examples/form_validation.rs) | Validation error accumulation |
| [user_registration](examples/user_registration.rs) | Effect composition and I/O separation |
| [error_context](examples/error_context.rs) | Error trails for debugging |
| [data_pipeline](examples/data_pipeline.rs) | Real-world ETL pipeline |
| [testing_patterns](examples/testing_patterns.rs) | Testing pure vs effectful code |

## Documentation

- 📚 [User Guide](docs/guide/README.md) - Comprehensive tutorials
- 📖 [API Docs](https://docs.rs/stillwater) - Full API reference
- 🤔 [FAQ](docs/FAQ.md) - Common questions
- 🏛️ [Design](DESIGN.md) - Architecture and decisions
- 💭 [Philosophy](PHILOSOPHY.md) - Core principles

## Comparison to Other Libraries

| Library | Validation | Effects | Async | Learning Curve |
|---------|-----------|---------|-------|----------------|
| Stillwater | ✓ Accumulation | ✓ Composition | ✓ Native | Low |
| frunk | ✓ Basic | ✗ | ✗ | High (HLists) |
| monadic | ✗ | ✓ Basic | ✗ | Medium (macros) |
| anyhow | ✗ | ✗ | ✓ | Low |
| Hand-rolled | ✗ | ✗ | ✓ | N/A |

See [COMPARISON.md](docs/COMPARISON.md) for detailed comparison.

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Inspired by functional programming patterns from Haskell, Scala ZIO, and F# computation expressions.
```

### Guide Template (docs/guide/02-validation.md)

```markdown
# Validation with Error Accumulation

## The Problem

Standard `Result` types short-circuit on the first error:

```rust
fn validate_form(data: FormData) -> Result<ValidForm, Error> {
    let email = validate_email(data.email)?;     // ❌ Stops here if invalid
    let password = validate_password(data.pwd)?; // Never reached
    let age = validate_age(data.age)?;           // Never reached

    Ok(ValidForm { email, password, age })
}
```

If the email is invalid, the user doesn't learn about password or age errors. They have to submit the form multiple times, fixing one error at a time. Frustrating!

## The Solution: Validation

Stillwater's `Validation` type accumulates ALL errors:

```rust
use stillwater::prelude::*;

fn validate_form(data: FormData) -> Validation<ValidForm, Vec<Error>> {
    Validation::all((
        validate_email(data.email),
        validate_password(data.pwd),
        validate_age(data.age),
    ))
    .map(|(email, password, age)| ValidForm { email, password, age })
}
```

Now all three validations run, and the user sees all errors at once!

## Core API

### Creating Validations

```rust
// Success
let v = Validation::success(42);

// Failure
let v = Validation::failure(vec!["error"]);

// From Result
let v = Validation::from_result(Ok(42));
let v = Validation::from_result(Err("error"));
```

### Combining Validations

```rust
// Combine with tuples (up to 12 items)
let result = Validation::all((
    validate_email(email),
    validate_password(password),
    validate_age(age),
));

// Combine Vec of same type
let result = Validation::all_vec(vec![
    validate_item(item1),
    validate_item(item2),
    validate_item(item3),
]);
```

### Transforming Validations

```rust
// Transform success value
validation.map(|x| x * 2)

// Transform error value
validation.map_err(|e| format!("Error: {}", e))

// Chain dependent validation
validation.and_then(|x| validate_next(x))
```

## Error Accumulation with Semigroup

For `Validation::all()` to work, your error type must implement `Semigroup`:

```rust
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}
```

Common implementations:
- `Vec<T>`: Concatenate vectors
- `String`: Concatenate strings
- `(A, B) where A: Semigroup, B: Semigroup`: Combine components

Example:
```rust
impl Semigroup for Vec<ValidationError> {
    fn combine(mut self, mut other: Self) -> Self {
        self.append(&mut other);
        self
    }
}
```

## Real-World Example

```rust
#[derive(Debug)]
enum ValidationError {
    InvalidEmail(String),
    PasswordTooShort { min: usize, actual: usize },
    AgeTooYoung { min: u8, actual: u8 },
}

fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec![
            ValidationError::InvalidEmail(email.to_string())
        ])
    }
}

fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
    const MIN_LEN: usize = 8;
    if pwd.len() >= MIN_LEN {
        Validation::success(pwd.to_string())
    } else {
        Validation::failure(vec![
            ValidationError::PasswordTooShort {
                min: MIN_LEN,
                actual: pwd.len(),
            }
        ])
    }
}

fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
    const MIN_AGE: u8 = 18;
    if age >= MIN_AGE {
        Validation::success(age)
    } else {
        Validation::failure(vec![
            ValidationError::AgeTooYoung {
                min: MIN_AGE,
                actual: age,
            }
        ])
    }
}

fn validate_registration(
    email: &str,
    password: &str,
    age: u8,
) -> Validation<User, Vec<ValidationError>> {
    Validation::all((
        validate_email(email),
        validate_password(password),
        validate_age(age),
    ))
    .map(|(email, password, age)| User { email, password, age })
}

// Usage
match validate_registration("invalid", "short", 15) {
    Validation::Success(user) => println!("✓ Registered: {:?}", user),
    Validation::Failure(errors) => {
        println!("✗ {} errors:", errors.len());
        for err in errors {
            println!("  - {:?}", err);
        }
    }
}

// Output:
// ✗ 3 errors:
//   - InvalidEmail("invalid")
//   - PasswordTooShort { min: 8, actual: 5 }
//   - AgeTooYoung { min: 18, actual: 15 }
```

## When to Use Validation

**Use Validation when**:
- Validating user input (forms, APIs)
- You want to report ALL errors at once
- Validations are independent (order doesn't matter)

**Use Result when**:
- Operations depend on previous results
- Short-circuit is desired (fail fast)
- Single error is sufficient

## Patterns

### Independent Field Validation

```rust
// All fields validated independently
Validation::all((
    validate_email(data.email),
    validate_phone(data.phone),
    validate_address(data.address),
))
```

### Dependent Validation

```rust
// First validate, then check dependencies
validate_email(email)
    .and_then(|email| {
        check_email_not_taken(email)
    })
```

### Mixed Validation

```rust
// Combine independent and dependent
Validation::all((
    validate_email(email),
    validate_password(password),
))
.and_then(|(email, password)| {
    // Now check if combination is valid
    check_credentials_not_weak(email, password)
})
```

## Testing

Validation is pure - testing is trivial:

```rust
#[test]
fn test_valid_email() {
    let result = validate_email("user@example.com");
    assert!(result.is_success());
}

#[test]
fn test_invalid_email() {
    let result = validate_email("invalid");
    assert!(result.is_failure());
}

#[test]
fn test_accumulation() {
    let result = validate_registration("bad", "short", 15);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 3);
        }
        _ => panic!("Expected failure"),
    }
}
```

## Next Steps

- Learn about [Effect composition](03-effects.md)
- See [full example](../../examples/form_validation.rs)
- Read the [API docs](https://docs.rs/stillwater)
```

### docs/FAQ.md Template

```markdown
# Frequently Asked Questions

## General

### What is Stillwater?

Stillwater is a library providing pragmatic functional programming abstractions for Rust, focused on validation and effect composition.

### Why "Stillwater"?

"Still" represents pure logic (calm, unchanging). "Water" represents effects (flowing, dynamic). Together: "pure core, imperative shell."

### Is this a monad library?

Sort of. Validation is an Applicative Functor, Effect is a Reader/IO monad. But we focus on practical patterns, not category theory.

### What's the learning curve?

Low. If you understand Result and async/await, you can use Stillwater. Advanced patterns are optional.

## Validation

### Why not just use Result?

Result short-circuits on the first error. Validation accumulates all errors, providing better UX for forms and APIs.

### Can I use ? operator with Validation?

On nightly with `try_trait` feature, yes! But be aware: `?` fails fast (no accumulation). Use `Validation::all()` for accumulation.

### What if I need more than 12 validations?

Use `Validation::all_vec()` for homogeneous collections, or nest tuples: `Validation::all((all1, all2, all3))`.

### How do I convert Validation to Result?

```rust
validation.into_result()
```

## Effects

### Why Effect instead of just async fn?

Effect separates pure logic from I/O, making code more testable and composable. Pure functions need zero mocks!

### Do I need tokio?

For async Effects, yes (or async-std). For sync-only code, no runtime needed.

### Can I use Effect with sync code?

Yes! Use `Effect::from_fn()` for sync operations. They'll be wrapped in ready futures.

### How do I test Effects?

Create simple mock environments (just data structures). See [testing_patterns example](../examples/testing_patterns.rs).

## Error Handling

### Should I always use ContextError?

No. Use it at I/O boundaries and major operation boundaries where context helps debugging. Don't add context to pure functions.

### Can I mix Validation and Effect?

Yes! Validate first (pure), then lift to Effect if validation succeeds:

```rust
let validated = validate_data(data);
Effect::from_validation(validated)
    .and_then(|valid| save_to_db(valid))
```

## Performance

### Is there overhead?

Minimal. Effect boxes one function per creation. Validation is just an enum. Both compile to efficient code.

### Can I use no_std?

Not in MVP. Post-MVP we may add no_std support (requires async runtime considerations).

## Comparison

### vs anyhow/thiserror

Those are for error handling. Stillwater is for validation (accumulation) and effect composition (separation). Use together!

### vs frunk

frunk focuses on HLists and type-level programming. Stillwater focuses on practical validation and effects with lower learning curve.

### vs monadic

monadic uses macros for do-notation. Stillwater uses method chaining (more idiomatic Rust).

### vs hand-rolling

Hand-rolling works but requires boilerplate. Stillwater provides tested, composable abstractions.

## Contributing

### How can I help?

See [CONTRIBUTING.md](../CONTRIBUTING.md). We welcome:
- Bug reports
- Documentation improvements
- Examples
- Feature requests

### What's the roadmap?

See [DESIGN.md](../DESIGN.md) for planned features post-MVP.
```

## Dependencies

- **Prerequisites**: All core specs (001-006), Spec 008 (structure), Spec 009 (examples)
- **Affected Components**: Documentation, rustdoc, guides
- **External Dependencies**: None

## Testing Strategy

### Documentation Tests

```rust
// All code examples in rustdoc must compile and run
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctest;

// Test that guide examples compile
#[cfg(test)]
mod guide_tests {
    // Include guide code here
}
```

### Link Validation

```bash
# Check all internal links work
cargo doc --all-features --open
# Manually verify all links

# Check external links
markdown-link-check docs/**/*.md
```

### CI Checks

```yaml
- name: Check documentation
  run: cargo doc --all-features --no-deps
  env:
    RUSTDOCFLAGS: -D warnings

- name: Test documentation examples
  run: cargo test --doc --all-features
```

## Documentation Requirements

### Code Documentation

- Every public item has rustdoc
- Examples for all public methods
- Link to relevant guides
- Explain "why" not just "what"

### User Documentation

- Progressive disclosure (simple → complex)
- Runnable examples
- Clear problem/solution format
- Links between related concepts

### Architecture Updates

- DESIGN.md explains all major decisions
- PHILOSOPHY.md explains core principles
- CHANGELOG.md tracks all changes

## Implementation Notes

### Writing Style

- **Active voice**: "Validation accumulates errors"
- **Second person**: "You can combine validations"
- **Present tense**: "Effect separates I/O from logic"
- **Simple language**: Avoid jargon where possible
- **Examples first**: Show code before explaining

### Code Examples

- Must compile and run
- Show both success and failure
- Include output when helpful
- Keep short (< 30 lines ideal)
- Comment non-obvious parts only

### Cross-Referencing

Link liberally between docs:
```rust
/// See also: [`Validation::all`], [User Guide](https://docs.rs/stillwater/guide)
```

## Migration and Compatibility

No migration - this is initial documentation.

Future versions should:
- Maintain guide structure
- Add migration guides for breaking changes
- Update CHANGELOG.md

## Open Questions

1. Should we use mdbook for guides?
   - Decision: No for MVP, markdown in docs/ is simpler. Consider post-MVP.

2. Should we have video tutorials?
   - Decision: Defer to post-MVP based on user requests

3. Should guides have exercises?
   - Decision: Not for MVP, but good idea for future

4. Should we translate to other languages?
   - Decision: English only for MVP, community translations welcome later
