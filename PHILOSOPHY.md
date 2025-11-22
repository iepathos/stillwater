# Stillwater Philosophy

## The Name

**Stillwater** is more than a name - it's a mental model:

```
       Still Waters
      ╱            ╲
    Pure          Water
   (Logic)       (Effects)
     ↓              ↓
  Unchanging     Flowing
 Predictable    Performing I/O
  Testable      At boundaries
```

Like a still pond with water flowing through it, your application should have:
- **Still core**: Pure business logic that doesn't change
- **Flowing water**: Effects that move data in and out

## Core Beliefs

### 1. Pure Core, Imperative Shell

**The Problem:**
Most code mixes business logic with I/O, making it hard to:
- Test (need to mock databases, APIs, filesystems)
- Reason about (what does this function *actually* do?)
- Reuse (tightly coupled to infrastructure)

**The Stillwater Way:**
```rust
// ❌ Typical mixed code
fn process_user(id: UserId, db: &Database) -> Result<User, Error> {
    let user = db.fetch_user(id)?;  // I/O
    if user.age < 18 {              // Logic
        return Err(Error::TooYoung);
    }
    let discount = if user.premium { // Logic
        0.15
    } else {
        0.05
    };
    user.discount = discount;        // Logic
    db.save_user(&user)?;           // I/O
    Ok(user)
}

// ✓ Stillwater separated
// Pure logic (the "still" core)
fn calculate_discount(user: &User) -> f64 {
    if user.premium { 0.15 } else { 0.05 }
}

fn validate_age(age: u8) -> Result<(), Error> {
    if age >= 18 { Ok(()) } else { Err(Error::TooYoung) }
}

fn apply_discount(user: User, discount: f64) -> User {
    User { discount, ..user }
}

// Effects (the "water" shell)
fn process_user_effect(id: UserId) -> Effect<User, Error, AppEnv> {
    IO::query(|db| db.fetch_user(id))           // I/O
        .and_then(|user| {
            validate_age(user.age)?;            // Pure!
            let discount = calculate_discount(&user); // Pure!
            let updated = apply_discount(user, discount); // Pure!
            Effect::pure(updated)
        })
        .and_then(|user| {
            IO::execute(|db| db.save_user(&user)) // I/O
                .map(|_| user)
        })
}
```

**Benefits:**
- Pure functions: 100% testable, no mocks
- Clear data flow: see exactly what transforms what
- Reusable logic: discount calculation works anywhere
- Easy to reason about: no hidden side effects

### 2. Fail Fast vs Fail Completely

**The Problem:**
Validation usually stops at the first error. User submits a form with 5 fields, gets "email invalid" error, fixes it, submits again, gets "password too weak", etc. Frustrating!

**The Stillwater Way:**
```rust
// ❌ Standard Result: stops at first error
fn validate_user(input: UserInput) -> Result<User, Error> {
    let email = validate_email(&input.email)?;     // Stops here if invalid
    let password = validate_password(&input.pwd)?; // Never reached
    let age = validate_age(input.age)?;           // Never reached
    Ok(User { email, password, age })
}

// ✓ Stillwater: accumulates ALL errors
fn validate_user(input: UserInput) -> Validation<User, Vec<Error>> {
    Validation::all((
        validate_email(&input.email),
        validate_password(&input.pwd),
        validate_age(input.age),
    ))
    .map(|(email, password, age)| User { email, password, age })
}
// Returns: Err(vec![EmailError, PasswordError, AgeError])
```

**When to use which:**
- **Result (fail fast)**: Sequential operations where later steps depend on earlier
  - Example: Fetch user, then fetch their orders
- **Validation (fail completely)**: Independent validations that should all be checked
  - Example: Form validation, config validation

### 3. Errors Should Tell Stories

**The Problem:**
Deep call stacks lose context:
```
Error: No such file or directory
```
Which file? What were we trying to do? Why?

**The Stillwater Way:**
```rust
fetch_config()
    .context("Loading application configuration")
    .and_then(|cfg| parse_config(cfg))
    .context("Parsing YAML configuration")
    .and_then(|cfg| validate_config(cfg))
    .context("Validating configuration values")
```

Error output:
```
Error: No such file or directory
  -> Loading application configuration
  -> Parsing YAML configuration
  -> Validating configuration values
```

Now you know exactly what failed and why.

### 4. Composition Over Complexity

**The Problem:**
Large functions that do everything are hard to test and understand.

**The Stillwater Way:**
Build complex behavior from simple, composable pieces:

```rust
// Small, focused, pure functions
fn parse_line(line: &str) -> Result<Record, ParseError>;
fn validate_record(rec: Record) -> Validation<ValidRecord, Vec<Error>>;
fn enrich_record(rec: ValidRecord, ref_data: &RefData) -> EnrichedRecord;
fn aggregate(records: Vec<EnrichedRecord>) -> Report;

// Compose them
fn pipeline(input: String, ref_data: RefData) -> Effect<Report, Error, Env> {
    input.lines()
        .map(parse_line)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(validate_record)
        .collect::<Validation<Vec<_>, _>>()?
        .into_iter()
        .map(|r| enrich_record(r, &ref_data))
        .collect()
        |> aggregate
        |> Effect::pure
}
```

Each piece:
- Does one thing
- Is easily testable
- Is reusable
- Has clear types

### 5. Types Guide, Don't Restrict

**The Problem:**
Heavy type machinery (HKTs, complex traits) makes code hard to understand and compile errors cryptic.

**The Stillwater Way:**
Use types to make wrong code hard to write, but keep them simple:

