# IO API Analysis - read and write

## Status

Accepted and implemented.

The current IO helper API is:

```rust
IO::read(...)
IO::write(...)
IO::read_async(...)
IO::write_async(...)
```

The previous design discussion considered more database-oriented and generic names. This document preserves that rationale while describing the current implementation accurately.

## Current Design

The IO module provides a small namespace for creating boxed effects from service operations:

```rust
use stillwater::IO;

let find_user = IO::read(|db: &Database| db.find_user(id));
let save_user = IO::write(|repo: &UserRepository| repo.save_user(user));
```

Both `read` and `write` receive `&T`, not `&mut T`. This follows Stillwater's effect model: `Effect::run` receives `&Env`, and services that mutate state should expose safe interior mutation through handles such as connection pools, `Arc<Mutex<T>>`, channels, or client types that perform mutation behind an immutable reference.

Async operations use explicit async variants:

```rust
let find_user = IO::read_async(|db: &Database| async move {
    db.find_user(id).await
});

let save_user = IO::write_async(|repo: &UserRepository| async move {
    repo.save_user(user).await
});
```

## Intent Of The Names

The names are semantic, not borrow-mode distinctions:

- `read` means the operation is conceptually query-like and returns information.
- `write` means the operation is conceptually command-like and changes external state.
- `read_async` and `write_async` mirror the same split for futures.

This lets code communicate intent even though both closures receive immutable references:

```rust
IO::read(|cache: &Cache| cache.get(key))
IO::write(|cache: &Cache| cache.set(key, value))
```

## Environment Extraction

IO helpers use `AsRef<T>` to extract a service from a larger application environment:

```rust
#[derive(Clone)]
struct AppEnv {
    db: Database,
    cache: Cache,
}

impl AsRef<Database> for AppEnv {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

impl AsRef<Cache> for AppEnv {
    fn as_ref(&self) -> &Cache {
        &self.cache
    }
}
```

The closure parameter tells the compiler which service type to extract:

```rust
let user_effect = IO::read(|db: &Database| db.find_user(id));
let cache_effect = IO::write(|cache: &Cache| cache.set(key, value));
```

This keeps the effect code decoupled from the concrete layout of `AppEnv`.

## Mutation Pattern

Because the environment is immutable, mutable services should use interior mutability or handle types that are already designed for shared access.

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Cache {
    entries: Arc<Mutex<HashMap<String, String>>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        self.entries.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.entries.lock().unwrap().insert(key, value);
    }
}

let read_effect = IO::read(|cache: &Cache| {
    cache.get("theme")
});

let write_effect = IO::write(|cache: &Cache| {
    cache.set("theme".to_string(), "dark".to_string());
});
```

This pattern aligns with async Rust service handles such as database pools and HTTP clients, which are commonly cloneable and internally synchronized.

## Naming Alternatives Considered

### `read` / `write`

```rust
IO::read(|db: &Database| db.find_user(id))
IO::write(|repo: &UserRepository| repo.save_user(user))
```

Pros:

- Clear across files, databases, network clients, caches, and loggers.
- Matches common I/O vocabulary.
- Short and easy to scan in effect pipelines.
- Communicates conceptual side effect without exposing mutable borrows.

Cons:

- `write` can sound storage-specific, even though it also covers commands such as sending email or publishing events.
- Operations that both read and write require judgment.

Verdict: best default.

### `access` / `modify`

```rust
IO::access(|db: &Database| db.find_user(id))
IO::modify(|repo: &UserRepository| repo.save_user(user))
```

Pros:

- General enough for any resource.
- `modify` clearly suggests state change.

Cons:

- `access` is vague.
- Less familiar than read/write.

Verdict: reasonable but less readable.

### `get` / `set`

```rust
IO::get(|cache: &Cache| cache.get(key))
IO::set(|cache: &Cache| cache.set(key, value))
```

Pros:

- Short and familiar.

Cons:

- Too narrow for deletes, inserts, publishes, and sends.
- Suggests property access rather than general I/O.

Verdict: too limited.

### `run` / `run_mut`

```rust
IO::run(|db: &Database| db.find_user(id))
IO::run_mut(|repo: &UserRepository| repo.save_user(user))
```

Pros:

- Generic.
- Familiar Rust suffix pattern.

Cons:

- Loses the semantic distinction between query-like and command-like operations.
- `run_mut` would be misleading because the current API does not pass `&mut T`.

Verdict: too generic and no longer matches the implementation model.

### Database-Oriented Names

Database APIs often use names equivalent to "query" for reads and "execute" for writes. That convention is familiar in database code, but Stillwater's IO helpers are not database-specific. They also wrap file systems, caches, message buses, HTTP clients, loggers, and application services.

Verdict: too database-centric for a general effect library.

## Comparison Matrix

| Option | Clarity | Generality | Matches Current Env Model | Verdict |
|--------|---------|------------|---------------------------|---------|
| `read` / `write` | High | High | Yes | Accepted |
| `access` / `modify` | Medium | High | Yes | Acceptable but less clear |
| `get` / `set` | Medium | Low | Yes | Too limited |
| `run` / `run_mut` | Low | High | No | Misleading |
| Database-oriented names | Medium | Low | Mixed | Too narrow |

## Real-World Examples

### Database

```rust
IO::read(|db: &Database| db.find_user(id))
IO::write(|db: &Database| db.insert_user(user))
```

### Cache

```rust
IO::read(|cache: &Cache| cache.get(key))
IO::write(|cache: &Cache| cache.set(key, value))
```

### Logger

```rust
IO::read(|logger: &Logger| logger.level())
IO::write(|logger: &Logger| logger.info("user registered"))
```

### File System Abstraction

```rust
IO::read(|fs: &FileSystem| fs.read_to_string(path))
IO::write(|fs: &FileSystem| fs.write_string(path, contents))
```

### Async HTTP Client

```rust
IO::read_async(|client: &HttpClient| async move {
    client.get_json(url).await
})

IO::write_async(|client: &HttpClient| async move {
    client.post_json(url, body).await
})
```

## Decision

Use `read` and `write` as the conceptual split:

```rust
IO::read(...)
IO::write(...)
IO::read_async(...)
IO::write_async(...)
```

The helpers intentionally use `AsRef<T>` and immutable service references. This keeps environments shareable and compatible with the broader `Effect` model. Services that need mutation should provide internally safe mutation, not require `&mut Env`.

## Consequences

Positive:

- The API works consistently with `Effect::run(&env)`.
- Environments remain easy to share across async and parallel effects.
- Service extraction is explicit through `AsRef<T>`.
- The names are understandable outside database code.

Tradeoffs:

- `write` is semantic rather than enforced by the borrow type.
- Users must understand that mutation requires interior mutability or shared service handles.
- A composite environment can only implement one `AsRef<T>` per service type, so duplicate services of the same concrete type may need newtype wrappers.

## Guidance

Prefer `IO::read` for:

- Lookup operations
- Configuration access
- Cache reads
- Database selects
- HTTP GET-like operations

Prefer `IO::write` for:

- Inserts, updates, and deletes
- Cache writes
- Logging
- Event publishing
- Email sending
- HTTP POST/PUT/PATCH-like operations

Use `from_fn` or `from_async` directly when the operation needs the whole environment or when service extraction through `AsRef<T>` is not the right fit.
