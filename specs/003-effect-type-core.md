---
number: 003
title: Effect Type with Async-First Design
category: foundation
priority: critical
status: draft
dependencies: [002]
created: 2025-11-21
---

# Specification 003: Effect Type with Async-First Design

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: Spec 002 (Validation Type)

## Context

Modern Rust applications are async-first for I/O operations. The Effect type is the core abstraction for separating pure business logic from side effects (I/O, network, database operations).

Following the "pure core, imperative shell" philosophy, Effect allows us to:
1. Keep business logic pure and testable (no I/O, no side effects)
2. Compose effects declaratively
3. Push I/O to the boundaries of the application
4. Test effectful code with mock environments

The Effect type wraps an async computation that depends on an environment and may fail with an error.

## Objective

Implement an async-first `Effect<T, E, Env>` type that enables composable, testable I/O operations while maintaining a clean separation between pure logic and side effects.

## Requirements

### Functional Requirements

- Define Effect type wrapping async functions
- Support both sync and async operations seamlessly
- Implement core combinators: `map`, `and_then`, `map_err`
- Provide constructors: `pure`, `fail`, `from_fn`, `from_async`, `from_result`
- Convert Validation to Effect
- Execute effects with environment via `run()` method
- Ensure proper error propagation
- Maintain type safety (T, E, Env are distinct)

### Non-Functional Requirements

