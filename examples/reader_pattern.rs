use stillwater::Effect;

/// Application configuration
#[derive(Clone, Debug)]
struct AppConfig {
    api_key: String,
    timeout_ms: u64,
    max_retries: u32,
    log_level: LogLevel,
}

#[derive(Clone, Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Database configuration
#[derive(Clone, Debug)]
struct DbConfig {
    host: String,
    port: u16,
    max_connections: u32,
}

/// Combined environment for application
#[derive(Clone)]
struct AppEnv {
    config: AppConfig,
    db: DbConfig,
}

#[derive(Debug, PartialEq)]
enum AppError {
    Unauthorized,
    Timeout,
    DatabaseError(String),
}

// Example 1: Using ask() to access the entire environment
fn get_environment() -> Effect<AppEnv, AppError, AppEnv> {
    Effect::<AppEnv, AppError, AppEnv>::ask()
}

// Example 2: Using asks() to extract specific configuration
fn get_timeout() -> Effect<u64, AppError, AppEnv> {
    Effect::<(), AppError, AppEnv>::asks(|env: &AppEnv| env.config.timeout_ms)
}

fn get_api_key() -> Effect<String, AppError, AppEnv> {
    Effect::<(), AppError, AppEnv>::asks(|env: &AppEnv| env.config.api_key.clone())
}

fn get_log_level() -> Effect<LogLevel, AppError, AppEnv> {
    Effect::<(), AppError, AppEnv>::asks(|env: &AppEnv| env.config.log_level.clone())
}

// Example 3: Composing asks() queries
fn get_db_connection_string() -> Effect<String, AppError, AppEnv> {
    Effect::<(), AppError, AppEnv>::asks(|env: &AppEnv| {
        format!("postgres://{}:{}", env.db.host, env.db.port)
    })
}

// Example 4: Using environment values in computations
fn fetch_user_data(user_id: u32) -> Effect<String, AppError, AppEnv> {
    get_api_key().and_then(move |api_key| {
        get_timeout().map(move |timeout| {
            format!(
                "Fetching user {} with key {} (timeout={}ms)",
                user_id, api_key, timeout
            )
        })
    })
}

// Example 5: Conditional logic based on environment
fn log_message(msg: String, level: LogLevel) -> Effect<(), AppError, AppEnv> {
    get_log_level().map(move |configured_level| {
        let should_log = match (&configured_level, &level) {
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warn, LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        };

        if should_log {
            match level {
                LogLevel::Debug => println!("[DEBUG] {}", msg),
                LogLevel::Info => println!("[INFO] {}", msg),
                LogLevel::Warn => println!("[WARN] {}", msg),
                LogLevel::Error => println!("[ERROR] {}", msg),
            }
        }
    })
}

// Example 5a: Helper functions for different log levels
fn log_debug(msg: String) -> Effect<(), AppError, AppEnv> {
    log_message(msg, LogLevel::Debug)
}

fn log_info(msg: String) -> Effect<(), AppError, AppEnv> {
    log_message(msg, LogLevel::Info)
}

fn log_warn(msg: String) -> Effect<(), AppError, AppEnv> {
    log_message(msg, LogLevel::Warn)
}

fn log_error(msg: String) -> Effect<(), AppError, AppEnv> {
    log_message(msg, LogLevel::Error)
}

// Example 5b: Validation that can fail with specific errors
fn validate_api_key() -> Effect<(), AppError, AppEnv> {
    get_api_key().and_then(|key| {
        if key.is_empty() {
            Effect::fail(AppError::Unauthorized)
        } else {
            Effect::pure(())
        }
    })
}

fn check_timeout() -> Effect<(), AppError, AppEnv> {
    get_timeout().and_then(|timeout| {
        if timeout == 0 {
            Effect::fail(AppError::Timeout)
        } else {
            Effect::pure(())
        }
    })
}

fn safe_database_query(query: String) -> Effect<Vec<String>, AppError, AppEnv> {
    if query.contains("DROP") || query.contains("DELETE") {
        Effect::fail(AppError::DatabaseError(
            "Dangerous query blocked".to_string(),
        ))
    } else {
        query_database(query)
    }
}

// Example 6: Using local() to temporarily modify environment
fn with_debug_logging<T: Send + 'static>(
    effect: Effect<T, AppError, AppEnv>,
) -> Effect<T, AppError, AppEnv> {
    Effect::local(
        |env: &AppEnv| AppEnv {
            config: AppConfig {
                log_level: LogLevel::Debug,
                ..env.config.clone()
            },
            ..env.clone()
        },
        effect,
    )
}

// Example 7: Extending timeout for specific operations
fn with_extended_timeout<T: Send + 'static>(
    multiplier: u64,
    effect: Effect<T, AppError, AppEnv>,
) -> Effect<T, AppError, AppEnv> {
    Effect::local(
        move |env: &AppEnv| AppEnv {
            config: AppConfig {
                timeout_ms: env.config.timeout_ms * multiplier,
                ..env.config.clone()
            },
            ..env.clone()
        },
        effect,
    )
}

