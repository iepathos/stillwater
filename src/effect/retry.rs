//! Retry support for zero-cost effects.
//!
//! This module provides retry combinators that integrate with the existing
//! `crate::retry::{RetryPolicy, RetryEvent, RetryExhausted, TimeoutError}` types.

use std::time::{Duration, Instant};

use crate::effect::boxed::BoxedEffect;
use crate::effect::ext::EffectExt;
use crate::effect::trait_def::Effect;
use crate::retry::{RetryEvent, RetryExhausted, RetryPolicy, TimeoutError};

/// Retry an effect using a factory function.
///
/// Each retry creates a fresh effect via the factory. This is semantically
/// correct for I/O operations which should be recreated (fresh connections,
/// new request IDs, etc.) rather than "cloned."
///
/// # Why a factory function?
///
/// Effects use `FnOnce` internally and are consumed on execution. Rather
/// than adding complexity to make effects cloneable, we embrace Rust's
/// ownership model: retry means "try this operation again from scratch."
///
/// # Returns
///
/// Returns a `BoxedEffect` because the retry loop creates dynamic control flow
/// that cannot be represented as a zero-cost combinator type.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::effect::retry::retry;
/// use stillwater::RetryPolicy;
/// use std::time::Duration;
///
/// let effect = retry(
///     || pure::<_, String, ()>(42),
///     RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3)
/// );
///
/// let result = effect.execute(&()).await.unwrap();
/// assert_eq!(result.into_value(), 42);
/// ```
#[cfg(feature = "async")]
pub fn retry<T, E, Env, F, Eff>(
    make_effect: F,
    policy: RetryPolicy,
) -> BoxedEffect<RetryExhausted<T>, RetryExhausted<E>, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    F: Fn() -> Eff + Send + 'static,
    Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
{
    crate::effect::constructors::from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let start = Instant::now();
            let mut attempt = 0u32;
            let mut prev_delay: Option<Duration> = None;

            loop {
                let effect = make_effect();
                match effect.run(&env).await {
                    Ok(value) => {
                        return Ok(RetryExhausted::new(value, attempt + 1, start.elapsed()));
                    }
                    Err(error) => {
                        let delay = policy.delay_with_jitter(attempt, prev_delay);

                        match delay {
                            Some(d) => {
                                tokio::time::sleep(d).await;
                                prev_delay = Some(d);
                                attempt += 1;
                            }
                            None => {
                                return Err(RetryExhausted::new(
                                    error,
                                    attempt + 1,
                                    start.elapsed(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    })
    .boxed()
}

/// Retry only when the predicate returns true for the error.
///
/// Non-retryable errors immediately propagate without retry attempts.
/// Useful for distinguishing transient errors (retry) from permanent
/// errors (fail fast).
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::effect::retry::retry_if;
/// use stillwater::RetryPolicy;
/// use std::time::Duration;
///
/// #[derive(Debug, PartialEq, Clone)]
/// enum AppError { Transient, Permanent }
///
/// let effect = retry_if(
///     || fail::<(), _, ()>(AppError::Permanent),
///     RetryPolicy::constant(Duration::from_millis(10)).with_max_retries(3),
///     |err| matches!(err, AppError::Transient)
/// );
///
/// // Permanent errors are not retried
/// let result = effect.execute(&()).await;
/// assert_eq!(result, Err(AppError::Permanent));
/// ```
#[cfg(feature = "async")]
pub fn retry_if<T, E, Env, F, P, Eff>(
    make_effect: F,
    policy: RetryPolicy,
    should_retry: P,
) -> BoxedEffect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    F: Fn() -> Eff + Send + 'static,
    P: Fn(&E) -> bool + Send + Sync + 'static,
    Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
{
    crate::effect::constructors::from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let mut attempt = 0u32;
            let mut prev_delay: Option<Duration> = None;

            loop {
                let effect = make_effect();
                match effect.run(&env).await {
                    Ok(value) => return Ok(value),
                    Err(error) => {
                        if !should_retry(&error) {
                            return Err(error);
                        }

                        let delay = policy.delay_with_jitter(attempt, prev_delay);

                        match delay {
                            Some(d) => {
                                tokio::time::sleep(d).await;
                                prev_delay = Some(d);
                                attempt += 1;
                            }
                            None => {
                                return Err(error);
                            }
                        }
                    }
                }
            }
        }
    })
    .boxed()
}

/// Retry with hooks for observability.
///
/// The `on_retry` callback is invoked before each retry attempt,
/// receiving information about the failed attempt. The callback
/// is synchronous and should not block; use it for logging/metrics.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::effect::retry::retry_with_hooks;
/// use stillwater::{RetryPolicy, RetryEvent};
/// use std::time::Duration;
///
/// let effect = retry_with_hooks(
///     || pure::<_, String, ()>(42),
///     RetryPolicy::exponential(Duration::from_millis(10)).with_max_retries(3),
///     |event: &RetryEvent<'_, String>| {
///         println!(
///             "Attempt {} failed: {:?}, next delay: {:?}",
///             event.attempt, event.error, event.next_delay
///         );
///     }
/// );
/// ```
#[cfg(feature = "async")]
pub fn retry_with_hooks<T, E, Env, F, H, Eff>(
    make_effect: F,
    policy: RetryPolicy,
    on_retry: H,
) -> BoxedEffect<RetryExhausted<T>, RetryExhausted<E>, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    F: Fn() -> Eff + Send + 'static,
    H: Fn(&RetryEvent<'_, E>) + Send + Sync + 'static,
    Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
{
    crate::effect::constructors::from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            let start = Instant::now();
            let mut attempt = 0u32;
            let mut prev_delay: Option<Duration> = None;

            loop {
                let effect = make_effect();
                match effect.run(&env).await {
                    Ok(value) => {
                        return Ok(RetryExhausted::new(value, attempt + 1, start.elapsed()));
                    }
                    Err(error) => {
                        let delay = policy.delay_with_jitter(attempt, prev_delay);

                        // Call the hook before retrying
                        {
                            let event = RetryEvent {
                                attempt: attempt + 1,
                                error: &error,
                                next_delay: delay,
                                elapsed: start.elapsed(),
                            };
                            on_retry(&event);
                        }

                        match delay {
                            Some(d) => {
                                tokio::time::sleep(d).await;
                                prev_delay = Some(d);
                                attempt += 1;
                            }
                            None => {
                                return Err(RetryExhausted::new(
                                    error,
                                    attempt + 1,
                                    start.elapsed(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    })
    .boxed()
}

/// Add a timeout to an effect.
///
/// If the effect doesn't complete within the duration, it fails
/// with a timeout error.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::effect::retry::with_timeout;
/// use stillwater::TimeoutError;
/// use std::time::Duration;
///
/// let effect = with_timeout(
///     from_async(|_: &()| async {
///         tokio::time::sleep(Duration::from_secs(10)).await;
///         Ok::<_, String>(42)
///     }),
///     Duration::from_millis(10)
/// );
///
/// match effect.execute(&()).await {
///     Err(TimeoutError::Timeout { duration }) => {
///         assert_eq!(duration, Duration::from_millis(10));
///     }
///     _ => panic!("Expected timeout"),
/// }
/// ```
#[cfg(feature = "async")]
pub fn with_timeout<T, E, Env, Eff>(
    effect: Eff,
    duration: Duration,
) -> BoxedEffect<T, TimeoutError<E>, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
{
    crate::effect::constructors::from_async(move |env: &Env| {
        let env = env.clone();
        async move {
            match tokio::time::timeout(duration, effect.run(&env)).await {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(e)) => Err(TimeoutError::Inner(e)),
                Err(_) => Err(TimeoutError::Timeout { duration }),
            }
        }
    })
    .boxed()
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, from_async, from_fn, pure};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // ==========================================================================
    // Helper functions for creating test effects
    // ==========================================================================

    /// Creates an effect that fails `fail_count` times before succeeding with `success_value`.
    fn flaky_effect<E: Clone + Send + 'static>(
        attempt_counter: Arc<AtomicU32>,
        fail_until: u32,
        success_value: i32,
        error: E,
    ) -> impl Effect<Output = i32, Error = E, Env = ()> {
        let error_clone = error.clone();
        from_fn(move |_: &()| {
            let current = attempt_counter.fetch_add(1, Ordering::SeqCst);
            if current < fail_until {
                Err(error_clone.clone())
            } else {
                Ok(success_value)
            }
        })
    }

    // ==========================================================================
    // Tests for retry() function
    // ==========================================================================

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        // Test: Operation succeeds on the first attempt (no retries needed)
        let effect = retry(
            || pure::<_, String, ()>(42),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 42); // Note: final_error holds success value in Ok case
        assert_eq!(result.attempts, 1);
        assert!(result.total_duration < Duration::from_millis(100)); // Should be nearly instant
    }

    #[tokio::test]
    async fn test_retry_success_after_multiple_retries() {
        // Test: Operation fails 2 times, then succeeds on 3rd attempt
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry(
            move || flaky_effect(counter_clone.clone(), 2, 42, "transient error".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 42);
        assert_eq!(result.attempts, 3); // 2 failures + 1 success
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhaustion_all_failures() {
        // Test: All attempts fail, retry exhausted
        let effect = retry(
            || fail::<i32, _, ()>("always fails".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(2),
        );

        let result = effect.execute(&()).await.unwrap_err();
        assert_eq!(result.final_error, "always fails".to_string());
        assert_eq!(result.attempts, 3); // 1 initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_attempt_count_accuracy() {
        // Test: Verify exact attempt counting with different max_retries settings
        for max_retries in [0, 1, 3, 5] {
            let effect = retry(
                || fail::<i32, _, ()>("error".to_string()),
                RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(max_retries),
            );

            let result = effect.execute(&()).await.unwrap_err();
            assert_eq!(
                result.attempts,
                max_retries + 1,
                "Expected {} attempts for max_retries={}, got {}",
                max_retries + 1,
                max_retries,
                result.attempts
            );
        }
    }

    #[tokio::test]
    async fn test_retry_elapsed_time_tracking() {
        // Test: Verify elapsed time is tracked correctly
        let effect = retry(
            || fail::<i32, _, ()>("error".to_string()),
            RetryPolicy::constant(Duration::from_millis(10)).with_max_retries(2),
        );

        let result = effect.execute(&()).await.unwrap_err();
        // Should have waited at least 20ms (2 retries * 10ms each)
        assert!(
            result.total_duration >= Duration::from_millis(15),
            "Expected total_duration >= 15ms, got {:?}",
            result.total_duration
        );
    }

    #[tokio::test]
    async fn test_retry_with_exponential_policy() {
        // Test: Retry with exponential backoff policy
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry(
            move || flaky_effect(counter_clone.clone(), 2, 100, "error".to_string()),
            RetryPolicy::exponential(Duration::from_millis(1)).with_max_retries(5),
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 100);
        assert_eq!(result.attempts, 3);
    }

    #[tokio::test]
    async fn test_retry_with_linear_policy() {
        // Test: Retry with linear backoff policy
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry(
            move || flaky_effect(counter_clone.clone(), 1, 50, "error".to_string()),
            RetryPolicy::linear(Duration::from_millis(1)).with_max_retries(3),
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 50);
        assert_eq!(result.attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_zero_max_retries() {
        // Test: With max_retries=0, only the initial attempt is made
        let effect = retry(
            || fail::<i32, _, ()>("error".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(0),
        );

        let result = effect.execute(&()).await.unwrap_err();
        assert_eq!(result.attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_factory_creates_fresh_effect() {
        // Test: Each retry creates a fresh effect via the factory
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let effect = retry(
            move || {
                // Increment counter each time factory is called
                call_count_clone.fetch_add(1, Ordering::SeqCst);
                fail::<i32, _, ()>("error".to_string())
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
        );

        let _ = effect.execute(&()).await;
        // Factory should be called 4 times (1 initial + 3 retries)
        assert_eq!(call_count.load(Ordering::SeqCst), 4);
    }

    // ==========================================================================
    // Tests for retry_if() function
    // ==========================================================================

    #[tokio::test]
    async fn test_retry_if_retryable_errors_are_retried() {
        // Test: Retryable errors trigger retry attempts
        #[derive(Debug, PartialEq, Clone)]
        enum TestError {
            Transient(u32),
            #[allow(dead_code)]
            Permanent,
        }

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_if(
            move || {
                let counter = counter_clone.clone();
                from_fn(move |_: &()| {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(TestError::Transient(count))
                    } else {
                        Ok(42)
                    }
                })
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
            |err| matches!(err, TestError::Transient(_)),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Ok(42));
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_if_non_retryable_errors_fail_immediately() {
        // Test: Non-retryable errors fail immediately without retry
        #[derive(Debug, PartialEq, Clone)]
        #[allow(dead_code)]
        enum TestError {
            Transient,
            Permanent,
        }

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_if(
            move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                fail::<(), _, ()>(TestError::Permanent)
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
            |err| matches!(err, TestError::Transient),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Err(TestError::Permanent));
        // Only one attempt should be made
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_if_mixed_errors() {
        // Test: Mix of transient errors followed by permanent error
        #[derive(Debug, PartialEq, Clone)]
        enum TestError {
            Transient,
            Permanent,
        }

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_if(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    fail::<i32, _, ()>(TestError::Transient)
                } else {
                    fail::<i32, _, ()>(TestError::Permanent)
                }
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
            |err| matches!(err, TestError::Transient),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Err(TestError::Permanent));
        // 3 attempts: 2 transient + 1 permanent
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_if_predicate_receives_error() {
        // Test: Predicate receives the correct error reference
        #[derive(Debug, PartialEq, Clone)]
        struct ErrorWithCode(u32);

        let error_codes_seen = Arc::new(std::sync::Mutex::new(Vec::new()));
        let codes_clone = error_codes_seen.clone();

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_if(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                fail::<i32, _, ()>(ErrorWithCode(count))
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
            move |err: &ErrorWithCode| {
                codes_clone.lock().unwrap().push(err.0);
                err.0 < 2 // Retry for codes 0, 1; fail on 2+
            },
        );

        let _ = effect.execute(&()).await;
        let codes = error_codes_seen.lock().unwrap().clone();
        // Predicate should have seen errors with codes 0, 1, 2
        assert_eq!(codes, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn test_retry_if_exhaustion_for_retryable_errors() {
        // Test: Retryable errors eventually exhaust retries
        #[derive(Debug, PartialEq, Clone)]
        struct RetryableError;

        let effect = retry_if(
            || fail::<i32, _, ()>(RetryableError),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(2),
            |_: &RetryableError| true, // Always retryable
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Err(RetryableError));
    }

    #[tokio::test]
    async fn test_retry_if_success_on_first_attempt() {
        // Test: Success on first attempt, predicate never called
        let predicate_called = Arc::new(AtomicU32::new(0));
        let pred_clone = predicate_called.clone();

        let effect = retry_if(
            || pure::<_, String, ()>(42),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
            move |_: &String| {
                pred_clone.fetch_add(1, Ordering::SeqCst);
                true
            },
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Ok(42));
        assert_eq!(predicate_called.load(Ordering::SeqCst), 0);
    }

    // ==========================================================================
    // Tests for retry_with_hooks() function
    // ==========================================================================

    #[tokio::test]
    async fn test_retry_with_hooks_on_retry_called() {
        // Test: on_retry hook is called for each retry attempt
        let hook_calls = Arc::new(AtomicU32::new(0));
        let hook_clone = hook_calls.clone();

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_with_hooks(
            move || flaky_effect(counter_clone.clone(), 2, 42, "error".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
            move |_event: &RetryEvent<'_, String>| {
                hook_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 42);
        // Hook should be called twice (for the 2 failures)
        assert_eq!(hook_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_with_hooks_event_data() {
        // Test: Hook receives correct RetryEvent information
        let events = Arc::new(std::sync::Mutex::new(Vec::<(u32, String, bool)>::new()));
        let events_clone = events.clone();

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_with_hooks(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                fail::<i32, _, ()>(format!("error_{}", count))
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(2),
            move |event: &RetryEvent<'_, String>| {
                events_clone.lock().unwrap().push((
                    event.attempt,
                    event.error.clone(),
                    event.next_delay.is_some(),
                ));
            },
        );

        let _ = effect.execute(&()).await;
        let recorded_events = events.lock().unwrap().clone();

        // Should have 3 events (for attempts 1, 2, 3)
        assert_eq!(recorded_events.len(), 3);

        // First failure (attempt 1)
        assert_eq!(recorded_events[0].0, 1);
        assert_eq!(recorded_events[0].1, "error_0");
        assert!(recorded_events[0].2); // has next_delay

        // Second failure (attempt 2)
        assert_eq!(recorded_events[1].0, 2);
        assert_eq!(recorded_events[1].1, "error_1");
        assert!(recorded_events[1].2); // has next_delay

        // Third failure (attempt 3) - exhausted
        assert_eq!(recorded_events[2].0, 3);
        assert_eq!(recorded_events[2].1, "error_2");
        assert!(!recorded_events[2].2); // no next_delay (exhausted)
    }

    #[tokio::test]
    async fn test_retry_with_hooks_not_called_on_success() {
        // Test: Hook is not called when first attempt succeeds
        let hook_called = Arc::new(AtomicU32::new(0));
        let hook_clone = hook_called.clone();

        let effect = retry_with_hooks(
            || pure::<_, String, ()>(42),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
            move |_event: &RetryEvent<'_, String>| {
                hook_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 42);
        assert_eq!(hook_called.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_retry_with_hooks_elapsed_time_in_event() {
        // Test: RetryEvent contains increasing elapsed time
        let elapsed_times = Arc::new(std::sync::Mutex::new(Vec::<Duration>::new()));
        let times_clone = elapsed_times.clone();

        let effect = retry_with_hooks(
            || fail::<i32, _, ()>("error".to_string()),
            RetryPolicy::constant(Duration::from_millis(5)).with_max_retries(2),
            move |event: &RetryEvent<'_, String>| {
                times_clone.lock().unwrap().push(event.elapsed);
            },
        );

        let _ = effect.execute(&()).await;
        let times = elapsed_times.lock().unwrap().clone();

        // Each elapsed time should be greater than the previous
        for window in times.windows(2) {
            assert!(
                window[1] >= window[0],
                "Elapsed times should be non-decreasing: {:?} vs {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[tokio::test]
    async fn test_retry_with_hooks_success_after_retries() {
        // Test: Hook is called for failures, not for the final success
        let hook_calls = Arc::new(AtomicU32::new(0));
        let hook_clone = hook_calls.clone();

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_with_hooks(
            move || flaky_effect(counter_clone.clone(), 3, 99, "error".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
            move |_: &RetryEvent<'_, String>| {
                hook_clone.fetch_add(1, Ordering::SeqCst);
            },
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 99);
        assert_eq!(result.attempts, 4);
        // Hook called 3 times for the 3 failures
        assert_eq!(hook_calls.load(Ordering::SeqCst), 3);
    }

    // ==========================================================================
    // Tests for with_timeout() function
    // ==========================================================================

    #[tokio::test]
    async fn test_with_timeout_completes_before_timeout() {
        // Test: Operation completes successfully before timeout
        let effect = with_timeout(pure::<_, String, ()>(42), Duration::from_secs(1));

        let result = effect.execute(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_with_timeout_operation_times_out() {
        // Test: Operation times out correctly
        let effect = with_timeout(
            from_async(|_: &()| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok::<_, String>(42)
            }),
            Duration::from_millis(10),
        );

        let result = effect.execute(&()).await;
        match result {
            Err(TimeoutError::Timeout { duration }) => {
                assert_eq!(duration, Duration::from_millis(10));
            }
            _ => panic!("Expected TimeoutError::Timeout, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_with_timeout_inner_error_propagated() {
        // Test: Inner error is propagated with TimeoutError::Inner variant
        let effect = with_timeout(
            fail::<i32, _, ()>("inner error".to_string()),
            Duration::from_secs(1),
        );

        let result = effect.execute(&()).await;
        match result {
            Err(TimeoutError::Inner(e)) => {
                assert_eq!(e, "inner error");
            }
            _ => panic!("Expected TimeoutError::Inner, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_with_timeout_zero_duration() {
        // Test: Zero duration timeout (edge case)
        let effect = with_timeout(
            from_async(|_: &()| async {
                // Even a minimal operation might timeout with zero duration
                Ok::<_, String>(42)
            }),
            Duration::ZERO,
        );

        let result = effect.execute(&()).await;
        // With zero duration, the result depends on scheduling - could be either
        // success or timeout. Just verify it returns a valid result.
        assert!(result.is_ok() || matches!(result, Err(TimeoutError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_with_timeout_async_operation_success() {
        // Test: Async operation that takes some time but completes before timeout
        let effect = with_timeout(
            from_async(|_: &()| async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                Ok::<_, String>(100)
            }),
            Duration::from_millis(100),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Ok(100));
    }

    #[tokio::test]
    async fn test_with_timeout_async_operation_failure() {
        // Test: Async operation that fails (not times out)
        let effect = with_timeout(
            from_async(|_: &()| async {
                tokio::time::sleep(Duration::from_millis(5)).await;
                Err::<i32, _>("async error".to_string())
            }),
            Duration::from_millis(100),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Err(TimeoutError::Inner("async error".to_string())));
    }

    #[tokio::test]
    async fn test_with_timeout_preserves_value_type() {
        // Test: Timeout wrapper preserves the value type correctly
        #[derive(Debug, PartialEq)]
        struct CustomValue {
            x: i32,
            y: String,
        }

        let effect = with_timeout(
            pure::<_, String, ()>(CustomValue {
                x: 42,
                y: "test".to_string(),
            }),
            Duration::from_secs(1),
        );

        let result = effect.execute(&()).await;
        assert_eq!(
            result,
            Ok(CustomValue {
                x: 42,
                y: "test".to_string()
            })
        );
    }

    #[tokio::test]
    async fn test_with_timeout_preserves_error_type() {
        // Test: Timeout wrapper preserves the error type correctly
        #[derive(Debug, PartialEq)]
        struct CustomError {
            code: u32,
            message: String,
        }

        let effect = with_timeout(
            fail::<i32, _, ()>(CustomError {
                code: 500,
                message: "internal error".to_string(),
            }),
            Duration::from_secs(1),
        );

        let result = effect.execute(&()).await;
        assert_eq!(
            result,
            Err(TimeoutError::Inner(CustomError {
                code: 500,
                message: "internal error".to_string()
            }))
        );
    }

    // ==========================================================================
    // Additional edge case and integration tests
    // ==========================================================================

    #[tokio::test]
    async fn test_retry_with_environment() {
        // Test: Retry works correctly with a non-unit environment
        #[derive(Clone)]
        struct Config {
            multiplier: i32,
        }

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry(
            move || {
                let counter = counter_clone.clone();
                from_fn(move |env: &Config| {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err::<i32, _>("not ready".to_string())
                    } else {
                        Ok(env.multiplier * 10)
                    }
                })
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
        );

        let result = effect.execute(&Config { multiplier: 5 }).await.unwrap();
        assert_eq!(result.final_error, 50);
    }

    #[tokio::test]
    async fn test_retry_if_with_environment() {
        // Test: retry_if works correctly with a custom environment
        #[derive(Clone)]
        struct AppConfig {
            threshold: u32,
        }

        #[derive(Debug, PartialEq, Clone)]
        #[allow(dead_code)]
        enum AppError {
            BelowThreshold(u32),
            AboveThreshold,
        }

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let effect = retry_if(
            move || {
                let counter = counter_clone.clone();
                from_fn(move |env: &AppConfig| {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < env.threshold {
                        Err::<i32, _>(AppError::BelowThreshold(count))
                    } else {
                        Ok((count + 1) as i32)
                    }
                })
            },
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(10),
            |err| matches!(err, AppError::BelowThreshold(_)),
        );

        let result = effect.execute(&AppConfig { threshold: 3 }).await;
        assert_eq!(result, Ok(4));
    }
}
