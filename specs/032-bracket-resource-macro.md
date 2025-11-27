---
number: 32
title: Bracket Resource Macro
category: foundation
priority: medium
status: draft
dependencies: [2]
created: 2025-11-27
revised: 2025-11-27
---

# Specification 032: Bracket Resource Macro

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 002 (Resource Scopes and Bracket Pattern)

## Context

Spec 002 introduced the `Acquiring` builder for managing multiple resources with
guaranteed cleanup. While functional, the syntax is verbose:

```rust
// Current: verbose builder pattern
Effect::acquiring(
    order_lock(order_id.clone()),
    |lock| async move { lock.release().await.map_err(OrderError::Lock) },
)
.and(
    db_connection(),
    |conn| async move { conn.close().await.map_err(OrderError::Database) },
)
.and(
    db_transaction(),
    |tx| async move { tx.rollback().await.map_err(OrderError::Database) },
)
.with_flat(|lock, conn, tx| {
    do_work(lock, conn, tx)
})
```

The repetitive structure obscures the intent: "acquire these resources, use them, clean up."
A declarative macro can provide clearer syntax while generating the same code.

### Prior Art

- **Haskell**: `bracket acquire release use` function
- **Scala cats-effect**: `Resource.make(acquire)(release).use { r => ... }`
- **Python**: `async with` context manager syntax
- **C#**: `await using var x = ...` declaration syntax

### Philosophy Alignment

From PHILOSOPHY.md: *"Composition over complexity"*

The `bracket!` macro composes with existing APIs rather than replacing them:

1. Generates `Effect::acquiring().and().with_flat()` chains
2. No new runtime behavior - pure syntax sugar
3. Works alongside manual builder usage
4. Integrates with `effect!` macro (Spec 031)

## Objective

Add a `bracket!` macro to stillwater that:

1. Provides declarative syntax for resource acquisition with cleanup
2. Supports multiple resources with automatic LIFO cleanup ordering
3. Eliminates boilerplate for cleanup function syntax
4. Generates optimal `Acquiring` builder code
5. Composes with `effect!` macro for the use body

## Requirements

### Functional Requirements

#### FR-1: Single Resource Syntax

Acquire one resource with cleanup:

```rust
bracket! {
    conn <- db_connection(), |c| c.close();
    =>
    fetch_user(&conn, user_id)
}

// Expands to:
Effect::acquiring(
    db_connection(),
    |c| async move { c.close().await }
)
.with(|conn| {
    fetch_user(conn, user_id)
})
```

#### FR-2: Multiple Resources with LIFO Cleanup

Acquire multiple resources, released in reverse order:

```rust
bracket! {
    lock <- order_lock(id), |l| l.release();
    conn <- db_connection(), |c| c.close();
    tx   <- db_transaction(), |t| t.rollback();
    =>
    process_order(&lock, &conn, &tx)
}

// Expands to:
Effect::acquiring(order_lock(id), |l| async move { l.release().await })
    .and(db_connection(), |c| async move { c.close().await })
    .and(db_transaction(), |t| async move { t.rollback().await })
    .with_flat(|lock, conn, tx| {
        process_order(lock, conn, tx)
    })
```

#### FR-3: Async Cleanup Functions

The cleanup closure is automatically wrapped in `async move`:

```rust
bracket! {
    // User writes:
    conn <- db_connection(), |c| c.close();
    =>
    use_conn(&conn)
}

// Macro generates:
Effect::acquiring(
    db_connection(),
    |c| async move { c.close().await }  // async move added automatically
)
.with(|conn| use_conn(conn))
```

#### FR-4: Cleanup with Error Mapping

Support explicit error mapping in cleanup:

```rust
bracket! {
    conn <- db_connection(), |c| c.close().map_err(AppError::Db);
    =>
    use_conn(&conn)
}

// Expands to:
Effect::acquiring(
    db_connection(),
    |c| async move { c.close().await.map_err(AppError::Db) }
)
.with(|conn| use_conn(conn))
```

#### FR-5: Discard Resource Name

Use `_` when the resource handle isn't needed (only cleanup matters):

```rust
bracket! {
    _ <- acquire_lock(id), |l| l.release();  // Lock acquired but not used
    conn <- db_connection(), |c| c.close();
    =>
    use_conn(&conn)  // Can't use lock here, only conn
}
```

#### FR-6: Compose with effect! Macro

The body can use `effect!` for sequential operations:

```rust
bracket! {
    conn <- db_connection(), |c| c.close();
    tx   <- db_transaction(), |t| t.rollback();
    =>
    effect! {
        order <- fetch_order(&conn, order_id);
        _ <- validate_order(&tx, &order);
        _ <- update_inventory(&tx, &order);
        _ <- tx.commit();
        Effect::pure(order)
    }
}
```

