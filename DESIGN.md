# Stillwater - Design Document

## Philosophy

**Stillwater** embodies the principle of **pure core, imperative shell**:
- **Still** = Pure functions (unchanging, referentially transparent, predictable)
- **Water** = Effects (flowing, moving, performing I/O)

Like a still pond with water flowing through it, stillwater keeps your pure business logic calm and testable while effects flow at the boundaries.

## Core Problems Solved

### 1. Effect Separation
Pure business logic mixed with I/O makes testing painful and reasoning difficult.

**Stillwater's approach:** Make effects explicit in types, push I/O to boundaries.

### 2. Validation Accumulation
Standard Result/Option stops at first error. Forms need ALL errors at once.

**Stillwater's approach:** `Validation` type that accumulates errors using semigroups.

### 3. Error Context Loss
Errors bubble up losing "why" and "where" context.

**Stillwater's approach:** Context chaining that preserves error trails.

### 4. Dependency Threading
Passing config, database, logger through every function is verbose and brittle.

**Stillwater's approach:** Reader pattern for implicit environment access.

### 5. Effect Composition
Combining `Result<Option<T>>`, async, and other effects is painful.

**Stillwater's approach:** Composable effect types with clean combinators.

## Design Principles

### 1. Rust-First, Not Haskell-in-Rust
- Work with ownership and borrowing, not against it
- Integrate with `?` operator, don't replace it
- Use concrete types, not impossible abstractions (no HKT requirements)
- Zero-cost via generics and monomorphization

### 2. Progressive Disclosure
Simple things should be simple, complex things possible:
```rust
// Level 1: Just like Result
validate_email(input)?

// Level 2: Accumulate errors
Validation::all((email, age, name))?

// Level 3: Full effect composition
Effect::from_validation(validated)
    .and_then(save_to_db)
    .run(&env)?
```

### 3. Pure Core, Imperative Shell
Make the pattern ergonomic:
- Pure functions take data, return data
- Effects are explicitly typed
- I/O happens at boundaries via `run()`

### 4. Testability First
- Pure functions need zero mocking
- Effects run with test environments
- Composition is deterministic

### 5. No Magic
- No heavy macros (monadic crate style)
- Zero-cost by default, boxing only when type erasure is needed
- Clear types, obvious behavior
- Error messages that help

## Core Types

### Effect Trait (Zero-Cost)

Stillwater uses a trait-based Effect design following the `futures` crate pattern:
**zero-cost by default, explicit boxing when needed**.

```rust
/// Core effect trait - implemented by all effect combinators
pub trait Effect {
    type Output;
    type Error;
    type Env;

    /// Run the effect, consuming it
    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>>;
}
```

**Why a trait?**

Each combinator returns a concrete type that implements `Effect`:

```rust
use stillwater::prelude::*;

pure(42)            // Returns: Pure<i32, E, Env>
    .map(|x| x + 1) // Returns: Map<Pure<...>, F>
    .and_then(...)  // Returns: AndThen<Map<...>, F>
```

This enables:
- **Zero heap allocations** for effect chains
- **Full inlining** by the compiler
- **Predictable performance** characteristics

### BoxedEffect (When Type Erasure is Needed)

When you need type erasure, use `.boxed()`:

```rust
pub type BoxedEffect<T, E, Env> = Box<dyn Effect<Output = T, Error = E, Env = Env> + Send>;
```

Use boxing for:
- Collections of effects
- Recursive effects
- Match arms with different effect types

### Constructors

Effects are created using free functions:

```rust
use stillwater::prelude::*;

// Pure value (no effects)
let effect = pure::<_, String, ()>(42);

// Failure
let effect = fail::<i32, _, ()>("error".to_string());

// From synchronous function
let effect = from_fn(|env: &Env| Ok::<_, String>(env.value));

// From async function
let effect = from_async(|env: &Env| async { Ok(env.fetch().await) });

// From Result
let effect = from_result::<_, String, ()>(Ok(42));

// From Option
let effect = from_option::<_, _, ()>(Some(42), || "missing");

// From Validation
let effect = from_validation(validation);
```

