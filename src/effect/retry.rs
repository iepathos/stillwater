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
    use crate::effect::constructors::{fail, pure};

    #[tokio::test]
    async fn test_retry_success() {
        let effect = retry(
            || pure::<_, String, ()>(42),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
        );

        let result = effect.execute(&()).await.unwrap();
        assert_eq!(result.final_error, 42);
        assert_eq!(result.attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let effect = retry(
            || fail::<i32, _, ()>("error".to_string()),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(2),
        );

        let result = effect.execute(&()).await.unwrap_err();
        assert_eq!(result.final_error, "error".to_string());
        assert_eq!(result.attempts, 3); // 1 initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_if_non_retryable() {
        #[derive(Debug, PartialEq, Clone)]
        enum TestError {
            Transient,
            Permanent,
        }

        let effect = retry_if(
            || fail::<(), _, ()>(TestError::Permanent),
            RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
            |err| matches!(err, TestError::Transient),
        );

        let result = effect.execute(&()).await;
        assert_eq!(result, Err(TestError::Permanent));
    }

    #[tokio::test]
    async fn test_with_timeout_success() {
        let effect = with_timeout(pure::<_, String, ()>(42), Duration::from_secs(1));

        let result = effect.execute(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_with_timeout_timeout() {
        use crate::effect::constructors::from_async;

        let effect = with_timeout(
            from_async(|_: &()| async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok::<_, String>(42)
            }),
            Duration::from_millis(10),
        );

        let result = effect.execute(&()).await;
        assert!(matches!(result, Err(TimeoutError::Timeout { .. })));
    }

    #[tokio::test]
    async fn test_with_timeout_inner_error() {
        let effect = with_timeout(
            fail::<i32, _, ()>("inner error".to_string()),
            Duration::from_secs(1),
        );

        let result = effect.execute(&()).await;
        assert!(matches!(result, Err(TimeoutError::Inner(e)) if e == "inner error"));
    }
}
