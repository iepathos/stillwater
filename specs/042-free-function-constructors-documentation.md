---
number: 042
title: Free Function Constructors Documentation Enhancement
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 042: Free Function Constructors Documentation Enhancement

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

Stillwater provides "free functions" (standalone functions) as an alternative to associated function syntax for creating effects. For example, instead of writing:

```rust
Effect::asks(|env: &MapEnv| env.worktree_manager.clone())
```

Users can write:

```rust
use stillwater::effect::prelude::*;

asks(|env: &MapEnv| env.worktree_manager.clone())
```

This pattern is common in functional programming libraries and provides:
- More concise, readable code
- Better alignment with FP idioms (Haskell, Scala ZIO, etc.)
- Reduced visual noise when composing multiple effects
- Natural function composition without type prefixes

### Current State

The free functions already exist in `src/effect/constructors.rs`:
- `pure` - Create a pure effect with a value
- `fail` - Create a failing effect with an error
- `from_fn` - Create effect from a synchronous function
- `from_async` - Create effect from an async function
- `from_result` - Create effect from a Result
- `from_option` - Create effect from an Option
- `from_validation` - Create effect from a Validation
- `ask` - Get the entire environment (Reader monad)
- `asks` - Query a value from the environment (Reader monad)
- `local` - Run an effect with a modified environment
- `zip3` through `zip8` - Combine multiple effects

These are exported via:
- `src/effect/mod.rs` - Direct re-exports
- `src/effect/prelude.rs` - Prelude re-exports

### Gap Analysis

While the implementation is complete, documentation and examples are inconsistent:
- Some doc examples still use `Effect::pure(x)` style (legacy API)
- `docs/PATTERNS.md` mixes old and new styles
- No dedicated section explaining the free function pattern
- Examples don't consistently import from prelude
- The ergonomic benefits aren't clearly communicated

## Objective

Enhance documentation to prominently feature the free function pattern, update all examples to use the ergonomic style, and add comprehensive documentation explaining when and why to use free functions vs. associated functions.

## Requirements

### Functional Requirements

#### FR1: Free Functions Documentation Section

Add a dedicated documentation section covering free function constructors:

- **MUST** explain what free functions are and their benefits
- **MUST** list all available free functions with signatures
- **MUST** show import patterns (`use stillwater::effect::prelude::*`)
- **MUST** compare before/after for readability improvement
- **SHOULD** reference Haskell/Scala origins for FP-aware developers

#### FR2: Constructor Module Documentation Enhancement

Improve `src/effect/constructors.rs` documentation:

- **MUST** add module-level documentation explaining the free function pattern
- **MUST** update all doc examples to use ````rust` (not `ignore`)
- **MUST** ensure all examples compile and are tested
- **SHOULD** add "See Also" cross-references between related functions

#### FR3: Prelude Documentation Enhancement

Improve `src/effect/prelude.rs` documentation:

- **MUST** document that this is the recommended import for most use cases
- **MUST** show example of typical usage pattern
- **MUST** list all re-exported free functions
- **SHOULD** explain when direct imports might be preferred

#### FR4: PATTERNS.md Updates

Update `docs/PATTERNS.md` to consistently use free functions:

- **MUST** update all Effect pattern examples to use free function style
- **MUST** remove/update any `Effect::pure()`, `Effect::fail()` patterns
- **MUST** add imports to all examples
- **SHOULD** add a section on "Functional Style with Free Functions"

#### FR5: Examples Directory Updates

Update example files to showcase free function pattern:

- **MUST** update `examples/effects.rs` to use free functions
- **MUST** ensure examples compile with `cargo run --example`
- **SHOULD** add comments explaining the free function pattern

#### FR6: API Comparison Table

Add a clear comparison in documentation:

| Associated Function Style | Free Function Style |
|--------------------------|---------------------|
| `Effect::pure(42)` | `pure(42)` |
| `Effect::fail(err)` | `fail(err)` |
| `Effect::asks(\|e\| ...)` | `asks(\|e\| ...)` |
| `Effect::from_fn(f)` | `from_fn(f)` |

- **MUST** include in constructors.rs module docs
- **SHOULD** include in MIGRATION.md or README

### Non-Functional Requirements

#### NFR1: Consistency

- All documentation must use consistent terminology
- All examples must use the same import style
- No mixing of old/new API styles within a single file

#### NFR2: Compilation Verification

- All doc examples must compile (`cargo test --doc`)
- All example files must run successfully
- No `#[ignore]` attributes on working examples

#### NFR3: Discoverability

- Free functions should be easily discoverable in docs.rs
- IDE autocomplete should suggest free functions
- Module documentation should guide users to the right imports

## Acceptance Criteria

### Documentation Updates

- [ ] **AC1**: `constructors.rs` has comprehensive module-level documentation explaining free functions
- [ ] **AC2**: All `constructors.rs` doc examples compile (remove `ignore` attribute)
- [ ] **AC3**: `prelude.rs` has updated documentation explaining recommended usage
- [ ] **AC4**: `mod.rs` module docs mention free function pattern as preferred style

### PATTERNS.md Updates

- [ ] **AC5**: All Effect patterns use free function imports
- [ ] **AC6**: No `Effect::pure()` or `Effect::fail()` patterns remain (except in migration/legacy context)
- [ ] **AC7**: New section added: "Free Function Style"
- [ ] **AC8**: All examples include appropriate `use` statements

