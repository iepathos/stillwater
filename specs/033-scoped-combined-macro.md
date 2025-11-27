---
number: 33
title: Scoped Combined Macro
category: foundation
priority: low
status: draft
dependencies: [2, 31, 32]
created: 2025-11-27
revised: 2025-11-27
---

# Specification 033: Scoped Combined Macro

**Category**: foundation
**Priority**: low
**Status**: draft
**Dependencies**: Spec 002 (Resource Scopes), Spec 031 (effect!), Spec 032 (bracket!)

## Context

Specs 031 and 032 provide `effect!` (do-notation) and `bracket!` (resource management)
macros respectively. While powerful individually, combining them still requires nesting:

```rust
// Current: bracket! with effect! nested inside
bracket! {
    lock <- order_lock(id), |l| l.release();
    conn <- db_connection(), |c| c.close();
    =>
    effect! {
        order <- fetch_order(&conn, order_id);
        _ <- validate_order(&order);
        result <- process_order(&conn, &order);
        Effect::pure(result)
    }
}
```

This is already a significant improvement, but the separation between resource
acquisition and sequential effect composition creates visual noise. A unified
`scoped!` macro can provide the cleanest possible syntax.

### Inspiration

The `scoped!` macro draws inspiration from:

- **Python**: `async with` + normal sequential code
- **C#**: `await using` + normal sequential code
- **Haskell**: `ResourceT` monad transformer allowing interleaved resource/effect operations
- **ZIO 2**: Scopes with `ZIO.acquireRelease` integrated into for-comprehensions

### Philosophy Alignment

From PHILOSOPHY.md: *"Pragmatism over purity"*

The `scoped!` macro is the most ambitious of the three. It should only be implemented
if real-world usage of `effect!` and `bracket!` demonstrates the need. The spec is
written to capture the design intent, not mandate immediate implementation.

## Objective

Add a `scoped!` macro to stillwater that:

1. Unifies resource acquisition and effect sequencing in one syntax
2. Allows interleaved `use` (resources) and `let` (effects) statements
3. Provides the most ergonomic experience for complex Effect code
4. Compiles to optimal nested bracket/and_then chains
5. Maintains full type safety and inference

## Requirements

### Functional Requirements

#### FR-1: Unified Syntax

Combine resource acquisition and effect binding in one block:

```rust
scoped! {
    use lock = order_lock(id) => |l| l.release();
    use conn = db_connection() => |c| c.close();

    let order = fetch_order(&conn, &order_id)?;
    validate_order(&order)?;

    use tx = db_transaction() => |t| t.rollback();

    update_status(&tx, &order.id, Fulfilled)?;
    decrement_inventory(&tx, &order)?;
    tx.commit()?;

    Order { status: Fulfilled, ..order }
}
```

This would expand to properly nested `bracket!` and `effect!` calls.

#### FR-2: Resource Statements (`use`)

Declare resources with cleanup:

```rust
use name = acquire_effect => |var| cleanup_expr;
```

- Resources are acquired in order
- Cleanup runs in reverse order (LIFO)
- Resources are available to all subsequent statements

#### FR-3: Effect Binding with `?`

Bind effect results using `?` operator syntax:

```rust
let order = fetch_order(&conn, id)?;  // Binds Ok value, propagates Err
```

This is syntactic sugar for:
```rust
order <- fetch_order(&conn, id);  // In effect! notation
```

#### FR-4: Pure Let Statements

Regular `let` without `?` for pure computations:

```rust
let total = order.items.iter().map(|i| i.price).sum();
```

#### FR-5: Expression Statements with `?`

Execute effects for side effects:

```rust
validate_order(&order)?;  // Execute, propagate error, discard Ok value
```

Equivalent to:
```rust
_ <- validate_order(&order);  // In effect! notation
```

#### FR-6: Final Expression

The block must end with a value expression (the success value):

```rust
scoped! {
    use conn = db_connection() => |c| c.close();
    let data = fetch_data(&conn)?;
    data.process()  // Final expression - must be an Effect or convertible to one
}
```

#### FR-7: Nested Scopes

Support nesting for conditional resource acquisition:

```rust
scoped! {
    use conn = db_connection() => |c| c.close();
    let order = fetch_order(&conn, id)?;

    if order.needs_lock {
        scoped! {
            use lock = acquire_lock(order.id) => |l| l.release();
            process_with_lock(&conn, &lock, &order)?
        }
    } else {
        process_without_lock(&conn, &order)?
    }
}
```

