# Error Context at Boundaries

Keep domain errors structured for decisions and matching. Add contextual information
when an infrastructure operation crosses an application boundary.

## Wrap an error

```rust
use stillwater::ContextError;

let error = ContextError::new("connection refused")
    .context("loading registration facts");
assert_eq!(error.inner(), &"connection refused");
assert_eq!(error.context_trail(), &["loading registration facts"]);
```

## Add context to an effect

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let effect = fail::<(), _, ()>("connection refused")
        .context("loading registration facts")
        .context_chain("registering user");
    let error = effect.run(&()).await.unwrap_err();
    assert_eq!(error.inner(), &"connection refused");
    assert_eq!(
        error.context_trail(),
        &["loading registration facts", "registering user"],
    );
});
```

The first `context` wraps the error in `ContextError`. Use `context_chain` to append
information to that same wrapper; calling `context` again would nest wrappers.

## Preserve successful values

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let effect = pure::<_, &str, ()>(42).context("calculating total");
    assert_eq!(effect.run(&()).await, Ok(42));
});
```

Context strings can allocate. Put useful operation names and identifiers at boundaries,
avoid secrets such as passwords or tokens, and retain the underlying error for callers.
A context trail does not imply that preceding writes were rolled back.

See [effects](03-effects.md) for separating domain rejection from infrastructure failures,
and [testing](15-testing.md) for testing error propagation.
