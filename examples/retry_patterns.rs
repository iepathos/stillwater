//! Retry Patterns Example
//!
//! Demonstrates retry and resilience patterns for Effect-based computations.
//! Shows practical patterns including:
//! - Basic retry with different backoff strategies
//! - Conditional retry (retry_if)
//! - Retry with observability hooks
//! - Timeout handling
//! - Combining retry with timeout for robust I/O operations

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use stillwater::effect::prelude::*;
use stillwater::{RetryPolicy, TimeoutError};

// ==================== Basic Retry ====================

/// Example 1: Basic retry with exponential backoff
///
/// Demonstrates retrying an operation that fails transiently.
async fn example_basic_retry() {
    println!("\n=== Example 1: Basic Retry ===");

    // Track the number of attempts
    let attempts = Arc::new(AtomicU32::new(0));

    let effect = retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        println!("  Attempt {}", n + 1);
                        if n < 2 {
                            Err("transient failure")
                        } else {
                            Ok("success!")
                        }
                    }
                })
            }
        },
        RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(5),
    );

    let result = effect.run_standalone().await;

    match result {
        Ok(success) => {
            let attempts = success.attempts;
            let value = success.into_value();
            println!("Success after {} attempts: {}", attempts, value);
        }
        Err(exhausted) => {
            println!(
                "Failed after {} attempts: {}",
                exhausted.attempts, exhausted.final_error
            );
        }
    }
}

// ==================== Different Backoff Strategies ====================

/// Example 2: Comparing different backoff strategies
///
/// Shows how delay increases with different strategies.
async fn example_backoff_strategies() {
    println!("\n=== Example 2: Backoff Strategies ===");

    // Constant delay
    let constant = RetryPolicy::constant(Duration::from_millis(100)).with_max_retries(5);
    println!("Constant delays:");
    for i in 0..5 {
        if let Some(d) = constant.delay_for_attempt(i) {
            println!("  Attempt {}: {:?}", i + 1, d);
        }
    }

    // Linear delay
    let linear = RetryPolicy::linear(Duration::from_millis(100)).with_max_retries(5);
    println!("\nLinear delays:");
    for i in 0..5 {
        if let Some(d) = linear.delay_for_attempt(i) {
            println!("  Attempt {}: {:?}", i + 1, d);
        }
    }

    // Exponential delay
    let exponential = RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(5);
    println!("\nExponential delays:");
    for i in 0..5 {
        if let Some(d) = exponential.delay_for_attempt(i) {
            println!("  Attempt {}: {:?}", i + 1, d);
        }
    }

    // Fibonacci delay
    let fibonacci = RetryPolicy::fibonacci(Duration::from_millis(100)).with_max_retries(5);
    println!("\nFibonacci delays:");
    for i in 0..5 {
        if let Some(d) = fibonacci.delay_for_attempt(i) {
            println!("  Attempt {}: {:?}", i + 1, d);
        }
    }
}

// ==================== Conditional Retry ====================

/// Example 3: Retry only on specific errors
///
/// Demonstrates retry_if to distinguish transient from permanent errors.
async fn example_conditional_retry() {
    println!("\n=== Example 3: Conditional Retry ===");

    #[derive(Debug, Clone, PartialEq)]
    enum AppError {
        Transient(String),
        Permanent(String),
    }

    let attempts = Arc::new(AtomicU32::new(0));

    // This effect returns a Permanent error - should NOT retry
    let effect = retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        println!("  Attempting...");
                        Err::<(), _>(AppError::Permanent("invalid credentials".to_string()))
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(100)).with_max_retries(5),
        |err| matches!(err, AppError::Transient(_)),
    );

    let result = effect.run_standalone().await;
    println!("Permanent error (no retries): {:?}", result.unwrap_err());
    println!("Total attempts: {}", attempts.load(Ordering::SeqCst));

    // Reset attempts
    attempts.store(0, Ordering::SeqCst);

    // This effect returns Transient errors then succeeds
    let effect = retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        println!("  Attempt {}", n + 1);
                        if n < 2 {
                            Err(AppError::Transient("connection timeout".to_string()))
                        } else {
                            Ok("connected!")
                        }
                    }
                })
            }
        },
        RetryPolicy::constant(Duration::from_millis(50)).with_max_retries(5),
        |err| matches!(err, AppError::Transient(_)),
    );

    let result = effect.run_standalone().await;
    println!("\nTransient errors then success: {:?}", result.unwrap());
    println!("Total attempts: {}", attempts.load(Ordering::SeqCst));
}