#### FR-8: Resource Scope Boundaries

Resources acquired in a `scoped!` block are released when the block exits:

```rust
scoped! {
    use conn = db_connection() => |c| c.close();

    let result = scoped! {
        use tx = begin_transaction(&conn) => |t| t.rollback();
        do_work(&tx)?;
        tx.commit()?
        // tx released here
    };

    // conn still available here
    log_result(&conn, &result)?

    // conn released here
}
```

### Non-Functional Requirements

#### NFR-1: Zero Runtime Overhead

Expands to nested `bracket!` and `effect!` calls, which expand to builder chains.
No additional allocations or runtime dispatch.

#### NFR-2: Reasonable Error Messages

While macro error messages are inherently limited, provide:

- Clear errors for syntax mistakes
- Type errors point to user code, not macro internals
- Suggestions for common mistakes

#### NFR-3: Predictable Expansion

Users should understand what code is generated:

1. `use` statements group into `bracket!` calls
2. `let x = expr?` becomes effect binding
3. `expr?` becomes discard binding
4. Final expression is the body

## Acceptance Criteria

### Must Have

- [ ] `scoped!` macro with `use` and `let ... ?` syntax
- [ ] Support for interleaved resources and effects
- [ ] LIFO cleanup ordering for resources
- [ ] Pure `let` bindings (no `?`)
- [ ] Expression statements with `?`
- [ ] Final expression requirement
- [ ] Unit tests for various combinations
- [ ] Documentation with examples

### Should Have

- [ ] Nested `scoped!` support
- [ ] Helpful error messages for common mistakes
- [ ] Integration examples showing real-world patterns

### Won't Have (This Version)

- [ ] `if`/`else`/`match` as statements (use as expressions in Effect context)
- [ ] `for` loops (use `Effect::traverse` or similar)
- [ ] `while` loops (Effect is not designed for imperative loops)
- [ ] Automatic error type coercion (use `.map_error()` explicitly)

## Technical Details

### Implementation Approach

The `scoped!` macro is the most complex of the three. It requires:

1. **Parsing**: Distinguish `use`, `let ... ?`, `expr?`, and `let` statements
2. **Grouping**: Consecutive `use` statements form a single `bracket!` block
3. **Code Generation**: Emit nested `bracket!` and `effect!` invocations

#### Conceptual Expansion

**Input:**
```rust
scoped! {
    use conn = db_connection() => |c| c.close();
    use tx = begin_transaction(&conn) => |t| t.rollback();

    let order = fetch_order(&conn, id)?;
    validate(&order)?;

    use lock = acquire_lock(order.id) => |l| l.release();

    process(&tx, &lock, &order)?;
    tx.commit()?;

    order
}
```

**Expands to (conceptually):**
```rust
bracket! {
    conn <- db_connection(), |c| c.close();
    tx <- begin_transaction(&conn), |t| t.rollback();
    =>
    effect! {
        order <- fetch_order(&conn, id);
        _ <- validate(&order);

        result <- bracket! {
            lock <- acquire_lock(order.id), |l| l.release();
            =>
            effect! {
                _ <- process(&tx, &lock, &order);
                _ <- tx.commit();
                Effect::pure(order)
            }
        };

        Effect::pure(result)
    }
}
```

#### Macro Structure (Procedural Macro Recommended)

Due to the complexity of parsing interleaved statements, a **procedural macro**
is recommended over `macro_rules!`. This allows:

1. Proper parsing of Rust syntax
2. Better error messages with spans
3. More sophisticated code generation

```rust
// In a proc-macro crate (e.g., stillwater-macros)

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Block, Stmt, Expr};

#[proc_macro]
pub fn scoped(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as ScopedBlock);

    // Parse statements into:
    // - UseStmt: use name = expr => |var| cleanup;
    // - EffectBind: let name = expr?;
    // - EffectExec: expr?;
    // - PureLet: let name = expr;
    // - FinalExpr: expr (last item, no semicolon)

    // Group consecutive UseStmts into bracket blocks
    // Interleave with effect! sequences

    let expanded = generate_code(&block);
    TokenStream::from(expanded)
}
```

