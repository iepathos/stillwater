# IO API Analysis - query vs execute

## Current Design

```rust
// Read-only operation (immutable borrow)
IO::query(|db: &Database| db.find_user(id))

// Mutating operation (mutable borrow)
IO::execute(|db: &mut Database| db.save_user(&user))
```

**Intention:**
- `query` = read-only, no side effects
- `execute` = writes/mutations, has side effects

---

## Is This Clear Enough?

### Arguments FOR Current Naming

**1. Database Familiarity**
```sql
-- Query = read
SELECT * FROM users WHERE id = 1;

-- Execute = write
INSERT INTO users VALUES (...);
UPDATE users SET ...;
DELETE FROM users WHERE ...;
```

Most developers know "query" = read from databases.

**2. CQRS Pattern**
- Command/Query Responsibility Segregation
- "Query" = read-only
- "Command" = mutates state
- Well-known pattern in backend development

**3. Rust Conventions**
```rust
// Immutable borrow
fn query(&self) -> Data

// Mutable borrow
fn execute(&mut self) -> ()
```

The `&` vs `&mut` distinction is fundamental to Rust.

**4. Self-Documenting**
```rust
// It's obvious this reads
IO::query(|cache: &Cache| cache.get(key))

// It's obvious this writes
IO::execute(|cache: &mut Cache| cache.set(key, value))
```

---

### Arguments AGAINST Current Naming

**1. "Execute" is Vague**

What does "execute" mean?
- Execute a command?
- Execute a transaction?
- Execute arbitrary code?

It doesn't clearly communicate "mutation" or "side effect."

**2. Not All Queries Are Reads**

```rust
// This "queries" but also mutates (connection pool)
IO::query(|db: &Database| {
    db.run_query("SELECT ...") // Internally mutates pool
})

// This "executes" but might not mutate
IO::execute(|logger: &mut Logger| {
    logger.log("message") // Appends to buffer, but conceptually a write
})
```

The naming suggests one thing, but Rust's borrowing tells the truth.

**3. Overloaded Terms**

In SQL databases:
- "query" can be any SQL statement
- "execute" is a method name, not a concept

In general programming:
- "execute" = run code (too generic)
- "query" = ask a question (specific)

**4. Confusion with Environment Access**

```rust
// Which do I use for reading config?
IO::query(|env: &AppEnv| env.config.database_url)

// Or this?
IO::execute(|env: &AppEnv| env.config.database_url)
```

Answer: `query` (immutable borrow), but it's not obvious from the name.

---

## Alternative Naming Schemes

### Option 1: read / write

