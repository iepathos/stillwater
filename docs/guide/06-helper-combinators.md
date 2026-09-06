# Helper Combinators

Use composition to describe a small operation clearly. Prefer a named domain struct or
ordinary function when a long chain makes the data flow difficult to follow.

## Accumulate independent validations

```rust
use stillwater::Validation;

let values = Validation::<i32, Vec<&str>>::all((
    Validation::<_, Vec<&str>>::success(1),
    Validation::success(2),
));
assert_eq!(values.into_result(), Ok((1, 2)));

let failures = Validation::<i32, Vec<&str>>::all_vec(vec![
    Validation::failure(vec!["name"]),
    Validation::failure(vec!["email"]),
]);
assert_eq!(failures.into_result(), Err(vec!["name", "email"]));
```

## Map, chain, and translate errors

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let effect = pure::<_, &str, ()>(21)
        .map(|value| value * 2)
        .and_then(|value| from_result(Ok(value + 1)))
        .map_err(str::to_string);
    assert_eq!(effect.run(&()).await, Ok(43));
});
```

`map` transforms a value; `and_then` constructs a dependent effect; `map_err` translates
an error at a boundary. None of these automatically accumulates failures.

## Write a small reusable wrapper

Accept an effect through its trait instead of using the removed generic Effect struct.

```rust
use stillwater::prelude::*;

fn require_positive<E>(effect: E) -> impl Effect<Output = i32, Error = &'static str, Env = E::Env>
where
    E: Effect<Output = i32, Error = &'static str>,
{
    effect.ensure(|value| *value > 0, "must be positive")
}

tokio_test::block_on(async {
    assert_eq!(require_positive(pure::<_, &str, ()>(1)).run(&()).await, Ok(1));
    assert_eq!(require_positive(pure::<_, &str, ()>(0)).run(&()).await, Err("must be positive"));
});
```

Retry requires a factory because running an effect consumes it. Use the existing
[retry and timeout helpers](12-retry.md) rather than a wrapper that pretends to rerun
the same effect. See [effects](03-effects.md) for the core architecture.
