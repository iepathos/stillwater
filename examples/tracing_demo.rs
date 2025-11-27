//! Demonstrates tracing + context integration for comprehensive debugging
//!
//! Run with: cargo run --example tracing_demo --features "tracing async"
//!
//! This example shows two complementary debugging tools:
//! - **Tracing**: WHERE things happen (file/line, timing, observability)
//! - **Context**: WHAT was being attempted (error narrative, semantic meaning)
//!
//! Together they give you the full debugging story.

use stillwater::prelude::*;
use stillwater::ContextError;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    // Set up tracing subscriber that shows span lifecycle events
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    tracing::info!("=== Successful workflow ===");
    run_successful_workflow().await;

    tracing::info!("\n=== Failing workflow (tracing + context) ===");
    run_failing_workflow().await;
}

async fn run_successful_workflow() {
    // Tracing only - shows timing and file/line
    let result = fetch_user(1)
        .traced()
        .and_then(|user| validate_user(user).traced())
        .and_then(|user| save_user(user))
        .run(&())
        .await;

    match result {
        Ok(id) => tracing::info!("Success: saved user with id {}", id),
        Err(e) => tracing::error!("Failed: {}", e),
    }
}

async fn run_failing_workflow() {
    // Tracing shows WHERE (file:line, timing)
    // Context shows WHAT (semantic operation being attempted)
    //
    // Note: .context() wraps errors in ContextError, so we apply it
    // at workflow boundaries to build the error narrative
    let result = fetch_user(1)
        .traced()
        .context("loading user profile") // WHAT: semantic meaning
        .and_then(|user| validate_user(user).traced().map_err(ContextError::new))
        .context("validating user permissions")
        .and_then(|_| {
            fail_at_database()
                .instrument(tracing::error_span!("database-write")) // WHERE
                .map_err(ContextError::new)
        })
        .context("persisting workflow state") // This is where it fails
        .and_then(|id| cleanup(id).traced().map_err(ContextError::new))
        .context("finalizing transaction")
        .run(&())
        .await;

    match result {
        Ok(id) => tracing::info!("Success: {}", id),
        Err(e) => {
            // The error now tells a complete story:
            // - WHAT was being attempted (context trail)
            // - Plus tracing output shows WHERE and timing
            tracing::error!("Workflow failed!\n{}", e);
        }
    }
}

// Simple effect builders - tracing/context applied at call site
fn fetch_user(id: i32) -> Effect<String, String, ()> {
    Effect::from_fn(move |_| {
        tracing::debug!("fetching user {}", id);
        Ok(format!("user-{}", id))
    })
}

fn validate_user(user: String) -> Effect<String, String, ()> {
    Effect::from_fn(move |_| {
        tracing::debug!("validating {}", user);
        Ok(user)
    })
}

fn save_user(user: String) -> Effect<i32, String, ()> {
    Effect::from_fn(move |_| {
        tracing::debug!("saving {}", user);
        Ok(42)
    })
    .named("save-user") // WHERE: named span
}

fn fail_at_database() -> Effect<i32, String, ()> {
    Effect::from_fn(|_| {
        tracing::error!("database connection refused!");
        Err("connection refused".to_string())
    })
}

fn cleanup(id: i32) -> Effect<i32, String, ()> {
    Effect::from_fn(move |_| {
        tracing::debug!("cleanup for {}", id);
        Ok(id)
    })
}
