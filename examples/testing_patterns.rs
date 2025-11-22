//! Testing Patterns Example
//!
//! Demonstrates testing strategies for code using Validation and Effect.
//! Shows how the "pure core, imperative shell" pattern makes testing easy.
//!
//! Patterns covered:
//! - Testing pure validation functions
//! - Testing effects with mock environments
//! - Testing effect composition
//! - Verifying side effects through the environment

use std::sync::{Arc, Mutex};
use stillwater::{Effect, Validation};

// ==================== Testing Pure Validations ====================

/// Pure validation functions are trivial to test - no mocks needed!
fn validate_age(age: i32) -> Validation<i32, Vec<String>> {
    if (0..=150).contains(&age) {
        Validation::success(age)
    } else {
        Validation::failure(vec![format!("Invalid age: {}", age)])
    }
}

fn example_testing_validations() {
    println!("\n=== Example 1: Testing Pure Validations ===");
    println!("Pure validation functions are easy to test - just call them!");

    // Test valid age
    let result = validate_age(25);
    assert!(matches!(result, Validation::Success(25)));
    println!("✓ Valid age test passed");

    // Test invalid age
    let result = validate_age(-5);
    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 1);
            println!("✓ Invalid age test passed");
        }
        _ => panic!("Expected failure"),
    }

    // Test boundary conditions
    let result = validate_age(0);
    assert!(matches!(result, Validation::Success(0)));
    let result = validate_age(150);
    assert!(matches!(result, Validation::Success(150)));
    let result = validate_age(151);
    assert!(matches!(result, Validation::Failure(_)));
    println!("✓ Boundary condition tests passed");
}

// ==================== Testing Effects with Mock Environment ====================

/// Example service that we'll mock for testing
trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: String);
}

/// Production implementation (real database)
#[allow(dead_code)]
struct PostgresStorage {
    // connection details, etc.
}

#[allow(dead_code)]
impl Storage for PostgresStorage {
    fn get(&self, key: &str) -> Option<String> {
        // Real database query
        unimplemented!("Would query real database for key: {}", key)
    }

    fn set(&self, key: &str, value: String) {
        // Real database write
        unimplemented!("Would write to database: {} = {}", key, value)
    }
}

/// Test implementation (in-memory mock)
struct MockStorage {
    data: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn verify_contains(&self, key: &str, expected: &str) -> bool {
        self.data
            .lock()
            .unwrap()
            .get(key)
            .map(|v| v == expected)
            .unwrap_or(false)
    }
}

impl Storage for MockStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: String) {
        self.data.lock().unwrap().insert(key.to_string(), value);
    }
}

/// Test environment using mock
struct TestEnv {
    storage: MockStorage,
}

impl TestEnv {
    fn new() -> Self {
        Self {
            storage: MockStorage::new(),
        }
    }
}

impl AsRef<MockStorage> for TestEnv {
    fn as_ref(&self) -> &MockStorage {
        &self.storage
    }
}

/// Function to test - uses Effect
fn save_user_preference<Env: AsRef<MockStorage> + Sync + 'static>(
    user_id: u64,
    preference: String,
) -> Effect<(), String, Env> {
    Effect::from_fn(move |env: &Env| {
        let storage: &MockStorage = env.as_ref();
        let key = format!("user:{}:preference", user_id);
        storage.set(&key, preference.clone());
        Ok(())
    })
}

fn load_user_preference<Env: AsRef<MockStorage> + Sync + 'static>(
    user_id: u64,
) -> Effect<Option<String>, String, Env> {
    Effect::from_fn(move |env: &Env| {
        let storage: &MockStorage = env.as_ref();
        let key = format!("user:{}:preference", user_id);
        Ok(storage.get(&key))
    })
}

async fn example_testing_effects() {
    println!("\n=== Example 2: Testing Effects with Mock Environment ===");
    println!("Effects are easy to test with mock environments!");

    let env = TestEnv::new();

    // Test saving
    save_user_preference(42, "dark_mode".to_string())
        .run(&env)
        .await
        .unwrap();
    assert!(env
        .storage
        .verify_contains("user:42:preference", "dark_mode"));
    println!("✓ Save effect test passed");

    // Test loading
    let result = load_user_preference(42).run(&env).await.unwrap();
    assert_eq!(result, Some("dark_mode".to_string()));
    println!("✓ Load effect test passed");

    // Test loading non-existent
    let result = load_user_preference(99).run(&env).await.unwrap();
    assert_eq!(result, None);
    println!("✓ Load non-existent test passed");
}

// ==================== Testing Effect Composition ====================

/// Combined operation that we want to test
fn update_and_verify<Env: AsRef<MockStorage> + Sync + 'static>(
    user_id: u64,
    new_preference: String,
) -> Effect<bool, String, Env> {
    save_user_preference(user_id, new_preference.clone())
        .and_then(move |_| load_user_preference(user_id))
        .map(move |loaded| loaded == Some(new_preference.clone()))
}

async fn example_testing_composition() {
    println!("\n=== Example 3: Testing Effect Composition ===");
    println!("Composed effects can be tested end-to-end!");

    let env = TestEnv::new();

    let success = update_and_verify(10, "compact_view".to_string())
        .run(&env)
        .await
        .unwrap();
    assert!(success);
    println!("✓ Composition test passed");
}

