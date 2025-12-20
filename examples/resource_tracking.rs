//! Compile-Time Resource Tracking Example
//!
//! Demonstrates compile-time resource tracking for type-safe resource management.
//! Unlike the runtime bracket pattern (see `resource_scopes.rs`), this approach
//! uses the type system to track resource acquisition and release at compile time.
//!
//! Key concepts:
//! - Resource markers (FileRes, DbRes, TxRes, etc.) for type-level tracking
//! - ResourceEffect trait with Acquires/Releases associated types
//! - Extension methods: `.acquires()`, `.releases()`, `.neutral()`
//! - Builder pattern: `Bracket::<R>::new()` for ergonomic resource brackets
//! - Zero runtime overhead - all tracking is compile-time only

use stillwater::effect::resource::{
    assert_resource_neutral, bracket, resource_bracket, Bracket, DbRes, Empty, FileRes, Has,
    ResourceEffect, ResourceEffectExt, ResourceKind, TrackedExt, TxRes,
};
use stillwater::{pure, Effect};

// =============================================================================
// Example 1: Basic Resource Annotation
// =============================================================================

/// Demonstrates marking effects with resource acquisition.
///
/// The `.acquires::<FileRes>()` method wraps the effect to indicate
/// it acquires a file resource at the type level.
fn open_file(
    path: &str,
) -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Has<FileRes>,
    Releases = Empty,
> {
    // In a real application, this would actually open a file
    println!("  Opening file: {}", path);
    pure::<_, String, ()>(format!("FileHandle({})", path)).acquires::<FileRes>()
}

/// Demonstrates marking effects with resource release.
///
/// The `.releases::<FileRes>()` method indicates the effect
/// releases a file resource.
fn close_file(
    handle: String,
) -> impl ResourceEffect<Output = (), Error = String, Env = (), Acquires = Empty, Releases = Has<FileRes>>
{
    println!("  Closing file: {}", handle);
    pure::<_, String, ()>(()).releases::<FileRes>()
}

/// A resource-neutral operation that neither acquires nor releases.
fn read_contents(
    handle: &str,
) -> impl ResourceEffect<Output = String, Error = String, Env = (), Acquires = Empty, Releases = Empty>
{
    println!("  Reading contents from: {}", handle);
    // This operation is neutral with respect to resources
    pure::<_, String, ()>(format!("Contents of {}", handle)).neutral()
}

async fn example_basic_annotation() {
    println!("\n=== Example 1: Basic Resource Annotation ===");

    // Create an effect that acquires a file resource
    let acquire = open_file("data.txt");

    // Run it (type system knows this acquires FileRes)
    let handle = acquire.run(&()).await.unwrap();
    println!("  Got handle: {}", handle);

    // Read contents (neutral operation)
    let contents = read_contents(&handle).run(&()).await.unwrap();
    println!("  Contents: {}", contents);

    // Close the file (releases FileRes)
    close_file(handle).run(&()).await.unwrap();
    println!("  File closed successfully");
}

// =============================================================================
// Example 2: Resource Bracket Pattern
// =============================================================================

/// Demonstrates the resource_bracket for guaranteed resource safety.
///
/// The bracket ensures:
/// - Acquire runs first
/// - Use runs with the acquired resource
/// - Release ALWAYS runs (even on error)
/// - The bracket as a whole is resource-neutral
async fn example_resource_bracket() {
    println!("\n=== Example 2: Resource Bracket Pattern ===");

    // Option A: Turbofish syntax (explicit but verbose)
    println!("\n  Using turbofish syntax:");
    let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
        pure::<_, String, ()>("file_handle".to_string()),
        |handle: String| {
            println!("    Releasing: {}", handle);
            async { Ok(()) }
        },
        |handle: &String| {
            println!("    Using handle: {}", handle);
            pure::<_, String, ()>(format!("Processed: {}", handle))
        },
    )
    .run(&())
    .await;
    println!("    Result: {:?}", result);

    // Option B: Builder pattern (ergonomic, single type parameter)
    println!("\n  Using builder pattern:");
    let result = Bracket::<FileRes>::new()
        .acquire(pure::<_, String, ()>("file_handle".to_string()))
        .release(|handle: String| {
            println!("    Releasing: {}", handle);
            async { Ok(()) }
        })
        .use_fn(|handle: &String| {
            println!("    Using handle: {}", handle);
            pure::<_, String, ()>(format!("Processed: {}", handle))
        })
        .run(&())
        .await;
    println!("    Result: {:?}", result);

    // Option C: bracket() convenience function
    println!("\n  Using bracket() function:");
    let result = bracket::<FileRes>()
        .acquire(pure::<_, String, ()>("file_handle".to_string()))
        .release(|handle: String| {
            println!("    Releasing: {}", handle);
            async { Ok(()) }
        })
        .use_fn(|handle: &String| {
            println!("    Using handle: {}", handle);
            pure::<_, String, ()>(format!("Processed: {}", handle))
        })
        .run(&())
        .await;
    println!("    Result: {:?}", result);
}

// =============================================================================
// Example 3: Transaction Protocol Enforcement
// =============================================================================

/// Begin a database transaction.
/// Type signature enforces that TxRes is acquired.
fn begin_transaction() -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Has<TxRes>,
    Releases = Empty,
> {
    println!("  BEGIN TRANSACTION");
    pure::<_, String, ()>("tx_12345".to_string()).acquires::<TxRes>()
}

/// Commit the transaction.
/// Type signature enforces that TxRes is released.
fn commit_transaction(
    _tx: String,
) -> impl ResourceEffect<Output = (), Error = String, Env = (), Acquires = Empty, Releases = Has<TxRes>>
{
    println!("  COMMIT");
    pure::<_, String, ()>(()).releases::<TxRes>()
}

