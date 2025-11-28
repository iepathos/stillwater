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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, from_async, pure};
    use crate::effect::ext::EffectExt;
    use std::time::{Duration, Instant};

    // ==================== Test Helpers ====================

    /// Creates an effect that succeeds after a delay.
    fn delayed_success<T: Clone + Send + 'static>(
        value: T,
        delay: Duration,
    ) -> BoxedEffect<T, String, ()> {
        from_async(move |_: &()| {
            let value = value.clone();
            async move {
                tokio::time::sleep(delay).await;
                Ok(value)
            }
        })
        .boxed()
    }

    /// Creates an effect that fails after a delay.
    fn delayed_failure<T: Send + 'static>(
        error: String,
        delay: Duration,
    ) -> BoxedEffect<T, String, ()> {
        from_async(move |_: &()| {
            let error = error.clone();
            async move {
                tokio::time::sleep(delay).await;
                Err(error)
            }
        })
        .boxed()
    }

    // ==================== par_all Tests ====================

    #[tokio::test]
    async fn test_par_all_all_succeed() {
        let effects: Vec<BoxedEffect<i32, String, ()>> =
            vec![pure(1).boxed(), pure(2).boxed(), pure(3).boxed()];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_par_all_accumulates_all_errors() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            fail("error1".to_string()).boxed(),
            pure(3).boxed(),
            fail("error2".to_string()).boxed(),
        ];

        let result = par_all(effects, &()).await;
        // Should collect ALL errors, not just the first
        assert_eq!(
            result,
            Err(vec!["error1".to_string(), "error2".to_string()])
        );
    }

    #[tokio::test]
    async fn test_par_all_all_fail() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            fail("error1".to_string()).boxed(),
            fail("error2".to_string()).boxed(),
            fail("error3".to_string()).boxed(),
        ];

        let result = par_all(effects, &()).await;
        assert_eq!(
            result,
            Err(vec![
                "error1".to_string(),
                "error2".to_string(),
                "error3".to_string()
            ])
        );
    }

    #[tokio::test]
    async fn test_par_all_empty_collection() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Ok(vec![]));
    }

    #[tokio::test]
    async fn test_par_all_single_effect_success() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![pure(42).boxed()];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Ok(vec![42]));
    }

    #[tokio::test]
    async fn test_par_all_single_effect_failure() {
        let effects: Vec<BoxedEffect<i32, String, ()>> =
            vec![fail("single error".to_string()).boxed()];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Err(vec!["single error".to_string()]));
    }

    #[tokio::test]
    async fn test_par_all_runs_in_parallel() {
        let delay = Duration::from_millis(50);
        let effects = vec![
            delayed_success(1, delay),
            delayed_success(2, delay),
            delayed_success(3, delay),
        ];

        let start = Instant::now();
        let result = par_all(effects, &()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // If parallel: ~50ms, if sequential: ~150ms
        // Use 100ms threshold for robustness
        assert!(
            elapsed < Duration::from_millis(100),
            "Expected parallel execution (<100ms), got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_par_all_error_order_matches_input_order() {
        // Errors from later effects should still appear in input order
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            delayed_failure("first".to_string(), Duration::from_millis(30)),
            delayed_failure("second".to_string(), Duration::from_millis(10)),
            delayed_failure("third".to_string(), Duration::from_millis(20)),
        ];

        let result = par_all(effects, &()).await;
        // Errors collected in order they appear in input, regardless of completion time
        assert_eq!(
            result,
            Err(vec![
                "first".to_string(),
                "second".to_string(),
                "third".to_string()
            ])
        );
    }

    // ==================== par_try_all Tests ====================

    #[tokio::test]
    async fn test_par_try_all_all_succeed() {
        let effects: Vec<BoxedEffect<i32, String, ()>> =
            vec![pure(1).boxed(), pure(2).boxed(), pure(3).boxed()];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_par_try_all_returns_first_error_by_position() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            fail("first_error".to_string()).boxed(),
            pure(3).boxed(),
            fail("second_error".to_string()).boxed(),
        ];

        let result = par_try_all(effects, &()).await;
        // Returns first error by position in input vec
        assert_eq!(result, Err("first_error".to_string()));
    }

    #[tokio::test]
    async fn test_par_try_all_first_effect_fails() {
        let effects: Vec<BoxedEffect<i32, String, ()>> =
            vec![fail("error".to_string()).boxed(), pure(2).boxed()];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_par_try_all_last_effect_fails() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            pure(2).boxed(),
            fail("error".to_string()).boxed(),
        ];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_par_try_all_empty_collection() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Ok(vec![]));
    }

    #[tokio::test]
    async fn test_par_try_all_single_success() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![pure(42).boxed()];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Ok(vec![42]));
    }

    #[tokio::test]
    async fn test_par_try_all_single_failure() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![fail("error".to_string()).boxed()];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Err("error".to_string()));
    }

    // ==================== race Tests ====================

    #[tokio::test]
    async fn test_race_first_to_complete_wins() {
        let effects = vec![
            delayed_success(1, Duration::from_millis(10)), // Winner (fastest)
            delayed_success(2, Duration::from_millis(100)),
            delayed_success(3, Duration::from_millis(100)),
        ];

        let result = race(effects, &()).await;
        assert_eq!(result, Ok(1));
    }

    #[tokio::test]
    async fn test_race_timing_verification() {
        let effects = vec![
            delayed_success(1, Duration::from_millis(100)),
            delayed_success(2, Duration::from_millis(10)), // Winner (fastest)
            delayed_success(3, Duration::from_millis(100)),
        ];

        let start = Instant::now();
        let result = race(effects, &()).await;
        let elapsed = start.elapsed();

        assert_eq!(result, Ok(2));
        // Should complete around 10ms, not 100ms
        assert!(
            elapsed < Duration::from_millis(50),
            "Expected race winner at ~10ms, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_race_first_success_wins_over_later_failures() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            delayed_success(1, Duration::from_millis(10)), // Wins with success
            delayed_failure("error".to_string(), Duration::from_millis(100)),
        ];

        let result = race(effects, &()).await;
        assert_eq!(result, Ok(1));
    }

    #[tokio::test]
    async fn test_race_single_effect() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![pure(42).boxed()];

        let result = race(effects, &()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_race_single_failure() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![fail("error".to_string()).boxed()];

        let result = race(effects, &()).await;
        assert_eq!(result, Err("error".to_string()));
    }

    #[tokio::test]
    #[should_panic(expected = "race called with empty effects vec")]
    async fn test_race_empty_panics() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![];
        let _ = race(effects, &()).await;
    }

    // Note: The current race implementation uses select_all which returns
    // the first to complete, whether success or failure. The remaining
    // futures are dropped.

    // ==================== par_all_limit Tests ====================

    #[tokio::test]
    async fn test_par_all_limit_all_succeed() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = (1..=5).map(|i| pure(i).boxed()).collect();

        let result = par_all_limit(effects, 3, &()).await;
        // Note: buffer_unordered may return in different order
        let mut values = result.unwrap();
        values.sort();
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_par_all_limit_with_errors() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            fail("error1".to_string()).boxed(),
            pure(3).boxed(),
            fail("error2".to_string()).boxed(),
        ];

        let result = par_all_limit(effects, 2, &()).await;
        // Errors are collected (order may vary with buffer_unordered)
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
        assert!(errors.contains(&"error1".to_string()));
        assert!(errors.contains(&"error2".to_string()));
    }

    #[tokio::test]
    async fn test_par_all_limit_empty() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![];

        let result = par_all_limit(effects, 3, &()).await;
        assert_eq!(result, Ok(vec![]));
    }

    #[tokio::test]
    async fn test_par_all_limit_respects_concurrency() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let effects: Vec<BoxedEffect<i32, String, ()>> = (0..6)
            .map(|i| {
                let cc = concurrent_count.clone();
                let mc = max_concurrent.clone();
                from_async(move |_: &()| {
                    let cc = cc.clone();
                    let mc = mc.clone();
                    async move {
                        // Increment concurrent count
                        let current = cc.fetch_add(1, Ordering::SeqCst) + 1;
                        // Track max
                        mc.fetch_max(current, Ordering::SeqCst);

                        // Simulate work
                        tokio::time::sleep(Duration::from_millis(20)).await;

                        // Decrement concurrent count
                        cc.fetch_sub(1, Ordering::SeqCst);

                        Ok(i)
                    }
                })
                .boxed()
            })
            .collect();

        let result = par_all_limit(effects, 2, &()).await;

        assert!(result.is_ok());
        // Max concurrent should be <= limit (2)
        let observed_max = max_concurrent.load(Ordering::SeqCst);
        assert!(
            observed_max <= 2,
            "Expected max concurrency <= 2, got {}",
            observed_max
        );
    }

    #[tokio::test]
    async fn test_par_all_limit_of_one_sequential() {
        let delay = Duration::from_millis(20);
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            delayed_success(1, delay),
            delayed_success(2, delay),
            delayed_success(3, delay),
        ];

        let start = Instant::now();
        let result = par_all_limit(effects, 1, &()).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // With limit=1, should run sequentially: ~60ms
        assert!(
            elapsed >= Duration::from_millis(50),
            "Expected sequential execution (>=50ms), got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_par_all_limit_large_enough_is_parallel() {
        let delay = Duration::from_millis(30);
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            delayed_success(1, delay),
            delayed_success(2, delay),
            delayed_success(3, delay),
        ];

        let start = Instant::now();
        let result = par_all_limit(effects, 10, &()).await; // Limit >= count
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should run in parallel: ~30ms
        assert!(
            elapsed < Duration::from_millis(60),
            "Expected parallel execution (<60ms), got {:?}",
            elapsed
        );
    }

    // ==================== par2, par3, par4 Tests ====================

    #[tokio::test]
    async fn test_par2_both_succeed() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = pure::<_, String, ()>("hello".to_string());

        let (r1, r2) = par2(e1, e2, &()).await;
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Ok("hello".to_string()));
    }

    #[tokio::test]
    async fn test_par2_first_fails() {
        let e1 = fail::<i32, _, ()>("error".to_string());
        let e2 = pure::<_, String, ()>("hello".to_string());

        let (r1, r2) = par2(e1, e2, &()).await;
        assert_eq!(r1, Err("error".to_string()));
        assert_eq!(r2, Ok("hello".to_string()));
    }

    #[tokio::test]
    async fn test_par2_second_fails() {
        let e1 = pure::<_, String, ()>(42);
        let e2 = fail::<String, _, ()>("error".to_string());

        let (r1, r2) = par2(e1, e2, &()).await;
        assert_eq!(r1, Ok(42));
        assert_eq!(r2, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_par2_both_fail() {
        let e1 = fail::<i32, _, ()>("error1".to_string());
        let e2 = fail::<String, _, ()>("error2".to_string());

        let (r1, r2) = par2(e1, e2, &()).await;
        assert_eq!(r1, Err("error1".to_string()));
        assert_eq!(r2, Err("error2".to_string()));
    }

    #[tokio::test]
    async fn test_par3_all_succeed() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = pure::<_, String, ()>(2);
        let e3 = pure::<_, String, ()>(3);

        let (r1, r2, r3) = par3(e1, e2, e3, &()).await;
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Ok(2));
        assert_eq!(r3, Ok(3));
    }

    #[tokio::test]
    async fn test_par3_mixed_results() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = fail::<i32, _, ()>("error".to_string());
        let e3 = pure::<_, String, ()>(3);

        let (r1, r2, r3) = par3(e1, e2, e3, &()).await;
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Err("error".to_string()));
        assert_eq!(r3, Ok(3));
    }

    #[tokio::test]
    async fn test_par4_all_succeed() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = pure::<_, String, ()>(2);
        let e3 = pure::<_, String, ()>(3);
        let e4 = pure::<_, String, ()>(4);

        let (r1, r2, r3, r4) = par4(e1, e2, e3, e4, &()).await;
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Ok(2));
        assert_eq!(r3, Ok(3));
        assert_eq!(r4, Ok(4));
    }

    // ==================== Environment Sharing Tests ====================

    #[tokio::test]
    async fn test_par_all_shares_environment() {
        #[derive(Clone)]
        struct Env {
            multiplier: i32,
        }

        let effects: Vec<BoxedEffect<i32, String, Env>> = vec![
            from_async(|env: &Env| {
                let m = env.multiplier;
                async move { Ok(m) }
            })
            .boxed(),
            from_async(|env: &Env| {
                let m = env.multiplier;
                async move { Ok(2 * m) }
            })
            .boxed(),
            from_async(|env: &Env| {
                let m = env.multiplier;
                async move { Ok(3 * m) }
            })
            .boxed(),
        ];

        let env = Env { multiplier: 10 };
        let result = par_all(effects, &env).await;
        // Values: multiplier=10, multiplier*2=20, multiplier*3=30
        assert_eq!(result, Ok(vec![10, 20, 30]));
    }

    #[tokio::test]
    async fn test_par2_shares_environment() {
        #[derive(Clone)]
        struct Env {
            value: i32,
        }

        let e1 = from_async(|env: &Env| {
            let v = env.value;
            async move { Ok::<_, String>(v) }
        });
        let e2 = from_async(|env: &Env| {
            let v = env.value;
            async move { Ok::<_, String>(v * 2) }
        });

        let env = Env { value: 21 };
        let (r1, r2) = par2(e1, e2, &env).await;
        assert_eq!(r1, Ok(21));
        assert_eq!(r2, Ok(42));
    }

    #[tokio::test]
    async fn test_par_all_limit_shares_environment() {
        #[derive(Clone)]
        struct Env {
            prefix: String,
        }

        let effects: Vec<BoxedEffect<String, String, Env>> = (1..=3)
            .map(|i| {
                from_async(move |env: &Env| {
                    let prefix = env.prefix.clone();
                    async move { Ok(format!("{}-{}", prefix, i)) }
                })
                .boxed()
            })
            .collect();

        let env = Env {
            prefix: "item".to_string(),
        };
        let result = par_all_limit(effects, 2, &env).await;
        let mut values = result.unwrap();
        values.sort();
        assert_eq!(values, vec!["item-1", "item-2", "item-3"]);
    }

    // ==================== par! Macro Tests ====================

    #[tokio::test]
    async fn test_par_macro_two_effects() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = pure::<_, String, ()>(2);

        let (r1, r2) = crate::par!(&(); e1, e2);
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Ok(2));
    }

    #[tokio::test]
    async fn test_par_macro_three_effects() {
        let e1 = pure::<_, String, ()>(1);
        let e2 = pure::<_, String, ()>(2);
        let e3 = pure::<_, String, ()>(3);

        let (r1, r2, r3) = crate::par!(&(); e1, e2, e3);
        assert_eq!(r1, Ok(1));
        assert_eq!(r2, Ok(2));
        assert_eq!(r3, Ok(3));
    }

    #[tokio::test]
    async fn test_par_macro_with_environment() {
        #[derive(Clone)]
        struct Env {
            value: i32,
        }

        let e1 = from_async(|env: &Env| {
            let v = env.value;
            async move { Ok::<_, String>(v) }
        });
        let e2 = from_async(|env: &Env| {
            let v = env.value;
            async move { Ok::<_, String>(v * 2) }
        });

        let env = Env { value: 21 };
        let (r1, r2) = crate::par!(&env; e1, e2);
        assert_eq!(r1, Ok(21));
        assert_eq!(r2, Ok(42));
    }
}