// Example 8: Real-world scenario - API request with retry
fn make_api_request(endpoint: String) -> Effect<String, AppError, AppEnv> {
    get_api_key()
        .and_then(move |api_key| {
            get_timeout().and_then(move |timeout| {
                Effect::<(), AppError, AppEnv>::asks(move |env: &AppEnv| {
                    format!(
                        "POST {} [key={}, timeout={}ms, retries={}]",
                        endpoint, api_key, timeout, env.config.max_retries
                    )
                })
            })
        })
        .and_then(|result| log_info(result.clone()).map(move |_| result))
}

// Example 9: Database operation with connection pooling
fn query_database(query: String) -> Effect<Vec<String>, AppError, AppEnv> {
    get_db_connection_string()
        .and_then(move |conn_str| {
            Effect::<(), AppError, AppEnv>::asks(move |env: &AppEnv| {
                println!(
                    "Executing query on {} (max_connections={})",
                    conn_str, env.db.max_connections
                );
                vec![format!("Result for: {}", query)]
            })
        })
        .and_then(|results| {
            log_info(format!("Query returned {} rows", results.len())).map(move |_| results)
        })
}

// Example 10: Complex workflow combining multiple patterns
fn process_user_request(user_id: u32) -> Effect<String, AppError, AppEnv> {
    // Validate before processing
    validate_api_key()
        .and_then(move |_| check_timeout())
        .and_then(move |_| {
            // Log the request with debug logging enabled
            with_debug_logging(log_debug(format!(
                "Processing request for user {}",
                user_id
            )))
        })
        .and_then(move |_| {
            // Fetch user data with standard timeout
            fetch_user_data(user_id)
        })
        .and_then(move |user_data| {
            // Query database with extended timeout (safe query)
            with_extended_timeout(
                3,
                safe_database_query(format!("SELECT * FROM users WHERE id = {}", user_id)),
            )
            .map(move |db_results| (user_data, db_results))
        })
        .and_then(|(user_data, db_results)| {
            // Make API request to external service
            make_api_request("/api/enrich".to_string()).map(move |api_response| {
                format!(
                    "User: {}, DB: {:?}, API: {}",
                    user_data, db_results, api_response
                )
            })
        })
}

