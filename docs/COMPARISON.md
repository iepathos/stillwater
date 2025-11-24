# Comparison to Other Libraries

This document compares Stillwater to other Rust libraries providing similar functionality.

## Quick Comparison Table

| Feature | Stillwater | frunk | monadic | anyhow | validator |
|---------|-----------|-------|---------|--------|-----------|
| Error accumulation | ✓ | ✓ | ✗ | ✗ | ✗ |
| Effect composition | ✓ | ✗ | ✓ | ✗ | ✗ |
| Async support | ✓ | ✗ | ✗ | ✓ | ✗ |
| Learning curve | Low | High | Medium | Low | Low |
| Type-level programming | ✗ | ✓ | ✗ | ✗ | ✗ |
| Macro DSL | ✗ | ✗ | ✓ | ✗ | ✓ |
| Pure Rust idioms | ✓ | Partial | Partial | ✓ | ✓ |
| Dependencies | 0 (core) | Many | Few | Few | Many |

## vs frunk

**frunk** focuses on type-level functional programming with HLists, Generic, and other advanced concepts.

### Similarities
- Both provide Validation with error accumulation
- Both implement Semigroup

### Differences

**Stillwater**:
- ✓ Practical focus on common patterns
- ✓ Effect composition for I/O separation
- ✓ Lower learning curve
- ✓ Better documentation for beginners
- ✓ Async support

**frunk**:
- ✓ More advanced type-level features (HLists, Generic)
- ✓ Powerful generic programming
- ✗ Steeper learning curve
- ✗ No effect system
- ✗ No async support

### When to use frunk
- Type-level programming is important
- You need HList transformations
- You're comfortable with advanced type theory

### When to use Stillwater
- Validation and effects are your primary needs
- You want a gentler learning curve
- You need async support

## vs monadic

**monadic** provides Reader/Writer/State monads with macro-based do-notation.

### Similarities
- Both provide effect composition
- Both handle dependencies (Reader monad)

### Differences

**Stillwater**:
- ✓ No macro DSL (more idiomatic Rust)
- ✓ Method chaining instead of do-notation
- ✓ Validation with error accumulation
- ✓ Async support
- ✓ Zero dependencies

**monadic**:
- ✓ More monad types (Writer, State)
- ✓ Do-notation via macros
- ✗ Macro-heavy syntax (`rdrdo!`)
- ✗ No validation
- ✗ No async support

### When to use monadic
- You want Haskell-style do-notation
- You need Writer or State monads
- You're porting Haskell code

### When to use Stillwater
- You prefer Rust idioms over Haskell syntax
- You need validation and effects together
- You want async support

## vs anyhow / thiserror

**anyhow** provides ergonomic error handling. **thiserror** provides derive macros for error types.

### Similarities
- All handle errors
- All work with Result

### Differences

**Stillwater**:
- ✓ Error accumulation (Validation)
- ✓ Effect composition
- ✓ ContextError for trails
- ✗ Less focused on error handling alone

**anyhow/thiserror**:
- ✓ Excellent error handling ergonomics
- ✓ Great for error propagation
- ✗ No error accumulation
- ✗ No effect system

### When to use anyhow/thiserror
- Error handling is your only need
- You want minimal boilerplate
- Short-circuiting errors are fine

### When to use Stillwater
- You need error accumulation
- You want effect composition
- You're building validation-heavy apps

**Recommendation**: Use both! Stillwater for business logic, anyhow for error propagation:

```rust
use stillwater::{Validation, Effect};
use anyhow::{Result, Context};

fn validate_and_process(input: Data) -> Result<Output> {
    let validated = Validation::all((
        validate_email(&input.email),
        validate_age(input.age),
    ))
    .into_result()
    .context("validating input")?;

    process(validated)
        .run(&env)
        .await
        .context("processing data")?;

    Ok(output)
}
```

## vs validator

**validator** provides derive macros for common validation rules.

### Similarities
- Both validate data
- Both can accumulate errors

### Differences

**Stillwater**:
- ✓ Functional composition
- ✓ Effect system
- ✓ Custom validation logic
- ✗ No derive macros
- ✗ More verbose for simple cases

**validator**:
- ✓ Derive macros for common validations
- ✓ Less boilerplate for simple cases
- ✗ No effect system
- ✗ Less flexible for complex logic

