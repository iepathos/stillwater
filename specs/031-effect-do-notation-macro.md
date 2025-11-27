---
number: 31
title: Effect Do-Notation Macro
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-27
revised: 2025-11-27
---

# Specification 031: Effect Do-Notation Macro

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None (builds on existing Effect type)

## Context

The Effect type provides powerful monadic composition through `and_then`, `map`, and other
combinators. However, chaining multiple effects leads to deeply nested closures that are
difficult to read and write:

```rust
// Current: deeply nested, hard to follow
fetch_order(&conn, order_id)
    .and_then(|order| {
        validate_order(&order)
            .and_then(|_| {
                fetch_inventory(&conn, &order)
                    .and_then(|inventory| {
                        check_stock(&inventory, &order)
                            .and_then(|_| {
                                process_order(&order)
                                    .map(|result| {
                                        (order, result)
                                    })
                            })
                    })
            })
    })
```

This is a well-known problem in functional programming, solved by "do-notation" in Haskell
and "for-comprehensions" in Scala. Several Rust crates provide this pattern:

- `do-notation` crate
- `mdo` crate
- `frunk` HList operations

However, none are designed specifically for our Effect type with its environment parameter.

### Philosophy Alignment

From PHILOSOPHY.md: *"We're not trying to be Haskell. We're trying to be better Rust."*

The `effect!` macro provides Haskell-style ergonomics while remaining idiomatic Rust:

1. **Familiar syntax** - Rust developers understand `let` bindings
2. **No hidden magic** - Expands to standard `and_then` chains
3. **Type-safe** - Full type inference and error checking
4. **Composable** - Works with all existing Effect combinators

## Objective

Add an `effect!` macro to stillwater that:

1. Provides flat, sequential syntax for Effect composition
2. Eliminates nested closure boilerplate
3. Maintains full type safety and inference
4. Generates zero runtime overhead (pure syntax sugar)
5. Produces helpful error messages on misuse

## Requirements

### Functional Requirements

#### FR-1: Basic Sequential Binding

Bind effect results to variables for use in subsequent effects:

```rust
effect! {
    order <- fetch_order(&conn, order_id);
    inventory <- fetch_inventory(&conn, &order);
    result <- process_order(&order, &inventory);
    Effect::pure(result)
}

// Expands to:
fetch_order(&conn, order_id)
    .and_then(|order| {
        fetch_inventory(&conn, &order)
            .and_then(|inventory| {
                process_order(&order, &inventory)
                    .and_then(|result| {
                        Effect::pure(result)
                    })
            })
    })
```

#### FR-2: Discard Binding

Use `_` to execute an effect for its side effects without binding the result:

```rust
effect! {
    order <- fetch_order(&conn, order_id);
    _ <- log_order_access(&order);  // Result discarded
    _ <- validate_order(&order);     // Result discarded
    Effect::pure(order)
}

// Expands to:
fetch_order(&conn, order_id)
    .and_then(|order| {
        log_order_access(&order)
            .and_then(|_| {
                validate_order(&order)
                    .and_then(|_| {
                        Effect::pure(order)
                    })
            })
    })
```

#### FR-3: Pure Let Bindings

Use `let` for pure (non-Effect) computations:

```rust
effect! {
    order <- fetch_order(&conn, order_id);
    let total = order.items.iter().map(|i| i.price).sum::<Money>();
    let tax = total * TAX_RATE;
    _ <- validate_total(total + tax);
    Effect::pure((order, total, tax))
}

// Expands to:
fetch_order(&conn, order_id)
    .and_then(|order| {
        let total = order.items.iter().map(|i| i.price).sum::<Money>();
        let tax = total * TAX_RATE;
        validate_total(total + tax)
            .and_then(|_| {
                Effect::pure((order, total, tax))
            })
    })
```

#### FR-4: Pattern Destructuring

Support pattern matching in bindings:

```rust
effect! {
    (user, profile) <- fetch_user_with_profile(user_id);
    UserSettings { theme, locale, .. } <- fetch_settings(user.id);
    Effect::pure(UserView { user, profile, theme, locale })
}
```

#### FR-5: Early Return with `?` Syntax (Optional Enhancement)

Convert Result-returning expressions to Effect:

```rust
effect! {
    order <- fetch_order(&conn, order_id);
    // ? converts Result<T, E> to Effect<T, E, Env>
    validated = validate_order(&order)?;
    Effect::pure(validated)
}

// Expands to:
fetch_order(&conn, order_id)
    .and_then(|order| {
        Effect::from_result(validate_order(&order))
            .and_then(|validated| {
                Effect::pure(validated)
            })
    })
```

