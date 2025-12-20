---
number: 36
title: Compile-Time Resource Tracking
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-12-19
---

# Specification 036: Compile-Time Resource Tracking

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (builds on existing Effect trait and bracket module)

## Context

Stillwater provides runtime resource safety through the `bracket` pattern, which guarantees that cleanup code runs even when the "use" phase fails. This is powerful but has limitations:

1. **No compile-time leak detection**: Nothing prevents returning an acquired resource without releasing it
2. **No protocol enforcement**: Can't enforce patterns like "begin transaction → operations → commit/rollback"
3. **No documentation in types**: Function signatures don't communicate resource requirements
4. **Runtime-only safety**: Resource leaks are only caught (if at all) at runtime

Other languages have demonstrated that resource tracking can be lifted to the type level:
- Rust's ownership system tracks memory resources
- Linear types (Clean, ATS) track general resources
- Effect systems (Koka, Eff) track computational effects

This specification adds **compile-time resource tracking** to stillwater's effect system, enabling:
- Type-level documentation of resource acquisition/release
- Compile-time detection of resource leaks
- Protocol enforcement through types
- Zero runtime overhead (purely type-level)

## Objective

Extend the Effect system with optional resource tracking that:

1. Tracks resource acquisition and release in effect types
2. Provides compile-time warnings/errors for resource leaks
3. Enables protocol enforcement (e.g., transaction begin/end)
4. Maintains zero runtime overhead
5. Integrates seamlessly with existing Effect combinators
6. Remains optional (existing code continues to work)

## Requirements

### Functional Requirements

#### 1. Resource Kind Markers

- Define a `ResourceKind` trait for marking resource types
- Provide common resource markers: `FileRes`, `DbRes`, `LockRes`, `TxRes`, `SocketRes`
- Enable users to define custom resource kinds
- Resource markers must be zero-sized (no runtime cost)

#### 2. Type-Level Resource Sets

- Implement type-level sets for tracking multiple resources
- Provide `Empty` (no resources) and `Has<R, Rest>` (resource R plus rest)
- Implement type-level operations: union, contains, subset checking
- All operations must be compile-time only

#### 3. ResourceEffect Trait

- Extend Effect with resource tracking via `ResourceEffect` trait
- Track `Acquires` (resources created) and `Releases` (resources consumed)
- Default both to `Empty` for backward compatibility
- Resource tracking must compose through combinators

#### 4. Tracked Wrapper Type

- Provide `Tracked<Eff, Acquires, Releases>` wrapper for adding resource annotations
- Wrapper must have zero runtime overhead (delegates to inner effect)
- Implement Effect trait for Tracked with proper delegation

#### 5. Extension Methods for Marking Resources

- Provide `.acquires::<R>()` method to mark resource acquisition
- Provide `.releases::<R>()` method to mark resource release
- Provide `.scoped::<R>()` method for effects that acquire and release same resource
- Methods should be chainable

#### 6. Resource-Aware Bracket

- Provide `resource_bracket` that enforces resource neutrality
- Type signature must guarantee: acquire creates R, release consumes R
- The bracket as a whole must have `Acquires = Empty, Releases = Empty`
- Integrate with existing bracket implementations

#### 7. Combinator Resource Propagation

- `AndThen<A, B>` accumulates resources: `Acquires = Union<A::Acquires, B::Acquires>`
- `Map<Eff, F>` preserves resources from inner effect
- `Pure`, `Fail` are resource-neutral (`Empty`, `Empty`)
- All existing combinators must have ResourceEffect implementations

#### 8. Resource Neutrality Assertion

- Provide `assert_resource_neutral()` helper function
- Function accepts only effects with `Acquires = Empty, Releases = Empty`
- Use for function return types to enforce no leaked resources

### Non-Functional Requirements

- **Zero runtime overhead**: All tracking is compile-time only
- **Backward compatibility**: Existing code must continue to work unchanged
- **Ergonomic API**: Resource tracking should feel natural, not burdensome
- **Clear error messages**: Compiler errors should guide users to fixes
- **Composable**: Resource tracking must work with all existing Effect patterns
- **Extensible**: Users can define custom resource kinds

## Acceptance Criteria

