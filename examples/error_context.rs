//! Error Context Example
//!
//! Demonstrates ContextError and error trails for better debugging.
//! Shows practical patterns including:
//! - Creating context errors
//! - Adding context breadcrumbs
//! - Using .context() with Effects
//! - Building error trails through call stacks
//! - Formatted error display

use stillwater::{ContextError, Effect, EffectContext};

// ==================== Basic Context Errors ====================

/// Example 1: Creating and displaying context errors
///
/// Demonstrates basic ContextError construction and display.
fn example_basic_context() {
    println!("\n=== Example 1: Basic Context Errors ===");

    // Simple error with no context
    let err1: ContextError<&str> = ContextError::new("file not found");
    println!("Basic error:\n{}\n", err1);

    // Error with single context
    let err2 = ContextError::new("file not found").context("reading config file");
    println!("With context:\n{}\n", err2);

    // Error with multiple context layers
    let err3 = ContextError::new("connection refused")
        .context("connecting to database")
        .context("initializing application")
        .context("startup sequence");
    println!("Multiple contexts:\n{}\n", err3);
}

// ==================== Building Error Trails ====================

/// Example 2: Building error trails through function calls
///
/// Demonstrates how context accumulates as errors bubble up.
fn example_error_trails() {
    println!("\n=== Example 2: Error Trails ===");

    // Simulated function call stack
    fn read_file(path: &str) -> Result<String, ContextError<String>> {
        Err(ContextError::new(format!("No such file: {}", path)))
    }

    fn load_config() -> Result<String, ContextError<String>> {
        read_file("/etc/app/config.toml")
            .map_err(|e| e.context("loading configuration file".to_string()))
    }

    fn initialize_database() -> Result<(), ContextError<String>> {
        load_config().map_err(|e| e.context("reading database connection string".to_string()))?;
        Ok(())
    }

    fn start_application() -> Result<(), ContextError<String>> {
        initialize_database()
            .map_err(|e| e.context("initializing database connection".to_string()))?;
        Ok(())
    }

    // Run and see the full error trail
    match start_application() {
        Ok(_) => println!("Application started successfully"),
        Err(e) => println!("Application startup failed:\n{}", e),
    }
}

// ==================== Context with Effects ====================

/// Example 3: Using context() with Effect
///
/// Demonstrates how to add context to Effect errors.
async fn example_effect_context() {
    println!("\n=== Example 3: Context with Effects ===");

    struct Database {
        connected: bool,
    }

    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    // Effect that fails
    fn connect_to_db() -> Effect<(), String, Env> {
        Effect::from_fn(|env: &Env| {
            if env.db.connected {
                Ok(())
            } else {
                Err("connection refused".to_string())
            }
        })
    }

    // Add context to the effect
    let effect = connect_to_db().context("connecting to primary database");

    let env = Env {
        db: Database { connected: false },
    };

    match effect.run(&env).await {
        Ok(_) => println!("Connected successfully"),
        Err(e) => println!("Connection failed:\n{}", e),
    }
}

// ==================== Layered Context in Workflows ====================

/// Example 4: Building context through effect composition
///
/// Demonstrates context accumulation in complex effect workflows.
async fn example_layered_context() {
    println!("\n=== Example 4: Layered Context in Workflows ===");

    #[derive(Clone)]
    struct User {
        id: u64,
        name: String,
    }

    struct Database {
        users: Vec<User>,
    }

    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    // Low-level: fetch user from database
    fn fetch_user(user_id: u64) -> Effect<User, ContextError<String>, Env> {
        Effect::from_fn(move |env: &Env| {
            env.db
                .users
                .iter()
                .find(|u| u.id == user_id)
                .cloned()
                .ok_or_else(|| format!("user id {} not found", user_id))
        })
        .context(format!("fetching user {}", user_id))
    }

    // Mid-level: load user profile
    fn load_user_profile(user_id: u64) -> Effect<User, ContextError<String>, Env> {
        fetch_user(user_id).context(format!("loading profile for user {}", user_id))
    }

    // High-level: display user dashboard
    fn display_dashboard(user_id: u64) -> Effect<String, ContextError<String>, Env> {
        load_user_profile(user_id)
            .map(|user| format!("Dashboard for {}", user.name))
            .context(format!("rendering dashboard for user {}", user_id))
    }

    let env = Env {
        db: Database { users: vec![] }, // Empty database - will fail
    };

    match display_dashboard(42).run(&env).await {
        Ok(dashboard) => println!("{}", dashboard),
        Err(e) => println!("Failed to display dashboard:\n{}", e),
    }
}

// ==================== Practical Error Handling ====================