#### FR-7: Nested Brackets

Support nesting for conditional resource acquisition:

```rust
bracket! {
    conn <- db_connection(), |c| c.close();
    =>
    effect! {
        order <- fetch_order(&conn, order_id);
        result <- if order.needs_lock {
            bracket! {
                lock <- acquire_lock(order.id), |l| l.release();
                =>
                process_locked(&conn, &lock, &order)
            }
        } else {
            process_unlocked(&conn, &order)
        };
        Effect::pure(result)
    }
}
```

### Non-Functional Requirements

#### NFR-1: Zero Runtime Overhead

Expands to standard `Acquiring` builder calls - no additional allocations.

#### NFR-2: Clear Error Messages

Provide helpful compile-time errors:

- Missing `=>` separator
- Missing cleanup function
- Invalid resource pattern

#### NFR-3: Predictable Expansion

Users should be able to mentally expand the macro:

| Resources | Generates |
|-----------|-----------|
| 1 | `.with(\|a\| ...)` |
| 2 | `.with_flat(\|a, b\| ...)` |
| 3 | `.with_flat(\|a, b, c\| ...)` |
| 4 | `.with_flat(\|a, b, c, d\| ...)` |
| 5+ | Compile error (use nested brackets) |

## Acceptance Criteria

### Must Have

- [ ] `bracket!` macro with `name <- acquire, cleanup;` syntax
- [ ] Support for 1-4 resources
- [ ] Automatic `async move` wrapping for cleanup functions
- [ ] `=>` separator between resources and body
- [ ] Support for `_` discard pattern
- [ ] LIFO cleanup order preserved
- [ ] Unit tests for all resource counts (1-4)
- [ ] Documentation with examples

### Should Have

- [ ] Helpful compile error for 5+ resources (suggesting nested brackets)
- [ ] Integration examples with `effect!` macro
- [ ] Compile-fail tests for invalid syntax

### Won't Have (This Version)

- [ ] More than 4 resources (use nested brackets or manual builder)
- [ ] Conditional cleanup (always runs)
- [ ] Custom cleanup error handling (uses default logging)
- [ ] Resource type inference from cleanup (explicit acquire required)

## Technical Details

### Implementation Approach

#### Macro Definition

