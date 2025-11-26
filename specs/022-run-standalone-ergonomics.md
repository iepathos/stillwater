---
number: 022
title: Effect::run_standalone() Ergonomic API
category: optimization
priority: medium
status: draft
dependencies: []
created: 2025-11-26
---

# Specification 022: Effect::run_standalone() Ergonomic API

**Category**: optimization
**Priority**: medium
**Status**: draft
**Dependencies**: none

## Context

When using `Effect<T, E, ()>` (effects that don't require an environment), users must currently pass an awkward `&()` reference:

```rust
let result = effect.run(&()).await;
```

This is syntactically ugly and confusing for newcomers who don't understand why they need to pass a reference to an empty tuple. The `()` environment means "no dependencies required" - the effect is self-contained. We should provide a cleaner API for this common case.

## Objective

Add an ergonomic `run_standalone()` method for effects that don't require an environment (`Env = ()`), eliminating the need to write `run(&())`. Update all examples and documentation to use this cleaner API.

## Requirements

### Functional Requirements

1. **Add `run_standalone()` method**
   - Implement on `Effect<T, E, ()>` specifically (not generic `Env`)
   - Signature: `pub async fn run_standalone(self) -> Result<T, E>`
   - Implementation: delegates to `self.run(&())`
   - Requires `async` feature to be enabled

2. **Method constraints**
   - Only available when `Env = ()` (unit type)
   - Same trait bounds as `run()`: `T: Send + 'static`, `E: Send + 'static`
   - Returns same `Result<T, E>` as `run()`

3. **Update all examples using `run(&())`**
   - `examples/retry_patterns.rs`
   - `examples/effects.rs`
   - `examples/traverse.rs`
   - `examples/testing_patterns.rs`
   - Any other examples with `run(&())`

4. **Update all documentation using `run(&())`**
   - `docs/guide/03-effects.md`
   - `docs/guide/11-parallel-effects.md`
   - `docs/guide/12-traverse-patterns.md`
   - `docs/guide/15-retry.md`
   - `src/retry/mod.rs` module docs
   - `src/effect.rs` doc comments
   - Any other documentation with `run(&())`

5. **Update internal code using `run(&())`**
   - `src/testing.rs`
   - `src/traverse.rs`
   - `src/context.rs`
   - `src/retry/tests.rs`
   - `src/retry/error.rs`
   - `tests/helper_combinators_integration.rs`
   - Any other test or implementation code

### Non-Functional Requirements

- Zero runtime overhead (method is just a wrapper)
- No breaking changes to existing API
- Backwards compatible - `run(&())` continues to work
- Clear documentation explaining when to use each method

## Acceptance Criteria

- [ ] `Effect::run_standalone()` method exists and compiles
- [ ] Method is only available for `Effect<T, E, ()>` (unit environment)
- [ ] Method requires `async` feature flag
- [ ] All 163 occurrences of `run(&())` are updated to `run_standalone()` where applicable
- [ ] `examples/retry_patterns.rs` uses `run_standalone()` (8 occurrences)
- [ ] `examples/effects.rs` uses `run_standalone()` (2 occurrences)
- [ ] `examples/traverse.rs` uses `run_standalone()` (6 occurrences)
- [ ] `examples/testing_patterns.rs` uses `run_standalone()` (2 occurrences)
- [ ] `src/effect.rs` doc tests updated (86 occurrences to review)
- [ ] `src/testing.rs` updated (7 occurrences)
- [ ] `src/traverse.rs` updated (9 occurrences)
- [ ] `src/retry/tests.rs` updated (11 occurrences)
- [ ] Documentation guides updated to show `run_standalone()` as preferred for unit env
- [ ] All tests pass
- [ ] All doc tests pass
- [ ] Clippy passes with no warnings

## Technical Details

### Implementation Approach

Add a specialized impl block for `Effect<T, E, ()>`:

```rust
#[cfg(feature = "async")]
impl<T, E> Effect<T, E, ()>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Run an effect that doesn't require an environment.
    ///
    /// This is a convenience method for effects with `Env = ()`. Instead of
    /// writing `effect.run(&()).await`, you can write `effect.run_standalone().await`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect: Effect<i32, String, ()> = Effect::pure(42);
    /// let result = effect.run_standalone().await;
    /// assert_eq!(result, Ok(42));
    /// # });
    /// ```
    pub async fn run_standalone(self) -> Result<T, E> {
        self.run(&()).await
    }
}
```

### Architecture Changes

None - this is a purely additive API.

### Files to Modify

**Implementation:**
- `src/effect.rs` - Add `run_standalone()` method

**Examples (update `run(&())` → `run_standalone()`):**
- `examples/retry_patterns.rs` (8 occurrences)
- `examples/effects.rs` (2 occurrences)
- `examples/traverse.rs` (6 occurrences)
- `examples/testing_patterns.rs` (2 occurrences)

**Documentation:**
- `docs/guide/03-effects.md` (2 occurrences)
- `docs/guide/11-parallel-effects.md` (5 occurrences)
- `docs/guide/12-traverse-patterns.md` (3 occurrences)
- `docs/guide/15-retry.md` (update to use `run_standalone()`)
- `src/retry/mod.rs` (1 occurrence in module docs)
- `src/effect.rs` (86 occurrences in doc tests/comments)

**Internal code:**
- `src/testing.rs` (7 occurrences)
- `src/traverse.rs` (9 occurrences)
- `src/context.rs` (1 occurrence)
- `src/retry/tests.rs` (11 occurrences)
- `src/retry/error.rs` (2 occurrences)
- `tests/helper_combinators_integration.rs` (2 occurrences)

**Specs (leave as-is or update):**
- `specs/002-resource-scopes.md` (14 occurrences) - may leave as design doc

### API Documentation

Add to `src/effect.rs`:

```rust
/// Run an effect that doesn't require an environment.
///
/// This is a convenience method for effects with `Env = ()` (unit type).
/// When your effect doesn't need any external dependencies, use this method
/// instead of `run(&())` for cleaner code.
///
/// # When to Use
///
/// Use `run_standalone()` when:
/// - Your effect type is `Effect<T, E, ()>`
/// - The effect doesn't access any environment via `ask()` or `asks()`
/// - You're writing examples or tests with simple effects
///
/// Use `run(env)` when:
/// - Your effect requires an actual environment (database, config, etc.)
/// - The effect type is `Effect<T, E, MyEnv>` where `MyEnv` is not `()`
///
/// # Example
///
/// ```rust
/// use stillwater::Effect;
///
/// # tokio_test::block_on(async {
/// // Instead of this:
/// let result = Effect::<_, String, ()>::pure(42).run(&()).await;
///
/// // Write this:
/// let result = Effect::<_, String, ()>::pure(42).run_standalone().await;
/// assert_eq!(result, Ok(42));
/// # });
/// ```
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**: `Effect` type
- **External Dependencies**: None (uses existing async runtime)