/// Example 5: Real-world error handling with context
///
/// Demonstrates using context for a realistic file processing scenario.
async fn example_realistic_error_handling() {
    println!("\n=== Example 5: Realistic Error Handling ===");

    #[derive(Debug)]
    struct Config {
        timeout: u32,
    }

    struct FileSystem {
        files: Vec<String>,
    }

    struct Env {
        fs: FileSystem,
    }

    impl AsRef<FileSystem> for Env {
        fn as_ref(&self) -> &FileSystem {
            &self.fs
        }
    }

    // Read file
    fn read_file(path: String) -> Effect<String, ContextError<String>, Env> {
        let path_for_context = path.clone();
        Effect::from_fn(move |env: &Env| {
            env.fs
                .files
                .iter()
                .find(|f| f.starts_with(&path))
                .cloned()
                .ok_or_else(|| format!("file not found: {}", path))
        })
        .context(format!("reading file '{}'", path_for_context))
    }

    // Parse config
    fn parse_config(content: String) -> Effect<Config, ContextError<String>, Env> {
        Effect::from_fn(move |_env: &Env| {
            if content.contains("timeout") {
                Ok(Config { timeout: 30 })
            } else {
                Err("missing required field 'timeout'".to_string())
            }
        })
        .context("parsing configuration".to_string())
    }

    // Load and parse config
    fn load_config(path: String) -> Effect<Config, ContextError<String>, Env> {
        read_file(path.clone())
            .and_then(parse_config)
            .context(format!("loading config from '{}'", path))
    }

    let env1 = Env {
        fs: FileSystem {
            files: vec!["config.toml: timeout=30".to_string()],
        },
    };

    // Success case
    match load_config("config.toml".to_string()).run(&env1).await {
        Ok(config) => println!("Config loaded: timeout={}", config.timeout),
        Err(e) => println!("Failed:\n{}", e),
    }

    // Failure case: file not found
    let env2 = Env {
        fs: FileSystem { files: vec![] },
    };

    match load_config("missing.toml".to_string()).run(&env2).await {
        Ok(_) => println!("Config loaded"),
        Err(e) => println!("\nFailed:\n{}", e),
    }

    // Failure case: parse error
    let env3 = Env {
        fs: FileSystem {
            files: vec!["bad.toml: invalid content".to_string()],
        },
    };

    match load_config("bad.toml".to_string()).run(&env3).await {
        Ok(_) => println!("Config loaded"),
        Err(e) => println!("\nFailed:\n{}", e),
    }
}

// ==================== Custom Error Types ====================

/// Example 6: Using ContextError with custom error types
///
/// Demonstrates wrapping domain-specific errors.
async fn example_custom_errors() {
    println!("\n=== Example 6: Custom Error Types ===");

    #[derive(Debug, Clone)]
    enum AppError {
        NotFound,
        PermissionDenied,
        InvalidInput(String),
    }

    impl std::fmt::Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AppError::NotFound => write!(f, "resource not found"),
                AppError::PermissionDenied => write!(f, "permission denied"),
                AppError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            }
        }
    }

    struct Env;

    fn validate_user_input(input: String) -> Effect<String, AppError, Env> {
        Effect::from_fn(move |_env: &Env| {
            if input.is_empty() {
                Err(AppError::InvalidInput("input cannot be empty".to_string()))
            } else if input.len() > 100 {
                Err(AppError::InvalidInput(
                    "input exceeds maximum length".to_string(),
                ))
            } else {
                Ok(input.clone())
            }
        })
    }

    fn check_permissions(user_id: u64) -> Effect<(), AppError, Env> {
        Effect::from_fn(move |_env: &Env| {
            if user_id == 0 {
                Err(AppError::PermissionDenied)
            } else {
                Ok(())
            }
        })
    }

    fn find_resource(id: u64) -> Effect<String, AppError, Env> {
        Effect::from_fn(move |_env: &Env| {
            if id == 99 {
                Ok("resource data".to_string())
            } else {
                Err(AppError::NotFound)
            }
        })
    }

    let env = Env;

    // Valid input
    match validate_user_input("hello".to_string()).run(&env).await {
        Ok(s) => println!("Valid input: {}", s),
        Err(e) => println!("Error:\n{}", e),
    }

    // Invalid input
    println!();
    match validate_user_input("".to_string()).run(&env).await {
        Ok(s) => println!("Valid input: {}", s),
        Err(e) => println!("Error:\n{}", e),
    }

    // Permission denied
    println!();
    match check_permissions(0).run(&env).await {
        Ok(_) => println!("Permission granted"),
        Err(e) => println!("Error:\n{}", e),
    }

    // Not found
    println!();
    match find_resource(42).run(&env).await {
        Ok(data) => println!("Found: {}", data),
        Err(e) => println!("Error:\n{}", e),
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Error Context Examples");
    println!("======================");

    example_basic_context();
    example_error_trails();
    example_effect_context().await;
    example_layered_context().await;
    example_realistic_error_handling().await;
    example_custom_errors().await;

    println!("\n=== All examples completed successfully! ===");
}