/// Rollback the transaction.
/// Type signature also releases TxRes.
#[allow(dead_code)]
fn rollback_transaction(
    _tx: String,
) -> impl ResourceEffect<Output = (), Error = String, Env = (), Acquires = Empty, Releases = Has<TxRes>>
{
    println!("  ROLLBACK");
    pure::<_, String, ()>(()).releases::<TxRes>()
}

/// Execute a query within a transaction.
/// This is resource-neutral (doesn't acquire or release).
fn execute_query(
    tx: &str,
    query: &str,
) -> impl ResourceEffect<
    Output = Vec<String>,
    Error = String,
    Env = (),
    Acquires = Empty,
    Releases = Empty,
> {
    println!("  QUERY [{}]: {}", tx, query);
    pure::<_, String, ()>(vec!["row1".to_string(), "row2".to_string()]).neutral()
}

async fn example_transaction_protocol() {
    println!("\n=== Example 3: Transaction Protocol ===");

    // The resource_bracket ensures transactions are always closed
    let result = resource_bracket::<TxRes, _, _, _, _, _, _, _, _, _>(
        begin_transaction(),
        |tx: String| async move {
            // In real code, you might choose commit or rollback based on result
            commit_transaction(tx).run(&()).await
        },
        |tx: &String| {
            // Multiple queries within the transaction
            println!("  Executing queries...");
            let _rows1 = execute_query(tx, "SELECT * FROM users");
            let _rows2 = execute_query(tx, "UPDATE users SET active = true");
            pure::<_, String, ()>("Transaction completed".to_string())
        },
    )
    .run(&())
    .await;

    println!("  Result: {:?}", result);
}

// =============================================================================
// Example 4: Multiple Resource Types
// =============================================================================

/// Demonstrates tracking multiple resource types simultaneously.
async fn example_multiple_resources() {
    println!("\n=== Example 4: Multiple Resource Types ===");

    // Track both file and database resources
    let effect = pure::<_, String, ()>(42)
        .acquires::<FileRes>()
        .also_acquires::<DbRes>();

    // The type system now knows this effect acquires both FileRes and DbRes
    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);

    // You can also release multiple resources
    let release_both = pure::<_, String, ()>(())
        .releases::<FileRes>()
        .also_releases::<DbRes>();

    release_both.run(&()).await.unwrap();
    println!("  Released both resources");
}

// =============================================================================
// Example 5: Compile-Time Neutrality Assertion
// =============================================================================

/// Demonstrates compile-time verification of resource neutrality.
///
/// The `assert_resource_neutral` function only accepts effects
/// that are guaranteed to be resource-neutral at compile time.
fn safe_file_operation(
    path: &str,
) -> impl ResourceEffect<Output = String, Error = String, Env = (), Acquires = Empty, Releases = Empty>
{
    let path = path.to_string();

    // This bracket is guaranteed to be resource-neutral
    let effect = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
        pure::<_, String, ()>(format!("handle_{}", path)),
        |_handle: String| async { Ok(()) },
        |handle: &String| pure::<_, String, ()>(format!("Read from {}", handle)),
    );

    // Compile-time assertion that this is neutral
    assert_resource_neutral(effect)
}

async fn example_neutrality_assertion() {
    println!("\n=== Example 5: Compile-Time Neutrality Assertion ===");

    // This function is guaranteed at compile time to be resource-neutral
    let result = safe_file_operation("config.json").run(&()).await;
    println!("  Result: {:?}", result);
    println!("  The type system guarantees no resource leaks!");
}

// =============================================================================
// Example 6: Custom Resource Kinds
// =============================================================================

/// Define a custom resource kind for connection pools.
struct PoolRes;

impl ResourceKind for PoolRes {
    const NAME: &'static str = "ConnectionPool";
}

fn acquire_pool_connection() -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Has<PoolRes>,
    Releases = Empty,
> {
    println!("  Acquiring connection from pool...");
    pure::<_, String, ()>("conn_42".to_string()).acquires::<PoolRes>()
}

fn release_pool_connection(
    _conn: String,
) -> impl ResourceEffect<Output = (), Error = String, Env = (), Acquires = Empty, Releases = Has<PoolRes>>
{
    println!("  Returning connection to pool");
    pure::<_, String, ()>(()).releases::<PoolRes>()
}

async fn example_custom_resource() {
    println!("\n=== Example 6: Custom Resource Kinds ===");

    let result = resource_bracket::<PoolRes, _, _, _, _, _, _, _, _, _>(
        acquire_pool_connection(),
        |conn: String| async move { release_pool_connection(conn).run(&()).await },
        |conn: &String| {
            println!("  Using connection: {}", conn);
            pure::<_, String, ()>(format!("Query result from {}", conn))
        },
    )
    .run(&())
    .await;

    println!("  Result: {:?}", result);
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    println!("Compile-Time Resource Tracking Examples");
    println!("========================================");
    println!();
    println!("This demonstrates type-level resource tracking with ZERO runtime overhead.");
    println!("All resource safety is verified at compile time.");

    example_basic_annotation().await;
    example_resource_bracket().await;
    example_transaction_protocol().await;
    example_multiple_resources().await;
    example_neutrality_assertion().await;
    example_custom_resource().await;

    println!("\n=== All examples completed! ===");
    println!();
    println!("Key takeaways:");
    println!("  - Use .acquires::<R>() to mark resource acquisition");
    println!("  - Use .releases::<R>() to mark resource release");
    println!("  - Use resource_bracket for guaranteed cleanup");
    println!("  - Use assert_resource_neutral for compile-time safety checks");
    println!("  - Define custom ResourceKind types for domain-specific resources");
    println!("  - ALL tracking is compile-time only - zero runtime overhead!");
}