```rust
/// Bracket macro for resource acquisition with guaranteed cleanup.
///
/// Provides declarative syntax for acquiring resources and ensuring they
/// are released in reverse order (LIFO), even on error.
///
/// # Syntax
///
/// ```text
/// bracket! {
///     name <- acquire_effect, |var| cleanup_expr;
///     name2 <- acquire_effect2, |var| cleanup_expr2;
///     =>
///     use_effect
/// }
/// ```
///
/// # Example
///
/// ```rust
/// use stillwater::{Effect, bracket};
///
/// fn process_order(order_id: OrderId) -> Effect<Order, Error, AppEnv> {
///     bracket! {
///         lock <- acquire_lock(order_id), |l| l.release();
///         conn <- db_connection(), |c| c.close();
///         =>
///         effect! {
///             order <- fetch_order(&conn, order_id);
///             _ <- validate(&order);
///             Effect::pure(order)
///         }
///     }
/// }
/// ```
///
/// # Cleanup Ordering
///
/// Resources are released in reverse order of acquisition (LIFO):
///
/// ```rust
/// bracket! {
///     a <- acquire_a(), |a| release_a(a);  // Released 3rd
///     b <- acquire_b(), |b| release_b(b);  // Released 2nd
///     c <- acquire_c(), |c| release_c(c);  // Released 1st
///     =>
///     use_all(&a, &b, &c)
/// }
/// ```
#[macro_export]
macro_rules! bracket {
    // Single resource
    ($name:pat <- $acquire:expr, |$cleanup_var:ident| $cleanup:expr; => $body:expr) => {
        $crate::Effect::acquiring(
            $acquire,
            |$cleanup_var| async move { $cleanup.await }
        ).with(|$name| $body)
    };

    // Two resources
    (
        $name1:pat <- $acquire1:expr, |$cv1:ident| $cleanup1:expr;
        $name2:pat <- $acquire2:expr, |$cv2:ident| $cleanup2:expr;
        => $body:expr
    ) => {
        $crate::Effect::acquiring($acquire1, |$cv1| async move { $cleanup1.await })
            .and($acquire2, |$cv2| async move { $cleanup2.await })
            .with_flat(|$name1, $name2| $body)
    };

    // Three resources
    (
        $name1:pat <- $acquire1:expr, |$cv1:ident| $cleanup1:expr;
        $name2:pat <- $acquire2:expr, |$cv2:ident| $cleanup2:expr;
        $name3:pat <- $acquire3:expr, |$cv3:ident| $cleanup3:expr;
        => $body:expr
    ) => {
        $crate::Effect::acquiring($acquire1, |$cv1| async move { $cleanup1.await })
            .and($acquire2, |$cv2| async move { $cleanup2.await })
            .and($acquire3, |$cv3| async move { $cleanup3.await })
            .with_flat(|$name1, $name2, $name3| $body)
    };

    // Four resources
    (
        $name1:pat <- $acquire1:expr, |$cv1:ident| $cleanup1:expr;
        $name2:pat <- $acquire2:expr, |$cv2:ident| $cleanup2:expr;
        $name3:pat <- $acquire3:expr, |$cv3:ident| $cleanup3:expr;
        $name4:pat <- $acquire4:expr, |$cv4:ident| $cleanup4:expr;
        => $body:expr
    ) => {
        $crate::Effect::acquiring($acquire1, |$cv1| async move { $cleanup1.await })
            .and($acquire2, |$cv2| async move { $cleanup2.await })
            .and($acquire3, |$cv3| async move { $cleanup3.await })
            .and($acquire4, |$cv4| async move { $cleanup4.await })
            .with_flat(|$name1, $name2, $name3, $name4| $body)
    };

    // Error case: too many resources
    (
        $name1:pat <- $acquire1:expr, |$cv1:ident| $cleanup1:expr;
        $name2:pat <- $acquire2:expr, |$cv2:ident| $cleanup2:expr;
        $name3:pat <- $acquire3:expr, |$cv3:ident| $cleanup3:expr;
        $name4:pat <- $acquire4:expr, |$cv4:ident| $cleanup4:expr;
        $name5:pat <- $acquire5:expr, |$cv5:ident| $cleanup5:expr;
        $($rest:tt)*
    ) => {
        compile_error!("bracket! supports at most 4 resources. Use nested bracket! for more.")
    };
}
```

#### Alternative: Recursive Macro (More Flexible)

A more sophisticated approach using recursive parsing:

```rust
#[macro_export]
macro_rules! bracket {
    // Entry point: start accumulating resources
    ($($tokens:tt)*) => {
        $crate::bracket_internal!(@parse [] $($tokens)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! bracket_internal {
    // Accumulate resource bindings until we hit =>
    (@parse [$($resources:tt)*] $name:pat <- $acquire:expr, |$cv:ident| $cleanup:expr; $($rest:tt)*) => {
        $crate::bracket_internal!(@parse [$($resources)* ($name, $acquire, $cv, $cleanup)] $($rest)*)
    };

    // Hit the => separator, now emit code
    (@parse [($n1:pat, $a1:expr, $cv1:ident, $c1:expr)] => $body:expr) => {
        $crate::Effect::acquiring($a1, |$cv1| async move { $c1.await })
            .with(|$n1| $body)
    };

    (@parse [($n1:pat, $a1:expr, $cv1:ident, $c1:expr) ($n2:pat, $a2:expr, $cv2:ident, $c2:expr)] => $body:expr) => {
        $crate::Effect::acquiring($a1, |$cv1| async move { $c1.await })
            .and($a2, |$cv2| async move { $c2.await })
            .with_flat(|$n1, $n2| $body)
    };

    // ... patterns for 3 and 4 resources ...

    // Error: no resources
    (@parse [] => $body:expr) => {
        compile_error!("bracket! requires at least one resource")
    };
}
```

### Cleanup Function Handling

The cleanup closure needs special handling:

1. **Wrap in `async move`**: User writes `|c| c.close()`, macro generates `|c| async move { c.close().await }`
2. **Await the expression**: The cleanup expression is awaited inside the async block
3. **Move semantics**: `async move` ensures the resource is moved into the cleanup future

**Edge case - already async:**

If users write `|c| async { c.close().await }`, the macro would generate:
`|c| async move { (async { c.close().await }).await }`

This is slightly redundant but semantically correct. For simplicity, we document
that users should write just the expression, not the async block.

### Architecture Changes

New file structure:

```
src/
├── lib.rs
├── macros/
│   ├── mod.rs       # Re-export macros
│   ├── effect.rs    # effect! macro (Spec 031)
│   └── bracket.rs   # bracket! macro (this spec)
```

## Dependencies

- **Prerequisites**: Spec 002 (Resource Scopes) - must be implemented first
- **Affected Components**:
  - `lib.rs` - Export macro
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn bracket_single_resource() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket! {
            val <- Effect::<_, String, ()>::pure(42),
                   |_v| { released_clone.store(true, Ordering::SeqCst); async { Ok(()) } };
            =>
            Effect::pure(val * 2)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bracket_two_resources_lifo() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();

        let result = bracket! {
            a <- Effect::<_, String, ()>::pure("first"),
                 |_| { order1.lock().unwrap().push("release_first"); async { Ok(()) } };
            b <- Effect::pure("second"),
                 |_| { order2.lock().unwrap().push("release_second"); async { Ok(()) } };
            =>
            Effect::pure(format!("{} {}", a, b))
        }
        .run(&())
        .await;

        assert_eq!(result, Ok("first second".to_string()));

        let releases = order.lock().unwrap();
        assert_eq!(*releases, vec!["release_second", "release_first"]);
    }

    #[tokio::test]
    async fn bracket_three_resources() {
        let counter = Arc::new(AtomicU32::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let result = bracket! {
            a <- Effect::<_, String, ()>::pure(1),
                 |_| { c1.fetch_add(1, Ordering::SeqCst); async { Ok(()) } };
            b <- Effect::pure(2),
                 |_| { c2.fetch_add(1, Ordering::SeqCst); async { Ok(()) } };
            c <- Effect::pure(3),
                 |_| { c3.fetch_add(1, Ordering::SeqCst); async { Ok(()) } };
            =>
            Effect::pure(a + b + c)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(6));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn bracket_cleanup_on_error() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket! {
            _val <- Effect::<i32, String, ()>::pure(42),
                    |_| { released_clone.store(true, Ordering::SeqCst); async { Ok(()) } };
            =>
            Effect::<i32, String, ()>::fail("error".to_string())
        }
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(released.load(Ordering::SeqCst), "cleanup must run on error");
    }

    #[tokio::test]
    async fn bracket_discard_pattern() {
        let acquired = Arc::new(AtomicBool::new(false));
        let acquired_clone = acquired.clone();

        let result = bracket! {
            _ <- Effect::<_, String, ()>::from_fn(move |_| {
                     acquired_clone.store(true, Ordering::SeqCst);
                     Ok(())
                 }),
                 |_| async { Ok(()) };
            =>
            Effect::pure(42)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(42));
        assert!(acquired.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bracket_with_effect_macro() {
        let result = bracket! {
            conn <- Effect::<_, String, ()>::pure("connection"),
                    |_| async { Ok(()) };
            =>
            effect! {
                data <- Effect::pure(format!("{}_data", conn));
                Effect::pure(data.len())
            }
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(15)); // "connection_data".len()
    }
}
```

### Compile-Fail Tests

```rust
// tests/ui/bracket_too_many_resources.rs
fn main() {
    let _ = bracket! {
        a <- Effect::pure(1), |_| async { Ok(()) };
        b <- Effect::pure(2), |_| async { Ok(()) };
        c <- Effect::pure(3), |_| async { Ok(()) };
        d <- Effect::pure(4), |_| async { Ok(()) };
        e <- Effect::pure(5), |_| async { Ok(()) }; // 5th resource - error!
        =>
        Effect::pure(a + b + c + d + e)
    };
}
// Expected error: "bracket! supports at most 4 resources"
```

## Documentation Requirements

- **Code Documentation**: Doc comments with examples on macro
- **User Documentation**: README section on resource management with bracket!
- **Examples**: `examples/bracket_macro.rs` showing real-world patterns

## Implementation Notes

### Why Limit to 4 Resources?

1. **Spec 002 limitation**: `with_flat` only implemented for 2-4 resources
2. **Complexity**: Each resource count needs a separate macro arm
3. **Practical**: 4+ resources is rare; nesting works for edge cases
4. **Error messages**: Clear guidance to use nesting for more

### Cleanup Expression Evaluation

The cleanup expression is evaluated at cleanup time, not macro expansion time:

```rust
bracket! {
    conn <- open_conn(), |c| c.close();  // c.close() called during cleanup
    =>
    use_conn(&conn)
}
```

This is correct behavior - we want cleanup to run with the actual resource.

### Integration with Spec 031 (effect!)

The macros compose naturally:

```rust
bracket! {
    conn <- db_conn(), |c| c.close();
    =>
    effect! {                          // effect! in body
        x <- fetch(&conn);
        Effect::pure(x)
    }
}
```

## Migration and Compatibility

### Backward Compatibility

Purely additive - existing `Acquiring` builder usage unchanged.

### Migration Path

```rust
// Before: manual builder
Effect::acquiring(open_conn(), |c| async move { c.close().await })
    .and(open_file(), |f| async move { f.close().await })
    .with_flat(|conn, file| do_work(conn, file))

// After: bracket! macro
bracket! {
    conn <- open_conn(), |c| c.close();
    file <- open_file(), |f| f.close();
    =>
    do_work(&conn, &file)
}
```

## Related Specifications

- **Spec 002**: Resource Scopes and Bracket Pattern - Foundation this builds on
- **Spec 031**: Effect Do-Notation Macro - Composes with bracket! for body
- **Spec 033**: scoped! Macro (planned) - Combines effect! and bracket!

---

*"Acquire, use, release. Make it look easy."*