#### Alternative: Declarative Macro (Simpler but Limited)

A `macro_rules!` version is possible but with limitations:

```rust
#[macro_export]
macro_rules! scoped {
    // Base case: final expression
    (@body [] $final:expr) => {
        $crate::Effect::pure($final)
    };

    // Resource statement
    (@body [use $name:ident = $acquire:expr => |$cv:ident| $cleanup:expr; $($rest:tt)*] $final:expr) => {
        $crate::bracket! {
            $name <- $acquire, |$cv| $cleanup;
            =>
            $crate::scoped!(@body [$($rest)*] $final)
        }
    };

    // Effect binding: let x = expr?;
    (@body [let $name:ident = $e:expr ?; $($rest:tt)*] $final:expr) => {
        $e.and_then(|$name| $crate::scoped!(@body [$($rest)*] $final))
    };

    // Effect execution: expr?;
    (@body [$e:expr ?; $($rest:tt)*] $final:expr) => {
        $e.and_then(|_| $crate::scoped!(@body [$($rest)*] $final))
    };

    // Pure let binding
    (@body [let $name:pat = $e:expr; $($rest:tt)*] $final:expr) => {
        {
            let $name = $e;
            $crate::scoped!(@body [$($rest)*] $final)
        }
    };

    // Entry point: collect all statements
    ({ $($stmt:tt)* }) => {
        // This is where it gets tricky with macro_rules!
        // We need to separate statements from final expression
        $crate::scoped!(@parse {} $($stmt)*)
    };
}
```

**Limitation**: `macro_rules!` struggles to distinguish `expr?;` from `expr` at
the statement level without significant complexity.

### Recommended Implementation Path

Given the complexity:

1. **Phase 1**: Implement `effect!` and `bracket!` as `macro_rules!` (simpler)
2. **Phase 2**: Evaluate real-world usage patterns
3. **Phase 3**: If `scoped!` is needed, implement as proc-macro for proper parsing

### Architecture Changes

If implementing as proc-macro:

```
stillwater/
├── Cargo.toml           # Depends on stillwater-macros
├── src/
│   └── lib.rs           # Re-exports macros
└── stillwater-macros/   # New proc-macro crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── effect.rs    # effect! (could move here from macro_rules!)
        ├── bracket.rs   # bracket! (could move here)
        └── scoped.rs    # scoped! proc-macro
```

## Dependencies

- **Prerequisites**:
  - Spec 002 (Resource Scopes) - Foundation
  - Spec 031 (effect!) - Do-notation macro
  - Spec 032 (bracket!) - Resource macro
- **Affected Components**:
  - New proc-macro crate (recommended)
  - `lib.rs` - Re-export macro
