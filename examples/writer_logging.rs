//! Writer Effect example demonstrating accumulating logs/audit events alongside computation.
//!
//! The Writer Effect allows you to accumulate values (like logs, metrics, or audit trails)
//! without threading state through every function.
//!
//! Run with: cargo run --example writer_logging

use stillwater::effect::prelude::*;
use stillwater::effect::writer::prelude::*;
use stillwater::monoid::Sum;

// ============================================================================
// Example 1: Basic Logging with Vec<String>
// ============================================================================

/// Simple logging - accumulate log messages alongside computation
fn basic_logging() -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>>
{
    tell_one::<_, String, ()>("Starting computation".to_string())
        .and_then(|_| into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(10)))
        .tap_tell(|n| vec![format!("Got initial value: {}", n)])
        .map(|n| n * 2)
        .tap_tell(|n| vec![format!("After doubling: {}", n)])
        .and_then(|n| tell_one(format!("Final result: {}", n)).map(move |_| n))
}

// ============================================================================
// Example 2: Audit Trail with Custom Events
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum AuditEvent {
    OperationStarted { operation: String },
    StepCompleted { step: String, result: String },
    OperationCompleted { operation: String, success: bool },
}

/// Track audit events for compliance/debugging
fn audit_trail(
    user_id: u32,
) -> impl WriterEffect<Output = String, Error = String, Env = (), Writes = Vec<AuditEvent>> {
    tell_one::<_, String, ()>(AuditEvent::OperationStarted {
        operation: format!("process_user_{}", user_id),
    })
    .and_then(move |_| {
        // Simulate fetching user
        into_writer::<_, _, Vec<AuditEvent>>(pure::<_, String, ()>(format!("User-{}", user_id)))
            .tap_tell(|name| {
                vec![AuditEvent::StepCompleted {
                    step: "fetch_user".to_string(),
                    result: name.clone(),
                }]
            })
    })
    .and_then(|user_name| {
        // Simulate validation
        into_writer::<_, _, Vec<AuditEvent>>(pure::<_, String, ()>(true))
            .tap_tell(|valid| {
                vec![AuditEvent::StepCompleted {
                    step: "validate_user".to_string(),
                    result: format!("valid={}", valid),
                }]
            })
            .map(move |_| user_name)
    })
    .tap_tell(|_| {
        vec![AuditEvent::OperationCompleted {
            operation: "process_user".to_string(),
            success: true,
        }]
    })
}

// ============================================================================
// Example 3: Metrics Collection with Sum Monoid
// ============================================================================

/// Count operations using the Sum monoid
fn count_operations(
) -> impl WriterEffect<Output = Vec<i32>, Error = String, Env = (), Writes = Sum<u32>> {
    // Each operation increments the counter
    tell::<_, String, ()>(Sum(1))
        .map(|_| 10)
        .and_then(|n| tell::<Sum<u32>, String, ()>(Sum(1)).map(move |_| n * 2))
        .and_then(|n| tell::<Sum<u32>, String, ()>(Sum(1)).map(move |_| n + 5))
        .map(|final_val| vec![final_val])
}

// ============================================================================
// Example 4: Processing Pipeline with traverse_writer
// ============================================================================

/// Process multiple items, accumulating logs for each
fn process_items(
    items: Vec<i32>,
) -> impl WriterEffect<Output = Vec<i32>, Error = String, Env = (), Writes = Vec<String>> {
    traverse_writer(items, |item| {
        tell_one::<_, String, ()>(format!("Processing item: {}", item))
            .map(move |_| item * 10)
            .tap_tell(|result| vec![format!("  -> Result: {}", result)])
    })
}

// ============================================================================
// Example 5: Filtering Logs with censor
// ============================================================================

/// Use censor to filter out verbose debug messages
fn filtered_logging(
) -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>> {
    tell_one::<_, String, ()>("DEBUG: entering function".to_string())
        .and_then(|_| tell_one("INFO: processing data".to_string()))
        .and_then(|_| tell_one("DEBUG: intermediate state".to_string()))
        .and_then(|_| tell_one("INFO: completed".to_string()))
        .map(|_| 42)
        // Filter out DEBUG messages
        .censor(|logs| {
            logs.into_iter()
                .filter(|log| !log.starts_with("DEBUG"))
                .collect()
        })
}

