---
number: 025
title: Documentation Update for Zero-Cost Effect API
category: foundation
priority: high
status: draft
dependencies: [024]
created: 2025-11-26
---

# Specification 025: Documentation Update for Zero-Cost Effect API

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

Spec 024 introduces a significant API redesign for Stillwater's Effect system, moving from always-boxed to zero-cost-by-default with opt-in boxing. This change affects all documentation across the project.

### Current Documentation Issues

The current documentation contains inaccurate claims about boxing behavior:

1. **PHILOSOPHY.md** (previously fixed but needs updating for new API)
   - Now accurately describes per-combinator boxing
   - Needs to be updated to describe zero-cost default

2. **README.md**
   - All Effect examples use old API
   - Performance claims need updating

3. **DESIGN.md**
   - Effect struct definition outdated
   - Boxing discussion needs reframing

4. **docs/FAQ.md**
   - Performance overhead answers need updating
   - New FAQ items needed for when to use `.boxed()`

5. **docs/guide/03-effects.md**
   - All examples use old API
   - Performance section needs complete rewrite

6. **All examples/**
   - Every example file uses old `Effect<T, E, Env>` syntax
   - Need migration to new `impl Effect<...>` or `BoxedEffect`

## Objective

Update all documentation to accurately reflect the new zero-cost Effect API, providing clear guidance on:

1. When to use the zero-cost API (default)
2. When to use `.boxed()` (collections, recursion, match arms)
3. Migration from old API to new API
4. Performance characteristics

## Requirements

### Functional Requirements

#### FR1: README.md Updates

- **MUST** update all Effect examples to new API
- **MUST** update "Why Stillwater?" section for zero-cost claims
- **MUST** update comparison section
- **MUST** add "Zero-Cost Abstractions" section highlighting the design

#### FR2: PHILOSOPHY.md Updates

- **MUST** update "Why Box in some places?" section
- **MUST** explain zero-cost default vs opt-in boxing
- **MUST** add "When to Box" guidance
- **SHOULD** add comparison with `futures` crate pattern

#### FR3: DESIGN.md Updates

- **MUST** update Effect type definition
- **MUST** update Effect methods documentation
- **MUST** add section on trait-based design
- **MUST** update architecture diagrams if present

#### FR4: FAQ.md Updates

- **MUST** update performance overhead answers
- **MUST** add FAQ: "When should I use .boxed()?"
- **MUST** add FAQ: "Why did the API change?"
- **MUST** add FAQ: "How do I migrate from the old API?"

#### FR5: Guide Updates

- **MUST** update docs/guide/03-effects.md completely
- **MUST** add new section on boxing decisions
- **MUST** update all code examples
- **MUST** update performance considerations section

#### FR6: Example Updates

- **MUST** update all example files to new API
- **SHOULD** add example showing when to use `.boxed()`
- **SHOULD** add example comparing boxed vs unboxed performance

#### FR7: Migration Guide

- **MUST** create docs/MIGRATION.md for 0.7.x to 0.8.x migration
- **MUST** document all breaking changes
- **MUST** provide before/after code examples
- **MUST** explain compatibility module usage

### Non-Functional Requirements

#### NFR1: Consistency

- All documentation MUST use consistent terminology
- All examples MUST follow the same patterns
- All performance claims MUST be accurate and verifiable

#### NFR2: Clarity

- New users MUST understand when to use boxed vs unboxed
- Migration path MUST be clear for existing users
- Type annotations in examples MUST be minimal but sufficient

## Acceptance Criteria

### README.md

- [ ] **AC1**: Effect examples use new `impl Effect<...>` syntax
- [ ] **AC2**: Zero-cost claims are prominent and accurate
- [ ] **AC3**: Quick comparison table updated
- [ ] **AC4**: No references to old `Effect<T, E, Env>` struct

### PHILOSOPHY.md

- [ ] **AC5**: "Why Box" section explains opt-in boxing
- [ ] **AC6**: New "When to Box" section added
- [ ] **AC7**: Comparison with `futures` crate pattern included

### DESIGN.md

- [ ] **AC8**: Effect trait definition documented
- [ ] **AC9**: Combinator types documented
- [ ] **AC10**: BoxedEffect documented with use cases

### FAQ.md

- [ ] **AC11**: "When should I use .boxed()?" FAQ added
- [ ] **AC12**: "How do I migrate?" FAQ added
- [ ] **AC13**: Performance FAQ updated

### Guide

- [ ] **AC14**: 03-effects.md fully updated
- [ ] **AC15**: Boxing decision guide section added
- [ ] **AC16**: All code examples compile with new API

### Examples

- [ ] **AC17**: All example files updated
- [ ] **AC18**: New boxing_decisions.rs example added
- [ ] **AC19**: All examples pass `cargo test --examples`

### Migration Guide

- [ ] **AC20**: MIGRATION.md created
- [ ] **AC21**: All breaking changes documented
- [ ] **AC22**: Compatibility module documented

## Technical Details

### Key Documentation Changes

#### README.md - Before/After

**Before:**
```markdown
## Why Stillwater?

**vs. monadic:**
- ✓ Minimal overhead (one small Box per combinator, negligible for I/O-bound work)
```

**After:**
```markdown
## Why Stillwater?

**vs. monadic:**
- ✓ Zero-cost abstractions (no allocation by default, opt-in boxing when needed)
- ✓ Follows the `futures` crate pattern familiar to Rustaceans
```

**Before (example):**
```rust
use stillwater::Effect;

fn create_user(email: String) -> Effect<User, AppError, AppEnv> {
    Effect::pure(email)
        .and_then(|e| validate_email(e))
        .and_then(|e| save_user(e))
}
```

**After (example):**
```rust
use stillwater::effect::prelude::*;

fn create_user(email: String) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    pure(email)
        .and_then(validate_email)
        .and_then(save_user)
}

// Or when you need type erasure:
fn create_user_boxed(email: String) -> BoxedEffect<User, AppError, AppEnv> {
    pure(email)
        .and_then(validate_email)
        .and_then(save_user)
        .boxed()
}
```

#### PHILOSOPHY.md - New Section

```markdown
### Why Box in some places?

Stillwater follows the **`futures` crate pattern**: zero-cost by default, explicit boxing when needed.

**Zero-cost by default:**

Each combinator returns a concrete type that encodes the operation:

```rust
pure(42)            // Type: Pure<i32, E, Env>
    .map(|x| x + 1) // Type: Map<Pure<i32, E, Env>, impl FnOnce...>
    .and_then(...)  // Type: AndThen<Map<...>, impl FnOnce...>
```

No heap allocation occurs. The compiler can inline everything.

**Opt-in boxing with `.boxed()`:**

Use `.boxed()` when you need type erasure:

```rust
// Store different effects in a collection
let effects: Vec<BoxedEffect<i32, E, Env>> = vec![
    pure(1).boxed(),
    pure(2).map(|x| x * 2).boxed(),
];

// Recursive effects
fn countdown(n: i32) -> BoxedEffect<i32, E, Env> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n).and_then(move |x| countdown(x - 1)).boxed()
    }
}

// Match arms with different effect types
fn get_user(source: Source) -> BoxedEffect<User, E, Env> {
    match source {
        Source::Cache => from_cache(id).boxed(),
        Source::Database => from_db(id).map(|u| u.clone()).boxed(),
    }
}
```

### When to Box

| Situation | Box? | Reason |
|-----------|------|--------|
| Simple effect chain | No | Zero-cost default |
| Return `impl Effect` | No | Concrete type inferred |
| Store in `Vec`/`HashMap` | Yes | Need uniform type |
| Recursive function | Yes | Break infinite type |
| Match with different effect types | Yes | All arms same type |
| Cross-crate API boundary | Maybe | `impl Effect` often works |
```

#### New Migration Guide

```markdown
# Migration Guide: Stillwater 0.7.x to 0.8.x

## Overview

Stillwater 0.8.0 introduces a zero-cost Effect API, following the `futures` crate pattern. This is a breaking change that requires updating your code.

## Key Changes

| 0.7.x | 0.8.x |
|-------|-------|
| `Effect<T, E, Env>` struct | `impl Effect<Output=T, Error=E, Env=Env>` trait |
| `Effect::pure(x)` | `pure::<_, E, Env>(x)` |
| `Effect::fail(e)` | `fail::<T, _, Env>(e)` |
| `Effect::from_fn(f)` | `from_fn(f)` |
| Always boxed | Zero-cost, opt-in `.boxed()` |

## Migration Steps

### Step 1: Update Imports

```rust
// Before
use stillwater::Effect;

// After
use stillwater::effect::prelude::*;
```

### Step 2: Update Return Types

```rust
// Before
fn my_effect() -> Effect<i32, String, ()> {
    Effect::pure(42)
}

// After - Option A: Zero-cost (preferred)
fn my_effect() -> impl Effect<Output = i32, Error = String, Env = ()> {
    pure(42)
}

// After - Option B: Boxed (when needed)
fn my_effect() -> BoxedEffect<i32, String, ()> {
    pure(42).boxed()
}
```

### Step 3: Update Constructor Calls

```rust
// Before
Effect::pure(42)
Effect::fail("error")
Effect::from_fn(|env| Ok(env.value))

// After
pure(42)
fail("error")
from_fn(|env| Ok(env.value))
```

### Step 4: Add `.boxed()` Where Needed

If you're storing effects in collections, using recursion, or returning different effect types from match arms, add `.boxed()`:

```rust
// Collections
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).boxed(),
];

// Recursion
fn recursive(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n).and_then(move |_| recursive(n - 1)).boxed()
    }
}
```

## Using the Compatibility Module

For gradual migration, use the compatibility module:

```rust
use stillwater::compat::Effect; // Type alias for BoxedEffect

// Old code continues to work (with deprecation warnings)
fn my_effect() -> Effect<i32, String, ()> {
    Effect::pure(42)
}
```

This is deprecated and will be removed in 0.9.0. Migrate to the new API as soon as possible.

## Common Issues

### "expected struct, found opaque type"

You're returning `impl Effect` but the caller expects a concrete type. Either:
1. Use `.boxed()` to get `BoxedEffect`
2. Update the caller to accept `impl Effect`

### "cannot infer type"

Add type annotations to constructor functions:
```rust
pure::<_, String, ()>(42)  // Specify error and env types
```

### "the trait bound is not satisfied"

Make sure your closures are `Send`:
```rust
// Before (might not be Send)
.map(|x| x + some_local_ref)

// After (capture by value)
let value = *some_local_ref;
.map(move |x| x + value)
```
```

#### New Example File

```rust
// examples/boxing_decisions.rs

//! Demonstrates when to use `.boxed()` vs zero-cost effects.
//!
//! Run with: cargo run --example boxing_decisions

use stillwater::effect::prelude::*;

// ============================================================================
// ZERO-COST: Simple effect chains
// ============================================================================

/// Zero-cost effect chain - no heap allocation.
fn zero_cost_example() -> impl Effect<Output = i32, Error = String, Env = ()> {
    pure(1)
        .map(|x| x + 1)
        .map(|x| x * 2)
        .and_then(|x| pure(x + 10))
}

// ============================================================================
// BOXED: Storing in collections
// ============================================================================

/// Must use `.boxed()` to store different effects in a Vec.
fn collection_example() -> Vec<BoxedEffect<i32, String, ()>> {
    vec![
        pure(1).boxed(),
        pure(2).map(|x| x * 2).boxed(),
        pure(3).and_then(|x| pure(x * 3)).boxed(),
    ]
}

// ============================================================================
// BOXED: Recursive effects
// ============================================================================

/// Recursive effects require `.boxed()` to break the infinite type.
fn recursive_sum(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n)
            .and_then(move |x| recursive_sum(x - 1).map(move |sum| x + sum))
            .boxed()
    }
}

// ============================================================================
// BOXED: Match arms with different types
// ============================================================================

enum DataSource {
    Cache,
    Database,
    Remote,
}

/// Different match arms have different types, need `.boxed()`.
fn fetch_data(source: DataSource) -> BoxedEffect<String, String, ()> {
    match source {
        DataSource::Cache => {
            pure("cached data".to_string()).boxed()
        }
        DataSource::Database => {
            pure("db")
                .map(|s| format!("{} data", s))
                .boxed()
        }
        DataSource::Remote => {
            pure("remote")
                .and_then(|s| pure(format!("{} data", s)))
                .boxed()
        }
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== Zero-Cost Effect Demo ===\n");

    // Zero-cost chain
    let result = zero_cost_example().execute(&()).await;
    println!("Zero-cost chain: {:?}", result);

    // Collection of effects
    let effects = collection_example();
    println!("\nCollection of {} effects:", effects.len());
    for (i, effect) in effects.into_iter().enumerate() {
        let result = effect.execute(&()).await;
        println!("  Effect {}: {:?}", i, result);
    }

    // Recursive effect
    let sum = recursive_sum(5).execute(&()).await;
    println!("\nRecursive sum(5): {:?}", sum);

    // Match arms
    for source in [DataSource::Cache, DataSource::Database, DataSource::Remote] {
        let data = fetch_data(source).execute(&()).await;
        println!("Fetch data: {:?}", data);
    }
}
```

### Files to Update

| File | Changes |
|------|---------|
| `README.md` | Examples, performance claims, comparison |
| `PHILOSOPHY.md` | Boxing section, new "When to Box" |
| `DESIGN.md` | Effect type definition, trait design |
| `docs/FAQ.md` | Performance FAQ, new FAQs |
| `docs/guide/03-effects.md` | Complete rewrite |
| `docs/MIGRATION.md` | New file |
| `examples/*.rs` | All examples |

## Dependencies

### Prerequisites
- Spec 024 implemented (Zero-Cost Effect Trait)

### Affected Components
- All markdown documentation
- All example files
- API documentation (rustdoc)

## Testing Strategy

### Documentation Tests

- All code examples in markdown MUST compile
- Use `cargo test --doc` to verify

### Example Tests

- All examples MUST run successfully
- Use `cargo run --example <name>` to verify each

### Link Verification

- All internal links MUST be valid
- Check with markdown link checker

## Implementation Notes

### Order of Updates

1. **MIGRATION.md** first - provides reference for other updates
2. **PHILOSOPHY.md** - core design rationale
3. **DESIGN.md** - technical architecture
4. **README.md** - main entry point
5. **FAQ.md** - common questions
6. **Guide** - detailed usage
7. **Examples** - working code

### Style Guidelines

- Use consistent terminology: "zero-cost", "boxed", "type erasure"
- Always show both zero-cost and boxed versions where relevant
- Include type annotations in examples for clarity
- Explain WHY, not just WHAT

## Success Metrics

### Quantitative
- All documentation code examples compile
- All examples run without errors
- No broken links

### Qualitative
- New users understand when to use `.boxed()`
- Existing users can migrate successfully
- Documentation accurately reflects implementation

---

*"Clear docs, confident users."*