- **External Dependencies**:
  - `syn` - Rust syntax parsing
  - `quote` - Code generation
  - `proc-macro2` - Proc-macro utilities

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scoped_single_resource_single_effect() {
        let result = scoped! {
            use val = Effect::<_, String, ()>::pure(42) => |_| async { Ok(()) };
            let doubled = val * 2;
            doubled
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
    }

    #[tokio::test]
    async fn scoped_multiple_resources() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let o1 = order.clone();
        let o2 = order.clone();

        let result = scoped! {
            use a = Effect::<_, String, ()>::pure(1)
                => |_| { o1.lock().unwrap().push("a"); async { Ok(()) } };
            use b = Effect::pure(2)
                => |_| { o2.lock().unwrap().push("b"); async { Ok(()) } };

            a + b
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(3));
        assert_eq!(*order.lock().unwrap(), vec!["b", "a"]); // LIFO
    }

    #[tokio::test]
    async fn scoped_effect_bindings() {
        let result = scoped! {
            use conn = Effect::<_, String, ()>::pure("conn") => |_| async { Ok(()) };

            let data = Effect::pure(format!("{}_data", conn))?;
            let len = data.len();

            len
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(9)); // "conn_data".len()
    }

    #[tokio::test]
    async fn scoped_effect_execution() {
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        let result = scoped! {
            use val = Effect::<_, String, ()>::pure(42) => |_| async { Ok(()) };

            Effect::from_fn(move |_| {
                executed_clone.store(true, Ordering::SeqCst);
                Ok(())
            })?;

            val
        }
        .run(&())
        .await;

        assert_eq!(result, Ok(42));
        assert!(executed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn scoped_interleaved_resources_and_effects() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let o1 = order.clone();
        let o2 = order.clone();

        let result = scoped! {
            use a = Effect::<_, String, ()>::pure("a")
                => |_| { o1.lock().unwrap().push("release_a"); async { Ok(()) } };

            let x = Effect::pure(1)?;

            use b = Effect::pure("b")
                => |_| { o2.lock().unwrap().push("release_b"); async { Ok(()) } };

            let y = Effect::pure(2)?;

            format!("{}{}:{}", a, b, x + y)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok("ab:3".to_string()));
        // b released first (inner scope), then a
        assert_eq!(*order.lock().unwrap(), vec!["release_b", "release_a"]);
    }

    #[tokio::test]
    async fn scoped_cleanup_on_error() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = scoped! {
            use _val = Effect::<i32, String, ()>::pure(42)
                => |_| { released_clone.store(true, Ordering::SeqCst); async { Ok(()) } };

            Effect::<i32, String, ()>::fail("error".to_string())?;

            42  // Never reached
        }
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn scoped_nested() {
        let result = scoped! {
            use conn = Effect::<_, String, ()>::pure("conn") => |_| async { Ok(()) };

            let inner = scoped! {
                use tx = Effect::pure("tx") => |_| async { Ok(()) };
                format!("{}_{}", conn, tx)
            }?;

            format!("{}_done", inner)
        }
        .run(&())
        .await;

        assert_eq!(result, Ok("conn_tx_done".to_string()));
    }
}
```

## Documentation Requirements

- **Code Documentation**: Comprehensive doc comments with examples
- **User Documentation**: README section showing the progression from manual → effect! → bracket! → scoped!
- **Examples**: `examples/scoped_macro.rs` with realistic scenarios
- **Migration Guide**: How to convert nested bracket!/effect! to scoped!

## Implementation Notes

### When to Implement

This spec should be implemented **only if**:

1. Spec 031 (effect!) and Spec 032 (bracket!) are implemented and stable
2. Real-world usage demonstrates that nesting them is cumbersome
3. The proc-macro complexity is justified by ergonomic gains

### Complexity Budget

The `scoped!` macro is the most complex:

| Macro | Implementation | Complexity |
|-------|---------------|------------|
| `effect!` | macro_rules! | Low |
| `bracket!` | macro_rules! | Medium |
| `scoped!` | proc-macro | High |

If the simpler macros provide sufficient ergonomics, `scoped!` can remain unimplemented.

### Alternative: Don't Implement

The combination of `bracket!` and `effect!` may be "good enough":

```rust
bracket! {
    conn <- db_connection(), |c| c.close();
    tx <- db_transaction(), |t| t.rollback();
    =>
    effect! {
        order <- fetch_order(&conn, id);
        _ <- validate(&order);
        _ <- process(&tx, &order);
        _ <- tx.commit();
        Effect::pure(order)
    }
}
```

This is already quite readable. The `scoped!` macro saves one level of nesting
and allows interleaving, but may not be worth the proc-macro complexity.

## Migration and Compatibility

### Backward Compatibility

Purely additive. Existing code using `effect!` and `bracket!` unchanged.

### Migration Path

```rust
// From bracket! + effect!:
bracket! {
    conn <- db_connection(), |c| c.close();
    =>
    effect! {
        order <- fetch_order(&conn, id);
        _ <- validate(&order);
        Effect::pure(order)
    }
}

// To scoped!:
scoped! {
    use conn = db_connection() => |c| c.close();
    let order = fetch_order(&conn, id)?;
    validate(&order)?;
    order
}
```

## Related Specifications

- **Spec 002**: Resource Scopes and Bracket Pattern - Foundation
- **Spec 031**: Effect Do-Notation Macro - `effect!` macro
- **Spec 032**: Bracket Resource Macro - `bracket!` macro

## Open Questions

1. **Proc-macro vs macro_rules!**: Is the added complexity of a proc-macro justified?
2. **? syntax ambiguity**: Does `let x = expr?` conflict with Rust's native `?` operator?
3. **Scope boundaries**: How should resource scopes interact with `if`/`match` expressions?
4. **Error types**: Should the macro help with error type unification?

These questions should be resolved through experimentation with the simpler macros first.

---

*"Simple things should be simple. Complex things should be possible."*
