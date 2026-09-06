# Helper Combinators

Stillwater provides helper functions for common patterns with Validation and Effect.

## Validation Combinators

### all() - Combine multiple validations

Already covered in [Validation guide](02-validation.md):

```text
use stillwater::Validation;

Validation::all((
    validate_email(email),
    validate_age(age),
    validate_password(password),
))
```

### all_vec() - Combine vector of validations

For homogeneous collections:

```text
use stillwater::Validation;

let validations: Vec<Validation<Item, Vec<Error>>> = items
    .into_iter()
    .map(|item| validate_item(item))
    .collect();

let result: Validation<Vec<Item>, Vec<Error>> = Validation::all_vec(validations);
```

## Effect Combinators

### map() - Transform success value

```text
effect.map(|user| user.email)
```

### and_then() - Chain dependent effects

```text
effect.and_then(|user| load_profile(user))
```

### map_err() - Transform error value

```text
effect.map_err(|e| format!("Failed: {}", e))
```

## Building Custom Combinators

You can build your own combinators for common patterns:

```text
use stillwater::{Effect, Validation};

// Retry combinator
fn retry<T, E, Env>(
    effect: Effect<T, E, Env>,
    times: usize
) -> Effect<T, E, Env>
where
    T: Clone,
    E: Clone,
{
    // Implementation left as exercise
    effect
}

// Timeout combinator
fn timeout<T, E, Env>(
    effect: Effect<T, E, Env>,
    duration: Duration
) -> Effect<T, TimeoutError<E>, Env> {
    // Implementation left as exercise
    todo!()
}
```

## Next Steps

- Learn about [Try Trait](07-try-trait.md) (nightly feature)
- See [Patterns](../PATTERNS.md) for more recipes