### Extension Trait (EffectExt)

All Effect types get combinator methods via `EffectExt`:

```rust
use stillwater::{Effect, EffectExt};

/// Extension trait providing combinator methods
pub trait EffectExt: Effect {
    /// Transform success value
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U;

    /// Transform error value
    fn map_err<F, E2>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E2;

    /// Chain dependent computations
    fn and_then<F, E2>(self, f: F) -> AndThen<Self, F>
    where
        F: FnOnce(Self::Output) -> E2,
        E2: Effect<Error = Self::Error, Env = Self::Env>;

    /// Recover from errors
    fn or_else<F, E2>(self, f: F) -> OrElse<Self, F>
    where
        F: FnOnce(Self::Error) -> E2,
        E2: Effect<Output = Self::Output, Env = Self::Env>;

    /// Type-erase to BoxedEffect
    fn boxed(self) -> BoxedEffect<Self::Output, Self::Error, Self::Env>;

    /// Side effect without modifying the value
    fn tap<F, E2>(self, f: F) -> Tap<Self, F>
    where
        F: FnOnce(&Self::Output) -> E2;

    /// Combine with another effect
    fn with<F, E2>(self, f: F) -> With<Self, F>
    where
        F: FnOnce(&Self::Output) -> E2;
}
```

### Validation<T, E>

Non-short-circuiting validation that accumulates all errors.

```rust
pub enum Validation<T, E> {
    Success(T),
    Failure(E),
}
```

**Key Methods:**
```rust
impl<T, E> Validation<T, E> {
    /// Create success
    pub fn success(value: T) -> Self;

    /// Create failure
    pub fn failure(error: E) -> Self;

    /// Convert from Result
    pub fn from_result(result: Result<T, E>) -> Self;
}

impl<T, E> Validation<T, E>
where
    E: Semigroup,  // Can combine errors (usually Vec<Error>)
{
    /// Validate all, accumulating errors
    pub fn all<I>(validations: I) -> Validation<Vec<T>, E>
    where
        I: IntoIterator<Item = Validation<T, E>>;

    /// Apply function if valid
    pub fn map<U, F>(self, f: F) -> Validation<U, E>
    where
        F: FnOnce(T) -> U;

    /// Chain validations, accumulating errors
    pub fn and<U>(self, other: Validation<U, E>) -> Validation<(T, U), E>;

    /// Convert to Result
    pub fn into_result(self) -> Result<T, E>;

    /// Convert to Effect
    pub fn into_effect<Env>(self) -> Effect<T, E, Env>;
}
```

### Reader Pattern Helpers

The Reader pattern provides functional dependency injection, allowing effects to access environment without explicit parameter passing.

```rust
impl<T, E, Env> Effect<T, E, Env> {
    /// Get the entire environment as an Effect
    pub fn ask() -> Effect<Env, E, Env>
    where
        Env: Clone + Send;

    /// Query a specific value from the environment
    pub fn asks<F, U>(f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(&Env) -> U + Send + 'static,
        U: Send + 'static;

    /// Run an effect with a modified environment
    pub fn local<F>(f: F, effect: Effect<T, E, Env>) -> Effect<T, E, Env>
    where
        F: FnOnce(&Env) -> Env + Send + 'static,
        Env: Clone + Send;
}
```

**Design rationale:**
- `ask()` provides access to the whole environment for cases where multiple fields are needed
- `asks(f)` is more efficient, extracting only what's needed without cloning the entire environment
- `local(f, effect)` enables temporary environment modifications without mutating the original
- All three compose naturally with other Effect combinators

**Usage patterns:**