- Async-first (all operations return Future)
- Zero-cost for sync operations (ready Future optimization)
- Type inference works for common cases
- Clear error messages
- Thread-safe (Send + 'static bounds where appropriate)
- Minimal boxing (one allocation per Effect creation)

## Acceptance Criteria

- [ ] Effect<T, E, Env> defined in `src/effect.rs`
- [ ] Wraps BoxFuture<'_, Result<T, E>>
- [ ] `pure()` creates successful effect
- [ ] `fail()` creates failed effect
- [ ] `from_fn()` wraps synchronous function
- [ ] `from_async()` wraps async function
- [ ] `from_result()` lifts Result to Effect
- [ ] `from_validation()` converts Validation to Effect
- [ ] `map()` transforms success value
- [ ] `and_then()` chains effects
- [ ] `map_err()` transforms error value
- [ ] `or_else()` recovers from errors
- [ ] `run(env)` executes effect and returns Future
- [ ] All methods compose correctly
- [ ] Comprehensive test coverage (>95%)
- [ ] Documentation with examples

## Technical Details

### Implementation Approach

```rust
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An effect that may perform I/O and depends on an environment
pub struct Effect<T, E = std::convert::Infallible, Env = ()> {
    run_fn: Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<T, E>> + Send>,
}

impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Create a pure value (no effects)
    pub fn pure(value: T) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { Ok(value) })),
        }
    }

    /// Create a failing effect
    pub fn fail(error: E) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { Err(error) })),
        }
    }

    /// Create from synchronous function
    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnOnce(&Env) -> Result<T, E> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                let result = f(env);
                Box::pin(async move { result })
            }),
        }
    }

    /// Create from async function
    pub fn from_async<F, Fut>(f: F) -> Self
    where
        F: FnOnce(&Env) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| Box::pin(f(env))),
        }
    }

    /// Create from Result
    pub fn from_result(result: Result<T, E>) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { result })),
        }
    }

    /// Chain effects
    pub fn and_then<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    let value = (self.run_fn)(env).await?;
                    let next = f(value);
                    (next.run_fn)(env).await
                })
            }),
        }
    }

    /// Transform success value
    pub fn map<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    (self.run_fn)(env).await.map(f)
                })
            }),
        }
    }

    /// Transform error value
    pub fn map_err<E2, F>(self, f: F) -> Effect<T, E2, Env>
    where
        F: FnOnce(E) -> E2 + Send + 'static,
        E2: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    (self.run_fn)(env).await.map_err(f)
                })
            }),
        }
    }

    /// Recover from errors
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> Effect<T, E, Env> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    match (self.run_fn)(env).await {
                        Ok(value) => Ok(value),
                        Err(err) => {
                            let recovery = f(err);
                            (recovery.run_fn)(env).await
                        }
                    }
                })
            }),
        }
    }

    /// Run the effect with the given environment
    pub async fn run(self, env: &Env) -> Result<T, E> {
        (self.run_fn)(env).await
    }
}

/// Convert Validation to Effect
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    pub fn from_validation(validation: Validation<T, E>) -> Self {
        match validation {
            Validation::Success(value) => Effect::pure(value),
            Validation::Failure(error) => Effect::fail(error),
        }
    }
}
```

### Architecture Changes

- New module: `src/effect.rs`
- Dependency on tokio or futures crate for BoxFuture
- Export from `src/lib.rs`
- Re-export in `prelude`

### Data Structures

```rust
pub struct Effect<T, E, Env> {
    run_fn: Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<T, E>> + Send>,
}
```

### APIs and Interfaces

See Implementation Approach above.

## Dependencies

- **Prerequisites**: Spec 002 (Validation type for conversion)
- **Affected Components**: None (new module)
- **External Dependencies**: None (uses std::future, consider tokio for testing)

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_pure() {
    let effect = Effect::<_, String, ()>::pure(42);
    assert_eq!(effect.run(&()).await, Ok(42));
}

#[tokio::test]
async fn test_fail() {
    let effect = Effect::<i32, _, ()>::fail("error");
    assert_eq!(effect.run(&()).await, Err("error"));
}

#[tokio::test]
async fn test_map() {
    let effect = Effect::<_, String, ()>::pure(5)
        .map(|x| x * 2);
    assert_eq!(effect.run(&()).await, Ok(10));
}

#[tokio::test]
async fn test_and_then() {
    let effect = Effect::<_, String, ()>::pure(5)
        .and_then(|x| Effect::pure(x * 2));
    assert_eq!(effect.run(&()).await, Ok(10));
}

#[tokio::test]
async fn test_from_fn_sync() {
    let effect = Effect::from_fn(|_: &()| Ok::<_, String>(42));
    assert_eq!(effect.run(&()).await, Ok(42));
}

#[tokio::test]
async fn test_from_async() {
    let effect = Effect::from_async(|_: &()| async { Ok::<_, String>(42) });
    assert_eq!(effect.run(&()).await, Ok(42));
}

#[tokio::test]
async fn test_mix_sync_and_async() {
    let effect = Effect::from_fn(|_: &()| Ok::<_, String>(5))
        .and_then(|x| Effect::from_async(move |_| async move { Ok(x * 2) }));
    assert_eq!(effect.run(&()).await, Ok(10));
}

#[tokio::test]
async fn test_or_else() {
    let effect = Effect::<i32, _, ()>::fail("error")
        .or_else(|_| Effect::pure(42));
    assert_eq!(effect.run(&()).await, Ok(42));
}

#[tokio::test]
async fn test_from_validation() {
    let validation = Validation::<_, String>::success(42);
    let effect = Effect::from_validation(validation);
    assert_eq!(effect.run(&()).await, Ok(42));

    let validation = Validation::<i32, _>::failure("error");
    let effect = Effect::from_validation(validation);
    assert_eq!(effect.run(&()).await, Err("error"));
}
```

### Integration Tests

```rust
// Test with environment
#[tokio::test]
async fn test_with_environment() {
    struct Env {
        value: i32,
    }

    let effect = Effect::from_fn(|env: &Env| Ok::<_, String>(env.value * 2));

    let env = Env { value: 21 };
    assert_eq!(effect.run(&env).await, Ok(42));
}

// Test error propagation
#[tokio::test]
async fn test_error_propagation() {
    let effect = Effect::<_, String, ()>::pure(5)
        .and_then(|_| Effect::fail("error"))
        .map(|x| x * 2);  // This shouldn't run

    assert_eq!(effect.run(&()).await, Err("error"));
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for Effect type
- Examples for each method
- Explain async-first design choice
- Show sync and async usage patterns
- Document environment pattern

### User Documentation

- Add "Effects" section to README
- Create guide in docs/guide/03-effects.md
- Explain "pure core, imperative shell" pattern
- Show testing with mock environments

### Architecture Updates

- Document Effect type in DESIGN.md
- Explain async-first decision in docs/async-design.md

## Implementation Notes

### Async Runtime

- Effect requires an async runtime (tokio, async-std)
- Tests use `#[tokio::test]`
- Document requirement in README

### Performance

- One Box allocation per Effect creation
- Sync functions wrapped in ready Future (optimized away by compiler)
- Minimal overhead for typical I/O-bound operations

### Lifetime Considerations

- `BoxFuture<'_, T>` borrows environment for lifetime of future
- Environment must outlive the future execution
- Cannot move Effect after calling run_fn (FnOnce)

### Type Inference

- Usually works well: `Effect::pure(42)` infers E and Env as ()
- Sometimes needs turbofish: `Effect::<_, String, ()>::pure(42)`
- Document common patterns

## Migration and Compatibility

No migration needed - this is a new feature.

## Open Questions

1. Should we support parallel execution? `Effect::all(vec![effect1, effect2])`
   - Decision: Defer to separate spec (helper combinators)

2. Should we integrate with Try trait for `?` operator?
   - Decision: Defer to separate spec (Try trait integration)

3. Should we provide `Effect::timeout()` helper?
   - Decision: Not for MVP, can add later

4. Should run() consume self or take &mut self for reusability?
   - Decision: Consume (FnOnce), more flexible
