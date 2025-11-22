---
number: 004
title: Context Error Handling with Error Trails
category: foundation
priority: high
status: draft
dependencies: [003]
created: 2025-11-21
---

# Specification 004: Context Error Handling with Error Trails

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 003 (Effect Type)

## Context

When errors bubble up through deep call stacks, they lose context about *what* was being attempted and *where* things went wrong. A simple "File not found" error doesn't tell you which file, what operation was trying to read it, or why.

Context error handling preserves a trail of breadcrumbs showing the path the error took through the code, making debugging significantly easier.

Example without context:
```
Error: No such file or directory
```

Example with context:
```
Error: No such file or directory
  -> Reading config file: /etc/myapp/config.toml
  -> Parsing TOML configuration
  -> Initializing application
```

## Objective

Implement a `ContextError<E>` wrapper type that accumulates context information as errors propagate up the call stack, and integrate it with the Effect type via a `.context()` method.

## Requirements

### Functional Requirements

- Define `ContextError<E>` wrapper type
- Store underlying error and context trail
- Implement `.context()` method on Effect
- Accumulate context messages in order (inner to outer)
- Provide access to underlying error
- Provide access to full context trail
- Implement Display for readable error output
- Implement Error trait
- Preserve error type information

### Non-Functional Requirements

- Minimal overhead (small Vec allocation for context)
- Clear error messages
- Works with any error type
- Integrates seamlessly with Effect
- Easy to add context (one method call)

## Acceptance Criteria

- [ ] ContextError<E> defined in `src/context.rs`
- [ ] Stores error and Vec<String> for context trail
- [ ] `.context()` method on Effect adds context layer
- [ ] Display implementation shows error and trail
- [ ] Error trait implemented
- [ ] `inner()` method returns &E
- [ ] `into_inner()` consumes and returns E
- [ ] `context_trail()` returns &[String]
- [ ] Effect::context() properly types as Effect<T, ContextError<E>, Env>
- [ ] Comprehensive tests (>95% coverage)
- [ ] Documentation with examples

## Technical Details

### Implementation Approach

```rust
use std::error::Error as StdError;
use std::fmt;

/// An error wrapper that accumulates context as it propagates
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextError<E> {
    error: E,
    context: Vec<String>,
}

impl<E> ContextError<E> {
    /// Create a new context error
    pub fn new(error: E) -> Self {
        ContextError {
            error,
            context: Vec::new(),
        }
    }

    /// Add a context layer
    pub fn context(mut self, msg: impl Into<String>) -> Self {
        self.context.push(msg.into());
        self
    }

    /// Get the underlying error
    pub fn inner(&self) -> &E {
        &self.error
    }

    /// Consume and return the underlying error
    pub fn into_inner(self) -> E {
        self.error
    }

    /// Get the context trail
    pub fn context_trail(&self) -> &[String] {
        &self.context
    }
}

impl<E: fmt::Display> fmt::Display for ContextError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display underlying error
        writeln!(f, "Error: {}", self.error)?;

        // Display context trail
        for (i, ctx) in self.context.iter().enumerate() {
            let indent = "  ".repeat(i + 1);
            writeln!(f, "{}-> {}", indent, ctx)?;
        }

        Ok(())
    }
}

impl<E: StdError + 'static> StdError for ContextError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

// Integration with Effect
impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Add context to errors from this effect
    pub fn context(self, msg: impl Into<String>) -> Effect<T, ContextError<E>, Env> {
        let msg = msg.into();
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    (self.run_fn)(env)
                        .await
                        .map_err(|err| ContextError::new(err).context(msg))
                })
            }),
        }
    }
}

// Chaining context on ContextError
impl<T, E, Env> Effect<T, ContextError<E>, Env>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Add another layer of context
    pub fn context(self, msg: impl Into<String>) -> Self {
        let msg = msg.into();
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    (self.run_fn)(env)
                        .await
                        .map_err(|err| err.context(msg))
                })
            }),
        }
    }
}
```

### Architecture Changes

- New module: `src/context.rs`
- New trait implementation on Effect for `.context()`
- Export from `src/lib.rs`
- Re-export in `prelude`

### Data Structures

```rust
pub struct ContextError<E> {
    error: E,
    context: Vec<String>,
}
```

### APIs and Interfaces

See Implementation Approach above.

## Dependencies

