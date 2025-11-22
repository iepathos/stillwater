# Async Design for Stillwater

## Critical Decision: Async from the Start

Since async is important for MVP, we need to design Effect with async as a first-class concern, not an afterthought.

## Design Options

### Option 1: Separate Sync and Async Types

```rust
// Sync version
struct Effect<T, E, Env> {
    run_fn: Box<dyn FnOnce(&Env) -> Result<T, E>>,
}

// Async version
struct AsyncEffect<T, E, Env> {
    run_fn: Box<dyn FnOnce(&Env) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>>,
}
```

**Pros:**
- ✅ Simple, clear separation
- ✅ No performance overhead for sync code
- ✅ Easy to understand which is which

**Cons:**
- ❌ Duplicates entire API (and_then, map, etc.)
- ❌ Can't easily mix sync and async
- ❌ Confusing for users: which one to use?

**Verdict:** Too much duplication, against DRY.

---

### Option 2: Unified Type with Async Methods

```rust
struct Effect<T, E, Env> {
    // Store function, not Future
    run_fn: Box<dyn FnOnce(&Env) -> BoxFuture<'static, Result<T, E>> + Send>,
}

impl<T, E, Env> Effect<T, E, Env> {
    // Create from sync function
    pub fn from_sync<F>(f: F) -> Self
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

    // Create from async function
    pub fn from_async<F, Fut>(f: F) -> Self
    where
        F: FnOnce(&Env) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| Box::pin(f(env))),
        }
    }

    // Run always returns Future
    pub async fn run(self, env: &Env) -> Result<T, E> {
        (self.run_fn)(env).await
    }
}
```

**Pros:**
- ✅ Single API, works for both sync and async
- ✅ Can freely mix sync and async effects
- ✅ Clean for users: one Effect type

**Cons:**
- ⚠️ Always returns Future (even for sync code)
- ⚠️ Requires async runtime even for pure sync code
- ⚠️ Boxing overhead

**Verdict:** Good, but forces async everywhere.

---

### Option 3: Generic Over Sync/Async (Type-State Pattern)

```rust
// Marker types
struct Sync;
struct Async;

struct Effect<T, E, Env, Runtime = Sync> {
    run_fn: Box<dyn ...>,  // Different based on Runtime
    _phantom: PhantomData<Runtime>,
}

impl<T, E, Env> Effect<T, E, Env, Sync> {
    pub fn run(self, env: &Env) -> Result<T, E> { ... }
}

impl<T, E, Env> Effect<T, E, Env, Async> {
    pub async fn run(self, env: &Env) -> Result<T, E> { ... }
}
```

**Pros:**
- ✅ Type system enforces sync vs async
- ✅ No runtime overhead for sync
- ✅ Can convert between them

**Cons:**
- ❌ Complex type signatures
- ❌ Hard to implement correctly
- ❌ Confusing for users
- ❌ Viral type parameter

**Verdict:** Too complex for the benefit.

---

### Option 4: Runtime-Agnostic with Trait-Based Execution (Recommended)

```rust
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct Effect<T, E, Env> {
    // Always async internally for maximum flexibility
    run_fn: Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<T, E>> + Send>,
}

impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Create from synchronous function
    pub fn pure(value: T) -> Self {
        Effect {
            run_fn: Box::new(move |_env| {
                Box::pin(async move { Ok(value) })
            }),
        }
    }

    /// Create from synchronous fallible function
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
                    let next_effect = f(value);
                    (next_effect.run_fn)(env).await
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
                    let value = (self.run_fn)(env).await?;
                    Ok(f(value))
                })
            }),
        }
    }

    /// Run the effect (always async)
    pub async fn run(self, env: &Env) -> Result<T, E> {
        (self.run_fn)(env).await
    }
}
```

**Pros:**
- ✅ Single, unified API
- ✅ Sync code works seamlessly (wrapped in ready Future)
- ✅ Natural async support
- ✅ Can mix sync and async effects freely
- ✅ Composable: async + sync + async chains work

**Cons:**
- ⚠️ Always requires async runtime (but acceptable in 2025)
- ⚠️ Some boxing overhead (minimal for I/O-bound code)
- ⚠️ Sync code has tiny wrapper cost (negligible)

**Verdict:** Best balance of simplicity and capability.

---

## Recommended Design: Option 4

**Rationale:**
1. Modern Rust is async-first for I/O
2. Tokio/async-std are standard in server apps
3. Wrapping sync in Future is cheap
4. Single API is much cleaner

### Core Effect Implementation

```rust
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An effect that may perform I/O and depends on an environment
pub struct Effect<T, E = Infallible, Env = ()> {
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

    /// Create an error
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

    /// Chain dependent effects
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
```

---

## Helper Methods (Idiomatic + Functional)

Based on your guidance to include helpers when appropriate:

### 1. Tap (Side Effect, Return Value)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + Clone + 'static,
    E: Send + 'static,
{
    /// Perform a side effect and return the original value
    pub fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Effect<(), E, Env> + Send + 'static,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }
}

// Usage:
user_effect
    .tap(|user| IO::write(|logger| logger.info(format!("Created user: {}", user.id))))
    // user is returned unchanged
```

### 2. Check (Conditional Failure)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Fail with error if predicate is false
    pub fn check<P, F>(self, predicate: P, error_fn: F) -> Self
    where
        P: FnOnce(&T) -> bool + Send + 'static,
        F: FnOnce() -> E + Send + 'static,
    {
        self.and_then(move |value| {
            if predicate(&value) {
                Effect::pure(value)
            } else {
                Effect::fail(error_fn())
            }
        })
    }
}

// Usage:
user_effect.check(
    |user| user.age >= 18,
    || AppError::AgeTooYoung
)
```

