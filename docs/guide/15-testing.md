# Testing with Stillwater

Test domain decisions as ordinary functions, then test the shell's observable ordering,
error propagation, and persisted state using explicit service implementations.

The snippets below execute their assertions as Cargo doctests. In application test files,
place equivalent bodies in ordinary Rust or Tokio tests. Do not hide doctest examples
inside test-only modules: a passing doctest can otherwise contain unchecked code.

## Pure decisions and accumulated errors

```rust
use stillwater::{Validation, assert_success, assert_failure, assert_validation_errors};

fn adult(age: u8) -> Validation<u8, Vec<&'static str>> {
    if age >= 18 { Validation::success(age) } else { Validation::failure(vec!["underage"]) }
}

assert_success!(adult(18));
assert_failure!(adult(17));
assert_validation_errors!(adult(17), vec!["underage"]);

let checks = Validation::<u8, Vec<&str>>::all((adult(16), adult(17)));
assert_eq!(checks.into_result(), Err(vec!["underage", "underage"]));
```

Prefer asserting exact outputs and errors over checking only success/failure. Include
boundary values and verify that independent failures accumulate.

## Explicit test environments

Small environments often need no builder. `MockEnv` is available when tuple composition
helps construct a fixture; each `with` adds one nested pair.

```rust
use stillwater::testing::MockEnv;

let (((), minimum_age), users) = MockEnv::new()
    .with(|| 18_u8)
    .with(|| Vec::<String>::new())
    .build();

assert_eq!(minimum_age, 18);
assert!(users.is_empty());
```

## Shell ordering and failure boundaries

```rust
use std::sync::{Arc, Mutex};
use stillwater::prelude::*;

#[derive(Clone, Default)]
struct Env { events: Arc<Mutex<Vec<&'static str>>> }

tokio_test::block_on(async {
    let env = Env::default();
    let save = from_fn(|env: &Env| {
        env.events.lock().unwrap().push("save");
        Err::<(), _>("database unavailable")
    });
    let workflow = save.and_then(|()| from_fn(|env: &Env| {
        env.events.lock().unwrap().push("email");
        Ok(())
    }));
    assert_eq!(workflow.run(&env).await, Err("database unavailable"));
    assert_eq!(*env.events.lock().unwrap(), vec!["save"]);
});
```

The [registration example](https://github.com/iepathos/stillwater/blob/master/examples/user_registration.rs)
also tests stale availability facts at commit time, rejected inputs, hashing failure,
save failure, and email failure after persistence. Check persisted state as well as return
values: an error does not imply that earlier operations were rolled back.

## Property tests

Add `proptest` as a development dependency. Stillwater's optional `proptest` feature
also supplies `Arbitrary` implementations for supported library types. A runner can
execute a property directly, which works in a doctest as well as in a normal test.

```rust
use proptest::{prelude::*, test_runner::TestRunner};
use stillwater::Validation;

TestRunner::default().run(&any::<i32>(), |value| {
    let result = Validation::<_, Vec<String>>::success(value).map(|x| x);
    prop_assert_eq!(result, Validation::Success(value));
    Ok(())
}).unwrap();
```

Choose domain properties too: for example, rejected registration decisions never emit
plans, and independent validation errors retain deterministic order. Bound arithmetic
inputs or use checked operations so overflow does not accidentally become the property.

## Run the relevant targets

```bash
cargo test
cargo test --all-features
cargo test --doc --all-features
cargo test --example user_registration --all-features
```

Default Cargo/nextest discovery does not run the tests embedded in this example. CI
selects it explicitly. The documentation contract also rejects test-only attributes
inside executable snippets and checks that every book chapter is in the Cargo manifest.
