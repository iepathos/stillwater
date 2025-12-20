//! Sink Effect example demonstrating streaming output patterns.
//!
//! This example shows how to use SinkEffect for:
//! - Real-time streaming to stdout
//! - Writing to files
//! - Testing with run_collecting
//! - High-volume processing with constant memory
//!
//! Run with: cargo run --example sink_streaming

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use stillwater::effect::prelude::*;
use stillwater::effect::sink::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// Example 1: Stream to stdout in real-time.
async fn stream_to_stdout() {
    println!("=== Streaming to stdout ===");

    let effect = emit::<_, String, ()>("Step 1: Initialize".into())
        .and_then(|_| emit("Step 2: Process".into()))
        .and_then(|_| emit("Step 3: Complete".into()))
        .map(|_| 42);

    let result = effect
        .run_with_sink(&(), |log| async move {
            println!("  {}", log);
        })
        .await;

    println!("Result: {:?}\n", result);
}

/// Example 2: Stream to a file.
async fn stream_to_file() {
    println!("=== Streaming to file ===");

    let file = Arc::new(Mutex::new(
        tokio::fs::File::create("/tmp/sink_demo.log").await.unwrap(),
    ));

    let items = vec![1, 2, 3, 4, 5];
    let effect = traverse_sink(items, |n| {
        emit::<_, String, ()>(format!("Processing item: {}", n)).map(move |_| n * 10)
    });

    let result = effect
        .run_with_sink(&(), move |log| {
            let file = file.clone();
            async move {
                let mut f = file.lock().await;
                f.write_all(format!("{}\n", log).as_bytes()).await.ok();
            }
        })
        .await;

    println!("Results: {:?}", result);
    println!("Logs written to /tmp/sink_demo.log\n");
}

/// Example 3: Testing pattern with run_collecting.
async fn testing_example() {
    println!("=== Testing with run_collecting ===");

    let effect = emit::<_, String, ()>("audit: user login".into())
        .and_then(|_| emit("audit: access granted".into()))
        .and_then(|_| emit("audit: resource fetched".into()))
        .map(|_| "success");

    let (result, collected) = effect.run_collecting(&()).await;

    println!("Result: {:?}", result);
    println!("Collected {} audit events:", collected.len());
    for event in &collected {
        println!("  - {}", event);
    }
    println!();
}

/// Example 4: High-volume processing with constant memory.
async fn high_volume_example() {
    println!("=== High-volume streaming (1000 items) ===");

    let items: Vec<i32> = (1..=1000).collect();
    let count = Arc::new(AtomicUsize::new(0));
    let count_clone = count.clone();

    let effect = traverse_sink(items, |n| {
        emit::<_, String, ()>(format!("Item {}", n)).map(move |_| n)
    });

    let _result = effect
        .run_with_sink(&(), move |_log| {
            let c = count_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    println!(
        "Streamed {} log entries with constant memory",
        count.load(Ordering::SeqCst)
    );
    println!();
}

/// Example 5: Error handling preserves prior emissions.
async fn error_handling_example() {
    println!("=== Error handling ===");

    let effect = emit::<_, String, String>("step 1: starting".into())
        .and_then(|_| emit("step 2: processing".into()))
        .and_then(|_| into_sink(fail::<i32, _, ()>("error: something went wrong".into())))
        .and_then(|n| emit("step 3: never reached".into()).map(move |_| n));

    let (result, collected) = effect.run_collecting(&()).await;

    println!("Result: {:?}", result);
    println!("Logs before failure:");
    for log in &collected {
        println!("  - {}", log);
    }
    println!();
}

/// Example 6: Using boxed sink effects for heterogeneous collections.
async fn boxed_example() {
    println!("=== Boxed sink effects ===");

    fn make_effect(flag: bool) -> BoxedSinkEffect<i32, String, (), String> {
        if flag {
            emit("operation enabled".to_string())
                .map(|_| 1)
                .boxed_sink()
        } else {
            into_sink::<_, _, String>(pure::<_, String, ()>(0)).boxed_sink()
        }
    }

    let effects = vec![make_effect(true), make_effect(false), make_effect(true)];

    let mut all_results = Vec::new();
    let mut all_logs = Vec::new();

    for effect in effects {
        let (result, logs) = effect.run_collecting(&()).await;
        all_results.push(result.unwrap());
        all_logs.extend(logs);
    }

    println!("Results: {:?}", all_results);
    println!("All logs: {:?}", all_logs);
    println!();
}

/// Example 7: Combining with Reader for environment-aware streaming.
async fn with_reader_example() {
    println!("=== With Reader pattern ===");

    #[derive(Clone)]
    struct AppEnv {
        prefix: String,
    }

    let effect = into_sink::<_, _, String>(asks::<_, String, AppEnv, _>(|env: &AppEnv| {
        env.prefix.clone()
    }))
    .and_then(|prefix| {
        emit(format!("{}: step 1", prefix))
            .and_then(move |_| emit(format!("{}: step 2", prefix.clone())))
            .map(|_| 42)
    })
    .tap_emit(|result| format!("Final result: {}", result));

    let env = AppEnv {
        prefix: "[APP]".to_string(),
    };
    let (result, logs) = effect.run_collecting(&env).await;

    println!("Result: {:?}", result);
    for log in &logs {
        println!("  {}", log);
    }
}

#[tokio::main]
async fn main() {
    stream_to_stdout().await;
    stream_to_file().await;
    testing_example().await;
    high_volume_example().await;
    error_handling_example().await;
    boxed_example().await;
    with_reader_example().await;

    println!("\n=== Benefits of Sink Effect ===");
    println!("- O(1) memory: items streamed immediately, not accumulated");
    println!("- Real-time output: logs appear as they happen");
    println!("- Flexible sinks: stdout, files, channels, databases");
    println!("- Testable: run_collecting bridges to Writer semantics");
    println!("- Async-ready: sinks can perform I/O without blocking");
}