// ==================== Verifying Side Effects ====================

/// Service that tracks calls for verification
struct SpyEmailService {
    sent: Arc<Mutex<Vec<(String, String)>>>,
}

impl SpyEmailService {
    fn new() -> Self {
        Self {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn send(&self, to: String, message: String) {
        self.sent.lock().unwrap().push((to, message));
    }

    fn verify_sent(&self, to: &str, message: &str) -> bool {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .any(|(t, m)| t == to && m == message)
    }

    fn count_sent(&self) -> usize {
        self.sent.lock().unwrap().len()
    }
}

struct EmailEnv {
    email: SpyEmailService,
}

impl EmailEnv {
    fn new() -> Self {
        Self {
            email: SpyEmailService::new(),
        }
    }
}

impl AsRef<SpyEmailService> for EmailEnv {
    fn as_ref(&self) -> &SpyEmailService {
        &self.email
    }
}

fn send_welcome_email<Env: AsRef<SpyEmailService> + Sync + 'static>(
    email: String,
    name: String,
) -> Effect<(), String, Env> {
    Effect::from_fn(move |env: &Env| {
        let service: &SpyEmailService = env.as_ref();
        let message = format!("Welcome, {}!", name);
        service.send(email.clone(), message);
        Ok(())
    })
}

async fn example_testing_side_effects() {
    println!("\n=== Example 4: Verifying Side Effects ===");
    println!("Use spy/mock services to verify side effects!");

    let env = EmailEnv::new();

    send_welcome_email("alice@example.com".to_string(), "Alice".to_string())
        .run(&env)
        .await
        .unwrap();

    assert!(env
        .email
        .verify_sent("alice@example.com", "Welcome, Alice!"));
    assert_eq!(env.email.count_sent(), 1);
    println!("✓ Side effect verification passed");
}

// ==================== Testing Error Cases ====================

fn divide<Env: Sync + 'static>(a: i32, b: i32) -> Effect<i32, String, Env> {
    Effect::from_fn(move |_env: &Env| {
        if b == 0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    })
}

async fn example_testing_errors() {
    println!("\n=== Example 5: Testing Error Cases ===");

    // Test success case
    let result = divide::<()>(10, 2).run(&()).await;
    assert_eq!(result, Ok(5));
    println!("✓ Success case test passed");

    // Test error case
    let result = divide::<()>(10, 0).run(&()).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Division by zero");
    println!("✓ Error case test passed");
}

// ==================== Property-Based Testing ====================

fn example_property_based_testing() {
    println!("\n=== Example 6: Property-Based Testing ===");
    println!("Testing properties that should always hold:");

    // Property: validation should be consistent
    for age in -10..200 {
        let result = validate_age(age);
        let is_valid = (0..=150).contains(&age);

        match result {
            Validation::Success(_) => assert!(is_valid, "Age {} should be invalid", age),
            Validation::Failure(_) => assert!(!is_valid, "Age {} should be valid", age),
        }
    }
    println!("✓ Validation consistency property holds");

    // Property: error accumulation should preserve all errors
    fn multi_error_validation(n: i32) -> Validation<(), Vec<String>> {
        let v1 = if n < 0 {
            Validation::failure(vec!["negative".to_string()])
        } else {
            Validation::success(())
        };

        let v2 = if n % 2 != 0 {
            Validation::failure(vec!["odd".to_string()])
        } else {
            Validation::success(())
        };

        Validation::<((), ()), Vec<String>>::all((v1, v2)).map(|_| ())
    }

    // Test that all errors are accumulated
    let result = multi_error_validation(-3);
    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"negative".to_string()));
            assert!(errors.contains(&"odd".to_string()));
        }
        _ => panic!("Expected failure with multiple errors"),
    }
    println!("✓ Error accumulation property holds");
}

// ==================== Test Organization Example ====================

fn example_test_organization() {
    println!("\n=== Example 7: Test Organization ===");
    println!("Organizing tests by behavior:");

    println!("\n  Unit Tests:");
    println!("  - Test pure functions in isolation");
    println!("  - Fast, no dependencies");
    println!("  - Example: validate_age()");

    println!("\n  Integration Tests:");
    println!("  - Test effect composition with mocks");
    println!("  - Verify interactions between components");
    println!("  - Example: update_and_verify()");

    println!("\n  Contract Tests:");
    println!("  - Verify mock behavior matches real services");
    println!("  - Run same tests against mock and real impl");
    println!("  - Example: Storage trait tests");

    println!("\n✓ All test organization patterns demonstrated");
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Testing Patterns Examples");
    println!("=========================");
    println!();
    println!("The pure core/imperative shell pattern makes testing easy:");
    println!("- Pure functions: Just call them");
    println!("- Effects: Inject mock environments");
    println!("- Side effects: Use spy services");

    example_testing_validations();
    example_testing_effects().await;
    example_testing_composition().await;
    example_testing_side_effects().await;
    example_testing_errors().await;
    example_property_based_testing();
    example_test_organization();

    println!("\n=== All examples completed successfully! ===");
}