### Examples Updates

- [ ] **AC9**: `examples/effects.rs` updated to use free functions
- [ ] **AC10**: All example files compile and run successfully

### Comparison/Migration Docs

- [ ] **AC11**: API comparison table added showing old vs new style
- [ ] **AC12**: Clear recommendation for new code to use free functions

### Testing

- [ ] **AC13**: `cargo test --doc` passes for all updated examples
- [ ] **AC14**: All examples in docs/ directory are validated

## Technical Details

### Module Documentation Template

For `src/effect/constructors.rs`:

```rust
//! Free function constructors for creating effects.
//!
//! This module provides standalone functions as an ergonomic alternative to
//! associated function syntax. Instead of writing `Effect::pure(x)`, you can
//! write simply `pure(x)`.
//!
//! # Quick Start
//!
//! ```rust
//! use stillwater::effect::prelude::*;
//!
//! // Free functions for concise, readable effect creation
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x * 2)
//!     .and_then(|x| pure(x + 1));
//! ```
//!
//! # Available Free Functions
//!
//! ## Value Constructors
//! - [`pure`] - Create effect that succeeds with a value
//! - [`fail`] - Create effect that fails with an error
//!
//! ## Conversion Constructors
//! - [`from_fn`] - Create effect from synchronous function
//! - [`from_async`] - Create effect from async function
//! - [`from_result`] - Lift a `Result` into an effect
//! - [`from_option`] - Lift an `Option` into an effect
//! - [`from_validation`] - Convert `Validation` to effect
//!
//! ## Reader Operations
//! - [`ask`] - Get the entire environment
//! - [`asks`] - Query a value from environment
//! - [`local`] - Run effect with modified environment
//!
//! ## Combinators
//! - [`zip3`] through [`zip8`] - Combine multiple effects
//!
//! # Why Free Functions?
//!
//! Free functions provide several benefits:
//!
//! 1. **Conciseness**: Less visual noise in effect chains
//! 2. **FP Idiom**: Familiar to users of Haskell, Scala ZIO, etc.
//! 3. **Composability**: Functions compose naturally
//! 4. **Readability**: Focus on what, not which type
//!
//! ## Comparison
//!
//! ```rust,ignore
//! // Associated function style (verbose)
//! Effect::asks(|env: &Env| env.db.clone())
//!     .and_then(|db| Effect::from_async(move |_| db.query()))
//!
//! // Free function style (concise)
//! asks(|env: &Env| env.db.clone())
//!     .and_then(|db| from_async(move |_| db.query()))
//! ```
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/effect/constructors.rs` | Enhanced module docs, fix doc examples |
| `src/effect/prelude.rs` | Enhanced module docs |
| `src/effect/mod.rs` | Add note about free function pattern |
| `docs/PATTERNS.md` | Update all examples to use free functions |
| `examples/effects.rs` | Update to showcase free function style |

## Dependencies

### Prerequisites
- None - implementation is already complete

### Affected Components
- `src/effect/constructors.rs` - documentation only
- `src/effect/prelude.rs` - documentation only
- `src/effect/mod.rs` - documentation only
- `docs/PATTERNS.md` - example updates
- `examples/effects.rs` - example updates

### External Dependencies
- None

## Testing Strategy

### Documentation Tests

- **Doc Tests**: All `/// ``` code blocks must compile via `cargo test --doc`
- **Example Files**: `cargo run --example effects` must succeed
- **Manual Review**: Documentation should be reviewed for clarity

### Verification Steps

1. Run `cargo test --doc` to verify all doc examples
2. Run `cargo run --example effects` to verify example file
3. Build docs with `cargo doc --open` and review
4. Check docs.rs preview for proper rendering

## Documentation Requirements

### Code Documentation
- Module-level documentation for constructors.rs
- Updated function-level documentation with working examples
- Cross-references between related functions

### User Documentation
- Updated PATTERNS.md with free function style
- API comparison showing old vs new style

## Implementation Notes

### Key Points

1. **Don't Remove Associated Functions**: They should remain for backwards compatibility and cases where explicit typing is preferred

2. **Consistent Import Pattern**: All examples should use:
   ```rust
   use stillwater::effect::prelude::*;
   ```

3. **Type Annotations**: Free functions sometimes need explicit type annotations:
   ```rust
   // Type annotation needed when types can't be inferred
   let effect = pure::<_, String, ()>(42);

   // No annotation needed when context provides types
   fn example() -> impl Effect<Output = i32, Error = String, Env = ()> {
       pure(42)  // Types inferred from return type
   }
   ```

4. **Focus on `asks`**: The `asks` function is particularly important as it's the primary way to access environment dependencies - ensure its documentation is comprehensive.

### Common Pitfalls

- Don't make examples that require complex setup
- Ensure examples show realistic use cases
- Remember to test async examples appropriately

## Migration and Compatibility

No breaking changes - this is purely documentation and example updates. The free functions have existed since the zero-cost API redesign and are already stable.

## Success Metrics

### Quantitative
- All doc examples compile (`cargo test --doc` passes)
- All example files run successfully
- Zero `Effect::pure()` patterns in new documentation

### Qualitative
- New users can discover and use free functions easily
- Documentation clearly explains when to use which style
- Examples are copy-paste ready
