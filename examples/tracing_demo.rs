//! Demonstrates tracing integration with Effects
//!
//! Run with: cargo run --example tracing_demo --features tracing

use stillwater::Effect;

#[tokio::main]
async fn main() {
    // Set up tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("Starting tracing demo");

    // Simulate a workflow with traced effects
    let result = fetch_data()
        .and_then(|data| process_data(data))
        .and_then(|processed| save_result(processed))
        .run(&())
        .await;

    match result {
        Ok(id) => tracing::info!("Workflow completed successfully: {}", id),
        Err(e) => tracing::error!("Workflow failed: {}", e),
    }

    // Demonstrate parallel tracing
    tracing::info!("Running parallel tasks");
    let parallel_result = run_parallel_tasks().await;
    tracing::info!("Parallel tasks result: {:?}", parallel_result);
}

fn fetch_data() -> Effect<String, String, ()> {
    Effect::pure("raw data".to_string()).named("fetch-data")
}

fn process_data(data: String) -> Effect<String, String, ()> {
    Effect::pure(format!("processed: {}", data)).traced()
}

fn save_result(_data: String) -> Effect<i32, String, ()> {
    Effect::pure(42).instrument(tracing::info_span!("save", operation = "database"))
}

async fn run_parallel_tasks() -> Result<Vec<i32>, Vec<String>> {
    let effects = vec![
        Effect::<_, String, ()>::pure(1).named("task-1"),
        Effect::pure(2).named("task-2"),
        Effect::pure(3).named("task-3"),
    ];

    Effect::par_all(effects).run(&()).await
}