// ============================================================================
// Example 6: Inspecting Writes with listen
// ============================================================================

/// Use listen to include accumulated writes in the output
fn inspect_writes(
) -> impl WriterEffect<Output = (i32, Vec<String>), Error = String, Env = (), Writes = Vec<String>>
{
    tell_one::<_, String, ()>("Step 1".to_string())
        .and_then(|_| tell_one("Step 2".to_string()))
        .map(|_| 42)
        .listen() // Output becomes (42, vec!["Step 1", "Step 2"])
}

// ============================================================================
// Example 7: Transforming Writes with pass
// ============================================================================

/// Use pass to let the computation decide how to transform writes
fn transform_writes(
) -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>> {
    tell::<_, String, ()>(vec![
        "log 1".to_string(),
        "log 2".to_string(),
        "log 3".to_string(),
        "log 4".to_string(),
    ])
    .map(|_| {
        // Return (value, transformation function)
        (
            42,
            |logs: Vec<String>| logs.into_iter().take(2).collect(), // Keep only first 2
        )
    })
    .pass()
}

// ============================================================================
// Example 8: Error Handling - writes are preserved up to failure point
// ============================================================================

fn error_handling_example(
) -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>> {
    tell_one::<_, String, ()>("Before error".to_string())
        .and_then(|_| tell_one("About to fail".to_string()))
        .and_then(|_| {
            // This fails
            into_writer::<_, _, Vec<String>>(fail::<i32, String, ()>("Something went wrong".into()))
        })
        .and_then(|n| tell_one("After error - never reached".to_string()).map(move |_| n))
}

// ============================================================================
// Example 9: Recovery with or_else
// ============================================================================

fn recovery_example(
) -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>> {
    tell_one::<_, String, ()>("Attempting operation".to_string())
        .and_then(|_| into_writer::<_, _, Vec<String>>(fail::<i32, String, ()>("Failed".into())))
        .or_else(|err| {
            tell_one::<_, String, ()>(format!("Recovering from: {}", err)).map(|_| 0)
            // Return default value
        })
        .tap_tell(|n| vec![format!("Final value: {}", n)])
}

// ============================================================================
// Example 10: fold_writer for Accumulation
// ============================================================================

fn fold_example() -> impl WriterEffect<Output = i32, Error = String, Env = (), Writes = Vec<String>>
{
    let items = vec![1, 2, 3, 4, 5];
    fold_writer(items, 0, |acc, n| {
        tell_one::<_, String, ()>(format!("Adding {} to {}", n, acc)).map(move |_| acc + n)
    })
}

// ============================================================================
// Example 11: Boxed Writer Effects in Collections
// ============================================================================

