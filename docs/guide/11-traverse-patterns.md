# Traverse and Sequence Patterns

Traversal maps a function over inputs and combines its results. Sequencing combines
results or effects that have already been constructed.

## Validation accumulates errors

`traverse` and `sequence` in this module operate on `Validation`. They preserve input
order and combine errors using `Semigroup`.

```rust
use stillwater::{Validation, traverse::{traverse, sequence}};

fn positive(value: i32) -> Validation<i32, Vec<i32>> {
    if value > 0 { Validation::success(value) } else { Validation::failure(vec![value]) }
}

assert_eq!(traverse([1, 2, 3], positive), Validation::Success(vec![1, 2, 3]));
assert_eq!(traverse([1, -2, -3], positive), Validation::Failure(vec![-2, -3]));
assert_eq!(traverse([], positive), Validation::Success(vec![]));

let checks = [positive(-1), positive(2), positive(-3)];
assert_eq!(sequence(checks), Validation::Failure(vec![-1, -3]));
```

## Sequential effect traversal

`traverse_effect_sequential` calls the factory for one item, awaits that child, and
only then proceeds. After an error, later child effects are neither constructed nor run.

```rust
use std::sync::{Arc, Mutex};
use stillwater::prelude::*;

tokio_test::block_on(async {
    let constructed = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&constructed);
    let effect = traverse_effect_sequential([1, 2, 3], move |item| {
        seen.lock().unwrap().push(item);
        if item == 2 { fail("stop").boxed() } else { pure(item).boxed() }
    });
    assert!(constructed.lock().unwrap().is_empty());
    assert_eq!(effect.run(&()).await, Err("stop"));
    assert_eq!(*constructed.lock().unwrap(), vec![1, 2]);
});
```

## Parallel effect traversal

`traverse_effect_parallel` constructs all children when the returned effect executes,
then polls them concurrently. It waits for the entire batch, even after an error.
Successful values retain input order; if multiple children fail, the error at the
earliest input position is returned.

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let effect = traverse_effect_parallel([1, 2, 3], |item| {
        from_async(move |_: &()| async move {
            tokio::task::yield_now().await;
            Ok::<_, &'static str>(item * 2)
        }).boxed()
    });
    assert_eq!(effect.run(&()).await, Ok(vec![2, 4, 6]));
});
```

Concurrent polling does not move CPU work onto separate threads. A child that blocks
its poll can block progress for the whole batch. For large I/O batches, use
`par_all_limit` or chunking to bound concurrency. Neither parallel traversal variant
provides rollback or keeps children running after the parent future is dropped.

## Sequence existing effects

```rust
use stillwater::prelude::*;

tokio_test::block_on(async {
    let ordered = sequence_effect_sequential(vec![
        pure::<_, &str, ()>(1).boxed(),
        pure(2).boxed(),
    ]);
    assert_eq!(ordered.run(&()).await, Ok(vec![1, 2]));

    let concurrent = sequence_effect_parallel(vec![
        fail::<i32, _, ()>("first input").boxed(),
        fail("second input").boxed(),
    ]);
    assert_eq!(concurrent.run(&()).await, Err("first input"));
});
```

Sequential sequencing stops running children after the first error, but their construction
has already happened. Use traversal when child construction must also be deferred.

## Construction, memory, and testing

All four effect helpers eagerly collect their input iterator when called. Side effects
in an iterator's `next` method therefore occur before execution. Traversal defers the
supplied child factory, not input enumeration. Inputs must be finite.

The effect helpers return `BoxedEffect`, require boxed children, and clone the environment
for execution. Prefer cheap shared service handles in the environment. Validation
traversal also materializes an intermediate vector; it makes no single-allocation or
speedup guarantee over a manual map and sequence.

Regression tests in
[`src/traverse.rs`](https://github.com/iepathos/stillwater/blob/master/src/traverse.rs)
cover child construction timing, strict sequential ordering, actual overlap, empty inputs,
input-ordered errors despite out-of-order completion, and completion after errors.
Use synchronization to control completion order in tests instead of assuming timer precision.

For the 1.x behavior and upgrade details, see [migration](../MIGRATION.md).