```rust
// Query configuration from environment
fn get_timeout() -> Effect<u64, Error, Config> {
    Effect::asks(|cfg: &Config| cfg.timeout)
}

// Temporarily modify environment for specific operation
fn with_extended_timeout<T>(effect: Effect<T, Error, Config>) -> Effect<T, Error, Config> {
    Effect::local(
        |cfg: &Config| Config { timeout: cfg.timeout * 2, ..*cfg },
        effect
    )
}

// Compose with other operations
fn fetch_with_config(url: String) -> Effect<Response, Error, AppEnv> {
    Effect::asks(|env: &AppEnv| env.config.timeout)
        .and_then(|timeout| fetch_with_timeout(url, timeout))
}
```

See [Reader Pattern guide](docs/guide/09-reader-pattern.md) for comprehensive examples.

### Parallel Effect Execution

Effect supports parallel execution of independent effects, enabling concurrent operations while preserving error handling and environment access patterns.

**Parallel Methods:**
```rust
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    /// Run multiple effects in parallel, collecting all results.
    /// Accumulates all errors if any fail.
    pub fn par_all<I>(effects: I) -> Effect<Vec<T>, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
        I::IntoIter: Send;

    /// Run effects in parallel with fail-fast semantics.
    /// Returns error immediately when first effect fails.
    pub fn par_try_all<I>(effects: I) -> Effect<Vec<T>, E, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
        I::IntoIter: Send;

    /// Race multiple effects, returning first successful result.
    /// Collects all errors if all effects fail.
    pub fn race<I>(effects: I) -> Effect<T, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
        I::IntoIter: Send;

    /// Run effects in parallel with concurrency limit.
    /// Useful for rate limiting or resource management.
    pub fn par_all_limit<I>(effects: I, limit: usize) -> Effect<Vec<T>, Vec<E>, Env>
    where
        I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
        I::IntoIter: Send;
}
```

**Usage patterns:**

```rust
// Fetch multiple users concurrently
fn fetch_users(ids: Vec<i32>) -> Effect<Vec<User>, DbError, AppEnv> {
    let effects = ids.into_iter().map(|id| fetch_user(id));
    Effect::par_all(effects)
}

// Race multiple data sources
fn fetch_with_fallback(url: String) -> Effect<Data, Error, AppEnv> {
    Effect::race([
        fetch_from_cache(url.clone()),
        fetch_from_primary(url.clone()),
        fetch_from_backup(url),
    ])
}

// Parallel validation with rate limiting
fn validate_batch(items: Vec<Item>) -> Effect<Vec<Result>, Error, AppEnv> {
    let validations = items.into_iter().map(|item| validate_item(item));
    Effect::par_all_limit(validations, 10) // Max 10 concurrent validations
}

// Fail-fast parallel operations
fn check_all_services() -> Effect<Vec<Status>, Error, AppEnv> {
    Effect::par_try_all([
        check_database(),
        check_cache(),
        check_queue(),
    ])
}
```

**Design characteristics:**
- Environment shared across all parallel tasks (must be `Sync`)
- True concurrency via async runtime (tokio, async-std)
- Type-safe composition with other Effect combinators
- Clear error semantics (accumulate vs fail-fast)
- Each effect in the collection has one Box allocation (negligible for I/O-bound work)

See [Parallel Effects guide](docs/guide/11-parallel-effects.md) for comprehensive examples and patterns.

### IO Module

Helper for creating I/O effects at boundaries.

```rust
pub struct IO;

impl IO {
    /// Create effect from database query
    pub fn query<T, E, F>(f: F) -> Effect<T, E, impl HasDatabase>
    where
        F: FnOnce(&Database) -> Result<T, E>;

    /// Create effect from database command
    pub fn execute<T, E, F>(f: F) -> Effect<T, E, impl HasDatabase>
    where
        F: FnOnce(&Database) -> Result<T, E>;

    /// Read file
    pub fn read_file<E>(path: impl AsRef<Path>) -> Effect<String, E, ()>
    where
        E: From<io::Error>;

    /// Write file
    pub fn write_file<E>(path: impl AsRef<Path>, content: impl AsRef<str>) -> Effect<(), E, ()>
    where
        E: From<io::Error>;
}
```