- [ ] `ResourceKind` trait defined with `NAME` constant for error messages
- [ ] Common resource markers implemented: `FileRes`, `DbRes`, `LockRes`, `TxRes`, `SocketRes`
- [ ] Type-level `Empty` and `Has<R, Rest>` resource sets implemented
- [ ] `ResourceSet` trait marks valid resource set types
- [ ] `ResourceEffect` trait extends Effect with `Acquires` and `Releases` associated types
- [ ] `Tracked<Eff, Acq, Rel>` wrapper implements both Effect and ResourceEffect
- [ ] `.acquires::<R>()` extension method works on any Effect
- [ ] `.releases::<R>()` extension method works on any Effect
- [ ] `resource_bracket` enforces acquire/release matching at compile time
- [ ] `AndThen` combinator properly unions acquired/released resources
- [ ] `Map`, `MapErr` preserve inner effect's resource tracking
- [ ] `Pure`, `Fail` are resource-neutral
- [ ] `assert_resource_neutral()` rejects effects with non-empty resource sets
- [ ] All new types implement `Debug`
- [ ] Comprehensive unit tests for all resource tracking functionality
- [ ] Documentation with examples for each feature
- [ ] Integration tests showing protocol enforcement patterns

## Technical Details

### Implementation Approach

The implementation adds a parallel type-level tracking system that shadows the runtime Effect behavior. The key insight is that resource tracking can be entirely separate from the actual effect execution.

### Module Structure

```
src/effect/resource/
├── mod.rs           # Module exports and documentation
├── markers.rs       # ResourceKind trait and common markers
├── sets.rs          # Type-level resource sets (Empty, Has)
├── tracked.rs       # Tracked wrapper and ResourceEffect trait
├── ext.rs           # Extension trait with .acquires()/.releases()
├── bracket.rs       # Resource-aware bracket
└── combinators.rs   # ResourceEffect impls for existing combinators
```

### Core Types

```rust
// markers.rs

/// Marker trait for resource kinds. Zero-sized, compile-time only.
pub trait ResourceKind: Send + Sync + 'static {
    /// Human-readable name for error messages
    const NAME: &'static str;
}

/// File handle resource
pub struct FileRes;
impl ResourceKind for FileRes {
    const NAME: &'static str = "File";
}

/// Database connection resource
pub struct DbRes;
impl ResourceKind for DbRes {
    const NAME: &'static str = "Database";
}

/// Lock/mutex resource
pub struct LockRes;
impl ResourceKind for LockRes {
    const NAME: &'static str = "Lock";
}

/// Transaction resource
pub struct TxRes;
impl ResourceKind for TxRes {
    const NAME: &'static str = "Transaction";
}

/// Network socket resource
pub struct SocketRes;
impl ResourceKind for SocketRes {
    const NAME: &'static str = "Socket";
}
```

```rust
// sets.rs

use std::marker::PhantomData;

/// Marker trait for valid resource sets
pub trait ResourceSet: Send + Sync + 'static {}

/// Empty resource set - no resources
pub struct Empty;
impl ResourceSet for Empty {}

/// Non-empty resource set - has resource R plus Rest
pub struct Has<R: ResourceKind, Rest: ResourceSet = Empty>(PhantomData<(R, Rest)>);
impl<R: ResourceKind, Rest: ResourceSet> ResourceSet for Has<R, Rest> {}

// Type-level operations would go here (Union, Contains, etc.)
// These use trait bounds to compute at compile time
```

```rust
// tracked.rs

use std::marker::PhantomData;
use crate::effect::Effect;

/// An effect with resource tracking
pub trait ResourceEffect: Effect {
    /// Resources this effect acquires (creates)
    type Acquires: ResourceSet;

    /// Resources this effect releases (consumes)
    type Releases: ResourceSet;
}

/// Wrapper that adds resource tracking to any effect
#[derive(Debug)]
pub struct Tracked<Eff, Acq: ResourceSet = Empty, Rel: ResourceSet = Empty> {
    inner: Eff,
    _phantom: PhantomData<(Acq, Rel)>,
}

impl<Eff, Acq: ResourceSet, Rel: ResourceSet> Tracked<Eff, Acq, Rel> {
    pub fn new(inner: Eff) -> Self {
        Self { inner, _phantom: PhantomData }
    }
}

// Effect implementation delegates to inner
impl<Eff: Effect, Acq: ResourceSet, Rel: ResourceSet> Effect for Tracked<Eff, Acq, Rel> {
    type Output = Eff::Output;
    type Error = Eff::Error;
    type Env = Eff::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        self.inner.run(env)
    }
}

impl<Eff: Effect, Acq: ResourceSet, Rel: ResourceSet> ResourceEffect for Tracked<Eff, Acq, Rel> {
    type Acquires = Acq;
    type Releases = Rel;
}
```

