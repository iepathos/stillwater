//! Comprehensive tests and examples for testing utilities
//!
//! This test suite demonstrates various patterns for testing Stillwater code.

use stillwater::prelude::*;
use stillwater::{assert_failure, assert_success, assert_validation_errors};

// Example domain types for testing
#[derive(Debug, Clone, PartialEq)]
struct User {
    email: String,
    age: i32,
}

#[derive(Debug, Clone, PartialEq)]
struct Database {
    users: Vec<User>,
}

impl Database {
    fn new() -> Self {
        Self { users: Vec::new() }
    }

    fn with_users(users: Vec<User>) -> Self {
        Self { users }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Config {
    min_age: i32,
    debug: bool,
}

impl Config {
    fn test_config() -> Self {
        Self {
            min_age: 18,
            debug: true,
        }
    }
}

// Validation functions
fn validate_email(email: &str) -> Validation<String, Vec<String>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec!["Email must contain @ and .".to_string()])
    }
}

fn validate_age(age: i32) -> Validation<i32, Vec<String>> {
    if age >= 18 {
        Validation::success(age)
    } else {
        Validation::failure(vec!["Must be 18 or older".to_string()])
    }
}

fn validate_user(email: &str, age: i32) -> Validation<User, Vec<String>> {
    Validation::<(String, i32), Vec<String>>::all((validate_email(email), validate_age(age)))
        .map(|(email, age)| User { email, age })
}

// Tests demonstrating assertion macros

#[test]
fn test_assert_success_with_valid_email() {
    let result = validate_email("user@example.com");
    assert_success!(result);
}

#[test]
fn test_assert_failure_with_invalid_email() {
    let result = validate_email("invalid");
    assert_failure!(result);
}

#[test]
fn test_assert_validation_errors_with_specific_error() {
    let result = validate_email("invalid");
    assert_validation_errors!(result, vec!["Email must contain @ and .".to_string()]);
}

#[test]
fn test_assert_success_with_age_validation() {
    let result = validate_age(25);
    assert_success!(result);
}

#[test]
fn test_assert_failure_with_underage() {
    let result = validate_age(15);
    assert_failure!(result);
}

#[test]
fn test_accumulating_multiple_errors() {
    let result = validate_user("invalid", 15);
    assert_failure!(result);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 2);
            assert!(errors.contains(&"Email must contain @ and .".to_string()));
            assert!(errors.contains(&"Must be 18 or older".to_string()));
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_successful_user_validation() {
    let result = validate_user("user@example.com", 25);
    assert_success!(result);

    match result {
        Validation::Success(user) => {
            assert_eq!(user.email, "user@example.com");
            assert_eq!(user.age, 25);
        }
        _ => panic!("Expected success"),
    }
}

// Tests demonstrating MockEnv builder

#[test]
#[allow(clippy::let_unit_value, clippy::unit_cmp)]
fn test_mock_env_empty() {
    let env = MockEnv::new().build();
    assert_eq!(env, ());
}

#[test]
fn test_mock_env_with_database() {
    let env = MockEnv::new().with(Database::new).build();

    let (_, db) = env;
    assert_eq!(db.users.len(), 0);
}

#[test]
fn test_mock_env_with_config_and_database() {
    let env = MockEnv::new()
        .with(Config::test_config)
        .with(Database::new)
        .build();

    let ((_, config), db) = env;
    assert_eq!(config.min_age, 18);
    assert!(config.debug);
    assert_eq!(db.users.len(), 0);
}

#[test]
fn test_mock_env_with_multiple_dependencies() {
    let env = MockEnv::new()
        .with(Config::test_config)
        .with(|| {
            Database::with_users(vec![User {
                email: "test@example.com".to_string(),
                age: 25,
            }])
        })
        .with(|| "auth_token_123")
        .build();

    let (((_, config), db), token) = env;
    assert_eq!(config.min_age, 18);
    assert_eq!(db.users.len(), 1);
    assert_eq!(token, "auth_token_123");
}

// Tests demonstrating validation composition

