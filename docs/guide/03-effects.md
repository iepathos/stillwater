# Effects: Pure Core, Imperative Shell

Use ordinary Rust functions for domain decisions and `Effect` for composing fallible
operations at application boundaries. An effect is consumed when it runs; retrying means
constructing a fresh effect.

## Start with a pure decision

Load the external facts a decision needs, pass input and facts to a pure function, then
interpret the resulting plain plan. The decision needs no runtime or service mocks.

```rust
use stillwater::{NonEmptyVec, Validation};

#[derive(Debug, PartialEq)]
struct Plan { username: String }

#[derive(Debug, PartialEq)]
enum Rejection { EmptyUsername, UsernameTaken }

fn decide(username: String, taken: bool) -> Validation<Plan, NonEmptyVec<Rejection>> {
    let name = if username.is_empty() {
        Validation::fail(Rejection::EmptyUsername)
    } else {
        Validation::success(())
    };
    let available = if taken {
        Validation::fail(Rejection::UsernameTaken)
    } else {
        Validation::success(())
    };
    Validation::<(), NonEmptyVec<Rejection>>::all((name, available))
        .map(|_| Plan { username })
}

assert_eq!(decide("calm".into(), false).into_result(), Ok(Plan { username: "calm".into() }));
assert_eq!(
    decide("".into(), true).into_result().unwrap_err().into_vec(),
    vec![Rejection::EmptyUsername, Rejection::UsernameTaken],
);
```

The complete [registration example](https://github.com/iepathos/stillwater/blob/master/examples/user_registration.rs)
connects this architecture to narrow repository, password-hasher, and email-sender traits.
Its shell loads facts, calls the decision, then hashes, saves, and sends email in order.

Facts are snapshots. The repository must enforce uniqueness atomically when saving,
using database constraints in a real adapter. A plan can therefore be rejected at commit
time even if the earlier decision accepted it. Domain rejection, commit conflicts, and
infrastructure failures have distinct outcomes.

Email failure leaves the user saved. Retrying the entire registration is not an email
retry protocol; use an application-owned outbox or an explicit compensation protocol
when delivery or rollback is a requirement. The example's hasher is a test double.

## Constructors and execution

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let chain = pure::<_, String, ()>(20)
        .map(|value| value + 1)
        .and_then(|value| pure(value * 2));
    assert_eq!(chain.run(&()).await, Ok(42));

    let failure = fail::<i32, _, ()>("unavailable");
    assert_eq!(failure.run(&()).await, Err("unavailable"));

    let existing = from_result::<_, &str, ()>(Ok(7));
    assert_eq!(existing.run(&()).await, Ok(7));
});
```

Use `from_fn` for synchronous fallible operations. Keep slow blocking work off an async
executor's worker thread using the runtime's blocking-work facilities.

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Env { limit: usize }

tokio_test::block_on(async {
    let effect = from_fn(|env: &Env| Ok::<_, &str>(env.limit))
        .ensure(|limit| *limit > 0, "limit must be positive");
    assert_eq!(effect.run(&Env { limit: 10 }).await, Ok(10));
});
```

`ensure` short-circuits on the first failed predicate. Use `Validation` in the pure
decision when independent checks must accumulate errors.

## Owned and borrowed async work

`from_async` accepts a future whose type does not borrow the supplied environment.
Clone a cheap shared handle before building that future.

```rust
use std::sync::Arc;
use stillwater::prelude::*;

#[derive(Clone)]
struct Env { name: Arc<String> }

tokio_test::block_on(async {
    let effect = from_async(|env: &Env| {
        let name = Arc::clone(&env.name);
        async move { Ok::<_, &str>(name.len()) }
    });
    assert_eq!(effect.run(&Env { name: Arc::new("water".into()) }).await, Ok(5));
});
```

Use `from_async_ref` when the operation must borrow the environment across an await.
It accepts a boxed future, adding one future allocation in the usual `Box::pin` usage.

```rust
use stillwater::prelude::*;

#[derive(Clone)]
struct Env { name: String }

tokio_test::block_on(async {
    let effect = from_async_ref(|env: &Env| {
        Box::pin(async move {
            tokio::task::yield_now().await;
            Ok::<_, &str>(env.name.len())
        })
    });
    assert_eq!(effect.run(&Env { name: "water".into() }).await, Ok(5));
});
```

## Concrete types and explicit type erasure

Most combinators store their inputs and closures directly. Return `impl Effect` when
callers do not need to name that concrete type. Use `BoxedEffect` at boundaries that
require a single erased type, such as heterogeneous collections or different branches.

```rust
use stillwater::prelude::*;

fn choose(cached: bool) -> BoxedEffect<i32, &'static str, ()> {
    if cached {
        pure(21).map(|value| value * 2).boxed()
    } else {
        fail("cache miss").boxed()
    }
}

tokio_test::block_on(async {
    assert_eq!(choose(true).run(&()).await, Ok(42));
    assert_eq!(choose(false).run(&()).await, Err("cache miss"));
});
```

Concrete types do not promise allocation-free execution or a fixed performance ratio.
`from_async_ref` boxes its future; helpers returning `BoxedEffect` (including `IO`,
effect traversal, and retry) erase types internally. User operations, captured data,
and environment clones may allocate too. Benchmark the actual workflow.

## Choose execution semantics explicitly

Use `and_then` when an operation needs an earlier output. Use sequential traversal for
ordered batches that stop on failure. Parallel traversal concurrently polls the batch,
waits for every result, and selects an error by input position.

These operations do not provide transactions or cancellation masking. Dropping the
parent future cancels unfinished work. Runtime bracket cleanup is an explicit protocol
for normal success/error paths, not a guarantee that async cleanup runs after cancellation
or panic; ordinary Rust ownership and `Drop` still matter.

See [traversal](11-traverse-patterns.md), [parallel effects](10-parallel-effects.md),
[reader helpers](09-reader-pattern.md), [testing](15-testing.md), and
[API tiers](17-api-tiers.md) for focused examples.