// Example 11: Testing with different environments
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_env() -> AppEnv {
        AppEnv {
            config: AppConfig {
                api_key: "test-key-123".to_string(),
                timeout_ms: 1000,
                max_retries: 3,
                log_level: LogLevel::Info,
            },
            db: DbConfig {
                host: "localhost".to_string(),
                port: 5432,
                max_connections: 10,
            },
        }
    }

    fn create_production_env() -> AppEnv {
        AppEnv {
            config: AppConfig {
                api_key: "prod-key-xyz".to_string(),
                timeout_ms: 5000,
                max_retries: 5,
                log_level: LogLevel::Warn,
            },
            db: DbConfig {
                host: "db.example.com".to_string(),
                port: 5432,
                max_connections: 50,
            },
        }
    }

    #[tokio::test]
    async fn test_ask_returns_environment() {
        let env = create_test_env();
        let result = get_environment().run(&env).await.unwrap();
        assert_eq!(result.config.api_key, "test-key-123");
    }

    #[tokio::test]
    async fn test_asks_extracts_timeout() {
        let env = create_test_env();
        let timeout = get_timeout().run(&env).await.unwrap();
        assert_eq!(timeout, 1000);
    }

    #[tokio::test]
    async fn test_local_modifies_timeout_temporarily() {
        let env = create_test_env();

        // Original timeout
        let original = get_timeout().run(&env).await.unwrap();
        assert_eq!(original, 1000);

        // Extended timeout in local scope
        let extended = with_extended_timeout(3, get_timeout())
            .run(&env)
            .await
            .unwrap();
        assert_eq!(extended, 3000);

        // Original still unchanged
        let still_original = get_timeout().run(&env).await.unwrap();
        assert_eq!(still_original, 1000);
    }

    #[tokio::test]
    async fn test_local_modifies_log_level() {
        let env = create_test_env();

        // Original log level
        let original = get_log_level().run(&env).await.unwrap();
        assert_eq!(original, LogLevel::Info);

        // Debug log level in local scope
        let debug = with_debug_logging(get_log_level()).run(&env).await.unwrap();
        assert_eq!(debug, LogLevel::Debug);
    }

    #[tokio::test]
    async fn test_different_environments() {
        let test_env = create_test_env();
        let prod_env = create_production_env();

        // Same code, different behavior based on environment
        let test_timeout = get_timeout().run(&test_env).await.unwrap();
        let prod_timeout = get_timeout().run(&prod_env).await.unwrap();

        assert_eq!(test_timeout, 1000);
        assert_eq!(prod_timeout, 5000);
    }

    #[tokio::test]
    async fn test_composition_with_environment() {
        let env = create_test_env();
        let result = fetch_user_data(42).run(&env).await.unwrap();
        assert!(result.contains("test-key-123"));
        assert!(result.contains("1000ms"));
    }

    #[tokio::test]
    async fn test_error_variants() {
        let env = AppEnv {
            config: AppConfig {
                api_key: "".to_string(),
                timeout_ms: 1000,
                max_retries: 3,
                log_level: LogLevel::Info,
            },
            db: create_test_env().db,
        };

        // Test unauthorized error
        let result = validate_api_key().run(&env).await;
        assert_eq!(result, Err(AppError::Unauthorized));

        // Test timeout error
        let timeout_env = AppEnv {
            config: AppConfig {
                api_key: "test".to_string(),
                timeout_ms: 0,
                max_retries: 3,
                log_level: LogLevel::Info,
            },
            db: create_test_env().db,
        };
        let result = check_timeout().run(&timeout_env).await;
        assert_eq!(result, Err(AppError::Timeout));

        // Test database error
        let result = safe_database_query("DROP TABLE users".to_string())
            .run(&create_test_env())
            .await;
        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }

    #[tokio::test]
    async fn test_log_levels() {
        // Test all log levels are used
        let debug_env = AppEnv {
            config: AppConfig {
                log_level: LogLevel::Debug,
                ..create_test_env().config
            },
            ..create_test_env()
        };

        log_debug("test".to_string()).run(&debug_env).await.unwrap();
        log_info("test".to_string()).run(&debug_env).await.unwrap();
        log_warn("test".to_string()).run(&debug_env).await.unwrap();
        log_error("test".to_string()).run(&debug_env).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("=== Reader Pattern Examples ===\n");

    // Setup environment
    let env = AppEnv {
        config: AppConfig {
            api_key: "secret-key-abc123".to_string(),
            timeout_ms: 2000,
            max_retries: 3,
            log_level: LogLevel::Info,
        },
        db: DbConfig {
            host: "localhost".to_string(),
            port: 5432,
            max_connections: 20,
        },
    };

    println!("1. Get entire environment:");
    let full_env = get_environment().run(&env).await?;
    println!("   Config: {:?}\n", full_env.config);

    println!("2. Extract specific values with asks():");
    let timeout = get_timeout().run(&env).await?;
    let api_key = get_api_key().run(&env).await?;
    println!("   Timeout: {}ms", timeout);
    println!("   API Key: {}\n", api_key);

    println!("3. Compose environment queries:");
    let conn_str = get_db_connection_string().run(&env).await?;
    println!("   Connection: {}\n", conn_str);

    println!("4. Use environment in computations:");
    let user_data = fetch_user_data(123).run(&env).await?;
    println!("   {}\n", user_data);

    println!("5. Temporarily extend timeout with local():");
    let standard = get_timeout().run(&env).await?;
    let extended = with_extended_timeout(5, get_timeout()).run(&env).await?;
    println!("   Standard timeout: {}ms", standard);
    println!("   Extended timeout: {}ms", extended);
    println!(
        "   Original unchanged: {}ms\n",
        get_timeout().run(&env).await?
    );

    println!("6. Complex workflow with environment:");
    let result = process_user_request(456).run(&env).await?;
    println!("   {}\n", result);

    println!("7. Database query with environment:");
    let db_results = query_database("SELECT * FROM orders".to_string())
        .run(&env)
        .await?;
    println!("   Results: {:?}\n", db_results);

    println!("8. Error handling with environment validation:");
    log_info("Demonstrating error handling".to_string())
        .run(&env)
        .await?;

    // Show safe query validation
    match safe_database_query("DROP TABLE users".to_string())
        .run(&env)
        .await
    {
        Err(AppError::DatabaseError(msg)) => {
            log_error(format!("Query blocked: {}", msg))
                .run(&env)
                .await?;
        }
        _ => {}
    }

    // Show successful safe query
    let safe_results = safe_database_query("SELECT * FROM users".to_string())
        .run(&env)
        .await?;
    println!("   Safe query results: {:?}\n", safe_results);

    println!("9. Different log levels:");
    log_debug("This is a debug message".to_string())
        .run(&env)
        .await?;
    log_warn("This is a warning message".to_string())
        .run(&env)
        .await?;
    println!();

    println!("=== Benefits of Reader Pattern ===");
    println!("✓ No global variables or thread-local storage");
    println!("✓ Easy to test with different configurations");
    println!("✓ Type-safe dependency injection");
    println!("✓ Functions don't need explicit config parameters");
    println!("✓ Temporary environment modifications with local()");
    println!("✓ Compose environment queries functionally");
    println!("✓ Clean error handling with context-aware validation");

    Ok(())
}
