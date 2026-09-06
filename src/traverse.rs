//! Traverse and sequence utilities for working with collections of effects
//!
//! This module provides the fundamental operations for working with collections of `Validation`
//! and `Effect` types. These operations allow you to transform collections of effectful
//! computations in a composable, functional way.
//!
//! # Core Concepts
//!
//! - **`sequence`**: Convert a collection of effects into an effect of a collection
//!   - `Vec<Validation<T, E>>` → `Validation<Vec<T>, E>`
//!   - `Vec<BoxedEffect<T, E, Env>>` → `BoxedEffect<Vec<T>, E, Env>`
//!
//! - **`traverse`**: Map a function over a collection and sequence the results
//!   - Equivalent to `map(f).sequence()` but more efficient
//!
//! # Examples
//!
//! ## Validation
//!
//! ```
//! use stillwater::{Validation, traverse::traverse};
//!
//! fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
//!     s.parse()
//!         .map(Validation::success)
//!         .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid number: {}", s)]))
//! }
//!
//! let numbers = vec!["1", "2", "3"];
//! let result = traverse(numbers, parse_number);
//! assert_eq!(result, Validation::Success(vec![1, 2, 3]));
//!
//! let mixed = vec!["1", "invalid", "3"];
//! let result = traverse(mixed, parse_number);
//! assert!(result.is_failure());
//! ```
//!
//! ## Effect
//!
//! ```
//! use stillwater::{BoxedEffect, traverse::traverse_effect_sequential};
//! use stillwater::effect::prelude::*;
//!
//! # tokio_test::block_on(async {
//! fn process(x: i32) -> BoxedEffect<i32, String, ()> {
//!     pure(x * 2).boxed()
//! }
//!
//! let numbers = vec![1, 2, 3];
//! let effect = traverse_effect_sequential(numbers, process);
//! let result = effect.run(&()).await;
//! assert_eq!(result, Ok(vec![2, 4, 6]));
//! # });
//! ```

use crate::{BoxedEffect, Semigroup, Validation};

/// Traverse a collection with a validation function.
///
/// Applies `f` to each element, accumulating all errors if any fail.
/// If all validations succeed, returns a success with a vector of all results.
///
/// # Type Parameters
///
/// * `T` - Input element type
/// * `U` - Output element type
/// * `E` - Error type (must implement `Semigroup` for error accumulation)
/// * `F` - Function type that transforms `T` into `Validation<U, E>`
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Validation, traverse::traverse};
///
/// fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
///     if x > 0 {
///         Validation::success(x)
///     } else {
///         Validation::failure(vec![format!("{} is not positive", x)])
///     }
/// }
///
/// let result = traverse(vec![1, 2, 3], validate_positive);
/// assert_eq!(result, Validation::Success(vec![1, 2, 3]));
///
/// let result = traverse(vec![1, -2, -3], validate_positive);
/// assert!(result.is_failure());
/// ```
pub fn traverse<T, U, E, F, I>(iter: I, f: F) -> Validation<Vec<U>, E>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Validation<U, E>,
    E: Semigroup,
{
    let validations: Vec<_> = iter.into_iter().map(f).collect();
    Validation::all_vec(validations)
}

/// Sequence a collection of validations.
///
/// Converts a collection of validations into a validation of a collection.
/// If all validations succeed, returns success with all values.
/// If any fail, accumulates all errors using `Semigroup`.
///
/// # Type Parameters
///
/// * `T` - Success value type
/// * `E` - Error type (must implement `Semigroup` for error accumulation)
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Validation, traverse::sequence};
///
/// let vals = vec![
///     Validation::<_, Vec<String>>::success(1),
///     Validation::success(2),
///     Validation::success(3),
/// ];
/// let result = sequence(vals);
/// assert_eq!(result, Validation::Success(vec![1, 2, 3]));
///
/// let vals = vec![
///     Validation::<i32, _>::failure(vec!["error1".to_string()]),
///     Validation::success(2),
///     Validation::failure(vec!["error2".to_string()]),
/// ];
/// let result = sequence(vals);
/// assert!(result.is_failure());
/// ```
pub fn sequence<T, E, I>(iter: I) -> Validation<Vec<T>, E>
where
    I: IntoIterator<Item = Validation<T, E>>,
    E: Semigroup,
{
    Validation::all_vec(iter.into_iter().collect())
}