### ContextError<E>

Error wrapper that preserves context chains.

```rust
pub struct ContextError<E> {
    error: E,
    context: Vec<String>,
}

impl<E> ContextError<E> {
    /// Add context layer
    pub fn context(self, msg: impl Into<String>) -> Self;

    /// Get the underlying error
    pub fn inner(&self) -> &E;

    /// Get context trail
    pub fn context_trail(&self) -> &[String];
}

impl<E: Display> Display for ContextError<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Error: {}", self.error)?;
        for (i, ctx) in self.context.iter().enumerate() {
            writeln!(f, "  {}-> {}", "  ".repeat(i), ctx)?;
        }
        Ok(())
    }
}
```

## Usage Examples

### Example 1: Simple Validation

```rust
use stillwater::{Validation, Semigroup};

#[derive(Debug)]
enum ValidationError {
    InvalidEmail(String),
    WeakPassword,
    AgeTooYoung(u8),
}

// Errors accumulate in Vec
impl Semigroup for Vec<ValidationError> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// Pure validation functions
fn validate_email(email: &str) -> Validation<Email, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(Email(email.to_string()))
    } else {
        Validation::failure(vec![ValidationError::InvalidEmail(email.to_string())])
    }
}

fn validate_password(pwd: &str) -> Validation<Password, Vec<ValidationError>> {
    if pwd.len() >= 8 {
        Validation::success(Password(pwd.to_string()))
    } else {
        Validation::failure(vec![ValidationError::WeakPassword])
    }
}

fn validate_age(age: u8) -> Validation<Age, Vec<ValidationError>> {
    if age >= 18 {
        Validation::success(Age(age))
    } else {
        Validation::failure(vec![ValidationError::AgeTooYoung(age)])
    }
}

// Validate all fields, get ALL errors
fn validate_user_input(input: UserInput) -> Validation<UserData, Vec<ValidationError>> {
    Validation::all((
        validate_email(&input.email),
        validate_password(&input.password),
        validate_age(input.age),
    ))
    .map(|(email, password, age)| UserData { email, password, age })
}

// Usage
let input = UserInput {
    email: "not-an-email",
    password: "weak",
    age: 15,
};

match validate_user_input(input) {
    Validation::Success(user) => println!("Valid: {:?}", user),
    Validation::Failure(errors) => {
        println!("Found {} errors:", errors.len());
        for err in errors {
            println!("  - {:?}", err);
        }
    }
}
// Output:
// Found 3 errors:
//   - InvalidEmail("not-an-email")
//   - WeakPassword
//   - AgeTooYoung(15)
```

### Example 2: Effect Composition with Context

```rust
use stillwater::{Effect, IO, ContextError};

// Define environment
struct AppEnv {
    db: Database,
    config: Config,
}

// Pure business logic (easily testable)
fn calculate_discount(customer: &Customer, total: Money) -> Money {
    match customer.tier {
        Tier::Gold => total * 0.15,
        Tier::Silver => total * 0.10,
        Tier::Bronze => total * 0.05,
    }
}

fn create_invoice(order_id: OrderId, total: Money) -> Invoice {
    Invoice {
        id: InvoiceId::generate(),
        order_id,
        total,
        created_at: Utc::now(),
    }
}

// Effect composition (I/O at boundaries)
fn process_order(order_id: OrderId) -> Effect<Invoice, ContextError<AppError>, AppEnv> {
    // Fetch order from DB
    IO::query(move |db: &Database| {
        db.find_order(order_id)
            .ok_or(AppError::OrderNotFound(order_id))
    })
    .context(format!("Fetching order {}", order_id))

    // Validate order
    .and_then(|order| {
        if order.items.is_empty() {
            Effect::fail(AppError::EmptyOrder)
        } else {
            Effect::pure(order)
        }
    })
    .context("Validating order")

    // Calculate total (pure!)
    .map(|order| {
        let total: Money = order.items.iter().map(|i| i.price).sum();
        (order, total)
    })

    // Fetch customer
    .and_then(|(order, total)| {
        IO::query(move |db: &Database| {
            db.find_customer(order.customer_id)
                .ok_or(AppError::CustomerNotFound(order.customer_id))
        })
        .map(move |customer| (order, customer, total))
    })
    .context("Fetching customer")

    // Apply discount (pure!)
    .map(|(order, customer, total)| {
        let discount = calculate_discount(&customer, total);
        let final_total = total - discount;
        (order.id, final_total)
    })

    // Create invoice (pure!)
    .map(|(order_id, total)| create_invoice(order_id, total))

    // Save invoice
    .and_then(|invoice| {
        let invoice_copy = invoice.clone();
        IO::execute(move |db: &Database| {
            db.save_invoice(&invoice)
        })
        .map(move |_| invoice_copy)
    })
    .context("Saving invoice")
}

// Run at application boundary
async fn handle_request(order_id: OrderId, env: AppEnv) -> Result<Invoice, ContextError<AppError>> {
    process_order(order_id).run_async(&env).await
}

// Error output with context:
// Error: OrderNotFound(12345)
//   -> Fetching order 12345
```

