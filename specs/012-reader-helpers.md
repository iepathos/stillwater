---
number: 012
title: Reader Pattern Helpers for Effect
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 012: Reader Pattern Helpers for Effect

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (extends existing Effect type)

## Context

Stillwater's `Effect<T, E, Env>` type implements the Reader monad pattern through its `Env` parameter. The environment provides dependency injection without globals or service locators.

Currently, effects can access the environment via `IO::read` and `IO::write`, but the Reader pattern has additional combinators that would improve ergonomics:

- **`ask()`** - Get the entire environment
- **`asks(f)`** - Query and transform the environment
- **`local(f, effect)`** - Temporarily modify environment for a sub-computation

These are standard Reader monad operations that make environment access more composable and explicit.

## Objective

Complete the Reader monad pattern in `Effect` by adding helper methods (`ask`, `asks`, `local`) that make environment access and manipulation more ergonomic and composable.

## Requirements

### Functional Requirements

- Add `Effect::ask()` method to get environment
- Add `Effect::asks(f)` method to query and transform environment
- Add `Effect::local(f, effect)` method to modify environment temporarily
- Ensure methods compose cleanly with existing combinators
- Maintain zero-cost abstraction guarantees
- Update IO module to use these helpers internally where appropriate
- Preserve async compatibility

### Non-Functional Requirements

- Zero runtime overhead compared to manual access
- Type-safe environment transformations
- Clear error messages for type mismatches
- Comprehensive documentation with examples
- No breaking changes to existing Effect API

## Acceptance Criteria

- [ ] `Effect::ask()` method implemented
- [ ] `Effect::asks(f)` method implemented
- [ ] `Effect::local(f, effect)` method implemented
- [ ] All methods work with sync and async effects
- [ ] Examples demonstrate practical use cases
- [ ] Tests verify Reader monad laws
- [ ] Documentation updated with Reader pattern guide
- [ ] Integration with existing Effect combinators works smoothly
- [ ] Zero performance regression
- [ ] All tests pass

## Technical Details

### Implementation Approach

#### Ask: Get Environment