// ==================== Retry with Observability ====================

/// Example 4: Retry with hooks for logging/metrics
///
/// Demonstrates retry_with_hooks for observability.
async fn example_retry_with_hooks() {
    println!("\n=== Example 4: Retry with Hooks ===");

    let attempts = Arc::new(AtomicU32::new(0));

    let effect = retry_with_hooks(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        if n < 3 {
                            Err(format!("error on attempt {}", n + 1))
                        } else {
                            Ok("finally succeeded!")
                        }
                    }
                })
            }
        },
        RetryPolicy::exponential(Duration::from_millis(50)).with_max_retries(5),
        |event| {
            // This hook is called before each retry
            println!(
                "  [HOOK] Attempt {} failed with: {:?}",
                event.attempt, event.error
            );
            if let Some(delay) = event.next_delay {
                println!("         Waiting {:?} before retry...", delay);
            } else {
                println!("         No more retries!");
            }
            println!("         Total elapsed: {:?}", event.elapsed);
        },
    );

    let result = effect.run_standalone().await;

    match result {
        Ok(success) => {
            let attempts = success.attempts;
            let value = success.into_value();
            println!("\nSuccess after {} attempts: {}", attempts, value);
        }
        Err(exhausted) => {
            println!(
                "\nFailed after {} attempts: {}",
                exhausted.attempts, exhausted.final_error
            );
        }
    }
}

// ==================== Timeout ====================

/// Example 5: Effect with timeout
///
/// Demonstrates with_timeout for operations that may hang.
async fn example_timeout() {
    println!("\n=== Example 5: Timeout ===");

    // Effect that takes too long
    let slow_effect = with_timeout(
        from_async(|_: &()| async {
            println!("  Starting slow operation...");
            tokio::time::sleep(Duration::from_secs(10)).await;
            Ok::<_, String>("done")
        }),
        Duration::from_millis(100),
    );

    println!("Running with 100ms timeout:");
    match slow_effect.run_standalone().await {
        Ok(value) => println!("  Completed: {}", value),
        Err(TimeoutError::Timeout { duration }) => {
            println!("  Timed out after {:?}", duration);
        }
        Err(TimeoutError::Inner(e)) => println!("  Inner error: {}", e),
    }

    // Effect that completes in time
    let fast_effect = with_timeout(
        from_async(|_: &()| async {
            println!("\n  Starting fast operation...");
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<_, String>("done quickly!")
        }),
        Duration::from_millis(100),
    );

    println!("Running with 100ms timeout:");
    match fast_effect.run_standalone().await {
        Ok(value) => println!("  Completed: {}", value),
        Err(TimeoutError::Timeout { duration }) => {
            println!("  Timed out after {:?}", duration);
        }
        Err(TimeoutError::Inner(e)) => println!("  Inner error: {}", e),
    }
}

// ==================== Retry with Per-Attempt Timeout ====================

/// Example 6: Combining retry with timeout
///
/// Demonstrates the recommended pattern for robust I/O operations:
/// - Per-attempt timeout to catch hanging operations
/// - Retry with backoff for transient failures
/// - Overall timeout for the entire retry sequence
async fn example_retry_with_timeout() {
    println!("\n=== Example 6: Retry with Timeout ===");

    let attempts = Arc::new(AtomicU32::new(0));

    // Each attempt has its own timeout, and we retry on timeout
    let effect = retry(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                with_timeout(
                    from_async(move |_: &()| {
                        let attempts = attempts.clone();
                        async move {
                            let n = attempts.fetch_add(1, Ordering::SeqCst);
                            println!("  Attempt {} starting...", n + 1);

                            if n < 2 {
                                // First two attempts are slow (will timeout)
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            }
                            // Third attempt is fast
                            tokio::time::sleep(Duration::from_millis(10)).await;
                            Ok::<_, String>("connected!")
                        }
                    }),
                    Duration::from_millis(100),
                )
                .map_err(|e| format!("{}", e))
            }
        },
        RetryPolicy::constant(Duration::from_millis(50)).with_max_retries(5),
    );

    let result = effect.run_standalone().await;

    match result {
        Ok(success) => {
            let attempts = success.attempts;
            let value = success.into_value();
            println!("\nSuccess after {} attempts: {}", attempts, value);
        }
        Err(exhausted) => {
            println!(
                "\nFailed after {} attempts: {}",
                exhausted.attempts, exhausted.final_error
            );
        }
    }
}