```rust
IO::read(|db: &Database| db.find_user(id))
IO::write(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ Crystal clear: read = get data, write = change data
- ✅ Matches filesystem operations (read file, write file)
- ✅ Matches networking (read socket, write socket)
- ✅ Universal concept across all I/O

**Cons:**
- ⚠️ "write" might imply only writes to storage
- ⚠️ What about operations that do both? (read-modify-write)

**Verdict:** Very clear, but "write" might be too specific.

---

### Option 2: access / modify

```rust
IO::access(|db: &Database| db.find_user(id))
IO::modify(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ "access" = read-only access
- ✅ "modify" = clearly indicates mutation
- ✅ Works for any resource, not just databases

**Cons:**
- ⚠️ "access" is a bit generic (access to do what?)
- ⚠️ Less familiar than read/write

**Verdict:** Clear, but slightly verbose.

---

### Option 3: get / set

```rust
IO::get(|db: &Database| db.find_user(id))
IO::set(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ Very short
- ✅ Familiar from getters/setters
- ✅ Clear pairing

**Cons:**
- ❌ "set" is wrong for operations like "delete" or "increment"
- ❌ Too simplistic for complex operations
- ❌ Implies property access, not general I/O

**Verdict:** Too limiting.

---

### Option 4: from / to

```rust
IO::from(|db: &Database| db.find_user(id))  // Get data from env
IO::to(|db: &mut Database| db.save_user(&user))  // Send data to env
```

**Pros:**
- ✅ Directional: from env, to env
- ✅ Clear data flow

**Cons:**
- ❌ Reads awkwardly: "IO from database"?
- ❌ Not idiomatic Rust
- ❌ Confusing with From/Into traits

**Verdict:** Creative but awkward.

---

### Option 5: run / run_mut

```rust
IO::run(|db: &Database| db.find_user(id))
IO::run_mut(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ Follows Rust convention (`_mut` suffix)
- ✅ Generic: "run this with environment"
- ✅ Familiar pattern from stdlib

**Cons:**
- ⚠️ "run" is very generic
- ⚠️ Doesn't communicate read vs write intent
- ⚠️ `run_mut` is a mouthful

**Verdict:** Idiomatic but loses semantic meaning.

---

### Option 6: query / mutate

```rust
IO::query(|db: &Database| db.find_user(id))
IO::mutate(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ "mutate" is clearer than "execute"
- ✅ "query" is familiar
- ✅ Clear distinction: one mutates, one doesn't

**Cons:**
- ⚠️ "query" still database-focused
- ⚠️ "mutate" might sound scary to some

**Verdict:** Better than current, but still database-centric.

---

### Option 7: with / with_mut

```rust
IO::with(|db: &Database| db.find_user(id))
IO::with_mut(|db: &mut Database| db.save_user(&user))
```

**Pros:**
- ✅ Follows Rust naming (RefCell::borrow / borrow_mut)
- ✅ Generic: "with this resource, do..."
- ✅ Common pattern in Rust

**Cons:**
- ⚠️ Very generic, loses intent
- ⚠️ "with" doesn't indicate I/O
- ⚠️ Looks like a combinator, not an effect

**Verdict:** Too generic, doesn't signal I/O.

---

## Comparison Matrix

| Option | Clarity | Familiarity | Generality | Rust Idioms | Verdict |
|--------|---------|-------------|------------|-------------|---------|
| **query / execute** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⚠️ Current |
| **read / write** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ✅ **Best** |
| **access / modify** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ Good |
| **get / set** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ❌ Too limited |
| **from / to** | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐ | ❌ Awkward |
| **run / run_mut** | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⚠️ Too generic |
| **query / mutate** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ Good |
| **with / with_mut** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ❌ Too generic |

---

## What Do Other Libraries Do?

### Rust Database Libraries

**SQLx:**
```rust
sqlx::query("SELECT ...").fetch_one(&pool)  // Generic "query"
sqlx::query("INSERT ...").execute(&pool)    // "execute" for mutations
```

**Diesel:**
```rust
users.filter(...).load(&conn)    // "load" = read
diesel::insert_into(...).execute(&conn)  // "execute" = write
```

**Tokio Postgres:**
```rust
client.query("SELECT ...", &[])     // "query" = read
client.execute("INSERT ...", &[])   // "execute" = write
```

**Pattern:** Database libraries use `query` for reads, `execute` for writes.

### Effect Systems

**Scala ZIO:**
```scala
ZIO.attempt(db.findUser(id))  // Generic "attempt" for any effect
```

**Haskell IO:**
```haskell
getUser :: IO User          -- Generic "IO" monad
saveUser :: User -> IO ()
```

**Pattern:** Effect systems often don't distinguish read/write at the type level.

### Rust Standard Library

**File I/O:**
```rust
std::fs::read("file.txt")      // "read" for reading
std::fs::write("file.txt", data)  // "write" for writing
```

**RefCell:**
```rust
refcell.borrow()      // Immutable access
refcell.borrow_mut()  // Mutable access
```

**Pattern:** Rust uses clear, specific names (read/write, borrow/borrow_mut).

---

## Real-World Usage Examples

Let's test each naming scheme with actual code:

### Scenario 1: Database Operations

```rust
// Current: query / execute
IO::query(|db: &Database| db.find_user(id))
IO::execute(|db: &mut Database| db.save_user(&user))

// Alternative: read / write
IO::read(|db: &Database| db.find_user(id))
IO::write(|db: &mut Database| db.save_user(&user))

// Alternative: access / modify
IO::access(|db: &Database| db.find_user(id))
IO::modify(|db: &mut Database| db.save_user(&user))
```

**Winner:** `read/write` - clearest intent

### Scenario 2: Logger

```rust
// Current: query / execute
IO::query(|logger: &Logger| logger.get_level())
IO::execute(|logger: &mut Logger| logger.log("message"))

// Alternative: read / write
IO::read(|logger: &Logger| logger.get_level())
IO::write(|logger: &mut Logger| logger.log("message"))

// Alternative: access / modify
IO::access(|logger: &Logger| logger.get_level())
IO::modify(|logger: &mut Logger| logger.log("message"))
```

**Winner:** `read/write` - "write to logger" is very clear

### Scenario 3: Cache

```rust
// Current: query / execute
IO::query(|cache: &Cache| cache.get(key))
IO::execute(|cache: &mut Cache| cache.set(key, value))

// Alternative: read / write
IO::read(|cache: &Cache| cache.get(key))
IO::write(|cache: &mut Cache| cache.set(key, value))

// Alternative: access / modify
IO::access(|cache: &Cache| cache.get(key))
IO::modify(|cache: &mut Cache| cache.set(key, value))
```

**Winner:** `read/write` - matches get/set semantics

### Scenario 4: File Operations

```rust
// Current: query / execute
IO::query(|fs: &FileSystem| fs.read_file(path))
IO::execute(|fs: &mut FileSystem| fs.write_file(path, data))

// Alternative: read / write
IO::read(|fs: &FileSystem| fs.read_file(path))
IO::write(|fs: &mut FileSystem| fs.write_file(path, data))

// Alternative: access / modify
IO::access(|fs: &FileSystem| fs.read_file(path))
IO::modify(|fs: &mut FileSystem| fs.write_file(path, data))
```

**Winner:** `read/write` - matches the operation names perfectly

### Scenario 5: Configuration

```rust
// Current: query / execute
IO::query(|config: &Config| config.database_url.clone())
IO::execute(|config: &mut Config| config.set_timeout(30))

// Alternative: read / write
IO::read(|config: &Config| config.database_url.clone())
IO::write(|config: &mut Config| config.set_timeout(30))

// Alternative: access / modify
IO::access(|config: &Config| config.database_url.clone())
IO::modify(|config: &mut Config| config.set_timeout(30))
```

**Winner:** `read/write` or `access/modify` both work well

---

## The Environment Problem

How does IO access specific services from the environment?

### Current Approach (Type-based)

```rust
// How does this work?
IO::query(|db: &Database| db.find_user(id))

// Does env need to be Database?
struct AppEnv {
    db: Database,
    cache: Cache,
    logger: Logger,
}

// Or do we extract via trait?
trait HasDatabase {
    fn database(&self) -> &Database;
}

impl HasDatabase for AppEnv {
    fn database(&self) -> &Database {
        &self.db
    }
}
```

This is **underspecified** in our current design!

### Option A: Direct Field Access

```rust
IO::read(|env: &AppEnv| &env.db.find_user(id))
//               ^^^^^    ^^^^^ - access db field
```

**Pros:** Simple, obvious
**Cons:** Couples to AppEnv structure

### Option B: Trait Extraction

```rust
IO::read_db(|db: &Database| db.find_user(id))
//      ^^^ - specialized for Database

// Implementation uses HasDatabase trait
```

**Pros:** Type-safe, decoupled
**Cons:** Need method for each service type?

### Option C: Generic Extraction

```rust
// User provides extraction function
Effect::from_env(|env| &env.db)
    .and_then(|db| Effect::pure(db.find_user(id)))
```

**Pros:** Flexible
**Cons:** Verbose

**We need to decide this!**

---

## Recommendation

### Primary Change: read / write

**Replace:**
```rust
IO::query(...)   // ❌ Database-centric
IO::execute(...) // ❌ Vague
```

**With:**
```rust
IO::read(...)    // ✅ Clear: reading data
IO::write(...)   // ✅ Clear: writing/mutating data
```

**Why:**
1. ✅ **Universal clarity** - works for any I/O (files, DB, network, cache)
2. ✅ **Matches Rust stdlib** - `std::fs::read`, `std::fs::write`
3. ✅ **Obvious intent** - read = no side effects, write = side effects
4. ✅ **Short and sweet** - 4-5 characters each
5. ✅ **Natural pairing** - everyone understands read/write

### Secondary Option: access / modify

If "write" feels too specific (though I don't think it is):

```rust
IO::access(...)  // Read-only access
IO::modify(...)  // Modifying access
```

This is also clear, just slightly more verbose.

### Environment Access Pattern

**Recommend Option B (Trait-based) with helper methods:**

```rust
// Provide common helpers
impl IO {
    pub fn read<T, R, F>(f: F) -> Effect<R, E, impl AsRef<T>>
    where
        F: FnOnce(&T) -> R,
    {
        Effect::from_fn(move |env| Ok(f(env.as_ref())))
    }

    pub fn write<T, R, F>(f: F) -> Effect<R, E, impl AsMut<T>>
    where
        F: FnOnce(&mut T) -> R,
    {
        Effect::from_fn(move |env| {
            let mut_ref = env.as_mut();
            Ok(f(mut_ref))
        })
    }
}

// User implements AsRef/AsMut:
impl AsRef<Database> for AppEnv {
    fn as_ref(&self) -> &Database {
        &self.db
    }
}

// Usage:
IO::read(|db: &Database| db.find_user(id))
//              ^^^^^^^^ - type inference figures out we need Database
```

---

## Final Verdict

**Current naming is NOT clear enough.**

**Problems:**
1. "query" is database-centric
2. "execute" is vague
3. Doesn't work well for non-DB I/O

**Solution:**
```rust
// Before
IO::query(|db: &Database| db.find_user(id))
IO::execute(|db: &mut Database| db.save_user(&user))

// After
IO::read(|db: &Database| db.find_user(id))
IO::write(|db: &mut Database| db.save_user(&user))
```

**Benefits:**
- ✅ Crystal clear for all I/O types
- ✅ Matches Rust conventions
- ✅ Universal concept (files, network, DB, cache, logger)
- ✅ Obvious which one mutates

**This is a clear improvement we should make.**

---

*Answer: No, `query/execute` is not clear enough. Switch to `read/write`.*