```rust
// ext.rs

/// Extension trait for adding resource tracking to effects
pub trait ResourceEffectExt: Effect + Sized {
    /// Mark that this effect acquires resource R
    fn acquires<R: ResourceKind>(self) -> Tracked<Self, Has<R>, Empty> {
        Tracked::new(self)
    }

    /// Mark that this effect releases resource R
    fn releases<R: ResourceKind>(self) -> Tracked<Self, Empty, Has<R>> {
        Tracked::new(self)
    }
}

impl<E: Effect> ResourceEffectExt for E {}

/// Assert that an effect is resource-neutral (compile-time check)
pub fn assert_resource_neutral<Eff>(effect: Eff) -> Eff
where
    Eff: ResourceEffect<Acquires = Empty, Releases = Empty>,
{
    effect
}
```

```rust
// bracket.rs

/// Resource-safe bracket with compile-time tracking
///
/// Type signature enforces:
/// - Acquire creates resource R
/// - Release consumes resource R
/// - Bracket as a whole is resource-neutral
pub fn resource_bracket<R, Acq, Use, Rel, UseEff, T, U, E, Env, RelFut>(
    acquire: Acq,
    release: Rel,
    use_fn: Use,
) -> impl ResourceEffect<Output = U, Error = E, Env = Env, Acquires = Empty, Releases = Empty>
where
    R: ResourceKind,
    Acq: ResourceEffect<Output = T, Error = E, Env = Env, Acquires = Has<R>, Releases = Empty>,
    Use: FnOnce(&T) -> UseEff + Send,
    UseEff: ResourceEffect<Output = U, Error = E, Env = Env, Acquires = Empty, Releases = Empty>,
    Rel: FnOnce(T) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    T: Send,
    U: Send,
    E: Send + Debug,
    Env: Clone + Send + Sync,
{
    // Implementation wraps existing bracket with resource tracking
    Tracked::new(bracket(acquire, release, use_fn))
}
```

### Combinator Implementations

```rust
// combinators.rs

// AndThen combines resources from both effects
impl<A, B, F> ResourceEffect for AndThen<A, B, F>
where
    A: ResourceEffect,
    B: ResourceEffect,
    // ... other bounds
{
    type Acquires = Union<A::Acquires, B::Acquires>;
    type Releases = Union<A::Releases, B::Releases>;
}

// Map preserves resources
impl<Eff, F, B> ResourceEffect for Map<Eff, F>
where
    Eff: ResourceEffect,
    // ... other bounds
{
    type Acquires = Eff::Acquires;
    type Releases = Eff::Releases;
}

// Pure is resource-neutral
impl<T, E, Env> ResourceEffect for Pure<T, E, Env> {
    type Acquires = Empty;
    type Releases = Empty;
}

// Fail is resource-neutral
impl<T, E, Env> ResourceEffect for Fail<T, E, Env> {
    type Acquires = Empty;
    type Releases = Empty;
}
```

### Usage Examples

```rust
use stillwater::effect::resource::*;

// Define operations with resource annotations
fn open_file(path: &str) -> impl ResourceEffect<
    Output = FileHandle,
    Acquires = Has<FileRes>,
    Releases = Empty,
> {
    pure(FileHandle::new(path)).acquires::<FileRes>()
}

fn close_file(handle: FileHandle) -> impl ResourceEffect<
    Output = (),
    Acquires = Empty,
    Releases = Has<FileRes>,
> {
    pure(()).releases::<FileRes>()  // Simplified; real impl would close
}

// This compiles: resource is properly managed
fn read_file_safe(path: &str) -> impl ResourceEffect<
    Acquires = Empty,
    Releases = Empty,
> {
    resource_bracket::<FileRes, _, _, _, _, _, _, _, _>(
        open_file(path),
        |h| async move { close_file(h).run(&()).await },
        |h| read_contents(h),
    )
}

// This would fail to compile: resource leak
fn read_file_leaky(path: &str) -> impl ResourceEffect<
    Acquires = Empty,  // Claims no acquisitions...
    Releases = Empty,
> {
    open_file(path)  // ...but actually acquires FileRes!
    // Compile error: expected Acquires = Empty, found Acquires = Has<FileRes>
}
```