### Example 3: Testing Pure vs Effects

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Pure functions: no mocking needed!
    #[test]
    fn test_calculate_discount() {
        let customer = Customer {
            tier: Tier::Gold,
            loyalty_points: 1000,
        };

        let discount = calculate_discount(&customer, Money(100.0));

        assert_eq!(discount, Money(15.0));
    }

    #[test]
    fn test_create_invoice() {
        let invoice = create_invoice(OrderId(123), Money(85.0));

        assert_eq!(invoice.order_id, OrderId(123));
        assert_eq!(invoice.total, Money(85.0));
    }

    // Effects: test with mock environment
    #[tokio::test]
    async fn test_process_order_effect() {
        let mut mock_db = MockDatabase::new();

        // Set up test data
        mock_db.insert_order(Order {
            id: OrderId(123),
            customer_id: CustomerId(456),
            items: vec![
                Item { price: Money(50.0) },
                Item { price: Money(30.0) },
            ],
        });

        mock_db.insert_customer(Customer {
            id: CustomerId(456),
            tier: Tier::Gold,
            loyalty_points: 1000,
        });

        let env = AppEnv {
            db: mock_db,
            config: Config::default(),
        };

        // Run the effect
        let result = process_order(OrderId(123))
            .run_async(&env)
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.total, Money(68.0)); // 80 - 15% discount
    }

    // Validation: test error accumulation
    #[test]
    fn test_validation_accumulates_all_errors() {
        let input = UserInput {
            email: "not-valid",
            password: "weak",
            age: 15,
        };

        let result = validate_user_input(input);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 3);
                assert!(matches!(errors[0], ValidationError::InvalidEmail(_)));
                assert!(matches!(errors[1], ValidationError::WeakPassword));
                assert!(matches!(errors[2], ValidationError::AgeTooYoung(_)));
            }
            _ => panic!("Expected validation to fail"),
        }
    }
}
```

### Example 4: Data Pipeline

```rust
use stillwater::{Effect, Validation, IO};

// Pure transformations
fn parse_csv_line(line: &str) -> Result<RawRecord, ParseError> {
    // parsing logic
}

fn validate_record(raw: RawRecord) -> Validation<ValidRecord, Vec<ValidationError>> {
    // validation logic
}

fn enrich_record(valid: ValidRecord, reference: &RefData) -> EnrichedRecord {
    // enrichment logic
}

fn aggregate_records(records: Vec<EnrichedRecord>) -> Report {
    // aggregation logic
}