### When to use validator
- Simple struct validation with standard rules
- You want derive macros
- Validation is your only need

### When to use Stillwater
- Complex validation logic
- Need effect composition
- Want full control over validation

**Recommendation**: Use both! validator for struct-level rules, Stillwater for complex logic:

```rust
use validator::Validate;
use stillwater::Validation;

#[derive(Validate)]
struct UserInput {
    #[validate(email)]
    email: String,
    #[validate(range(min = 18, max = 120))]
    age: u8,
}

fn validate_and_create(input: UserInput) -> Validation<User, Vec<Error>> {
    // First, struct-level validation
    input.validate()
        .map_err(|e| vec![Error::Validation(e)])?;

    // Then, custom business logic
    Validation::all((
        check_email_not_taken(&input.email),
        check_age_appropriate(input.age),
    ))
    .map(|_| User::from(input))
}
```

## vs Standard Library (Result, Option)

**Result** and **Option** are the foundation. When should you reach for Stillwater?

### Use Result when
- Short-circuiting is desired (fail fast)
- Single error is sufficient
- Simple error propagation

### Use Validation when
- You want ALL errors at once
- Validating forms or API requests
- Independent validations

### Use Effect when
- Separating I/O from logic
- Testing with mock environments
- Composing async operations

**Rule of thumb**: Start with Result. Reach for Stillwater when you need error accumulation or I/O separation.

## vs Other Languages

### Haskell

**Stillwater's Validation** ≈ Haskell's `Validation` from `Data.Validation`

**Stillwater's Effect** ≈ Haskell's `Reader` + `IO`

Key difference: Stillwater embraces Rust idioms (no HKTs, uses traits and generics).

### Scala ZIO

**Stillwater's Effect** shares philosophy with Scala's `ZIO`, but is much simpler:
- ZIO: Full-featured effect system with resource management, concurrency, streaming
- Stillwater: Focused on dependency injection and composition

### F# Computation Expressions

**Stillwater's validation** ≈ F#'s applicative validation

**Stillwater's effects** ≈ F#'s async workflows

Key difference: Rust's ownership and borrowing vs F#'s garbage collection.

## Summary

| Use Case | Best Choice |
|----------|-------------|
| Form validation | Stillwater Validation |
| API validation | Stillwater Validation |
| Testable I/O | Stillwater Effect |
| Error propagation | anyhow + Stillwater |
| Simple validations | validator + Stillwater |
| Type-level programming | frunk |
| Haskell-style monads | monadic |
| Generic error handling | anyhow/thiserror |

## Philosophy Comparison

| Library | Philosophy |
|---------|-----------|
| Stillwater | Pragmatic FP: common patterns, low learning curve |
| frunk | Academic FP: type-level programming, HLists |
| monadic | Haskell-style: monad abstraction, do-notation |
| anyhow | Ergonomic errors: minimal boilerplate |
| validator | Declarative: derive macros for common cases |

## Migration Guide

### From Result to Stillwater

```rust
// Before: Result (short-circuits)
fn validate(data: Data) -> Result<Valid, Error> {
    let email = validate_email(data.email)?;
    let age = validate_age(data.age)?;
    Ok(Valid { email, age })
}

// After: Validation (accumulates)
fn validate(data: Data) -> Validation<Valid, Vec<Error>> {
    Validation::all((
        validate_email(data.email),
        validate_age(data.age),
    ))
    .map(|(email, age)| Valid { email, age })
}
```

### From async fn to Effect

```rust
// Before: async fn (hard to test)
async fn create_user(email: String) -> Result<User, Error> {
    let user = User { email };
    DATABASE.save(&user).await?;
    Ok(user)
}

// After: Effect (testable)
fn create_user(email: String) -> Effect<User, Error, Env> {
    let user = User { email };
    IO::execute(|env: &Env| env.db.save(&user))
        .map(|_| user)
}
```

## Further Reading

- [Stillwater User Guide](guide/README.md)
- [frunk documentation](https://docs.rs/frunk)
- [monadic documentation](https://docs.rs/monadic)
- [anyhow documentation](https://docs.rs/anyhow)
- [validator documentation](https://docs.rs/validator)