```rust
impl<T, E, Env> Effect<T, E, Env> {
    /// Access the entire environment.
    ///
    /// This is the fundamental Reader operation - it gives you access to the
    /// environment so you can extract what you need.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// struct Config {
    ///     api_key: String,
    ///     timeout: u64,
    /// }
    ///
    /// let effect = Effect::ask()
    ///     .map(|config: &Config| config.api_key.clone());
    ///
    /// let config = Config {
    ///     api_key: "secret".into(),
    ///     timeout: 30,
    /// };
    ///
    /// let result = effect.run(&config).unwrap();
    /// assert_eq!(result, "secret");
    /// ```
    pub fn ask() -> Effect<Env, E, Env>
    where
        Env: Clone,
    {
        Effect::from_fn(|env| Ok(env.clone()))
    }
}
```

#### Asks: Query Environment

```rust
impl<T, E, Env> Effect<T, E, Env> {
    /// Query the environment and transform the result.
    ///
    /// This is a convenience method equivalent to `ask().map(f)`,
    /// useful when you only need a specific part of the environment.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// struct AppEnv {
    ///     database: Database,
    ///     config: Config,
    /// }
    ///
    /// // Extract just the config
    /// let effect = Effect::asks(|env: &AppEnv| env.config.clone());
    ///
    /// // Or query a specific field
    /// let timeout = Effect::asks(|env: &AppEnv| env.config.timeout);
    /// ```
    pub fn asks<F, U>(f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(&Env) -> U + 'static,
    {
        Effect::from_fn(move |env| Ok(f(env)))
    }
}
```

#### Local: Modify Environment

```rust
impl<T, E, Env> Effect<T, E, Env> {
    /// Run an effect with a modified environment.
    ///
    /// This allows you to temporarily change the environment for a
    /// sub-computation without affecting the rest of the chain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// struct Config {
    ///     debug: bool,
    ///     timeout: u64,
    /// }
    ///
    /// fn fetch_data() -> Effect<Data, Error, Config> {
    ///     Effect::asks(|cfg| cfg.timeout)
    ///         .and_then(|timeout| {
    ///             // Use timeout to fetch data
    ///             Effect::pure(Data::new())
    ///         })
    /// }
    ///
    /// // Run with modified timeout for this specific fetch
    /// let config = Config { debug: false, timeout: 30 };
    ///
    /// let effect = Effect::local(
    ///     |cfg: &Config| Config { timeout: 60, ..*cfg },
    ///     fetch_data()
    /// );
    ///
    /// // fetch_data sees timeout=60, but config still has timeout=30
    /// ```
    pub fn local<F>(f: F, effect: Effect<T, E, Env>) -> Effect<T, E, Env>
    where
        F: FnOnce(&Env) -> Env + 'static,
        Env: Clone,
    {
        Effect::from_fn(move |env| {
            let modified_env = f(env);
            effect.run(&modified_env)
        })
    }
}
```

### Architecture Changes

- Add methods to `src/effect.rs`
- Update `src/io.rs` to potentially use these helpers internally
- No new modules needed
- Update prelude to re-export if needed

### APIs and Interfaces

```rust
// Public API additions to Effect
impl<T, E, Env> Effect<T, E, Env> {
    pub fn ask() -> Effect<Env, E, Env> where Env: Clone;
    pub fn asks<F, U>(f: F) -> Effect<U, E, Env> where F: FnOnce(&Env) -> U;
    pub fn local<F>(f: F, effect: Effect<T, E, Env>) -> Effect<T, E, Env>
        where F: FnOnce(&Env) -> Env, Env: Clone;
}
```

### Integration Patterns

#### Pattern 1: Environment Extraction

```rust
// Before: manual extraction
let effect = IO::read(|env: &AppEnv| {
    let config = &env.config;
    let db = &env.database;
    process(config, db)
});

// After: using asks
let effect = Effect::asks(|env: &AppEnv| {
    (env.config.clone(), env.database.clone())
})
.and_then(|(config, db)| {
    process(&config, &db)
});
```

#### Pattern 2: Temporary Configuration

```rust
// Run effect with debug logging enabled
let debug_effect = Effect::local(
    |env: &AppEnv| AppEnv {
        config: Config { debug: true, ..env.config },
        ..env.clone()
    },
    risky_operation()
);
```

#### Pattern 3: Environment Composition

```rust
fn with_timeout<T, E>(
    timeout: u64,
    effect: Effect<T, E, AppEnv>
) -> Effect<T, E, AppEnv> {
    Effect::local(
        move |env| AppEnv {
            config: Config { timeout, ..env.config },
            ..env.clone()
        },
        effect
    )
}

// Usage
let effect = with_timeout(60, fetch_user(id));
```

## Dependencies

- **Prerequisites**: None (extends existing Effect)
- **Affected Components**:
  - `src/effect.rs` (add methods)
  - `src/io.rs` (potential internal use)
  - Examples demonstrating Reader pattern
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEnv {
        value: i32,
    }

    #[test]
    fn test_ask() {
        let env = TestEnv { value: 42 };
        let effect: Effect<TestEnv, (), TestEnv> = Effect::ask();
        let result = effect.run(&env).unwrap();
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_asks() {
        let env = TestEnv { value: 42 };
        let effect = Effect::asks(|e: &TestEnv| e.value * 2);
        let result = effect.run(&env).unwrap();
        assert_eq!(result, 84);
    }

    #[test]
    fn test_local() {
        let env = TestEnv { value: 10 };

        let inner = Effect::asks(|e: &TestEnv| e.value);
        let outer = Effect::local(
            |e: &TestEnv| TestEnv { value: e.value * 2 },
            inner
        );

        let result = outer.run(&env).unwrap();
        assert_eq!(result, 20); // Sees modified environment
        assert_eq!(env.value, 10); // Original unchanged
    }
}
```

### Reader Monad Laws

```rust
#[cfg(test)]
mod laws {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct Env(i32);

    // Law: ask >>= pure == pure
    #[test]
    fn test_ask_pure_identity() {
        let env = Env(42);

        let e1 = Effect::ask().and_then(|e: Env| Effect::pure(e));
        let e2: Effect<Env, (), Env> = Effect::pure(env.clone());

        assert_eq!(e1.run(&env).unwrap(), e2.run(&env).unwrap());
    }

    // Law: local id m == m
    #[test]
    fn test_local_identity() {
        let env = Env(42);
        let m = Effect::asks(|e: &Env| e.0);

        let e1 = Effect::local(|e: &Env| e.clone(), m.clone());
        let e2 = m;

        assert_eq!(e1.run(&env).unwrap(), e2.run(&env).unwrap());
    }

    // Law: local f (local g m) == local (g . f) m
    #[test]
    fn test_local_composition() {
        let env = Env(10);
        let m = Effect::asks(|e: &Env| e.0);

        let f = |e: &Env| Env(e.0 + 1);
        let g = |e: &Env| Env(e.0 * 2);

        let e1 = Effect::local(f, Effect::local(g, m.clone()));
        let e2 = Effect::local(|e: &Env| g(&f(e)), m);

        assert_eq!(e1.run(&env).unwrap(), e2.run(&env).unwrap());
    }
}
```

### Integration Tests

```rust
#[test]
fn test_reader_pattern_workflow() {
    #[derive(Clone)]
    struct AppEnv {
        db: MockDb,
        config: Config,
    }

    fn fetch_user(id: i32) -> Effect<User, Error, AppEnv> {
        Effect::asks(|env: &AppEnv| env.db.clone())
            .and_then(move |db| {
                db.find_user(id)
                    .ok_or(Error::NotFound)
                    .into()
            })
    }

    fn with_retry<T>(effect: Effect<T, Error, AppEnv>) -> Effect<T, Error, AppEnv>
    where
        T: Clone,
    {
        Effect::local(
            |env| AppEnv {
                config: Config { retries: 3, ..env.config },
                ..env.clone()
            },
            effect
        )
    }

    let env = AppEnv {
        db: MockDb::new(),
        config: Config::default(),
    };

    let result = with_retry(fetch_user(42)).run(&env);
    assert!(result.is_ok());
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for each method
- Examples showing practical use cases
- Explain relationship to Reader monad
- Document when to use each helper

### User Documentation

- Update `docs/guide/03-effects.md` with Reader section
- Add `docs/guide/09-reader-pattern.md` comprehensive guide
- Update README with Reader example
- Add FAQ: "What is the Reader pattern?"

### Architecture Updates

- Update DESIGN.md Reader pattern section
- Document environment composition patterns
- Explain `local` use cases

## Implementation Notes

### Design Decisions

**Why `ask()` instead of `get_env()`?**
- Standard Reader monad naming from Haskell/Scala
- Shorter, more ergonomic
- Familiar to FP practitioners

**Why require `Clone` for environment?**
- Enables `ask()` to return owned environment
- Allows `local()` to create modified copies
- Most environments are cheaply cloneable (Arc-wrapped services)

**Why `local` takes function instead of direct value?**
- More composable (can derive from current env)
- Allows partial updates
- Standard Reader pattern

### Gotchas

- `ask()` requires `Env: Clone` - may need Arc wrapping for large envs
- `local()` creates a copy - expensive for large environments
- Consider using `AsRef<T>` for service extraction (existing pattern)

### Best Practices

```rust
// Good: Extract only what you need
Effect::asks(|env: &AppEnv| env.config.timeout)

// Less good: Clone entire environment unnecessarily
Effect::ask().map(|env: AppEnv| env.config.timeout)

// Good: Use local for temporary config changes
Effect::local(|env| modify_config(env), effect)

// Good: Compose local for multiple changes
Effect::local(f, Effect::local(g, effect))
```

## Migration and Compatibility

### Breaking Changes

None - pure additions to existing API.

### Compatibility

- Fully backward compatible
- Existing Effect code works unchanged
- Opt-in usage of Reader helpers

### Migration Path

No migration needed. Users can adopt gradually:

```rust
// Before (still works)
IO::read(|env: &Config| env.value.clone())

// After (more composable)
Effect::asks(|env: &Config| env.value.clone())
```

## Related Patterns

- **Reader Monad**: This completes the implementation
- **Dependency Injection**: Environment provides DI
- **Configuration Management**: local() for temporary config

## Success Metrics

- Reader monad laws verified by tests
- Zero performance overhead vs manual implementation
- Positive user feedback on ergonomics
- Documentation clarity (measured by questions)

## Future Enhancements

- `ask_as<T>()` - automatic extraction via AsRef
- `local_async()` - async-specific local variant
- Environment composition helpers
- Reader transformers (ReaderT)