// ==================== Max Delay Cap ====================

/// Example 7: Using max_delay to cap exponential growth
///
/// Demonstrates how max_delay prevents delays from growing too large.
async fn example_max_delay() {
    println!("\n=== Example 7: Max Delay Cap ===");

    let policy = RetryPolicy::exponential(Duration::from_millis(100))
        .with_max_retries(10)
        .with_max_delay(Duration::from_millis(500));

    println!("Exponential backoff with 500ms cap:");
    for i in 0..10 {
        if let Some(d) = policy.delay_for_attempt(i) {
            println!("  Attempt {}: {:?}", i + 1, d);
        }
    }
}

// ==================== Real-World Pattern: HTTP Client ====================

/// Example 8: Simulated HTTP client with retry
///
/// Demonstrates a realistic pattern for API calls.
async fn example_http_pattern() {
    println!("\n=== Example 8: HTTP Client Pattern ===");

    // Simulated HTTP response
    #[derive(Debug, Clone)]
    enum HttpError {
        Timeout,
        ServerError(u16),
        ClientError(u16),
    }

    impl std::fmt::Display for HttpError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HttpError::Timeout => write!(f, "request timed out"),
                HttpError::ServerError(code) => write!(f, "server error: {}", code),
                HttpError::ClientError(code) => write!(f, "client error: {}", code),
            }
        }
    }

    // Only retry on timeouts and server errors, not client errors
    fn is_retryable(err: &HttpError) -> bool {
        matches!(err, HttpError::Timeout | HttpError::ServerError(_))
    }

    let attempts = Arc::new(AtomicU32::new(0));

    // Simulate an API that fails twice with server errors then succeeds
    let effect = retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        let n = attempts.fetch_add(1, Ordering::SeqCst);
                        println!("  HTTP request attempt {}", n + 1);

                        match n {
                            0 => Err(HttpError::ServerError(503)), // Service Unavailable
                            1 => Err(HttpError::Timeout),
                            _ => Ok("{ \"status\": \"ok\" }"),
                        }
                    }
                })
            }
        },
        RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(5)
            .with_max_delay(Duration::from_secs(2)),
        is_retryable,
    );

    let result = effect.run_standalone().await;

    match result {
        Ok(body) => println!("\nResponse: {}", body),
        Err(e) => println!("\nRequest failed: {}", e),
    }

    // Now demonstrate that client errors (4xx) are NOT retried
    println!("\n--- Client Error (should NOT retry) ---");
    let attempts = Arc::new(AtomicU32::new(0));

    let effect = retry_if(
        {
            let attempts = attempts.clone();
            move || {
                let attempts = attempts.clone();
                from_async(move |_: &()| {
                    let attempts = attempts.clone();
                    async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        println!("  HTTP request attempt");
                        // Client error - bad request, should NOT be retried
                        Err::<&str, _>(HttpError::ClientError(400))
                    }
                })
            }
        },
        RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(5)
            .with_max_delay(Duration::from_secs(2)),
        is_retryable,
    );

    let result = effect.run_standalone().await;

    match result {
        Ok(body) => println!("\nResponse: {}", body),
        Err(e) => println!("\nRequest failed (no retries for client error): {}", e),
    }
    println!("Total attempts: {}", attempts.load(Ordering::SeqCst));
}

#[tokio::main]
async fn main() {
    println!("======================================");
    println!("       Retry Patterns Example         ");
    println!("======================================");

    example_basic_retry().await;
    example_backoff_strategies().await;
    example_conditional_retry().await;
    example_retry_with_hooks().await;
    example_timeout().await;
    example_retry_with_timeout().await;
    example_max_delay().await;
    example_http_pattern().await;

    println!("\n======================================");
    println!("           Examples Complete           ");
    println!("======================================");
}
