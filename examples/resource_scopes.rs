//! Resource Scopes Example
//!
//! Demonstrates the bracket pattern for safe resource management.
//! The bracket pattern ensures resources are always released,
//! even when errors occur during use.
//!
//! Shows practical patterns including:
//! - Basic bracket usage for single resources
//! - Multiple resources with LIFO cleanup
//! - The `acquiring` builder for ergonomic multi-resource management
//! - Real-world file I/O with automatic cleanup
//! - Error handling during use and cleanup phases

use std::io::{self, Write as IoWrite};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use stillwater::effect::bracket::{acquiring, bracket, bracket2, bracket_full, BracketError};
use stillwater::{fail, from_fn, pure, Effect};

// ==================== Basic Bracket Pattern ====================

/// Example 1: Basic bracket pattern
///
/// Demonstrates the core acquire → use → release flow.
/// The release function always runs, even if use fails.
async fn example_basic_bracket() {
    println!("\n=== Example 1: Basic Bracket Pattern ===");

    // Track whether cleanup ran
    let cleaned_up = Arc::new(AtomicBool::new(false));
    let cleaned_up_clone = cleaned_up.clone();

    #[derive(Clone)]
    struct Resource {
        id: u32,
    }

    let effect = bracket(
        // Acquire: create the resource
        pure::<_, String, ()>(Resource { id: 42 }),
        // Release: cleanup (always runs)
        move |_resource: Resource| {
            cleaned_up_clone.store(true, Ordering::SeqCst);
            println!("  Resource cleaned up!");
            async { Ok(()) }
        },
        // Use: work with the resource
        |resource: &Resource| {
            println!("  Using resource with id: {}", resource.id);
            pure::<_, String, ()>(resource.id * 2)
        },
    );

    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);
    println!(
        "  Cleanup ran: {}",
        cleaned_up.load(Ordering::SeqCst)
    );
}

// ==================== Bracket with Use Failure ====================

/// Example 2: Bracket guarantees cleanup on failure
///
/// Even when the use function fails, the release function runs.
async fn example_bracket_use_failure() {
    println!("\n=== Example 2: Bracket Cleanup on Use Failure ===");

    let cleaned_up = Arc::new(AtomicBool::new(false));
    let cleaned_up_clone = cleaned_up.clone();

    let effect = bracket(
        // Acquire
        pure::<_, String, ()>("resource"),
        // Release
        move |_: &str| {
            cleaned_up_clone.store(true, Ordering::SeqCst);
            println!("  Cleanup ran despite failure!");
            async { Ok(()) }
        },
        // Use - fails!
        |_: &&str| fail::<i32, String, ()>("use phase failed".to_string()),
    );

    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);
    println!(
        "  Cleanup ran: {}",
        cleaned_up.load(Ordering::SeqCst)
    );
}

// ==================== Multiple Resources with LIFO Cleanup ====================

/// Example 3: Multiple resources with reverse cleanup order
///
/// Uses bracket2 to manage two resources. Resources are released
/// in reverse order (LIFO - Last In, First Out).
async fn example_bracket2_lifo() {
    println!("\n=== Example 3: Multiple Resources (LIFO Cleanup) ===");

    let cleanup_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let order1 = cleanup_order.clone();
    let order2 = cleanup_order.clone();

    let effect = bracket2(
        // Acquire first resource
        {
            println!("  Acquiring first resource...");
            pure::<_, String, ()>("database")
        },
        // Acquire second resource
        {
            println!("  Acquiring second resource...");
            pure::<_, String, ()>("file")
        },
        // Release first (runs second due to LIFO)
        move |_: &str| {
            order1.lock().unwrap().push("database");
            println!("  Released database");
            async { Ok(()) }
        },
        // Release second (runs first due to LIFO)
        move |_: &str| {
            order2.lock().unwrap().push("file");
            println!("  Released file");
            async { Ok(()) }
        },
        // Use both resources
        |db: &&str, file: &&str| {
            println!("  Using {} and {}", db, file);
            pure::<_, String, ()>(format!("{} + {}", db, file))
        },
    );

    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);
    println!(
        "  Cleanup order: {:?} (LIFO: file first, then database)",
        cleanup_order.lock().unwrap()
    );
}

// ==================== Acquiring Builder Pattern ====================

