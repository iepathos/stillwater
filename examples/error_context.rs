//! Error context example - testing context accumulation ergonomics
//!
//! This tests how well error context chains work in practice.

use std::path::{Path, PathBuf};
use stillwater::{ContextError, Effect, IO};

// ============================================================================
// Domain
// ============================================================================

#[derive(Debug, Clone)]
struct Config {
    database_url: String,
    api_key: String,
    max_connections: u32,
}

#[derive(Debug)]
struct AppState {
    config: Config,
    db_pool: DbPool,
}

#[derive(Debug, Clone)]
struct DbPool;

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug)]
enum AppError {
    FileNotFound(PathBuf),
    ParseError(String),
    ValidationError(String),
    DatabaseError(String),
    MissingField(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::FileNotFound(path) => write!(f, "File not found: {}", path.display()),
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for AppError {}

// ============================================================================
// Pure Functions
// ============================================================================

fn parse_toml(content: &str) -> Result<toml::Value, AppError> {
    // Simulated TOML parsing
    if content.contains("invalid") {
        Err(AppError::ParseError(
            "Invalid TOML syntax at line 10".to_string(),
        ))
    } else {
        Ok(toml::Value::String("parsed".to_string()))
    }
}

fn extract_database_url(config: &toml::Value) -> Result<String, AppError> {
    // Simulated extraction
    if config.as_str() == Some("no_db") {
        Err(AppError::MissingField("database.url".to_string()))
    } else {
        Ok("postgresql://localhost/mydb".to_string())
    }
}

fn extract_api_key(config: &toml::Value) -> Result<String, AppError> {
    Ok("secret_key_123".to_string())
}

fn extract_max_connections(config: &toml::Value) -> Result<u32, AppError> {
    Ok(10)
}

fn validate_config(config: &Config) -> Result<(), AppError> {
    if config.max_connections == 0 {
        Err(AppError::ValidationError(
            "max_connections must be > 0".to_string(),
        ))
    } else if config.api_key.is_empty() {
        Err(AppError::ValidationError(
            "api_key cannot be empty".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn create_config(db_url: String, api_key: String, max_conn: u32) -> Config {
    Config {
        database_url: db_url,
        api_key,
        max_connections: max_conn,
    }
}

fn create_db_pool(config: &Config) -> Result<DbPool, AppError> {
    if config.database_url.contains("invalid") {
        Err(AppError::DatabaseError("Failed to connect".to_string()))
    } else {
        Ok(DbPool)
    }
}

// Fake toml module
mod toml {
    #[derive(Debug)]
    pub enum Value {
        String(String),
    }

    impl Value {
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s),
            }
        }
    }
}

// ============================================================================
// Effects with Context
// ============================================================================

// This is the key test: Does context chaining feel natural?
// Can we trace errors back through the call stack?

fn load_config_file(path: PathBuf) -> Effect<String, ContextError<AppError>, ()> {
    // Simulate file read
    Effect::from_fn(move |_env| {
        if path.to_str().unwrap().contains("missing") {
            Err(AppError::FileNotFound(path.clone()))
        } else if path.to_str().unwrap().contains("permission") {
            Err(AppError::FileNotFound(path.clone()))
        } else {
            Ok("database.url = \"postgresql://localhost/mydb\"\napi_key = \"secret\"".to_string())
        }
    })
    .context(format!("Reading config file: {}", path.display()))
}

fn parse_config_content(content: String) -> Effect<toml::Value, ContextError<AppError>, ()> {
    Effect::from_result(parse_toml(&content)).context("Parsing TOML configuration")
}

fn extract_config_values(toml: toml::Value) -> Effect<Config, ContextError<AppError>, ()> {
    Effect::from_result(extract_database_url(&toml).and_then(|db_url| {
        extract_api_key(&toml).and_then(|api_key| {
            extract_max_connections(&toml).map(|max_conn| create_config(db_url, api_key, max_conn))
        })
    }))
    .context("Extracting configuration values")
}

fn validate_config_effect(config: Config) -> Effect<Config, ContextError<AppError>, ()> {
    Effect::from_result(validate_config(&config).map(|_| config))
        .context("Validating configuration")
}

fn initialize_database(config: Config) -> Effect<DbPool, ContextError<AppError>, ()> {
    Effect::from_result(create_db_pool(&config))
        .context(format!("Connecting to database: {}", config.database_url))
}

// Full initialization pipeline with context at each step
fn initialize_app(config_path: PathBuf) -> Effect<AppState, ContextError<AppError>, ()> {
    load_config_file(config_path.clone())
        .and_then(parse_config_content)
        .and_then(extract_config_values)
        .and_then(validate_config_effect)
        .and_then(|config| {
            initialize_database(config.clone()).map(move |db_pool| AppState { config, db_pool })
        })
        .context(format!(
            "Initializing application with config: {}",
            config_path.display()
        ))
}

// Alternative: more granular contexts?
fn initialize_app_verbose(config_path: PathBuf) -> Effect<AppState, ContextError<AppError>, ()> {
    load_config_file(config_path.clone())
        .context("Step 1/4: Loading configuration file")
        .and_then(parse_config_content)
        .context("Step 2/4: Parsing configuration")
        .and_then(extract_config_values)
        .context("Step 3/4: Extracting configuration values")
        .and_then(validate_config_effect)
        .context("Step 4/4: Validating configuration")
        .and_then(|config| {
            initialize_database(config.clone()).map(move |db_pool| AppState { config, db_pool })
        })
        .context("Step 5/5: Initializing database connection")
        .context(format!(
            "Application initialization from {}",
            config_path.display()
        ))
}

// ============================================================================
// Usage
// ============================================================================

fn main() {
    println!("=== Error Context Chaining Test ===\n");

    // Test 1: Success case (minimal context)
    println!("Test 1: Successful initialization");
    let effect = initialize_app(PathBuf::from("/etc/myapp/config.toml"));

    match effect.run(&()) {
        Ok(state) => {
            println!("✓ App initialized successfully");
            println!("  Database: {}", state.config.database_url);
            println!("  Max connections: {}", state.config.max_connections);
        }
        Err(err) => {
            println!("✗ Initialization failed:\n{}", err);
        }
    }

    println!("\n---\n");

    // Test 2: File not found - see context chain
    println!("Test 2: Missing config file");
    let effect = initialize_app(PathBuf::from("/etc/myapp/missing.toml"));

    match effect.run(&()) {
        Ok(_) => println!("✓ Unexpected success"),
        Err(err) => {
            println!("✗ Initialization failed:\n{}", err);
            println!("\nContext trail:");
            for (i, ctx) in err.context_trail().iter().enumerate() {
                println!("  {} {}", "  ".repeat(i), ctx);
            }
        }
    }

    println!("\n---\n");

    // Test 3: Parse error - see where in the chain it failed
    println!("Test 3: Invalid TOML syntax");
    let effect = initialize_app(PathBuf::from("/etc/myapp/invalid.toml"));

    match effect.run(&()) {
        Ok(_) => println!("✓ Unexpected success"),
        Err(err) => {
            println!("✗ Initialization failed:\n{}", err);
        }
    }

    println!("\n---\n");

    // Test 4: Verbose context
    println!("Test 4: Verbose context (step-by-step)");
    let effect = initialize_app_verbose(PathBuf::from("/etc/myapp/config.toml"));

    match effect.run(&()) {
        Ok(_) => println!("✓ Success with verbose context"),
        Err(err) => {
            println!("✗ Failed with context:\n{}", err);
        }
    }

    // Ergonomics questions:
    // 1. Is .context() easy to add at each step?
    // 2. Is the context trail readable?
    // 3. Should context be required or optional?
    // 4. How verbose should contexts be?
    // 5. Should we show file:line automatically?
    // 6. Is ContextError<E> the right wrapper?
}

/* Expected output:

=== Error Context Chaining Test ===

Test 1: Successful initialization
✓ App initialized successfully
  Database: postgresql://localhost/mydb
  Max connections: 10

---

Test 2: Missing config file
✗ Initialization failed:
File not found: /etc/myapp/missing.toml
  -> Reading config file: /etc/myapp/missing.toml
  -> Initializing application with config: /etc/myapp/missing.toml

Context trail:
   Reading config file: /etc/myapp/missing.toml
     Initializing application with config: /etc/myapp/missing.toml

---

Test 3: Invalid TOML syntax
✗ Initialization failed:
Parse error: Invalid TOML syntax at line 10
  -> Parsing TOML configuration
  -> Initializing application with config: /etc/myapp/invalid.toml

---

Test 4: Verbose context (step-by-step)
✓ Success with verbose context

*/