/// Traverse a collection sequentially with an effect function.
///
/// Constructs and runs one effect at a time, collecting results in input order.
/// On failure, later effects are neither constructed nor run.
///
/// # Type Parameters
///
/// * `T` - Input element type
/// * `U` - Output element type
/// * `E` - Error type
/// * `Env` - Environment type
/// * `F` - Function type that transforms `T` into `BoxedEffect<U, E, Env>`
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{BoxedEffect, traverse::traverse_effect_sequential};
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// fn double(x: i32) -> BoxedEffect<i32, String, ()> {
///     pure(x * 2).boxed()
/// }
///
/// let result = traverse_effect_sequential(vec![1, 2, 3], double);
/// assert_eq!(result.run(&()).await, Ok(vec![2, 4, 6]));
/// # });
/// ```
pub fn traverse_effect_sequential<T, U, E, Env, F, I>(
    iter: I,
    mut f: F,
) -> BoxedEffect<Vec<U>, E, Env>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T) -> BoxedEffect<U, E, Env> + Send + 'static,
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    use crate::effect::prelude::*;
    let items: Vec<_> = iter.into_iter().collect();
    from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(f(item).run(&env).await?);
            }
            Ok(values)
        }
    })
    .boxed()
}

/// Traverse a collection in parallel with an effect function.
///
/// Constructs all effects when the returned effect runs, then runs them
/// concurrently. All effects run to completion. Successful values and the
/// first returned error are ordered by input position, not completion time.
pub fn traverse_effect_parallel<T, U, E, Env, F, I>(
    iter: I,
    mut f: F,
) -> BoxedEffect<Vec<U>, E, Env>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T) -> BoxedEffect<U, E, Env> + Send + 'static,
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    use crate::effect::prelude::*;
    let items: Vec<_> = iter.into_iter().collect();
    from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let effects = items.into_iter().map(&mut f).collect();
            par_try_all(effects, &env).await
        }
    })
    .boxed()
}

/// Sequence a collection of effects sequentially.
///
/// Runs one effect at a time and stops at the first error. Effects after the
/// failure are not run. Successful values retain input order.
///
/// # Type Parameters
///
/// * `T` - Success value type
/// * `E` - Error type
/// * `Env` - Environment type
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{BoxedEffect, traverse::sequence_effect_sequential};
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effects = vec![
///     pure::<_, String, ()>(1).boxed(),
///     pure(2).boxed(),
///     pure(3).boxed(),
/// ];
/// let result = sequence_effect_sequential(effects);
/// assert_eq!(result.run(&()).await, Ok(vec![1, 2, 3]));
/// # });
/// ```
pub fn sequence_effect_sequential<T, E, Env, I>(iter: I) -> BoxedEffect<Vec<T>, E, Env>
where
    I: IntoIterator<Item = BoxedEffect<T, E, Env>>,
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    use crate::effect::prelude::*;
    let effects: Vec<BoxedEffect<T, E, Env>> = iter.into_iter().collect();
    from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let mut values = Vec::with_capacity(effects.len());
            for effect in effects {
                values.push(effect.run(&env).await?);
            }
            Ok(values)
        }
    })
    .boxed()
}

