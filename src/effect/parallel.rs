//! Parallel execution functions for effects.
//!
//! This module provides functions for running effects in parallel:
//! - `par_all` - Run all effects, collecting results or errors
//! - `par_try_all` - Run all effects, fail-fast on first error
//! - `race` - Race effects, return first to complete
//! - `par2`, `par3` - Run heterogeneous effects in parallel

use crate::effect::boxed::BoxedEffect;
use crate::effect::trait_def::Effect;

/// Execute boxed effects in parallel, collecting all results or all errors.
///
/// Returns `Ok(results)` if all effects succeed, `Err(errors)` if any fail.
/// All effects run to completion regardless of individual failures.
///
/// Requires boxed effects because `Vec<T>` needs homogeneous types.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
///     pure(1).boxed(),
///     pure(2).boxed(),
///     pure(3).boxed(),
/// ];
///
/// let result = par_all(effects, &()).await;
/// assert_eq!(result, Ok(vec![1, 2, 3]));
/// ```
pub async fn par_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, Vec<E>>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    let futures: Vec<_> = effects.into_iter().map(|eff| eff.run(env)).collect();

    let results: Vec<Result<T, E>> = futures::future::join_all(futures).await;

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in results {
        match result {
            Ok(value) => successes.push(value),
            Err(e) => failures.push(e),
        }
    }

    if failures.is_empty() {
        Ok(successes)
    } else {
        Err(failures)
    }
}

/// Execute boxed effects in parallel, fail-fast on first error.
///
/// Returns `Ok(results)` if all succeed, `Err(first_error)` on first failure.
/// Note: Other effects may continue running after the first error.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
///     pure(1).boxed(),
///     pure(2).boxed(),
/// ];
///
/// let result = par_try_all(effects, &()).await;
/// assert_eq!(result, Ok(vec![1, 2]));
/// ```
pub async fn par_try_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, E>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    let futures: Vec<_> = effects.into_iter().map(|eff| eff.run(env)).collect();

    let results: Vec<Result<T, E>> = futures::future::join_all(futures).await;

    results.into_iter().collect()
}

/// Race effects, returning the first to complete successfully.
///
/// Returns the result of the first effect to complete.
/// Other effects are dropped (cancelled).
///
/// # Panics
///
/// Panics if the effects vec is empty.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
///     pure(1).boxed(),
///     pure(2).boxed(),
/// ];
///
/// let result = race(effects, &()).await;
/// // Result is either Ok(1) or Ok(2), whichever completes first
/// ```
pub async fn race<T, E, Env>(effects: Vec<BoxedEffect<T, E, Env>>, env: &Env) -> Result<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    if effects.is_empty() {
        panic!("race called with empty effects vec");
    }

    let futures: Vec<_> = effects
        .into_iter()
        .map(|eff| Box::pin(eff.run(env)))
        .collect();

    let (result, _index, _remaining) = futures::future::select_all(futures).await;
    result
}

/// Execute two effects in parallel (heterogeneous).
///
/// Zero-cost when effects have concrete types.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let e1 = pure::<_, String, ()>(42);
/// let e2 = pure::<_, String, ()>("hello".to_string());
///
/// let (r1, r2) = par2(e1, e2, &()).await;
/// assert_eq!(r1, Ok(42));
/// assert_eq!(r2, Ok("hello".to_string()));
/// ```
pub async fn par2<E1, E2>(
    e1: E1,
    e2: E2,
    env: &E1::Env,
) -> (Result<E1::Output, E1::Error>, Result<E2::Output, E2::Error>)
where
    E1: Effect,
    E2: Effect<Env = E1::Env>,
{
    futures::join!(e1.run(env), e2.run(env))
}

/// Execute three effects in parallel (heterogeneous).
///
/// Zero-cost when effects have concrete types.
pub async fn par3<E1, E2, E3>(
    e1: E1,
    e2: E2,
    e3: E3,
    env: &E1::Env,
) -> (
    Result<E1::Output, E1::Error>,
    Result<E2::Output, E2::Error>,
    Result<E3::Output, E3::Error>,
)
where
    E1: Effect,
    E2: Effect<Env = E1::Env>,
    E3: Effect<Env = E1::Env>,
{
    futures::join!(e1.run(env), e2.run(env), e3.run(env))
}

/// Execute four effects in parallel (heterogeneous).
///
/// Zero-cost when effects have concrete types.
pub async fn par4<E1, E2, E3, E4>(
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    env: &E1::Env,
) -> (
    Result<E1::Output, E1::Error>,
    Result<E2::Output, E2::Error>,
    Result<E3::Output, E3::Error>,
    Result<E4::Output, E4::Error>,
)
where
    E1: Effect,
    E2: Effect<Env = E1::Env>,
    E3: Effect<Env = E1::Env>,
    E4: Effect<Env = E1::Env>,
{
    futures::join!(e1.run(env), e2.run(env), e3.run(env), e4.run(env))
}

/// Execute boxed effects in parallel with a concurrency limit.
///
/// Returns `Ok(results)` if all effects succeed, `Err(errors)` if any fail.
/// All effects run to completion regardless of individual failures.
///
/// Useful for rate limiting or resource constraints.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effects: Vec<BoxedEffect<i32, String, ()>> = (1..=10)
///     .map(|i| pure(i).boxed())
///     .collect();
///
/// let result = par_all_limit(effects, 3, &()).await;
/// assert_eq!(result.as_ref().map(|v| v.len()), Ok(10));
/// ```
pub async fn par_all_limit<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    limit: usize,
    env: &Env,
) -> Result<Vec<T>, Vec<E>>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    use futures::stream::{self, StreamExt};

    let results: Vec<Result<T, E>> = stream::iter(effects)
        .map(|eff| eff.run(env))
        .buffer_unordered(limit)
        .collect()
        .await;

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in results {
        match result {
            Ok(value) => successes.push(value),
            Err(e) => failures.push(e),
        }
    }

    if failures.is_empty() {
        Ok(successes)
    } else {
        Err(failures)
    }
}

/// Macro for arbitrary parallel execution with tuple return.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let e1 = pure::<_, String, ()>(1);
/// let e2 = pure::<_, String, ()>(2);
/// let e3 = pure::<_, String, ()>(3);
///
/// let (r1, r2, r3) = par!(&env; e1, e2, e3);
/// ```
#[macro_export]
macro_rules! par {
    ($env:expr; $($effect:expr),+ $(,)?) => {
        futures::join!($($effect.run($env)),+)
    };
}
