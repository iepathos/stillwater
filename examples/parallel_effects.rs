//! Parallel Effect Execution Examples
//!
//! This example demonstrates Stillwater's parallel effect execution capabilities,
//! showing how to run independent effects concurrently while preserving error
//! handling and environment access patterns.
//!
//! Run with: cargo run --example parallel_effects

use std::fmt;
use std::time::{Duration, Instant};
use stillwater::prelude::*;

// Mock types for demonstration
#[derive(Debug, Clone)]
struct User {
    id: i32,
    name: String,
}

#[derive(Debug)]
struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
struct AppEnv;

// Example 1: Basic par_all - Collect All Results
//
// Use par_all when you need all results and want to see all errors
// if multiple operations fail.
async fn example_par_all() {
    println!("\n=== Example 1: par_all - Collect All Results ===\n");

    let env = AppEnv;

    // Simulate fetching multiple users in parallel
    let effects = vec![
        fetch_user(1, 50).boxed(),
        fetch_user(2, 50).boxed(),
        fetch_user(3, 50).boxed(),
    ];

    let start = Instant::now();
    match par_all(effects, &env).await {
        Ok(users) => {
            println!("✓ Loaded {} users in {:?}", users.len(), start.elapsed());
            for user in users {
                println!("  - User {}: {}", user.id, user.name);
            }
        }
        Err(errors) => {
            println!("✗ Failed with {} errors:", errors.len());
            for error in errors {
                println!("  - {:?}", error);
            }
        }
    }

    println!("\nNote: All three 50ms tasks completed in ~50ms (parallel), not ~150ms (sequential)");
}

// Example 2: par_all with Mixed Success/Failure
//
// par_all collects ALL errors, not just the first one.
async fn example_par_all_with_errors() {
    println!("\n=== Example 2: par_all - Error Accumulation ===\n");

    let env = AppEnv;

    let effects = vec![
        pure(User {
            id: 1,
            name: "Alice".into(),
        })
        .boxed(),
        fail(AppError("Database timeout".into())).boxed(),
        fail(AppError("Network error".into())).boxed(),
    ];

    match par_all(effects, &env).await {
        Ok(_) => println!("All succeeded"),
        Err(errors) => {
            println!("✓ Collected {} errors:", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("  {}. {:?}", i + 1, error);
            }
        }
    }

    println!(
        "\nNote: par_all accumulates ALL errors, useful for showing users all validation failures"
    );
}

// Example 3: par_try_all - Fail Fast
//
// Use par_try_all when one failure means the entire operation should stop.
async fn example_par_try_all() {
    println!("\n=== Example 3: par_try_all - Fail Fast ===\n");

    let env = AppEnv;

    println!("Checking system health (all must succeed)...");

    let effects = vec![
        check_database().boxed(),
        check_cache().boxed(),
        check_queue().boxed(),
    ];

    let start = Instant::now();
    match par_try_all(effects, &env).await {
        Ok(statuses) => {
            println!(
                "✓ All {} services healthy in {:?}",
                statuses.len(),
                start.elapsed()
            );
        }
        Err(error) => {
            println!("✗ Health check failed in {:?}", start.elapsed());
            println!("  First error: {:?}", error);
            println!("\nNote: Stopped on first error, didn't wait for remaining checks");
        }
    }
}

// Example 4: race - First to Complete
//
// Use race when you have multiple equivalent sources and want the fastest response.
async fn example_race() {
    println!("\n=== Example 4: race - First Success Wins ===\n");

    let env = AppEnv;

    println!("Fetching data from multiple sources (using fastest)...");

    let effects = vec![
        fetch_from_source("cache", 30).boxed(),
        fetch_from_source("primary_db", 80).boxed(),
        fetch_from_source("backup_db", 120).boxed(),
    ];

    let start = Instant::now();
    match race(effects, &env).await {
        Ok(data) => {
            println!("✓ Got data from {} in {:?}", data, start.elapsed());
            println!("\nNote: Returned as soon as cache responded (~30ms)");
        }
        Err(error) => {
            println!("✗ First source failed: {:?}", error);
        }
    }
}

// Example 5: race with Timeout Pattern
//
// Use race to implement timeouts for slow operations.
async fn example_race_timeout() {
    println!("\n=== Example 5: race - Timeout Pattern ===\n");

    let env = AppEnv;

    println!("Fetching with 100ms timeout...");

    fn slow_operation(ms: u64) -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
        from_async(move |_env| async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            Ok(format!("Completed after {}ms", ms))
        })
    }

    let timeout_effect = from_async(|_env| async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Err(AppError("Operation timed out".into()))
    });

    // Try a slow operation with timeout
    let start = Instant::now();
    match race(
        vec![slow_operation(200).boxed(), timeout_effect.boxed()],
        &env,
    )
    .await
    {
        Ok(result) => println!("✓ {}", result),
        Err(error) => {
            println!("✗ Timed out after {:?}", start.elapsed());
            println!("  Error: {:?}", error);
        }
    }
}

