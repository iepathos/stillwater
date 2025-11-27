# Migration Guide: Stillwater 0.10.x to 0.11.0

## Overview

Stillwater 0.11.0 introduces a zero-cost Effect API, following the `futures` crate pattern. This is a breaking change that requires updating your code.

## Key Changes

| 0.10.x | 0.11.0 |
|--------|--------|
| `Effect<T, E, Env>` struct (boxed per combinator) | `impl Effect<Output=T, Error=E, Env=Env>` trait (zero-cost) |
| `Effect::pure(x)` | `pure(x)` or `pure::<_, E, Env>(x)` |
| `Effect::fail(e)` | `fail(e)` or `fail::<T, _, Env>(e)` |
| `Effect::from_fn(f)` | `from_fn(f)` |
| N/A | `from_async(f)`, `from_result(r)`, `from_option(o, err)` |
| N/A | `ask()`, `asks(f)`, `local(f, effect)` |
| `.run(&env).await` | `.run(&env).await` or `.execute(&env).await` |
| Always boxed | Zero-cost by default, opt-in `.boxed()` |

## Why the Change?

The old API boxed every combinator, allocating on the heap for each `.map()`, `.and_then()`, etc. While this was acceptable for I/O-bound work, it added unnecessary overhead for compute-bound code and prevented certain compiler optimizations.

The new API follows the pattern established by the `futures` crate:
- **Zero-cost by default**: Each combinator returns a concrete type, enabling full inlining
- **Explicit boxing**: Use `.boxed()` only when type erasure is needed

## Migration Steps

### Step 1: Update Imports

```rust
// Before
use stillwater::Effect;

// After - Option A: Use prelude (recommended)
use stillwater::prelude::*;
// or
use stillwater::effect::prelude::*;

// After - Option B: Direct imports
use stillwater::{pure, fail, from_fn, Effect, EffectExt, BoxedEffect};
```

### Step 2: Update Return Types

```rust
// Before
fn my_effect() -> Effect<i32, String, ()> {
    Effect::pure(42)
}

// After - Option A: Zero-cost (preferred)
fn my_effect() -> impl Effect<Output = i32, Error = String, Env = ()> {
    pure(42)
}

// After - Option B: Boxed (when needed)
fn my_effect_boxed() -> BoxedEffect<i32, String, ()> {
    pure(42).boxed()
}

// Running effects - both work:
let result = my_effect().run(&()).await;      // From Effect trait
let result = my_effect().execute(&()).await;  // Convenience method
```

### Step 3: Update Constructor Calls

```rust
// Before
Effect::pure(42)
Effect::fail("error")
Effect::from_fn(|env| Ok(env.value))

// After - basic constructors
pure(42)
fail("error")
from_fn(|env| Ok(env.value))

// After - additional constructors available
from_async(|env| async { Ok(value) })  // For async operations
from_result(Ok(42))                     // From Result
from_option(Some(42), || "missing")     // From Option with error
ask()                                   // Get entire environment
asks(|env| env.config.clone())          // Extract from environment
local(|env| modified_env, inner_effect) // Run with modified env
```

### Step 4: Add `.boxed()` Where Needed

If you're storing effects in collections, using recursion, or returning different effect types from match arms, add `.boxed()`:

```rust
use stillwater::{pure, BoxedEffect, EffectExt};

// Collections - need same type
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).boxed(),
];

// Recursion - need to break infinite type
fn recursive(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n)
            .and_then(move |_| recursive(n - 1))
            .boxed()
    }
}

// Match arms - need same type
fn conditional(flag: bool) -> BoxedEffect<i32, String, ()> {
    if flag {
        pure(1).boxed()
    } else {
        pure(2).map(|x| x * 2).boxed()
    }
}
```

## Using the Compatibility Module

For gradual migration, use the compatibility module:

```rust
#[allow(deprecated)]
use stillwater::LegacyEffect; // Type alias for BoxedEffect

// Old-style code (with deprecation warnings)
fn my_effect() -> LegacyEffect<i32, String, ()> {
    stillwater::pure(42).boxed()
}
```

The `LegacyEffect` type alias and `LegacyConstructors` trait are deprecated. Migrate to the new API as soon as possible.

## Common Issues

### "expected struct, found opaque type"

You're returning `impl Effect` but the caller expects a concrete type. Either:
1. Use `.boxed()` to get `BoxedEffect`
2. Update the caller to accept `impl Effect`

### "cannot infer type"

Add type annotations to constructor functions:
```rust
pure::<_, String, ()>(42)  // Specify error and env types
```

### "the trait bound is not satisfied"

Make sure your closures are `Send`:
```rust
// Before (might not be Send)
.map(|x| x + some_local_ref)

// After (capture by value)
let value = *some_local_ref;
.map(move |x| x + value)
```

### "recursive type has infinite size"

You need to use `.boxed()` for recursive effects:
```rust
fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n)
            .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
            .boxed()
    }
}
```

## Before/After Examples

### Simple Effect Chain

```rust
// Before
fn calculate() -> Effect<i32, String, AppEnv> {
    Effect::pure(42)
        .map(|x| x * 2)
        .and_then(|x| Effect::pure(x + 10))
}

// After
fn calculate() -> impl Effect<Output = i32, Error = String, Env = AppEnv> {
    pure(42)
        .map(|x| x * 2)
        .and_then(|x| pure(x + 10))
}
```

### Effect with Environment

```rust
// Before
fn fetch_config() -> Effect<String, AppError, AppEnv> {
    Effect::from_fn(|env: &AppEnv| {
        Ok(env.config.api_key.clone())
    })
}

// After
fn fetch_config() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    asks(|env: &AppEnv| env.config.api_key.clone())
}
```

### Async Effect

```rust
// Before
fn fetch_user(id: u64) -> Effect<User, DbError, AppEnv> {
    Effect::from_async(|env: &AppEnv| {
        let db = env.db.clone();
        async move {
            db.find_user(id).await
        }
    })
}

// After
fn fetch_user(id: u64) -> impl Effect<Output = User, Error = DbError, Env = AppEnv> {
    from_async(move |env: &AppEnv| {
        let db = env.db.clone();
        async move {
            db.find_user(id).await
        }
    })
}
```

### Parallel Effects

```rust
use stillwater::effect::prelude::*;

// Heterogeneous parallel (zero-cost) - par2, par3, par4
let effect = par2(
    pure::<_, String, ()>(1),
    pure::<_, String, ()>("hello".to_string()),
);
let (num, text) = effect.run(&()).await?;

// Homogeneous parallel (requires boxing) - par_all, race
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).boxed(),
    pure(3).boxed(),
];
let results = par_all(effects).run(&()).await?;
```

## Performance Implications

The new zero-cost API eliminates heap allocations for effect chains:

| Scenario | 0.10.x | 0.11.0 |
|----------|--------|--------|
| 10-combinator chain | 10 Box allocations | 0 allocations |
| Effect stored in collection | 1 Box per effect | 1 Box per effect (same) |
| Recursive effect | Multiple boxes | 1 Box per recursion level (same) |

For I/O-bound applications, this difference is negligible. For compute-bound code or code running in tight loops, the new API can provide meaningful performance improvements.

## Getting Help

- Check the [examples/](../examples/) directory for working code
- Read the [User Guide](guide/README.md) for comprehensive tutorials
- See [FAQ.md](FAQ.md) for common questions
- Open an issue on [GitHub](https://github.com/iepathos/stillwater/issues)