### Protocol Enforcement Example

```rust
// Transaction protocol enforcement
fn begin_tx() -> impl ResourceEffect<Acquires = Has<TxRes>> { ... }
fn commit(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
fn rollback(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
fn query(tx: &Tx) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> { ... }

// Correct: transaction is opened and closed
fn transfer_funds() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
    resource_bracket::<TxRes, _, _, _, _, _, _, _, _>(
        begin_tx(),
        |tx| async move { commit(tx).run(&()).await },
        |tx| {
            query(tx, "UPDATE accounts SET balance = balance - 100 WHERE id = 1")
                .and_then(|_| query(tx, "UPDATE accounts SET balance = balance + 100 WHERE id = 2"))
        },
    )
}

// Compile error: transaction never closed
fn bad_transfer() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
    begin_tx()
        .and_then(|tx| query(&tx, "UPDATE ..."))
    // Error: TxRes acquired but not released
}
```

## Dependencies

- **Prerequisites**: None - builds on existing Effect trait and bracket module
- **Affected Components**:
  - `src/effect/mod.rs` - add resource module export
  - `src/effect/ext.rs` - may add resource extension methods
  - `src/effect/combinators/` - add ResourceEffect implementations
  - `src/effect/bracket.rs` - add resource_bracket function
- **External Dependencies**: None (uses only std library)

## Testing Strategy

### Unit Tests

- Test all ResourceKind markers have correct NAME
- Test Empty and Has type construction
- Test Tracked wrapper delegates correctly to inner Effect
- Test `.acquires()` and `.releases()` extension methods
- Test `assert_resource_neutral` accepts/rejects correctly
- Test `resource_bracket` type constraints

### Integration Tests

- Test resource tracking through complex effect chains
- Test AndThen resource accumulation
- Test Map resource preservation
- Test real-world patterns (file handling, database transactions)
- Test protocol enforcement patterns
- Test nested resource brackets

### Compile-Fail Tests

Use `trybuild` or `compile_fail` doc tests:

- Verify leak detection catches unclosed resources
- Verify protocol violations are caught
- Verify type mismatches produce helpful errors

### Property Tests

- Property: `resource_bracket` always produces resource-neutral effect
- Property: `AndThen` correctly unions resources
- Property: `Map` preserves resources exactly

## Documentation Requirements

### Code Documentation

- Comprehensive doc comments on all public types and functions
- Examples in doc comments showing typical usage
- Doc comments explaining the type-level nature of tracking

### User Documentation

- Add "Resource Tracking" section to README.md
- Provide tutorial showing progression from basic to advanced usage
- Document common patterns: file handling, transactions, connection pools
- Document how to define custom resource kinds

### Architecture Updates

- Update DESIGN.md with resource tracking design rationale
- Add diagrams showing type-level resource flow
- Document relationship to bracket pattern

## Implementation Notes

### Zero-Cost Abstraction

The entire resource tracking system must be zero-cost:
- All marker types are zero-sized
- `Tracked` wrapper has same runtime behavior as inner effect
- Type-level computations (Union, etc.) happen at compile time only
- No runtime checks, allocations, or indirection

### Ergonomic Considerations

- Extension methods (`.acquires()`) are more ergonomic than manual `Tracked::new()`
- Resource kinds can be inferred in many contexts
- Default associated types (`= Empty`) reduce boilerplate
- `resource_bracket` should have similar API to existing `bracket`

### Error Message Quality

Compile errors involving resource mismatches can be confusing. Consider:
- Using `#[rustc_on_unimplemented]` attributes where possible
- Providing type aliases for common patterns
- Clear documentation of common error scenarios

### Future Extensions

This design allows for future extensions:
- Resource counts (e.g., "at most 5 connections") - would need runtime support
- Resource borrowing vs consuming distinction
- Integration with async/await resource cleanup
- Automatic resource inference via proc macros

## Migration and Compatibility

### Backward Compatibility

- All existing Effect code continues to work unchanged
- ResourceEffect is an optional extension, not a replacement
- Default implementations make adoption gradual

### Migration Path

1. **No change required**: Existing code works as-is
2. **Opt-in tracking**: Add `.acquires()` / `.releases()` to resource operations
3. **Full enforcement**: Use `resource_bracket` and `assert_resource_neutral`

### Breaking Changes

None. This is purely additive.