**Note**: This requires `Effect::from_result` to exist. If not present, this feature
can be deferred.

#### FR-6: Final Expression

The macro must end with an Effect expression (not a binding):

```rust
// Valid: ends with Effect
effect! {
    x <- get_x();
    Effect::pure(x * 2)
}

// Invalid: ends with binding (compile error)
effect! {
    x <- get_x();
    y <- get_y();  // Error: macro must end with an expression
}
```

### Non-Functional Requirements

#### NFR-1: Zero Runtime Overhead

The macro generates only `and_then` chains - no additional allocations or indirection.

#### NFR-2: Helpful Error Messages

When possible, provide clear compile-time errors:

- Missing final expression
- Type mismatches between effects
- Invalid binding patterns

#### NFR-3: IDE Compatibility

The generated code should work well with rust-analyzer:

- Type inference should work
- Go-to-definition should navigate correctly
- Autocomplete should function within the macro

## Acceptance Criteria

### Must Have

- [ ] `effect!` macro with `name <- effect;` binding syntax
- [ ] Support for `_ <- effect;` discard binding
- [ ] Support for `let name = expr;` pure bindings
- [ ] Support for pattern destructuring in bindings
- [ ] Final expression requirement enforced at compile time
- [ ] Zero runtime overhead (verified via expansion)
- [ ] Unit tests for all binding forms
- [ ] Documentation with examples

### Should Have

- [ ] `?` syntax for Result-to-Effect conversion (if `Effect::from_result` exists)
- [ ] Helpful custom error messages via `compile_error!`
- [ ] Integration examples with bracket/Acquiring

### Won't Have (This Version)

- [ ] `if`/`else` syntax inside macro (use regular Effect combinators)
- [ ] `match` syntax inside macro (use regular Rust match outside)
- [ ] `for` loop syntax (use `Effect::traverse` or similar)
- [ ] Automatic error type conversion (use `map_error` explicitly)

## Technical Details

### Implementation Approach

#### Macro Definition