/// Example 4: Acquiring builder for ergonomic multi-resource management
///
/// The `acquiring` builder provides a fluent API for managing multiple
/// resources without deeply nested brackets.
async fn example_acquiring_builder() {
    println!("\n=== Example 4: Acquiring Builder Pattern ===");

    let cleanup_order = Arc::new(std::sync::Mutex::new(Vec::new()));
    let order1 = cleanup_order.clone();
    let order2 = cleanup_order.clone();
    let order3 = cleanup_order.clone();

    let effect = acquiring(
        pure::<_, String, ()>("connection"),
        move |_: &str| {
            order1.lock().unwrap().push("connection");
            async { Ok(()) }
        },
    )
    .and(pure::<_, String, ()>("transaction"), move |_: &str| {
        order2.lock().unwrap().push("transaction");
        async { Ok(()) }
    })
    .and(pure::<_, String, ()>("lock"), move |_: &str| {
        order3.lock().unwrap().push("lock");
        async { Ok(()) }
    })
    // with_flat3 provides ergonomic access to all three resources
    .with_flat3(|conn: &&str, txn: &&str, lock: &&str| {
        println!("  Using: {}, {}, {}", conn, txn, lock);
        pure::<_, String, ()>(format!("{} -> {} -> {}", conn, txn, lock))
    });

    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);
    println!(
        "  Cleanup order (LIFO): {:?}",
        cleanup_order.lock().unwrap()
    );
}

// ==================== BracketFull for Explicit Error Handling ====================

/// Example 5: BracketFull for explicit error handling
///
/// bracket_full returns a BracketError that distinguishes between
/// acquire, use, and cleanup errors.
async fn example_bracket_full() {
    println!("\n=== Example 5: BracketFull for Explicit Errors ===");

    // Case 1: Use fails, cleanup succeeds
    println!("\n  Case 1: Use fails, cleanup succeeds");
    let result1 = bracket_full(
        pure::<_, String, ()>(42),
        |_: i32| async { Ok(()) },
        |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
    )
    .run(&())
    .await;

    match result1 {
        Err(BracketError::UseError(e)) => println!("    UseError: {}", e),
        other => println!("    Unexpected: {:?}", other),
    }

    // Case 2: Use succeeds, cleanup fails
    println!("\n  Case 2: Use succeeds, cleanup fails");
    let result2 = bracket_full(
        pure::<_, String, ()>(42),
        |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
        |val: &i32| pure::<_, String, ()>(*val * 2),
    )
    .run(&())
    .await;

    match result2 {
        Err(BracketError::CleanupError(e)) => println!("    CleanupError: {}", e),
        other => println!("    Unexpected: {:?}", other),
    }

    // Case 3: Both fail
    println!("\n  Case 3: Both use and cleanup fail");
    let result3 = bracket_full(
        pure::<_, String, ()>(42),
        |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
        |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
    )
    .run(&())
    .await;

    match result3 {
        Err(BracketError::Both {
            use_error,
            cleanup_error,
        }) => {
            println!("    Both failed!");
            println!("      Use error: {}", use_error);
            println!("      Cleanup error: {}", cleanup_error);
        }
        other => println!("    Unexpected: {:?}", other),
    }
}

// ==================== Real-World File I/O Example ====================