/// Sequence a collection of effects in parallel.
///
/// Runs all effects concurrently and waits for all of them to complete.
/// Successful values and the first returned error are ordered by input
/// position, not completion time.
pub fn sequence_effect_parallel<T, E, Env, I>(iter: I) -> BoxedEffect<Vec<T>, E, Env>
where
    I: IntoIterator<Item = BoxedEffect<T, E, Env>>,
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    use crate::effect::prelude::*;
    let effects: Vec<BoxedEffect<T, E, Env>> = iter.into_iter().collect();
    from_async(move |env: &Env| {
        let env = env.clone();
        async move { par_try_all(effects, &env).await }
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validation traverse tests
    #[test]
    fn test_traverse_all_success() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(vec![1, 2, 3], validate_positive);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    }

    #[test]
    fn test_traverse_with_failures() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(vec![1, -2, -3], validate_positive);
        assert!(result.is_failure());

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_traverse_empty() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(Vec::<i32>::new(), validate_positive);
        assert_eq!(result, Validation::Success(vec![]));
    }

    // Validation sequence tests
    #[test]
    fn test_sequence_all_success() {
        let vals = vec![
            Validation::<_, Vec<String>>::success(1),
            Validation::success(2),
            Validation::success(3),
        ];
        let result = sequence(vals);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    }

    #[test]
    fn test_sequence_with_failures() {
        let vals = vec![
            Validation::<i32, _>::failure(vec!["error1".to_string()]),
            Validation::success(2),
            Validation::failure(vec!["error2".to_string()]),
        ];
        let result = sequence(vals);
        assert!(result.is_failure());
    }

    #[test]
    fn test_sequence_empty() {
        let vals: Vec<Validation<i32, Vec<String>>> = vec![];
        let result = sequence(vals);
        assert_eq!(result, Validation::Success(vec![]));
    }

    // Effect traverse tests
    #[tokio::test]
    async fn test_traverse_effect_all_success() {
        use crate::effect::prelude::*;
        fn double(x: i32) -> BoxedEffect<i32, String, ()> {
            pure(x * 2).boxed()
        }

        let result = traverse_effect_sequential(vec![1, 2, 3], double);
        assert_eq!(result.run(&()).await, Ok(vec![2, 4, 6]));
    }

    #[tokio::test]
    async fn test_traverse_effect_with_failure() {
        use crate::effect::prelude::*;
        fn check_positive(x: i32) -> BoxedEffect<i32, String, ()> {
            if x > 0 {
                pure(x).boxed()
            } else {
                fail(format!("{} is not positive", x)).boxed()
            }
        }

        let result = traverse_effect_sequential(vec![1, -2, 3], check_positive);
        assert!(result.run(&()).await.is_err());
    }

    #[tokio::test]
    async fn test_traverse_effect_empty() {
        use crate::effect::prelude::*;
        fn double(x: i32) -> BoxedEffect<i32, String, ()> {
            pure(x * 2).boxed()
        }

        let result = traverse_effect_sequential(Vec::<i32>::new(), double);
        assert_eq!(result.run(&()).await, Ok(vec![]));
    }

    // Effect sequence tests
    #[tokio::test]
    async fn test_sequence_effect_all_success() {
        use crate::effect::prelude::*;
        let effects = vec![
            pure::<_, String, ()>(1).boxed(),
            pure(2).boxed(),
            pure(3).boxed(),
        ];
        let result = sequence_effect_sequential(effects);
        assert_eq!(result.run(&()).await, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_sequence_effect_with_failure() {
        use crate::effect::prelude::*;
        let effects = vec![
            pure::<_, String, ()>(1).boxed(),
            fail("error".to_string()).boxed(),
            pure(3).boxed(),
        ];
        let result = sequence_effect_sequential(effects);
        assert!(result.run(&()).await.is_err());
    }

    #[tokio::test]
    async fn test_sequence_effect_empty() {
        use crate::effect::prelude::*;
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![];
        let result = sequence_effect_sequential(effects);
        assert_eq!(result.run(&()).await, Ok(vec![]));
    }

    // Integration tests
    #[test]
    fn test_traverse_with_parse() {
        fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
            s.parse()
                .map(Validation::success)
                .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid number: {}", s)]))
        }

        let numbers = vec!["1", "2", "3"];
        let result = traverse(numbers, parse_number);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));

        let mixed = vec!["1", "invalid", "3"];
        let result = traverse(mixed, parse_number);
        assert!(result.is_failure());
    }

    #[tokio::test]
    async fn test_traverse_effect_with_env() {
        use crate::effect::prelude::*;

        #[derive(Clone)]
        struct Env {
            multiplier: i32,
        }

        fn multiply(x: i32) -> BoxedEffect<i32, String, Env> {
            asks(move |env: &Env| x * env.multiplier).boxed()
        }

        let env = Env { multiplier: 3 };
        let result = traverse_effect_sequential(vec![1, 2, 3], multiply);
        assert_eq!(result.run(&env).await, Ok(vec![3, 6, 9]));
    }

    #[tokio::test]
    async fn sequential_traversal_stops_before_constructing_later_effects() {
        use crate::effect::prelude::*;
        use std::sync::{Arc, Mutex};

        let constructed = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&constructed);
        let effect = traverse_effect_sequential(vec![1, 2, 3], move |item| {
            observed.lock().unwrap().push(item);
            if item == 2 {
                fail("stop").boxed()
            } else {
                pure(item).boxed()
            }
        });

        assert!(constructed.lock().unwrap().is_empty());
        assert_eq!(effect.run(&()).await, Err("stop"));
        assert_eq!(*constructed.lock().unwrap(), vec![1, 2]);
    }

    #[tokio::test]
    async fn sequential_sequence_runs_in_strict_order() {
        use crate::effect::prelude::*;
        use std::sync::{Arc, Mutex};

        let events = Arc::new(Mutex::new(Vec::new()));
        let effects = (1..=3)
            .map(|item| {
                let events = Arc::clone(&events);
                from_async(move |_: &()| async move {
                    events.lock().unwrap().push(format!("start-{item}"));
                    tokio::task::yield_now().await;
                    events.lock().unwrap().push(format!("end-{item}"));
                    Ok::<_, &'static str>(item)
                })
                .boxed()
            })
            .collect::<Vec<_>>();

        let result = sequence_effect_sequential(effects).run(&()).await;

        assert_eq!(result, Ok(vec![1, 2, 3]));
        assert_eq!(
            *events.lock().unwrap(),
            vec!["start-1", "end-1", "start-2", "end-2", "start-3", "end-3"]
        );
    }

    #[tokio::test]
    async fn parallel_traversal_overlaps_and_preserves_input_order() {
        use crate::effect::prelude::*;
        use std::sync::Arc;
        use std::time::Duration;
        use tokio::sync::Barrier;

        let barrier = Arc::new(Barrier::new(3));
        let effect = traverse_effect_parallel(vec![1_u64, 2, 3], move |item| {
            let barrier = Arc::clone(&barrier);
            from_async(move |_: &()| async move {
                barrier.wait().await;
                tokio::time::sleep(Duration::from_millis((4 - item) * 5)).await;
                Ok::<_, &'static str>(item)
            })
            .boxed()
        });

        let result = tokio::time::timeout(Duration::from_secs(1), effect.run(&()))
            .await
            .expect("parallel effects should all reach the barrier");

        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn parallel_sequence_waits_for_other_effects_after_failure() {
        use crate::effect::prelude::*;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let completed = Arc::new(AtomicBool::new(false));
        let completed_later = Arc::clone(&completed);
        let effects = vec![
            fail::<i32, _, ()>("first").boxed(),
            from_async(move |_: &()| async move {
                tokio::task::yield_now().await;
                completed_later.store(true, Ordering::SeqCst);
                Ok::<_, &'static str>(2)
            })
            .boxed(),
        ];

        let result = sequence_effect_parallel(effects).run(&()).await;

        assert_eq!(result, Err("first"));
        assert!(completed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn explicit_effect_traversals_accept_empty_inputs() {
        use crate::effect::prelude::*;

        let sequential =
            traverse_effect_sequential(Vec::<i32>::new(), |_| pure::<_, String, ()>(0).boxed());
        let parallel =
            traverse_effect_parallel(Vec::<i32>::new(), |_| pure::<_, String, ()>(0).boxed());
        let sequential_sequence =
            sequence_effect_sequential(Vec::<BoxedEffect<i32, String, ()>>::new());
        let parallel_sequence =
            sequence_effect_parallel(Vec::<BoxedEffect<i32, String, ()>>::new());

        assert_eq!(sequential.run(&()).await, Ok(vec![]));
        assert_eq!(parallel.run(&()).await, Ok(vec![]));
        assert_eq!(sequential_sequence.run(&()).await, Ok(vec![]));
        assert_eq!(parallel_sequence.run(&()).await, Ok(vec![]));
    }
}
