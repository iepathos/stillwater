//! IO Patterns Example
//!
//! Demonstrates the IO module's helpers for creating Effects from I/O operations.
//! Shows practical patterns including:
//! - Basic read/write operations
//! - Multiple services in environment
//! - Async operations
//! - Cache-aside pattern
//! - Effect composition with IO helpers

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stillwater::{Effect, IO};

// ==================== Basic Read/Write Operations ====================

/// Example 1: Simple read operation
///
/// Demonstrates using IO::read() to query data without modification.
async fn example_basic_read() {
    println!("\n=== Example 1: Basic Read Operation ===");

    // Define a simple database service
    struct Database {
        users: Vec<String>,
    }

    impl Database {
        fn count_users(&self) -> usize {
            self.users.len()
        }
    }

    // Define environment with database
    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    // Create environment
    let env = Env {
        db: Database {
            users: vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
            ],
        },
    };

    // Create effect using IO::read - type inference determines we need Database
    let effect = IO::read(|db: &Database| db.count_users());

    // Run effect
    let count = effect.run(&env).await.unwrap();
    println!("User count: {}", count);
}

/// Example 2: Write operation with interior mutability
///
/// Demonstrates using IO::write() for operations that modify state.
/// Note: Effect requires &Env, so we use Arc<Mutex<T>> for mutation.
async fn example_basic_write() {
    println!("\n=== Example 2: Basic Write Operation ===");

    // Logger service with interior mutability
    struct Logger {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl Logger {
        fn log(&self, msg: String) {
            self.messages.lock().unwrap().push(msg);
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }
    }

    struct Env {
        logger: Logger,
    }

    impl AsRef<Logger> for Env {
        fn as_ref(&self) -> &Logger {
            &self.logger
        }
    }

    let env = Env {
        logger: Logger {
            messages: Arc::new(Mutex::new(Vec::new())),
        },
    };

    // Create write effect
    let effect = IO::write(|logger: &Logger| {
        logger.log("Application started".to_string());
        logger.log("Processing request".to_string());
    });

    // Run effect
    effect.run(&env).await.unwrap();

    println!("Logged messages: {:?}", env.logger.get_messages());
}

// ==================== Multiple Services in Environment ====================

/// Example 3: Multiple services with type inference
///
/// Demonstrates how AsRef<T> allows the environment to provide multiple services,
/// with type inference automatically extracting the correct one.
async fn example_multiple_services() {
    println!("\n=== Example 3: Multiple Services ===");

    struct Database {
        name: String,
    }
    struct Cache {
        name: String,
    }
    struct Logger {
        name: String,
    }

    // Composite environment
    struct Env {
        db: Database,
        cache: Cache,
        logger: Logger,
    }

    // Implement AsRef for each service
    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }
    impl AsRef<Cache> for Env {
        fn as_ref(&self) -> &Cache {
            &self.cache
        }
    }
    impl AsRef<Logger> for Env {
        fn as_ref(&self) -> &Logger {
            &self.logger
        }
    }

    let env = Env {
        db: Database {
            name: "PostgreSQL".to_string(),
        },
        cache: Cache {
            name: "Redis".to_string(),
        },
        logger: Logger {
            name: "Syslog".to_string(),
        },
    };

    // Type inference figures out which service each effect needs
    let db_effect = IO::read(|db: &Database| db.name.clone());
    let cache_effect = IO::read(|cache: &Cache| cache.name.clone());
    let logger_effect = IO::read(|logger: &Logger| logger.name.clone());

    println!("Database: {}", db_effect.run(&env).await.unwrap());
    println!("Cache: {}", cache_effect.run(&env).await.unwrap());
    println!("Logger: {}", logger_effect.run(&env).await.unwrap());
}

// ==================== Async Operations ====================

/// Example 4: Async read and write operations
///
/// Demonstrates IO::read_async() and IO::write_async() for operations
/// that return futures, such as network calls or async database queries.
async fn example_async_operations() {
    use std::future::ready;

    println!("\n=== Example 4: Async Operations ===");

    struct ApiClient {
        base_url: String,
    }

    struct AuditLog {
        entries: Arc<Mutex<Vec<String>>>,
    }

    impl AuditLog {
        fn record_sync(&self, entry: String) {
            self.entries.lock().unwrap().push(entry);
        }
    }

    struct Env {
        api: ApiClient,
        audit: AuditLog,
    }

    impl AsRef<ApiClient> for Env {
        fn as_ref(&self) -> &ApiClient {
            &self.api
        }
    }

    impl AsRef<AuditLog> for Env {
        fn as_ref(&self) -> &AuditLog {
            &self.audit
        }
    }

    let env = Env {
        api: ApiClient {
            base_url: "https://api.example.com".to_string(),
        },
        audit: AuditLog {
            entries: Arc::new(Mutex::new(Vec::new())),
        },
    };

    // Async read operation - simulating an async query
    let fetch_effect = IO::read_async(|api: &ApiClient| {
        let base_url = api.base_url.clone();
        async move {
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            format!("User 42 from {}", base_url)
        }
    });
    let user = fetch_effect.run(&env).await.unwrap();
    println!("Fetched: {}", user);

    // Async write operation - using ready for demonstration
    let audit_effect = IO::write_async(|audit: &AuditLog| {
        audit.record_sync("User 42 accessed".to_string());
        ready(())
    });
    audit_effect.run(&env).await.unwrap();
    println!("Audit log: {:?}", env.audit.entries.lock().unwrap().clone());
}