```rust
/// Do-notation macro for Effect composition.
///
/// Provides flat, sequential syntax for chaining effects, eliminating
/// deeply nested `and_then` closures.
///
/// # Syntax
///
/// ```text
/// effect! {
///     pattern <- effect_expr;    // Bind effect result to pattern
///     _ <- effect_expr;          // Execute effect, discard result
///     let pattern = pure_expr;   // Pure (non-effect) binding
///     final_effect_expr          // Must end with an Effect
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use stillwater::{Effect, effect};
///
/// fn process_order(order_id: OrderId) -> Effect<Receipt, Error, AppEnv> {
///     effect! {
///         order <- fetch_order(order_id);
///         let total = calculate_total(&order);
///         _ <- validate_payment(&order, total);
///         receipt <- charge_customer(&order, total);
///         Effect::pure(receipt)
///     }
/// }
/// ```
#[macro_export]
macro_rules! effect {
    // Base case: final expression (no semicolon)
    ($e:expr) => { $e };

    // Binding with pattern <- effect;
    ($p:pat <- $e:expr; $($rest:tt)*) => {
        $e.and_then(|$p| $crate::effect!($($rest)*))
    };

    // Pure let binding
    (let $p:pat = $e:expr; $($rest:tt)*) => {
        {
            let $p = $e;
            $crate::effect!($($rest)*)
        }
    };

    // Optional: ? syntax for Result conversion (if Effect::from_result exists)
    // ($p:pat = $e:expr ?; $($rest:tt)*) => {
    //     $crate::Effect::from_result($e).and_then(|$p| $crate::effect!($($rest)*))
    // };
}
```

#### Expansion Examples

**Input:**
```rust
effect! {
    a <- get_a();
    b <- get_b(&a);
    let c = a + b;
    _ <- log_sum(c);
    Effect::pure(c)
}
```

**Expands to:**
```rust
get_a().and_then(|a| {
    get_b(&a).and_then(|b| {
        {
            let c = a + b;
            log_sum(c).and_then(|_| {
                Effect::pure(c)
            })
        }
    })
})
```

### Error Handling

The macro relies on Rust's type system for error handling. If effects have different
error types, users must explicitly convert:

```rust
effect! {
    // If fetch_order returns Effect<Order, DbError, Env>
    // and validate returns Effect<(), ValidationError, Env>
    // this won't compile without error mapping:

    order <- fetch_order(id).map_error(AppError::from);
    _ <- validate(&order).map_error(AppError::from);
    Effect::pure(order)
}
```

### Architecture Changes

New file structure:

```
src/
├── lib.rs           # Re-export macro
├── macros/
│   ├── mod.rs       # Module root
│   └── effect.rs    # effect! macro definition
```

Or simpler, add directly to `lib.rs` if macros are few.

## Dependencies

- **Prerequisites**: None (works with current Effect type)
- **Affected Components**:
  - `lib.rs` - Export macro
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn effect_macro_single_binding() {
        let result = effect! {
            x <- Effect::<_, String, ()>::pure(42);
            Effect::pure(x * 2)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
    }

    #[tokio::test]
    async fn effect_macro_multiple_bindings() {
        let result = effect! {
            a <- Effect::<_, String, ()>::pure(1);
            b <- Effect::pure(2);
            c <- Effect::pure(3);
            Effect::pure(a + b + c)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(6));
    }

    #[tokio::test]
    async fn effect_macro_discard_binding() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = effect! {
            x <- Effect::<_, String, ()>::pure(42);
            _ <- Effect::from_fn(move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
            Effect::pure(x)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn effect_macro_let_binding() {
        let result = effect! {
            x <- Effect::<_, String, ()>::pure(10);
            let doubled = x * 2;
            let tripled = x * 3;
            Effect::pure(doubled + tripled)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(50)); // 20 + 30
    }

    #[tokio::test]
    async fn effect_macro_pattern_destructure() {
        let result = effect! {
            (a, b) <- Effect::<_, String, ()>::pure((1, 2));
            Effect::pure(a + b)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(3));
    }

    #[tokio::test]
    async fn effect_macro_early_error() {
        let result = effect! {
            x <- Effect::<i32, String, ()>::pure(42);
            _ <- Effect::<(), String, ()>::fail("error".to_string());
            Effect::pure(x) // Never reached
        }
        .run(&())
        .await;

        assert_eq!(result, Err("error".to_string()));
    }

    #[tokio::test]
    async fn effect_macro_with_environment() {
        struct Env { multiplier: i32 }

        let result = effect! {
            x <- Effect::pure(10);
            m <- Effect::from_env(|env: &Env| Ok(env.multiplier));
            Effect::pure(x * m)
        }
        .run(&Env { multiplier: 5 })
        .await;

        assert_eq!(result, Ok(50));
    }
}
```

### Compile-Fail Tests

Using `trybuild` or similar for compile-time error testing:

```rust
// tests/ui/effect_macro_no_final_expr.rs
// This should fail to compile

fn main() {
    let _ = effect! {
        x <- Effect::pure(42);
        y <- Effect::pure(x);
        // Missing final expression!
    };
}
```

## Documentation Requirements

- **Code Documentation**: Comprehensive doc comments on macro with examples
- **User Documentation**: Section in README showing do-notation usage
- **Examples**: `examples/effect_macro.rs` demonstrating real-world usage

## Implementation Notes

### Hygiene Considerations

The macro should be hygienic - it should not capture or conflict with user variables.
Using `$crate::` prefix for internal references ensures this.

### Future Extensions

This macro is designed to compose with future macros:

- `bracket!` macro can use `effect!` internally for the use function
- `scoped!` macro will build on both `effect!` and `bracket!`

### IDE Support Tips

For better IDE support, consider:

1. Using `#[doc(hidden)]` helper macros sparingly
2. Keeping expansion simple and predictable
3. Testing with rust-analyzer to verify inference works

## Migration and Compatibility

### Backward Compatibility

This is a purely additive feature. Existing code using `and_then` chains continues
to work unchanged.

### Migration Path

Users can incrementally adopt `effect!`:

```rust
// Before
fetch_order(id)
    .and_then(|order| validate(&order).map(|_| order))
    .and_then(|order| process(&order))

// After
effect! {
    order <- fetch_order(id);
    _ <- validate(&order);
    process(&order)
}
```

The two forms are semantically identical and can be mixed in the same codebase.

## Related Specifications

- **Spec 002**: Resource Scopes and Bracket Pattern - Core resource management
- **Spec 032**: bracket! Macro (planned) - Resource acquisition syntax
- **Spec 033**: scoped! Macro (planned) - Combined resource + do-notation

---

*"Make the common case beautiful."*