## Testing Strategy

- **Unit Tests**: Add test for `run_standalone()` in `src/effect.rs` tests
- **Doc Tests**: Ensure all doc examples compile and pass
- **Integration Tests**: Existing tests updated to use new API
- **Regression**: Verify `run(&())` still works (backwards compatibility)

## Documentation Requirements

- **Code Documentation**: Comprehensive rustdoc on `run_standalone()` method
- **User Documentation**: Update guides to prefer `run_standalone()` for unit env
- **Architecture Updates**: None needed

## Implementation Notes

1. **Feature gating**: Method must be behind `#[cfg(feature = "async")]` since it's async
2. **Search pattern**: Use `run(&())` as search pattern to find all occurrences
3. **Preserve semantics**: Some doc tests intentionally show `run(&())` to demonstrate the pattern - consider if these should be updated or kept as examples of the alternative syntax
4. **Spec files**: The `specs/002-resource-scopes.md` file contains examples that may be intentionally showing the raw API - evaluate whether to update

## Migration and Compatibility

- **Breaking changes**: None
- **Backwards compatibility**: Full - `run(&())` continues to work
- **Migration path**: Optional - users can adopt `run_standalone()` at their own pace
- **Deprecation**: Not deprecating `run(&())` as it's still needed for non-unit environments