#[test]
fn test_validation_and_combinator() {
    let v1 = validate_email("user@example.com");
    let v2 = validate_age(25);

    let result = v1.and(v2);
    assert_success!(result);

    match result {
        Validation::Success((email, age)) => {
            assert_eq!(email, "user@example.com");
            assert_eq!(age, 25);
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_validation_and_accumulates_errors() {
    let v1 = validate_email("invalid");
    let v2 = validate_age(15);

    let result = v1.and(v2);
    assert_failure!(result);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 2);
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_validation_map() {
    let result = validate_email("user@example.com").map(|email| email.to_uppercase());

    assert_success!(result);
    match result {
        Validation::Success(email) => {
            assert_eq!(email, "USER@EXAMPLE.COM");
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_validation_map_error() {
    let result = validate_email("invalid").map_err(|errors: Vec<String>| {
        errors
            .iter()
            .map(|e| e.to_uppercase())
            .collect::<Vec<String>>()
    });

    assert_failure!(result);
    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors[0], "EMAIL MUST CONTAIN @ AND .");
        }
        _ => panic!("Expected failure"),
    }
}

// Tests demonstrating tuple validation

#[test]
fn test_validate_all_with_success() {
    use stillwater::validation::ValidateAll;

    let result = (
        Validation::<_, Vec<String>>::success(1),
        Validation::<_, Vec<String>>::success(2),
        Validation::<_, Vec<String>>::success(3),
    )
        .validate_all();

    assert_success!(result);
    match result {
        Validation::Success((a, b, c)) => {
            assert_eq!(a, 1);
            assert_eq!(b, 2);
            assert_eq!(c, 3);
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_validate_all_accumulates_errors() {
    use stillwater::validation::ValidateAll;

    let result = (
        Validation::<i32, _>::failure(vec!["error1"]),
        Validation::<i32, _>::failure(vec!["error2"]),
        Validation::<i32, _>::failure(vec!["error3"]),
    )
        .validate_all();

    assert_failure!(result);
    assert_validation_errors!(result, vec!["error1", "error2", "error3"]);
}

// Integration test with Effect

#[test]
fn test_effect_with_mock_env() {
    fn get_user_count(env: &Database) -> i32 {
        env.users.len() as i32
    }

    let env = MockEnv::new()
        .with(|| {
            Database::with_users(vec![
                User {
                    email: "user1@example.com".to_string(),
                    age: 25,
                },
                User {
                    email: "user2@example.com".to_string(),
                    age: 30,
                },
            ])
        })
        .build();

    let (_, db) = env;
    let count = get_user_count(&db);
    assert_eq!(count, 2);
}

// Property-based testing examples (only runs with proptest feature)

#[cfg(feature = "proptest")]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validation_success_is_always_success(value: i32) {
            let val = Validation::<_, Vec<String>>::success(value);
            assert!(val.is_success());
        }

        #[test]
        fn test_validation_failure_is_always_failure(errors: Vec<String>) {
            prop_assume!(!errors.is_empty());
            let val = Validation::<i32, _>::failure(errors);
            assert!(val.is_failure());
        }

        #[test]
        fn test_validation_map_preserves_success(value: i32) {
            let val = Validation::<_, Vec<String>>::success(value);
            let mapped = val.map(|x| x * 2);
            assert_success!(mapped);
        }

        #[test]
        fn test_validation_map_preserves_failure(errors: Vec<String>) {
            prop_assume!(!errors.is_empty());
            let val = Validation::<i32, _>::failure(errors.clone());
            let mapped = val.map(|x| x * 2);
            assert_failure!(mapped);
        }

        #[test]
        fn test_email_validation_with_at_and_dot(
            local in "[a-z]{1,10}",
            domain in "[a-z]{1,10}",
            tld in "[a-z]{2,5}"
        ) {
            let email = format!("{}@{}.{}", local, domain, tld);
            let result = validate_email(&email);
            assert_success!(result);
        }

        #[test]
        fn test_age_validation_above_threshold(age in 18..100i32) {
            let result = validate_age(age);
            assert_success!(result);
        }

        #[test]
        fn test_age_validation_below_threshold(age in 0..18i32) {
            let result = validate_age(age);
            assert_failure!(result);
        }
    }
}