### 3. With (Combine Effects, Keep Both Results)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Combine with another effect, returning both values
    pub fn with<U, F>(self, f: F) -> Effect<(T, U), E, Env>
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        self.and_then(move |value| {
            let effect = f(&value);
            effect.map(move |other| (value, other))
        })
    }
}

// Usage:
user_effect.with(|user| {
    IO::read(|db| db.find_orders(user.id))
})
// Returns: Effect<(User, Vec<Order>), E, Env>
```

### 4. Auto-Converting and_then

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Chain effect with automatic error conversion
    pub fn and_then_auto<U, E2, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> Effect<U, E2, Env> + Send + 'static,
        U: Send + 'static,
        E2: Send + 'static,
        E: From<E2>,
    {
        self.and_then(move |value| {
            f(value).map_err(E::from)
        })
    }
}

// Usage (no manual map_err needed):
user_effect
    .and_then_auto(|user| validate_user(user))  // Different error type, auto-converts!
    .and_then_auto(|user| save_user(user))      // Another different error, auto-converts!
```

### 5. Reference-Friendly and_then

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + Clone + 'static,
    E: Send + 'static,
{
    /// Chain effect by borrowing value, then returning it
    pub fn and_then_ref<U, F>(self, f: F) -> Effect<T, E, Env>
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }
}

// Usage:
user_effect
    .and_then_ref(|user| save_audit_log(user))  // Borrows user
    // user is returned (cloned once, not multiple times)
```

### 6. Parallel Execution (Future Enhancement)

```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Run multiple effects in parallel
    pub fn all<I>(effects: I) -> Effect<Vec<T>, E, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    let futures: Vec<_> = effects
                        .into_iter()
                        .map(|effect| (effect.run_fn)(env))
                        .collect();

                    // Run all futures concurrently
                    let results: Vec<Result<T, E>> = futures::future::join_all(futures).await;

                    // Collect all results, fail if any failed
                    results.into_iter().collect()
                })
            }),
        }
    }
}

// Usage:
let user_effects = user_ids.into_iter().map(|id| fetch_user(id));
Effect::all(user_effects)  // Fetches all users concurrently!
```

---

## IO Module for Async

```rust
pub struct IO;

impl IO {
    /// Read from environment (immutable borrow)
    pub fn read<T, R, F>(f: F) -> Effect<R, Infallible, T>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &T| Ok(f(env)))
    }

    /// Write to environment (mutable borrow) - requires RefCell or similar
    pub fn write<T, R, F>(f: F) -> Effect<R, Infallible, T>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &T| Ok(f(env)))
    }

    /// Async I/O operation
    pub fn read_async<T, R, F, Fut>(f: F) -> Effect<R, Infallible, T>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
    {
        Effect::from_async(move |env: &T| async move { Ok(f(env).await) })
    }
}
```

---

## Usage Examples

### Pure Sync Code

```rust
let effect = Effect::pure(42)
    .map(|x| x * 2)
    .map(|x| x + 10);

let result = effect.run(&()).await;  // Must use .await, but wrapping is cheap
assert_eq!(result, Ok(94));
```

### Async I/O

```rust
async fn fetch_user_async(id: UserId) -> Effect<User, AppError, AppEnv> {
    Effect::from_async(|env: &AppEnv| async move {
        env.db.query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one()
            .await
            .map_err(AppError::from)
    })
}
```

### Mixed Sync and Async

```rust
fn process_user(id: UserId) -> Effect<Invoice, AppError, AppEnv> {
    fetch_user_async(id)                          // Async I/O
        .and_then(|user| {
            let discount = calculate_discount(&user);  // Sync pure
            Effect::pure(discount)
        })
        .and_then(|discount| {
            save_discount_async(discount)          // Async I/O
        })
}

// All compose seamlessly!
```

---

## Performance Considerations

**Boxing Overhead:**
- One allocation per Effect creation
- Negligible for I/O-bound operations
- Database queries: ~1ms
- Network calls: ~10-100ms
- Boxing: ~50ns
- **Ratio: 0.005% overhead**

**Future Wrapping:**
- Sync code wrapped in ready Future
- Optimizer often eliminates this
- Zero runtime cost in practice

**Conclusion:** Async-first design has negligible overhead for typical use cases.

---

## Migration Path

Users can opt-in to sync-only if needed:

```rust
// If you really need blocking for some reason:
tokio::task::block_in_place(|| {
    let rt = tokio::runtime::Handle::current();
    rt.block_on(effect.run(&env))
})
```

But modern Rust apps should embrace async.

---

## Decision Summary

✅ **Use Option 4: Async-first unified Effect**

**Core Design:**
- Effect is always async internally
- Sync code wraps in ready Future (cheap)
- Single unified API
- Natural async support

**Helper Methods:**
- ✅ `.tap()` - side effects
- ✅ `.check()` - conditional failures
- ✅ `.with()` - combine effects
- ✅ `.and_then_auto()` - auto-convert errors
- ✅ `.and_then_ref()` - avoid cloning
- ✅ `Effect::all()` - parallel execution

**IO API:**
- `IO::read()` - immutable access
- `IO::write()` - mutable access
- `IO::read_async()` - async operations

---

*Async from the start is the right choice for modern Rust I/O libraries.*