// ==================== Cache-Aside Pattern ====================

/// Example 5: Real-world cache-aside pattern
///
/// Demonstrates a practical pattern combining multiple services:
/// 1. Check cache for data
/// 2. If miss, fetch from database
/// 3. Store in cache for next time
async fn example_cache_aside() {
    println!("\n=== Example 5: Cache-Aside Pattern ===");

    #[derive(Clone, Debug, PartialEq)]
    struct User {
        id: u64,
        name: String,
    }

    struct Database {
        users: HashMap<u64, User>,
    }

    struct Cache {
        data: Arc<Mutex<HashMap<u64, User>>>,
    }

    impl Cache {
        fn get(&self, id: u64) -> Option<User> {
            self.data.lock().unwrap().get(&id).cloned()
        }

        fn set(&self, id: u64, user: User) {
            self.data.lock().unwrap().insert(id, user);
        }
    }

    struct Env {
        db: Database,
        cache: Cache,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    impl AsRef<Cache> for Env {
        fn as_ref(&self) -> &Cache {
            &self.cache
        }
    }

    // Setup environment
    let mut users = HashMap::new();
    users.insert(
        1,
        User {
            id: 1,
            name: "Alice".to_string(),
        },
    );
    users.insert(
        2,
        User {
            id: 2,
            name: "Bob".to_string(),
        },
    );

    let env = Env {
        db: Database { users },
        cache: Cache {
            data: Arc::new(Mutex::new(HashMap::new())),
        },
    };

    // Function to get user with cache-aside pattern
    fn get_user(user_id: u64) -> Effect<Option<User>, std::convert::Infallible, Env> {
        // Check cache first
        IO::read(move |cache: &Cache| cache.get(user_id)).and_then(move |cached| {
            if let Some(user) = cached {
                println!("Cache hit for user {}", user_id);
                Effect::pure(Some(user))
            } else {
                println!("Cache miss for user {}", user_id);
                // Fetch from database (synchronous for simplicity)
                IO::read(move |db: &Database| db.users.get(&user_id).cloned()).and_then(
                    move |user| {
                        if let Some(ref u) = user {
                            let user_clone = u.clone();
                            // Store in cache
                            IO::write(move |cache: &Cache| {
                                cache.set(user_id, user_clone);
                            })
                            .map(move |_| user.clone())
                        } else {
                            Effect::pure(user)
                        }
                    },
                )
            }
        })
    }

    // First access - cache miss
    let user1 = get_user(1).run(&env).await.unwrap();
    println!("Got user: {:?}", user1);

    // Second access - cache hit
    let user1_again = get_user(1).run(&env).await.unwrap();
    println!("Got user again: {:?}", user1_again);

    // Different user - cache miss
    let user2 = get_user(2).run(&env).await.unwrap();
    println!("Got user: {:?}", user2);
}

// ==================== Effect Composition ====================

/// Example 6: Composing effects with map, and_then, etc.
///
/// Demonstrates how IO helpers integrate with Effect's combinators.
async fn example_effect_composition() {
    println!("\n=== Example 6: Effect Composition ===");

    struct Calculator {
        multiplier: i32,
    }

    struct Env {
        calc: Calculator,
    }

    impl AsRef<Calculator> for Env {
        fn as_ref(&self) -> &Calculator {
            &self.calc
        }
    }

    let env = Env {
        calc: Calculator { multiplier: 3 },
    };

    // Compose effects using map and and_then
    let effect = IO::read(|calc: &Calculator| calc.multiplier)
        .map(|x| x * 2) // Transform the value
        .and_then(|x| {
            // Chain another effect
            IO::read(move |calc: &Calculator| x + calc.multiplier)
        });

    let result = effect.run(&env).await.unwrap();
    println!("Result: {} (expected: {})", result, 3 * 2 + 3);
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("IO Patterns Examples");
    println!("====================");

    example_basic_read().await;
    example_basic_write().await;
    example_multiple_services().await;
    example_async_operations().await;
    example_cache_aside().await;
    example_effect_composition().await;

    println!("\n=== All examples completed successfully! ===");
}