// Example 6: par_all_limit - Bounded Concurrency
//
// Use par_all_limit to control resource usage and avoid overwhelming services.
async fn example_par_all_limit() {
    println!("\n=== Example 6: par_all_limit - Bounded Concurrency ===\n");

    let env = AppEnv;

    let user_ids: Vec<i32> = (1..=10).collect();
    println!(
        "Processing {} users with concurrency limit of 3...",
        user_ids.len()
    );

    let effects: Vec<_> = user_ids
        .into_iter()
        .map(|id| fetch_user(id, 30).boxed())
        .collect();

    let start = Instant::now();
    match par_all_limit(effects, 3, &env).await {
        Ok(users) => {
            println!("✓ Processed {} users in {:?}", users.len(), start.elapsed());
            println!("\nNote: Only 3 users processed at once, preventing resource exhaustion");
        }
        Err(errors) => {
            println!("✗ Failed with {} errors", errors.len());
        }
    }
}

// Example 7: Batch User Loading (Scatter-Gather Pattern)
//
// Load multiple users in parallel and combine results.
async fn example_batch_user_loading() {
    println!("\n=== Example 7: Batch User Loading (Scatter-Gather) ===\n");

    let env = AppEnv;

    println!("Loading multiple users in parallel...");

    let user_ids = [1, 2, 3, 4, 5];

    let start = Instant::now();

    // Load all users in parallel using par_all
    let user_effects: Vec<_> = user_ids
        .iter()
        .map(|&id| fetch_user(id, 40).boxed())
        .collect();

    match par_all(user_effects, &env).await {
        Ok(users) => {
            println!("✓ Loaded {} users in {:?}", users.len(), start.elapsed());
            for user in &users {
                println!("  - {}: {}", user.id, user.name);
            }
            println!(
                "\nNote: All {} users loaded in parallel (~40ms), not sequentially (~200ms)",
                users.len()
            );
        }
        Err(errors) => {
            println!("✗ Batch loading failed with {} errors", errors.len());
        }
    }
}

// Example 8: Graceful Degradation
//
// Use par_all with optional data to degrade gracefully on partial failures.
async fn example_graceful_degradation() {
    println!("\n=== Example 8: Graceful Degradation ===\n");

    let env = AppEnv;

    println!("Loading page with optional features...");

    // Core data (must succeed)
    let core_data = fetch_user(1, 20).run(&env).await;

    if let Ok(user) = core_data {
        // Optional features (failures are okay)
        let optional_effects = vec![
            fetch_recommendations(user.id, 30).map(Some).boxed(),
            fetch_recent_activity(user.id, 30).map(Some).boxed(),
            fail(AppError("Analytics unavailable".into())).boxed(), // This one fails
        ];

        match par_all(optional_effects, &env).await {
            Ok(_features) => {
                println!("✓ Core data loaded, all features available");
            }
            Err(errors) => {
                println!(
                    "✓ Core data loaded, {} optional features failed",
                    errors.len()
                );
                println!("  Failed features: {:?}", errors);
                println!("\nNote: Page still works with reduced functionality");
            }
        }
    }
}

// Helper functions for examples

fn fetch_user(
    id: i32,
    delay_ms: u64,
) -> impl Effect<Output = User, Error = AppError, Env = AppEnv> {
    from_async(move |_env| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(User {
            id,
            name: format!("User_{}", id),
        })
    })
}

fn check_database() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    from_async(|_env| async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok("Database OK".to_string())
    })
}

fn check_cache() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    from_async(|_env| async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        Ok("Cache OK".to_string())
    })
}

fn check_queue() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    from_async(|_env| async {
        tokio::time::sleep(Duration::from_millis(40)).await;
        Ok("Queue OK".to_string())
    })
}

fn fetch_from_source(
    source: &'static str,
    delay_ms: u64,
) -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    from_async(move |_env| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(source.to_string())
    })
}

fn fetch_recommendations(
    user_id: i32,
    delay_ms: u64,
) -> impl Effect<Output = Vec<String>, Error = AppError, Env = AppEnv> {
    from_async(move |_env| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(vec![
            format!("Recommendation 1 for user {}", user_id),
            format!("Recommendation 2 for user {}", user_id),
        ])
    })
}

fn fetch_recent_activity(
    user_id: i32,
    delay_ms: u64,
) -> impl Effect<Output = Vec<String>, Error = AppError, Env = AppEnv> {
    from_async(move |_env| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(vec![
            format!("Activity 1 for user {}", user_id),
            format!("Activity 2 for user {}", user_id),
        ])
    })
}

#[tokio::main]
async fn main() {
    println!("==============================================");
    println!("   Stillwater Parallel Effects Examples");
    println!("==============================================");

    println!("\nDemonstrates parallel execution of independent effects:");
    println!("- par_all: collect all results and errors");
    println!("- par_try_all: fail fast on first error");
    println!("- race: first success wins");
    println!("- par_all_limit: bounded concurrency");

    example_par_all().await;
    example_par_all_with_errors().await;
    example_par_try_all().await;
    example_race().await;
    example_race_timeout().await;
    example_par_all_limit().await;
    example_batch_user_loading().await;
    example_graceful_degradation().await;

    println!("\n==============================================");
    println!("         All examples completed!");
    println!("==============================================");
}