- **Prerequisites**: Spec 003 (Effect type for integration)
- **Affected Components**: Effect type (adds new method)
- **External Dependencies**: None (uses std::error)

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_context_error_new() {
    let err = ContextError::new("base error");
    assert_eq!(err.inner(), &"base error");
    assert_eq!(err.context_trail(), &[] as &[String]);
}

#[test]
fn test_context_accumulation() {
    let err = ContextError::new("base error")
        .context("first context")
        .context("second context");

    assert_eq!(err.context_trail(), &["first context", "second context"]);
}

#[test]
fn test_display_format() {
    let err = ContextError::new("file not found")
        .context("reading config")
        .context("initializing app");

    let output = format!("{}", err);
    assert!(output.contains("Error: file not found"));
    assert!(output.contains("-> reading config"));
    assert!(output.contains("-> initializing app"));
}

#[tokio::test]
async fn test_effect_context() {
    let effect = Effect::<i32, _, ()>::fail("base error")
        .context("operation failed");

    match effect.run(&()).await {
        Err(ctx_err) => {
            assert_eq!(ctx_err.inner(), &"base error");
            assert_eq!(ctx_err.context_trail(), &["operation failed"]);
        }
        Ok(_) => panic!("Expected error"),
    }
}

#[tokio::test]
async fn test_effect_multiple_contexts() {
    let effect = Effect::<i32, _, ()>::fail("base error")
        .context("step 1")
        .context("step 2")
        .context("step 3");

    match effect.run(&()).await {
        Err(ctx_err) => {
            assert_eq!(ctx_err.context_trail(), &["step 1", "step 2", "step 3"]);
        }
        Ok(_) => panic!("Expected error"),
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_real_world_error_trail() {
    #[derive(Debug, PartialEq)]
    enum AppError {
        FileNotFound(String),
        ParseError(String),
    }

    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                AppError::FileNotFound(path) => write!(f, "File not found: {}", path),
                AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            }
        }
    }

    impl StdError for AppError {}

    async fn read_config(path: &str) -> Effect<String, AppError, ()> {
        Effect::fail(AppError::FileNotFound(path.to_string()))
            .context(format!("Reading config file: {}", path))
    }

    async fn parse_config(content: String) -> Effect<Config, AppError, ()> {
        Effect::fail(AppError::ParseError("invalid TOML".to_string()))
            .context("Parsing TOML configuration")
    }

    async fn initialize_app() -> Effect<App, ContextError<AppError>, ()> {
        read_config("/etc/myapp/config.toml")
            .and_then(|content| parse_config(content))
            .context("Initializing application")
            .run(&())
            .await
    }

    match initialize_app().await {
        Err(err) => {
            assert!(matches!(err.inner(), AppError::FileNotFound(_)));
            let trail = err.context_trail();
            assert!(trail[0].contains("Reading config file"));
            assert!(trail[1].contains("Initializing application"));
        }
        Ok(_) => panic!("Expected error"),
    }
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for ContextError
- Examples showing context accumulation
- Explain integration with Effect
- Show Display output examples

### User Documentation

- Add "Error Context" section to README
- Create section in docs/guide/03-effects.md
- Show best practices for adding context
- Explain when to add context vs when not to

### Architecture Updates

- Document ContextError in DESIGN.md
- Add example to error_context.rs example file

## Implementation Notes

### Memory Overhead

- Vec<String> allocations for context trail
- Typically small (2-5 context layers)
- Negligible compared to I/O operations

### Display Formatting

- Indentation shows nesting depth
- Innermost error first, context follows
- Easy to read and understand

### When to Add Context

Good:
```rust
load_config(path)
    .context(format!("Loading config from {}", path))
    .and_then(parse_config)
    .context("Parsing configuration")
```

Bad (too much):
```rust
calculate_discount(customer)  // Pure function, no context needed
    .context("Calculating discount")  // Unnecessary!
```

Guideline: Add context at I/O boundaries and major operation boundaries.

### Alternative: Structured Context

Future enhancement could support structured context:
```rust
.context(ctx::Loading { path: "/etc/config" })
```

For MVP, strings are sufficient.

## Migration and Compatibility

No migration needed - this is a new feature.

## Open Questions

1. Should we support structured context (not just strings)?
   - Decision: Defer to future spec if users request it

2. Should we capture file:line automatically?
   - Decision: No, would require macro magic, against philosophy

3. Should ContextError implement From<E> to auto-wrap?
   - Decision: No, explicit wrapping is clearer

4. Should we limit context trail size to prevent memory issues?
   - Decision: No, typical trails are small, and limiting could hide useful info
