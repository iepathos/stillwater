//! Integration tests for retry functionality.

use super::*;
use crate::Effect;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_retry_succeeds_on_third_attempt() {
    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 2 {
                            Err("transient failure")
                        } else {
                            Ok("success")
                        }
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
    );

    let result = effect.run(&()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().final_error, "success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_exhausted_returns_final_error() {
    let effect = Effect::retry(
        || Effect::<(), _, ()>::fail("always fails"),
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
    );

    let result = effect.run(&()).await;

    assert!(result.is_err());
    let exhausted = result.unwrap_err();
    assert_eq!(exhausted.attempts, 4); // 1 initial + 3 retries
    assert_eq!(exhausted.final_error, "always fails");
}

#[tokio::test]
async fn test_retry_if_skips_non_retryable_errors() {
    #[derive(Debug, PartialEq, Clone)]
    #[allow(dead_code)]
    enum TestError {
        Transient,
        Permanent,
    }

    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        Err::<(), _>(TestError::Permanent)
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
        |err| matches!(err, TestError::Transient),
    );

    let result = effect.run(&()).await;

    assert_eq!(result, Err(TestError::Permanent));
    assert_eq!(attempts.load(Ordering::SeqCst), 1); // No retries for permanent error
}

#[tokio::test]
async fn test_retry_if_retries_transient_errors() {
    #[derive(Debug, PartialEq, Clone)]
    enum TestError {
        Transient,
        #[allow(dead_code)]
        Permanent,
    }

    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 2 {
                            Err::<&str, _>(TestError::Transient)
                        } else {
                            Ok("success")
                        }
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
        |err| matches!(err, TestError::Transient),
    );

    let result = effect.run(&()).await;

    assert_eq!(result, Ok("success"));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_with_hooks_calls_hook() {
    let attempts = Arc::new(AtomicU32::new(0));
    let hook_calls = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry_with_hooks(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 2 {
                            Err("transient")
                        } else {
                            Ok("success")
                        }
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
        {
            let hook_calls = hook_calls.clone();
            move |_event: &RetryEvent<'_, &str>| {
                hook_calls.fetch_add(1, Ordering::SeqCst);
            }
        },
    );

    let result = effect.run(&()).await;

    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert_eq!(hook_calls.load(Ordering::SeqCst), 2); // Called before each retry
}

#[tokio::test]
async fn test_timeout_triggers_correctly() {
    let effect = Effect::from_async(|_: &()| async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok::<_, String>(42)
    })
    .with_timeout(Duration::from_millis(10));

    let result = effect.run(&()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().is_timeout());
}

#[tokio::test]
async fn test_timeout_passes_through_success() {
    let effect = Effect::from_async(|_: &()| async { Ok::<_, String>(42) })
        .with_timeout(Duration::from_secs(1));

    let result = effect.run(&()).await;

    assert_eq!(result, Ok(42));
}

#[tokio::test]
async fn test_timeout_passes_through_inner_error() {
    let effect = Effect::from_async(|_: &()| async { Err::<i32, _>("inner error") })
        .with_timeout(Duration::from_secs(1));

    let result = effect.run(&()).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_inner());
    assert_eq!(err.into_inner(), Some("inner error"));
}

#[tokio::test]
async fn test_retry_with_timeout_per_attempt() {
    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 2 {
                            // First two attempts take too long
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                        Ok::<_, String>("success")
                    }
                })
                .with_timeout(Duration::from_millis(10))
                .map_err(|e| format!("{}", e))
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
    );

    let result = effect.run(&()).await;

    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_preserves_success_value() {
    let effect = Effect::retry(
        || Effect::<_, String, ()>::pure(42),
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(3),
    );

    let result = effect.run(&()).await;

    assert!(result.is_ok());
    // The success value is wrapped in Ok, and we get RetryExhausted on error
    // For success, we need to check the value is preserved
    let success = result.unwrap();
    assert_eq!(success.into_value(), 42);
}

#[tokio::test]
async fn test_exponential_backoff_timing() {
    use std::time::Instant;

    let start = Instant::now();
    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 3 {
                            Err("retry")
                        } else {
                            Ok("done")
                        }
                    }
                })
            }
        },
        RetryPolicy::exponential(Duration::from_millis(10)).with_max_retries(5),
    );

    let _ = effect.run(&()).await;
    let elapsed = start.elapsed();

    // With exponential backoff: 10ms + 20ms + 40ms = 70ms minimum
    // Add some tolerance for execution time
    assert!(
        elapsed >= Duration::from_millis(50),
        "Expected at least 50ms, got {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_retry_with_environment() {
    struct Env {
        fail_count: u32,
    }

    let attempts = Arc::new(AtomicU32::new(0));

    let effect = Effect::retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                Effect::from_async(move |env: &Env| {
                    let attempts = attempts.clone();
                    let fail_count = env.fail_count;
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < fail_count {
                            Err("retry")
                        } else {
                            Ok("success")
                        }
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(1)).with_max_retries(5),
    );

    let env = Env { fail_count: 2 };
    let result = effect.run(&env).await;

    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}