```rust
// Effect<T, E, Env> tells you:
// - T: what it produces
// - E: how it can fail
// - Env: what it needs to run

// You can't:
// - Run an effect without environment (compiler error)
// - Mix effects with different environments (type mismatch)
// - Forget to handle errors (must call .run())

// But you can:
// - Understand what's happening (no magic)
// - Get clear error messages (simple types)
// - Refactor safely (types guide you)
```

### 6. Pragmatism Over Purity

**The Stillwater Way:**
We're not trying to be Haskell. We're trying to be **better Rust**.

**What we DON'T do:**
- ❌ Force monad abstraction (Rust doesn't have HKTs)
- ❌ Fight the borrow checker (work with ownership)
- ❌ Replace standard library (integrate with Result/Option)
- ❌ Macro-heavy DSLs (prefer clear code)

**What we DO:**
- ✓ Provide concrete, useful types
- ✓ Work with `?` operator
- ✓ Zero-cost via generics
- ✓ Integrate with async/await
- ✓ Help you write better Rust

## Design Decisions

### Why not full Monad abstraction?

**Short answer:** Impossible in current Rust, and not actually needed.

**Long answer:**
Rust lacks Higher-Kinded Types (HKTs), making generic monad abstraction impossible. But that's okay! We don't need to abstract over "all monads" - we need concrete types that solve real problems:

- `Validation<T, E>` for error accumulation
- `Effect<T, E, Env>` for effect composition
- Each optimized for its use case

This is more Rusty anyway.

### Why Box in some places?

We minimize boxing, but use it strategically:

```rust
// Effect uses Box for the function
pub struct Effect<T, E, Env> {
    run_fn: Box<dyn FnOnce(&Env) -> Result<T, E>>,
}
```

**Why:**
- Allows recursive/self-referential effects
- Heap allocation only at creation, not per combinator
- Single allocation for entire effect chain (optimized)

**Alternative considered:**
Using generics everywhere means infinite type recursion:
```rust
Effect<T, E, Env, F1>
  .and_then(...)  -> Effect<U, E, Env, F2>
  .and_then(...)  -> Effect<V, E, Env, F3>
  // Type signature explodes
```

Boxing gives us finite, predictable types.

### Why separate Validation and Effect?

**Short answer:** Different use cases, different trade-offs.

**Validation:**
- Accumulates ALL errors (Applicative)
- Independent checks run in parallel
- Example: Form validation

**Effect:**
- Short-circuits on first error (Monad)
- Sequential operations with dependencies
- Example: Database queries

Having both gives you the right tool for each job.

### Why context instead of anyhow/eyre?

Stillwater's context is **composable** with effects:

```rust
// Automatic context propagation through effect chains
fetch_user(id)
    .context("Fetching user")
    .and_then(|user| process(user))
    .context("Processing")
    .run(&env)?

// vs anyhow (manual threading)
let user = fetch_user(id)
    .context("Fetching user")?;
let result = process(user)
    .context("Processing")?;
```

Also, `ContextError<E>` preserves the underlying error type, allowing pattern matching.

## When NOT to use Stillwater

**Don't use Stillwater if:**

1. **Your code is already simple**
   - Simple CRUD? Standard Result is fine
   - No complex validation? Don't overcomplicate

2. **You need maximum performance**
   - Hot path in game loop? Hand-optimize
   - Embedded system? Every allocation matters
   - Profile first, then decide

3. **Your team doesn't buy in**
   - Stillwater requires understanding the philosophy
   - If team prefers imperative, don't force it
   - Consistency > theoretical purity

4. **The problem is a better fit for other tools**
   - Need state machines? Use `typestate` pattern
   - Need async streams? Use `futures::Stream`
   - Need actors? Use `actix` or `tokio::mpsc`

## When TO use Stillwater

**Use Stillwater when:**

1. **You have complex validation**
   - Forms with many fields
   - Config files with multiple rules
   - API inputs that need comprehensive errors

2. **You want testable business logic**
   - Complex calculations
   - Business rules
   - Data transformations

3. **You have deep call stacks**
   - Need error context
   - Want clear failure trails
   - Debug production errors

4. **You value maintainability**
   - Code will live for years
   - Team is learning functional patterns
   - Want self-documenting types

## Mental Models

### The Pond Model

Imagine a still pond with streams flowing in and out:

```
  Stream In              Stream Out
     (I/O)                 (I/O)
       ↓                     ↑
    ┌─────────────────────┐
    │                     │
    │   Still  Water     │ ← Pure logic happens here
    │                     │   (calm, predictable)
    │   (Your Business)   │
    │                     │
    └─────────────────────┘
```

- **Streams** = Effects (IO, network, filesystem)
- **Pond** = Pure logic (transformations, calculations)
- **Still** = Unchanging, predictable, testable

### The Validation Funnel

```
  Input 1 ──→ ✓ or ✗ ─┐
  Input 2 ──→ ✓ or ✗ ─┤
  Input 3 ──→ ✓ or ✗ ─┼─→ All ✓ → Success
  Input 4 ──→ ✓ or ✗ ─┤   Any ✗ → All errors
  Input 5 ──→ ✓ or ✗ ─┘
```

Don't stop at first `✗` - collect them all!

### The Railway Tracks

```
Success Track:  ──→ transform ──→ validate ──→ save ──→ ✓
                         ↓            ↓          ↓
Error Track:            ✗ ──────────────────────────→ ✗
```

Once you hit an error, you're on the error track. Context accumulates along the way.

## Further Reading

- [DESIGN.md](./DESIGN.md) - Detailed API design and examples
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - Development roadmap
- **Railway Oriented Programming** - Scott Wlaschin
- **Functional Core, Imperative Shell** - Gary Bernhardt
- **Parse, don't validate** - Alexis King

---

*"The stillness at the center of your code is where truth lives. Effects are just water flowing around it."*
