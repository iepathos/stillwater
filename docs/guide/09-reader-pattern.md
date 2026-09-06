# Reader Helpers and Explicit Environments

An effect's environment contains the dependencies needed by its imperative boundary.
Keep business decisions in ordinary functions accepting plain data. Reader helpers
provide access to configuration and services without introducing global state.

## Project a value with asks

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Config { retry_limit: usize, region: String }

tokio_test::block_on(async {
    let effect = asks::<_, &str, Config, _>(|config| config.region.clone());
    let config = Config { retry_limit: 3, region: "west".into() };
    assert_eq!(effect.run(&config).await, Ok("west".to_string()));
});
```

Use `from_fn` for fallible synchronous access and `from_async_ref` for service calls
that borrow the environment across an await. `asks` itself produces a successful value.

## Clone the environment with ask

```rust
use stillwater::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct Config { limit: usize }

tokio_test::block_on(async {
    let env = Config { limit: 3 };
    assert_eq!(ask::<&str, Config>().run(&env).await, Ok(env));
});
```

`ask` clones the whole environment. Store shared services behind cheap handles such as
`Arc`; do not assume that a `Clone` bound means cloning is free.

## Scope configuration with local

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Config { limit: usize }

tokio_test::block_on(async {
    let read_limit = asks::<_, &str, Config, _>(|config| config.limit);
    let scoped = local(|config: &Config| Config { limit: config.limit + 1 }, read_limit);
    let env = Config { limit: 3 };
    assert_eq!(scoped.run(&env).await, Ok(4));
    assert_eq!(env.limit, 3);
});
```

The transformed environment is local to the inner effect. This does not clone or isolate
external state behind shared handles: two environments containing the same `Arc` still
refer to the same service.

The [registration example](https://github.com/iepathos/stillwater/blob/master/examples/user_registration.rs)
uses an environment of narrow service traits. See [effects](03-effects.md) for owned and
borrowed async constructors, and [API tiers](17-api-tiers.md) for choosing the smallest API.
