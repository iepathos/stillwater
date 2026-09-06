# Migration Guide

## Stillwater 1.x to 2.0

The core `Effect` trait remains asynchronous. This release removes deprecated
compatibility APIs and gives effect traversal explicit execution names.

### Choose traversal semantics deliberately

| 1.x API | Preserve concurrent execution | Opt into ordered, fail-fast execution |
|---------|-------------------------------|---------------------------------------|
| `traverse_effect(items, f)` | `traverse_effect_parallel(items, f)` | `traverse_effect_sequential(items, f)` |
| `sequence_effect(effects)` | `sequence_effect_parallel(effects)` | `sequence_effect_sequential(effects)` |

The 1.x implementation ran children concurrently and awaited all results, despite
documentation claiming sequential fail-fast behavior. The parallel variants preserve
that execution policy. Choosing sequential is an intentional behavior change: later
operations will not run after an earlier failure.

Traversal factory timing also changes. In 1.x, `traverse_effect` called `f` immediately
when building the traversal. Both new variants defer `f` until execution; parallel
constructs the full batch before polling children, while sequential constructs each
child only after the previous one succeeds. Move required construction-time work into
an explicit step rather than relying on the old factory timing.

All variants still eagerly enumerate their input iterators when called. They clone the
environment for execution and return boxed effects. Concurrent completion order does
not change output ordering or which error is returned: the first error by input position
wins after all children finish. Dropping the parent future cancels unfinished children.

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let batch = traverse_effect_parallel([1, 2, 3], |value| {
        pure::<_, &str, ()>(value * 2).boxed()
    });
    assert_eq!(batch.run(&()).await, Ok(vec![2, 4, 6]));

    let ordered = sequence_effect_sequential(vec![
        pure::<_, &str, ()>(1).boxed(),
        fail("stop").boxed(),
        pure(3).boxed(),
    ]);
    assert_eq!(ordered.run(&()).await, Err("stop"));
});
```

### Replace legacy aliases and constructors

`LegacyEffect` and `LegacyConstructors` have been removed, not just deprecated.
Use free constructors, return `impl Effect` for concrete composition, and use
`BoxedEffect` when a single erased type is needed. `RunStandalone` remains supported.

```rust
use stillwater::prelude::*;

fn concrete() -> impl Effect<Output = i32, Error = &'static str, Env = ()> {
    pure(42)
}

fn erased() -> BoxedEffect<i32, &'static str, ()> {
    concrete().boxed()
}

tokio_test::block_on(async {
    assert_eq!(concrete().run_standalone().await, Ok(42));
    assert_eq!(erased().run(&()).await, Ok(42));
});
```

### Replace bracket_simple

The removed function accepted `(acquire, use_fn, release_fn)`. The replacement
`bracket` accepts `(acquire, release_fn, use_fn)`.

The use callback now receives `&R`; release receives ownership of `R` and returns a
future with output `Result<(), E>`. It is an async callback, not an Effect.
The error type also needs `Debug`. If use needs owned data, explicitly clone the
necessary data before constructing its effect; the resource itself no longer needs
`Clone` just to enter the use callback.

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use stillwater::prelude::*;

tokio_test::block_on(async {
    let released = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&released);
    let effect = bracket(
        pure::<_, &str, ()>(String::from("connection")),
        move |_resource| async move {
            observed.store(true, Ordering::SeqCst);
            Ok(())
        },
        |resource: &String| pure(resource.len()),
    );
    assert_eq!(effect.run(&()).await, Ok(10));
    assert!(released.load(Ordering::SeqCst));
});
```

`bracket` logs cleanup errors and returns the use result. Choose `bracket_full` when
cleanup failure must be returned to the caller:

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let effect = bracket_full(
        pure::<_, &str, ()>(()),
        |_| async { Err("cleanup failed") },
        |_| pure(42),
    );
    assert_eq!(
        effect.run(&()).await,
        Err(BracketError::CleanupError("cleanup failed")),
    );
});
```

These async brackets do not mask cancellation or guarantee cleanup after panic.
Use normal ownership/RAII where possible and design asynchronous cleanup protocols explicitly.

### Features and allocation expectations

The `async` feature enables Tokio-backed retry and timeout helpers; it does not
switch `Effect` between sync and async modes. `IO` keeps its existing infallible outer
error channel and is not a replacement for fallible Effect constructors.

Most combinators store concrete fields without adding boxes. `from_async_ref` accepts
a boxed borrowed future; `IO`, traversal, and retry helpers return boxed effects.
Calls to `.boxed()` are therefore not the only possible allocation sites.

## Upgrading directly from pre-0.11 releases

Those releases used an `Effect<T, E, Env>` struct and associated constructors.
Use the current replacements above rather than migrating through removed aliases.

| Historical spelling | Current spelling |
|---------------------|------------------|
| `Effect::pure(value)` | `pure(value)` |
| `Effect::fail(error)` | `fail(error)` |
| `Effect::from_fn(f)` | `from_fn(f)` |
| `Effect<T, E, Env>` | `impl Effect<Output = T, Error = E, Env = Env>` or `BoxedEffect<T, E, Env>` |

Use [owned or borrowed async constructors](guide/03-effects.md) according to whether
the returned future borrows its environment. See the [API tiers](guide/17-api-tiers.md)
for the recommended 2.0 surface.
