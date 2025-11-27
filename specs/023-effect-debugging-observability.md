---
number: 023
title: Effect Tracing Integration
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-11-26
---

# Specification 023: Effect Tracing Integration

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None

## Context

When Effects fail or behave unexpectedly, debugging can be challenging because:

1. **Stack traces show closures**: Error locations show `effect.rs:{closure#0}` instead of the user's source code location
2. **No execution visibility**: No way to see what's happening during Effect execution
3. **No timing information**: Can't see how long each Effect takes
4. **No correlation**: Hard to trace parallel Effects back to their origin

### Current Pain Point

```rust
// User code
let result = fetch_user(id)
    .and_then(|user| process(user))
    .and_then(|data| save(data))
    .run(&env)
    .await;

// Error shows closures, not user code:
// Error at stillwater::effect::{closure#0}
```

### The Solution: Tracing Integration

The `tracing` crate is Rust's standard observability solution. By integrating with it:

- **See the full execution flow** - What happened, in what order, how long
- **Automatic location capture** - `#[track_caller]` captures where Effects are created
- **Zero cost when disabled** - Spans compile away completely
- **Rich ecosystem** - Works with OpenTelemetry, Jaeger, console subscribers, etc.

```
// With tracing enabled:
10:23:45.001 DEBUG effect{file="workflow.rs" line=127}: executing
10:23:45.050 DEBUG effect{file="workflow.rs" line=128}: executing
10:23:45.102 ERROR effect{file="workflow.rs" line=129}: failed err="connection refused"
```

## Objective

Add comprehensive tracing integration to Stillwater Effects:

1. **`.instrument(span)`** - Wrap any Effect in a tracing span
2. **`.traced()`** - Auto-capture source location into a span
3. **`.named(name)`** - Give Effect a debug name that appears in spans
4. **Feature-gated** - Zero overhead when `tracing` feature is disabled

## Requirements

### Functional Requirements

#### FR1: Span Instrumentation
- **MUST** provide `.instrument(span)` method to wrap Effect in a tracing Span
- **MUST** enter span when Effect starts executing
- **MUST** exit span when Effect completes (success or failure)
- **MUST** work with any user-created Span

#### FR2: Automatic Location Capture
- **MUST** provide `.traced()` method that auto-captures source location
- **MUST** use `#[track_caller]` to capture caller's file and line
- **MUST** create a debug span with file and line fields

#### FR3: Named Effects
- **MUST** provide `.named(name)` method for giving Effects debug names
- **MUST** create span with the provided name
- **SHOULD** work as alias for creating a named span

#### FR4: Feature Gating
- **MUST** be behind `tracing` feature flag
- **MUST NOT** add any runtime overhead when feature is disabled
- **MUST NOT** require tracing dependency when feature is disabled

### Non-Functional Requirements

#### NFR1: Zero-Cost Abstraction
- Tracing MUST compile away completely when feature disabled
- No binary size increase when feature disabled
- No runtime overhead beyond tracing crate's own overhead

#### NFR2: Ergonomics
- Simple API - single method call to instrument
- Works with existing tracing ecosystem (subscribers, etc.)
- Composable with all existing Effect combinators

#### NFR3: Async Correctness
- Spans MUST propagate correctly across async boundaries
- Span entry/exit MUST be correct even with concurrent Effects

## Acceptance Criteria

### Basic Instrumentation

- [ ] **AC1**: `.instrument(span)` wraps Effect in span
  ```rust
  let span = tracing::info_span!("fetch_user", user_id = %id);
  let effect = fetch_user(id).instrument(span);

  // Span is entered when effect.run() executes
  let result = effect.run(&env).await;
  ```

- [ ] **AC2**: Span entered on execution, exited on completion
  ```rust
  // Setup test subscriber that records spans
  let effect = Effect::<_, String, ()>::pure(42)
      .instrument(tracing::info_span!("test"));

  effect.run(&()).await;
  // Verify span was entered and exited
  ```

- [ ] **AC3**: Span captures success/failure
  ```rust
  let effect = Effect::<i32, _, ()>::fail("error")
      .instrument(tracing::info_span!("failing_op"));

  let _ = effect.run(&()).await;
  // Span should record the error
  ```

### Auto-Location Capture

- [ ] **AC4**: `.traced()` captures caller location
  ```rust
  fn my_function() -> Effect<i32, String, ()> {
      Effect::pure(42).traced()  // Captures this file:line
  }

  // Span has file="src/lib.rs", line=<this line number>
  ```

- [ ] **AC5**: Location is from caller, not internal
  ```rust
  // In user's workflow.rs line 100:
  let effect = do_something().traced();

  // Span shows workflow.rs:100, NOT effect.rs:<internal>
  ```

### Named Effects