// Pipeline composition
fn process_data_pipeline(
    input_path: PathBuf,
    output_path: PathBuf,
) -> Effect<Report, ContextError<PipelineError>, PipelineEnv> {
    // Read input file (I/O)
    IO::read_file(input_path.clone())
        .context(format!("Reading {}", input_path.display()))

    // Parse lines (pure, but can fail)
    .and_then(|content| {
        let lines: Vec<_> = content.lines().collect();
        let parsed: Result<Vec<_>, _> = lines
            .iter()
            .map(|line| parse_csv_line(line))
            .collect();

        Effect::from_result(parsed)
    })
    .context("Parsing CSV lines")

    // Validate all records (accumulate errors)
    .and_then(|raw_records| {
        let validations: Vec<_> = raw_records
            .into_iter()
            .map(validate_record)
            .collect();

        Validation::all(validations).into_effect()
    })
    .context("Validating records")

    // Load reference data (I/O)
    .and_then(|valid_records| {
        IO::query(|db: &Database| {
            db.load_reference_data()
        })
        .map(move |ref_data| (valid_records, ref_data))
    })
    .context("Loading reference data")

    // Enrich records (pure)
    .map(|(records, ref_data)| {
        records
            .into_iter()
            .map(|r| enrich_record(r, &ref_data))
            .collect()
    })

    // Aggregate (pure)
    .map(aggregate_records)

    // Save report (I/O)
    .and_then(move |report| {
        let report_json = serde_json::to_string_pretty(&report)
            .map_err(PipelineError::from)?;

        IO::write_file(output_path.clone(), report_json)
            .map(|_| report)
    })
    .context(format!("Writing {}", output_path.display()))
}
```

## Implementation Roadmap

### Phase 1: Core Types (MVP)
- [ ] `Validation<T, E>` with `Semigroup` trait
- [ ] Basic `Effect<T, E, Env>` with combinators
- [ ] `ContextError<E>` wrapper
- [ ] Integration with `?` operator via `Try` trait
- [ ] Comprehensive tests
- [ ] Examples directory

### Phase 2: Ergonomics
- [ ] `IO` module for common I/O patterns
- [ ] Async support (`run_async`)
- [ ] Better error messages
- [ ] Documentation and guides
- [ ] Real-world examples

### Phase 3: Advanced Features
- [ ] `OptionT` monad transformer
- [ ] `Reader` pattern helpers
- [ ] Parallel validation/effects
- [ ] Streaming/pipeline utilities
- [ ] Benchmarks vs hand-written code

### Phase 4: Ecosystem Integration
- [ ] Integration with popular frameworks (Axum, Actix)
- [ ] Database library integration (SQLx, Diesel)
- [ ] Validation helper macros (optional)
- [ ] Testing utilities

## Success Metrics

**Adoption:**
- Used in at least 3 real projects within 6 months
- 100+ GitHub stars within first year
- Positive feedback from Rust community

**Quality:**
- Zero-cost: same assembly as hand-written code
- Comprehensive docs with examples
- <5% compile time overhead
- Clear error messages

**Impact:**
- Makes testing easier (pure functions, mock-free)
- Reduces error handling boilerplate
- Encourages pure core, imperative shell pattern
- Helps developers write clearer, more maintainable code

## Non-Goals

- ❌ Perfect monad abstraction (impossible without HKTs)
- ❌ Haskell/Scala feature parity
- ❌ Heavy macro-based DSLs
- ❌ Runtime overhead for abstractions
- ❌ Fighting Rust idioms

## Questions to Resolve

1. **Boxing vs Generics:** When to box vs when to use generics?
   - Lean toward generics for zero-cost
   - Box only when necessary (e.g., recursive types)

2. **Async story:** How deep should async integration go?
   - Start with `run_async` for Effect
   - Add streaming later if needed

3. **Validation ergonomics:** Macro for `Validation::all`?
   - Start without macros
   - Add if community requests

4. **Reader vs explicit env:** Force Reader pattern or allow both?
   - Allow both, Reader as helper
   - Don't force paradigm

## Next Steps

1. Create Cargo project structure
2. Implement core `Validation` type
3. Implement basic `Effect` type
4. Write comprehensive tests
5. Create first examples
6. Gather early feedback

---

*Stillwater: Where pure logic flows through calm waters.*