async fn boxed_collection_example() {
    use std::convert::Infallible;

    let effects: Vec<BoxedWriterEffect<i32, Infallible, (), Vec<String>>> = vec![
        tell_one::<_, Infallible, ()>("Effect A".to_string())
            .map(|_| 1)
            .boxed_writer(),
        tell_one::<_, Infallible, ()>("Effect B".to_string())
            .map(|_| 2)
            .boxed_writer(),
        tell_one::<_, Infallible, ()>("Effect C".to_string())
            .map(|_| 3)
            .boxed_writer(),
    ];

    println!("11. Boxed writer effects in collection:");
    let mut all_results = Vec::new();
    let mut all_logs = Vec::new();

    for effect in effects {
        let (result, logs) = effect.run_writer(&()).await;
        all_results.push(result.unwrap());
        all_logs.extend(logs);
    }

    println!("    Results: {:?}", all_results);
    println!("    All logs: {:?}\n", all_logs);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_logging() {
        let (result, logs) = basic_logging().run_writer(&()).await;
        assert_eq!(result, Ok(20));
        assert_eq!(logs.len(), 4);
        assert!(logs[0].contains("Starting"));
        assert!(logs[3].contains("Final"));
    }

    #[tokio::test]
    async fn test_audit_trail() {
        let (result, events) = audit_trail(123).run_writer(&()).await;
        assert!(result.is_ok());
        assert_eq!(events.len(), 4); // Started, 2 steps, Completed
        assert!(matches!(
            &events[0],
            AuditEvent::OperationStarted { operation } if operation.contains("123")
        ));
    }

    #[tokio::test]
    async fn test_count_operations() {
        let (result, Sum(count)) = count_operations().run_writer(&()).await;
        assert!(result.is_ok());
        assert_eq!(count, 3); // 3 operations counted
    }

    #[tokio::test]
    async fn test_process_items() {
        let items = vec![1, 2, 3];
        let (result, logs) = process_items(items).run_writer(&()).await;
        assert_eq!(result, Ok(vec![10, 20, 30]));
        assert_eq!(logs.len(), 6); // 2 logs per item
    }

    #[tokio::test]
    async fn test_filtered_logging() {
        let (result, logs) = filtered_logging().run_writer(&()).await;
        assert_eq!(result, Ok(42));
        assert_eq!(logs.len(), 2); // Only INFO messages
        assert!(logs.iter().all(|l| l.starts_with("INFO")));
    }

    #[tokio::test]
    async fn test_inspect_writes() {
        let (result, logs) = inspect_writes().run_writer(&()).await;
        let (value, inner_logs) = result.unwrap();
        assert_eq!(value, 42);
        assert_eq!(inner_logs, logs); // listen includes writes in output
    }

    #[tokio::test]
    async fn test_error_handling() {
        let (result, logs) = error_handling_example().run_writer(&()).await;
        assert!(result.is_err());
        assert_eq!(logs.len(), 2); // Logs before error are preserved
    }

    #[tokio::test]
    async fn test_recovery() {
        let (result, logs) = recovery_example().run_writer(&()).await;
        assert_eq!(result, Ok(0));
        assert!(logs.iter().any(|l| l.contains("Recovering")));
    }

    #[tokio::test]
    async fn test_fold() {
        let (result, logs) = fold_example().run_writer(&()).await;
        assert_eq!(result, Ok(15)); // 1+2+3+4+5
        assert_eq!(logs.len(), 5);
    }
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== Writer Effect Examples ===\n");

    println!("1. Basic logging:");
    let (result, logs) = basic_logging().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Logs: {:?}\n", logs);

    println!("2. Audit trail with custom events:");
    let (result, events) = audit_trail(42).run_writer(&()).await;
    println!("   Result: {:?}", result);
    for event in &events {
        println!("   Event: {:?}", event);
    }
    println!();

    println!("3. Counting operations with Sum monoid:");
    let (result, Sum(count)) = count_operations().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Operation count: {}\n", count);

    println!("4. Processing items with traverse_writer:");
    let (result, logs) = process_items(vec![1, 2, 3]).run_writer(&()).await;
    println!("   Result: {:?}", result);
    for log in &logs {
        println!("   {}", log);
    }
    println!();

    println!("5. Filtered logging with censor:");
    let (result, logs) = filtered_logging().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Filtered logs: {:?}\n", logs);

    println!("6. Inspecting writes with listen:");
    let (result, logs) = inspect_writes().run_writer(&()).await;
    let (value, inner_logs) = result.unwrap();
    println!("   Value: {}", value);
    println!("   Inner logs: {:?}", inner_logs);
    println!("   Outer logs: {:?}\n", logs);

    println!("7. Transforming writes with pass:");
    let (result, logs) = transform_writes().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Logs (first 2 only): {:?}\n", logs);

    println!("8. Error handling - writes preserved up to failure:");
    let (result, logs) = error_handling_example().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Logs before error: {:?}\n", logs);

    println!("9. Recovery with or_else:");
    let (result, logs) = recovery_example().run_writer(&()).await;
    println!("   Result: {:?}", result);
    println!("   Logs: {:?}\n", logs);

    println!("10. Fold with logging:");
    let (result, logs) = fold_example().run_writer(&()).await;
    println!("    Result: {:?}", result);
    for log in &logs {
        println!("    {}", log);
    }
    println!();

    boxed_collection_example().await;

    println!("=== Benefits of Writer Effect ===");
    println!("- Accumulate logs/metrics without threading state");
    println!("- Type-safe accumulation with Monoid constraint");
    println!("- Logs preserved even on error (up to failure point)");
    println!("- Filter/transform logs with censor");
    println!("- Inspect accumulated writes with listen");
    println!("- Works with any Monoid: Vec, Sum, Product, etc.");
    println!("- Zero-cost by default, boxed when needed");
}