- [ ] **AC6**: `.named()` creates span with name
  ```rust
  let effect = fetch_user(id).named("fetch-user");

  // Equivalent to:
  // .instrument(tracing::debug_span!("effect", name = "fetch-user"))
  ```

- [ ] **AC7**: Named effects show in tracing output
  ```rust
  let effect = process_batch(items).named("batch-processor");
  effect.run(&env).await;

  // Output: DEBUG effect{name="batch-processor"}: ...
  ```

### Feature Gating

- [ ] **AC8**: Methods don't exist without feature
  ```rust
  #[cfg(not(feature = "tracing"))]
  fn test_no_tracing_methods() {
      let effect = Effect::<_, String, ()>::pure(42);
      // effect.instrument(span) - compile error: method not found
      // effect.traced() - compile error: method not found
  }
  ```

- [ ] **AC9**: Zero overhead when disabled
  - Compile without `tracing` feature
  - Binary size unchanged
  - No tracing symbols in output

### Composition

- [ ] **AC10**: Works with all combinators
  ```rust
  let effect = fetch_user(id)
      .traced()
      .and_then(|user| process(user).traced())
      .map(|result| result.id)
      .traced();

  // Each .traced() creates its own span
  ```

- [ ] **AC11**: Works with parallel execution
  ```rust
  let effects = vec![
      task1().traced(),
      task2().traced(),
      task3().traced(),
  ];

  Effect::par_all(effects).run(&env).await;
  // Each task has its own span, running concurrently
  ```

## Technical Details

### Implementation Approach

#### 1. New Tracing Module

```rust
// src/tracing_support.rs

#[cfg(feature = "tracing")]
use tracing::Span;

#[cfg(feature = "tracing")]
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Wrap this Effect in a tracing span
    ///
    /// The span is entered when the Effect executes and exited when it completes.
    /// This enables observability into Effect execution flow.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::Effect;
    /// use tracing::info_span;
    ///
    /// let effect = Effect::<_, String, ()>::pure(42)
    ///     .instrument(info_span!("my_operation", value = 42));
    /// ```
    pub fn instrument(self, span: Span) -> Self {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    span.in_scope(|| (self.run_fn)(env)).await
                })
            }),
        }
    }

    /// Auto-instrument with caller source location
    ///
    /// Creates a debug span capturing the file and line where `.traced()` was called.
    /// This makes it easy to see where in your code each Effect originates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// fn my_operation() -> Effect<i32, String, ()> {
    ///     Effect::pure(42).traced()  // Span captures this location
    /// }
    /// ```
    ///
    /// Output in logs:
    /// ```text
    /// DEBUG effect{file="src/lib.rs" line=42}: executing
    /// ```
    #[track_caller]
    pub fn traced(self) -> Self {
        let location = std::panic::Location::caller();
        let span = tracing::debug_span!(
            "effect",
            file = location.file(),
            line = location.line(),
        );
        self.instrument(span)
    }

    /// Give this Effect a name for tracing
    ///
    /// Creates a debug span with the given name. Useful for identifying
    /// specific operations in trace output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// let effect = fetch_user(id).named("fetch-user");
    /// ```
    ///
    /// Output in logs:
    /// ```text
    /// DEBUG effect{name="fetch-user"}: executing
    /// ```
    pub fn named(self, name: impl Into<String>) -> Self {
        let name = name.into();
        let span = tracing::debug_span!("effect", name = %name);
        self.instrument(span)
    }
}
```

#### 2. Module Organization

```rust
// src/lib.rs

// ... existing exports ...

#[cfg(feature = "tracing")]
mod tracing_support;

// Re-export tracing for convenience
#[cfg(feature = "tracing")]
pub use tracing;
```

#### 3. Cargo.toml Updates

```toml
[dependencies]
# ... existing deps ...
tracing = { version = "0.1", optional = true }

[features]
default = []
async = ["tokio"]
tracing = ["dep:tracing"]
# ... other features ...

[dev-dependencies]
# ... existing dev-deps ...
tracing-subscriber = { version = "0.3", features = ["fmt"] }
tracing-test = "0.2"
```

### Architecture

```
src/
├── lib.rs                 # Conditional export of tracing_support
├── effect.rs              # Core Effect (unchanged)
├── tracing_support.rs     # NEW: tracing methods (feature-gated)
└── ...
```

### Why This Design

| Choice | Rationale |
|--------|-----------|
| Separate module | Clean feature gating, no `#[cfg]` scattered through effect.rs |
| `span.in_scope()` | Correct async span propagation |
| `#[track_caller]` | Captures user's location, not internal location |
| `debug_span!` | Appropriate level for Effect tracing |
| Re-export tracing | Users don't need separate tracing import |

### Async Span Correctness

The `in_scope` pattern ensures spans work correctly with async:

```rust
// CORRECT - span covers the entire async operation
span.in_scope(|| async_operation()).await

// WRONG - span only covers creating the future, not awaiting
let _guard = span.enter();
async_operation().await  // Span may not be active here!
```

## Dependencies

### Prerequisites
- None

### External Dependencies
- `tracing` crate (optional, feature-gated)

### Affected Components
- Effect type (new methods added when feature enabled)
- lib.rs (conditional module export)

## Testing Strategy

### Unit Tests

```rust
#[cfg(feature = "tracing")]
mod tracing_tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_instrument_enters_span() {
        let effect = Effect::<_, String, ()>::pure(42)
            .instrument(tracing::info_span!("test_span"));

        let result = effect.run(&()).await;

        assert_eq!(result, Ok(42));
        assert!(logs_contain("test_span"));
    }

    #[traced_test]
    #[tokio::test]
    async fn test_traced_captures_location() {
        let effect = Effect::<_, String, ()>::pure(42).traced();

        effect.run(&()).await.unwrap();

        // Should contain this file
        assert!(logs_contain("tracing_tests"));
    }

    #[traced_test]
    #[tokio::test]
    async fn test_named_shows_name() {
        let effect = Effect::<_, String, ()>::pure(42)
            .named("my-operation");

        effect.run(&()).await.unwrap();

        assert!(logs_contain("my-operation"));
    }

    #[traced_test]
    #[tokio::test]
    async fn test_error_in_span() {
        let effect = Effect::<i32, _, ()>::fail("oops".to_string())
            .instrument(tracing::info_span!("failing"));

        let result = effect.run(&()).await;

        assert!(result.is_err());
        assert!(logs_contain("failing"));
    }

    #[traced_test]
    #[tokio::test]
    async fn test_nested_spans() {
        let inner = Effect::<_, String, ()>::pure(1).named("inner");
        let outer = inner.and_then(|x| {
            Effect::pure(x + 1).named("outer")
        });

        outer.run(&()).await.unwrap();

        assert!(logs_contain("inner"));
        assert!(logs_contain("outer"));
    }

    #[traced_test]
    #[tokio::test]
    async fn test_parallel_spans() {
        let effects = vec![
            Effect::<_, String, ()>::pure(1).named("task-1"),
            Effect::<_, String, ()>::pure(2).named("task-2"),
            Effect::<_, String, ()>::pure(3).named("task-3"),
        ];

        Effect::par_all(effects).run(&()).await.unwrap();

        assert!(logs_contain("task-1"));
        assert!(logs_contain("task-2"));
        assert!(logs_contain("task-3"));
    }
}
```

### Feature Gate Tests

```rust
// tests/no_tracing.rs
// Run with: cargo test --no-default-features

#[test]
fn test_compiles_without_tracing() {
    use stillwater::Effect;

    let effect = Effect::<_, String, ()>::pure(42);

    // These methods should NOT exist without tracing feature
    // Uncomment to verify compile error:
    // effect.instrument(todo!());
    // effect.traced();
    // effect.named("test");
}
```

### Integration Example

```rust
// examples/tracing_demo.rs

use stillwater::Effect;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Set up tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Simulate a workflow
    let result = fetch_data()
        .and_then(|data| process_data(data))
        .and_then(|processed| save_result(processed))
        .run(&())
        .await;

    match result {
        Ok(id) => tracing::info!("Workflow completed: {}", id),
        Err(e) => tracing::error!("Workflow failed: {}", e),
    }
}

fn fetch_data() -> Effect<String, String, ()> {
    Effect::pure("raw data".to_string())
        .named("fetch-data")
}

fn process_data(data: String) -> Effect<String, String, ()> {
    Effect::pure(format!("processed: {}", data))
        .traced()  // Auto-captures this location
}

fn save_result(data: String) -> Effect<i32, String, ()> {
    Effect::pure(42)
        .instrument(tracing::info_span!("save", data_len = data.len()))
}
```

## Documentation Requirements

### Code Documentation
- Document `.instrument()`, `.traced()`, `.named()` methods
- Document tracing feature flag
- Document async span behavior

### User Documentation
- Add "Tracing and Observability" section to README
- Show tracing subscriber setup
- Provide example output

### Examples
- `examples/tracing_demo.rs` - Basic tracing setup and usage

## Migration and Compatibility

### Breaking Changes
None - this is purely additive behind a feature flag.

### For Existing Users
No changes required. Opt-in by enabling `tracing` feature:

```toml
[dependencies]
stillwater = { version = "...", features = ["tracing"] }
```

## Success Metrics

### Quantitative
- Zero binary size increase when feature disabled
- No performance regression in benchmarks without feature
- All existing tests pass unchanged

### Qualitative
- Users can see Effect execution flow in traces
- Easy to identify where Effects originate
- Integrates seamlessly with existing tracing infrastructure
