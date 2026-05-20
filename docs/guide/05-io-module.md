# IO Module: Ergonomic Effect Creation

The IO module provides convenient helpers for creating Effects from I/O operations.

## Core Functions

### IO::read - Read-only operations

For queries that don't modify state:

```rust
use stillwater::IO;

struct Database { /* ... */ }

let effect = IO::read(|db: &Database| {
    db.fetch_user(123)
});
```

### IO::write - Mutating operations

For operations that modify state (uses interior mutability):

```rust
use stillwater::IO;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct Cache {
    data: Arc<Mutex<HashMap<u64, String>>>,
}

let effect = IO::write(|cache: &Cache| {
    cache.data.lock().unwrap().insert(key, value);
});
```

### IO::read_async and IO::write_async - Async operations

For async I/O, use `read_async` for query-style operations and `write_async` for operations that mutate through interior mutability:

```rust
use stillwater::IO;
use std::future::ready;

let read_effect = IO::read_async(|db: &Database| {
    let user = db.fetch_user(123);
    ready(user)
});

let write_effect = IO::write_async(|cache: &Cache| {
    cache.set(key, value);
    ready(())
});
```

## Environment Pattern

IO uses `AsRef<T>` for automatic dependency extraction:

```rust
struct AppEnv {
    db: Database,
    cache: Cache,
}

impl AsRef<Database> for AppEnv {
    fn as_ref(&self) -> &Database { &self.db }
}

impl AsRef<Cache> for AppEnv {
    fn as_ref(&self) -> &Cache { &self.cache }
}

// Type inference extracts the right dependency
let effect = IO::read(|db: &Database| db.fetch_user(123));
effect.run(&env).await  // AppEnv automatically provides Database
```

## Examples

See full examples in [examples/io_patterns.rs](https://github.com/iepathos/stillwater/blob/master/examples/io_patterns.rs).

## Next Steps

- Learn about [Helper Combinators](06-helper-combinators.md)
- Back to [Effects guide](03-effects.md)
