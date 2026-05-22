# Try Trait Support (Nightly Feature)

Stillwater provides experimental support for Rust's `?` operator with Validation on nightly Rust.

## Enabling

Add to `Cargo.toml`:

```toml
[dependencies]
stillwater = { version = "0.1", features = ["try_trait"] }
```

Use nightly Rust and opt into the unstable implementation explicitly:

```bash
RUSTFLAGS="--cfg try_trait_nightly" cargo +nightly test --features try_trait
```

Applications that use `?` with `Validation` must also add `#![feature(try_trait_v2)]`
to their crate root.

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

## Should You Use This Feature?

**Pros**:
- Familiar `?` syntax
- Cleaner for sequential operations

**Cons**:
- Requires nightly Rust
- Requires `RUSTFLAGS="--cfg try_trait_nightly"`
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