/// Example 6: Real-world file I/O with automatic cleanup
///
/// Demonstrates using the bracket pattern with actual file operations.
/// Uses interior mutability (Arc<Mutex>) since the bracket pattern
/// passes the resource by reference to the use function.
async fn example_file_io() {
    println!("\n=== Example 6: File I/O with Automatic Cleanup ===");

    // Track cleanup for demonstration
    let file_closed = Arc::new(AtomicBool::new(false));
    let file_closed_clone = file_closed.clone();

    // Create a temporary file path
    let temp_path = std::env::temp_dir().join("stillwater_bracket_example.txt");
    let temp_path_clone = temp_path.clone();

    // Use Arc<Mutex<>> for interior mutability - the bracket pattern
    // passes resources by reference to the use function
    #[derive(Debug, Clone)]
    struct FileHandle {
        path: PathBuf,
        content: Arc<std::sync::Mutex<Vec<String>>>,
    }

    let effect = bracket(
        // Acquire: create the file handle
        from_fn(move |_: &()| {
            println!("  Opening file: {:?}", temp_path_clone);
            Ok::<_, io::Error>(FileHandle {
                path: temp_path_clone.clone(),
                content: Arc::new(std::sync::Mutex::new(Vec::new())),
            })
        }),
        // Release: write to file and cleanup
        move |handle: FileHandle| {
            file_closed_clone.store(true, Ordering::SeqCst);
            let content = handle.content.lock().unwrap().clone();
            println!("  Closing file and writing {} lines", content.len());

            let path = handle.path.clone();
            async move {
                let mut file = std::fs::File::create(&path)?;
                for line in content {
                    writeln!(file, "{}", line)?;
                }
                println!("  File closed successfully");
                Ok(())
            }
        },
        // Use: write lines to the file handle (via interior mutability)
        |handle: &FileHandle| {
            let mut content = handle.content.lock().unwrap();
            content.push("Line 1: Hello from bracket pattern!".to_string());
            content.push("Line 2: Safe resource management".to_string());
            content.push("Line 3: Cleanup guaranteed".to_string());

            println!("  Writing {} lines to file", content.len());

            pure::<_, io::Error, ()>(content.len())
        },
    );

    let result = effect.run(&()).await;
    println!("  Result: {:?}", result);
    println!(
        "  File closed: {}",
        file_closed.load(Ordering::SeqCst)
    );

    // Verify the file was created
    if temp_path.exists() {
        let contents = std::fs::read_to_string(&temp_path).unwrap_or_default();
        println!("  File contents:\n{}", contents);
        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }
}

// ==================== Connection Pool Pattern ====================

/// Example 7: Connection pool pattern
///
/// Demonstrates using bracket for database-like connection management.
async fn example_connection_pool() {
    println!("\n=== Example 7: Connection Pool Pattern ===");

    // Simulated connection pool
    #[derive(Clone)]
    struct ConnectionPool {
        active_connections: Arc<AtomicUsize>,
        max_connections: usize,
    }

    #[derive(Debug)]
    struct Connection {
        id: usize,
    }

    impl ConnectionPool {
        fn new(max: usize) -> Self {
            ConnectionPool {
                active_connections: Arc::new(AtomicUsize::new(0)),
                max_connections: max,
            }
        }

        fn acquire(&self) -> Result<Connection, String> {
            let current = self.active_connections.fetch_add(1, Ordering::SeqCst);
            if current >= self.max_connections {
                self.active_connections.fetch_sub(1, Ordering::SeqCst);
                Err("Pool exhausted".to_string())
            } else {
                Ok(Connection { id: current + 1 })
            }
        }

        #[allow(dead_code)]
        fn release(&self, _conn: Connection) {
            self.active_connections.fetch_sub(1, Ordering::SeqCst);
        }
    }

    #[derive(Clone)]
    struct Env {
        pool: ConnectionPool,
    }

    let pool = ConnectionPool::new(5);
    let pool_for_release = pool.clone();

    let env = Env { pool };

    // Use bracket to ensure connection is always returned to pool
    let effect = bracket(
        from_fn(|env: &Env| {
            let conn = env.pool.acquire()?;
            println!("  Acquired connection #{}", conn.id);
            Ok::<_, String>(conn)
        }),
        move |conn: Connection| {
            println!("  Releasing connection #{}", conn.id);
            pool_for_release.active_connections.fetch_sub(1, Ordering::SeqCst);
            async move { Ok(()) }
        },
        |conn: &Connection| {
            println!("  Using connection #{} for query", conn.id);
            pure::<_, String, Env>(format!("Query result from connection #{}", conn.id))
        },
    );

    let result = effect.run(&env).await;
    println!("  Result: {:?}", result);
    println!(
        "  Active connections after: {}",
        env.pool.active_connections.load(Ordering::SeqCst)
    );
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Resource Scopes Examples");
    println!("========================");
    println!("Demonstrating the bracket pattern for safe resource management.");

    example_basic_bracket().await;
    example_bracket_use_failure().await;
    example_bracket2_lifo().await;
    example_acquiring_builder().await;
    example_bracket_full().await;
    example_file_io().await;
    example_connection_pool().await;

    println!("\n=== All examples completed successfully! ===");
}
