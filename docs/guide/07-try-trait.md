# Try Trait Support (Nightly Feature)

Stillwater provides experimental support for Rust's `?` operator with Validation and Effect on nightly Rust.

## Enabling

Add to `Cargo.toml`:

```toml
[dependencies]
stillwater = { version = "0.1", features = ["try_trait"] }
```

And use nightly Rust:

```bash
rustup override set nightly
```

## Using ? with Validation

```rust
#![feature(try_trait_v2)]

use stillwater::Validation;

fn validate_user(email: &str, age: u8) -> Validation<User, Vec<Error>> {
    let email = validate_email(email)?;  // ⚠️ Short-circuits!
    let age = validate_age(age)?;
    Ok(User { email, age })
}
```

**Warning**: Using `?` with Validation **short-circuits** on first error, defeating the purpose of error accumulation!

**Recommendation**: Don't use `?` with Validation. Use `Validation::all()` instead:

```rust
// ✓ Better: accumulates all errors
Validation::all((
    validate_email(email),
    validate_age(age),
))
.map(|(email, age)| User { email, age })
```

## Using ? with Effect

The `?` operator works well with Effect:

```rust
#![feature(try_trait_v2)]

use stillwater::Effect;

fn process_user(id: u64) -> Effect<Profile, Error, Env> {
    let user = fetch_user(id)?;          // Short-circuit on error
    let profile = load_profile(user)?;   // Short-circuit on error
    Ok(profile)
}
```

This is equivalent to:

```rust
fetch_user(id)
    .and_then(|user| load_profile(user))
```

Use whichever is more readable for your use case.

## Should You Use This Feature?

**Pros**:
- Familiar `?` syntax
- Cleaner for sequential operations

**Cons**:
- Requires nightly Rust
- Defeats Validation's purpose
- `and_then()` is just as readable

**Recommendation**: Wait for stable Rust support before using in production. The feature is mainly experimental.

## Migration Path

When try_trait_v2 stabilizes:

1. Update to stable Rust
2. Remove `#![feature(try_trait_v2)]`
3. Keep using `Validation::all()` for accumulation
4. Use `?` with Effect where it makes sense

## Next Steps

- Check the [FAQ](../FAQ.md) for common questions
- See [Patterns](../PATTERNS.md) for practical recipes
- Read [Comparison](../COMPARISON.md) vs other libraries
