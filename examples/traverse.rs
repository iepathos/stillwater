//! Traverse and Sequence Example
//!
//! Demonstrates traverse and sequence utilities for working with collections.
//! Shows practical patterns including:
//! - Traversing collections with validation functions
//! - Sequencing collections of validations
//! - Traversing with effect functions
//! - Batch processing patterns
//! - Error accumulation with traverse
//! - Practical use cases

use stillwater::prelude::*;
use stillwater::traverse::{sequence, sequence_effect, traverse, traverse_effect};
use stillwater::effect::{fail, from_fn, pure};
use stillwater::BoxedEffect;

// ==================== Basic Traverse ====================

/// Example 1: Basic traverse with validation
///
/// Demonstrates using traverse to apply a validation function to each element.
fn example_basic_traverse() {
    println!("\n=== Example 1: Basic Traverse ===");

    fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
        s.parse()
            .map(Validation::success)
            .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid number: {}", s)]))
    }

    // All valid numbers
    let numbers = vec!["1", "2", "3", "42"];
    let result = traverse(numbers, parse_number);
    println!("Parse valid numbers:");
    match result {
        Validation::Success(nums) => println!("  Parsed: {:?}", nums),
        Validation::Failure(errors) => println!("  Errors: {:?}", errors),
    }

    // Mix of valid and invalid - accumulates ALL errors
    let mixed = vec!["1", "invalid", "3", "bad"];
    let result = traverse(mixed, parse_number);
    println!("\nParse mixed input:");
    match result {
        Validation::Success(nums) => println!("  Parsed: {:?}", nums),
        Validation::Failure(errors) => {
            println!("  {} errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Sequence ====================

/// Example 2: Sequencing existing validations
///
/// Demonstrates using sequence to convert Vec<Validation<T, E>> to Validation<Vec<T>, E>.
fn example_sequence() {
    println!("\n=== Example 2: Sequence ===");

    // Create a collection of validations
    let validations = vec![
        Validation::<i32, Vec<String>>::success(1),
        Validation::success(2),
        Validation::success(3),
    ];

    let result = sequence(validations);
    println!("Sequence all success:");
    match result {
        Validation::Success(nums) => println!("  Values: {:?}", nums),
        Validation::Failure(errors) => println!("  Errors: {:?}", errors),
    }

    // Mix of success and failure
    let mixed_validations = vec![
        Validation::success(1),
        Validation::failure(vec!["error1".to_string()]),
        Validation::success(3),
        Validation::failure(vec!["error2".to_string()]),
    ];

    let result = sequence(mixed_validations);
    println!("\nSequence with failures:");
    match result {
        Validation::Success(nums) => println!("  Values: {:?}", nums),
        Validation::Failure(errors) => {
            println!("  Accumulated {} errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== User Registration Example ====================

/// Example 3: Validating multiple user registrations
///
/// Demonstrates a practical use case: batch user registration with validation.
fn example_batch_user_validation() {
    println!("\n=== Example 3: Batch User Validation ===");

    #[derive(Debug)]
    struct User {
        email: String,
        age: i32,
    }

    #[derive(Debug)]
    struct UserInput {
        email: String,
        age: i32,
    }

    fn validate_email(email: &str) -> Validation<String, Vec<String>> {
        if email.contains('@') && email.len() > 3 {
            Validation::success(email.to_string())
        } else {
            Validation::failure(vec![format!("Invalid email: {}", email)])
        }
    }

    fn validate_age(age: i32) -> Validation<i32, Vec<String>> {
        if (18..=120).contains(&age) {
            Validation::success(age)
        } else {
            Validation::failure(vec![format!("Invalid age: {} (must be 18-120)", age)])
        }
    }

    fn validate_user(input: UserInput) -> Validation<User, Vec<String>> {
        Validation::<(String, i32), Vec<String>>::all((
            validate_email(&input.email),
            validate_age(input.age),
        ))
        .map(|(email, age)| User { email, age })
    }

    let inputs = vec![
        UserInput {
            email: "alice@example.com".to_string(),
            age: 25,
        },
        UserInput {
            email: "bob@test.com".to_string(),
            age: 30,
        },
        UserInput {
            email: "charlie@mail.org".to_string(),
            age: 22,
        },
    ];

    println!("Validating batch of users:");
    let result = traverse(inputs, validate_user);
    match result {
        Validation::Success(users) => {
            println!("  All {} users valid:", users.len());
            for user in users {
                println!("    - {} (age {})", user.email, user.age);
            }
        }
        Validation::Failure(errors) => {
            println!("  Validation failed:");
            for error in errors {
                println!("    - {}", error);
            }
        }
    }

    // Now with some invalid users
    let mixed_inputs = vec![
        UserInput {
            email: "alice@example.com".to_string(),
            age: 25,
        },
        UserInput {
            email: "invalid".to_string(),
            age: 15,
        },
        UserInput {
            email: "bob@test.com".to_string(),
            age: 150,
        },
    ];

    println!("\nValidating batch with errors:");
    let result = traverse(mixed_inputs, validate_user);
    match result {
        Validation::Success(users) => println!("  All users valid: {:?}", users),
        Validation::Failure(errors) => {
            println!("  {} validation errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Effect Traverse ====================

/// Example 4: Traverse with effects
///
/// Demonstrates using traverse_effect for batch processing with effects.
#[tokio::main]
async fn example_effect_traverse() {
    println!("\n=== Example 4: Effect Traverse ===");

    fn process_number(x: i32) -> BoxedEffect<i32, String, ()> {
        pure(x * 2).boxed()
    }

    let numbers = vec![1, 2, 3, 4, 5];
    println!("Processing numbers with effect:");
    let effect = traverse_effect(numbers, process_number);
    match effect.run_standalone().await {
        Ok(results) => println!("  Results: {:?}", results),
        Err(error) => println!("  Error: {}", error),
    }

    // With validation
    fn validate_and_process(x: i32) -> BoxedEffect<i32, String, ()> {
        from_fn(move |_: &()| {
            if x > 0 {
                Ok(x * x)
            } else {
                Err(format!("Negative number: {}", x))
            }
        }).boxed()
    }

    let mixed = vec![1, 2, -3, 4];
    println!("\nProcessing with validation (fail-fast):");
    let effect = traverse_effect(mixed, validate_and_process);
    match effect.run_standalone().await {
        Ok(results) => println!("  Results: {:?}", results),
        Err(error) => println!("  Error (stopped at first): {}", error),
    }
}

// ==================== Batch File Processing ====================

/// Example 5: Simulated batch file processing
///
/// Demonstrates a practical pattern: processing multiple files.
#[tokio::main]
async fn example_batch_file_processing() {
    println!("\n=== Example 5: Batch File Processing ===");

    #[derive(Debug, Clone)]
    struct FileContent {
        path: String,
        lines: usize,
    }

    // Simulate reading a file
    fn read_file(path: String) -> BoxedEffect<FileContent, String, ()> {
        from_fn(move |_: &()| {
            // In real code, this would actually read files
            if path.ends_with(".txt") {
                Ok(FileContent {
                    path: path.clone(),
                    lines: 100, // Simulated
                })
            } else {
                Err(format!("Not a text file: {}", path))
            }
        }).boxed()
    }

    let files = vec![
        "file1.txt".to_string(),
        "file2.txt".to_string(),
        "file3.txt".to_string(),
    ];

    println!("Reading files:");
    let effect = traverse_effect(files, read_file);
    match effect.run_standalone().await {
        Ok(contents) => {
            println!("  Read {} files:", contents.len());
            for content in contents {
                println!("    - {} ({} lines)", content.path, content.lines);
            }
        }
        Err(error) => println!("  Error: {}", error),
    }

    let mixed_files = vec![
        "file1.txt".to_string(),
        "image.png".to_string(),
        "file2.txt".to_string(),
    ];

    println!("\nReading mixed files (fail-fast):");
    let effect = traverse_effect(mixed_files, read_file);
    match effect.run_standalone().await {
        Ok(contents) => println!("  Read: {:?}", contents),
        Err(error) => println!("  Error: {}", error),
    }
}

// ==================== Sequence Effect ====================

/// Example 6: Sequencing effects
///
/// Demonstrates using sequence_effect to convert Vec<Effect> to Effect<Vec>.
#[tokio::main]
async fn example_sequence_effect() {
    println!("\n=== Example 6: Sequence Effect ===");

    // Create a collection of effects
    let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
        pure(1).boxed(),
        pure(2).boxed(),
        pure(3).boxed(),
    ];

    println!("Sequence pure effects:");
    let result_effect = sequence_effect(effects);
    match result_effect.run_standalone().await {
        Ok(values) => println!("  Values: {:?}", values),
        Err(error) => println!("  Error: {}", error),
    }

    // Mix with failure
    let mixed_effects: Vec<BoxedEffect<i32, String, ()>> = vec![
        pure(1).boxed(),
        fail("something went wrong".to_string()).boxed(),
        pure(3).boxed(),
    ];

    println!("\nSequence with failure (fail-fast):");
    let result_effect = sequence_effect(mixed_effects);
    match result_effect.run_standalone().await {
        Ok(values) => println!("  Values: {:?}", values),
        Err(error) => println!("  Error: {}", error),
    }
}

// ==================== Practical Pattern: Config Validation ====================

/// Example 7: Validating configuration entries
///
/// Demonstrates validating a collection of configuration key-value pairs.
fn example_config_validation() {
    println!("\n=== Example 7: Config Validation ===");

    #[derive(Debug)]
    struct ConfigEntry {
        key: String,
        value: String,
    }

    fn validate_config_entry(key: &str, value: &str) -> Validation<ConfigEntry, Vec<String>> {
        let mut errors = Vec::new();

        if key.is_empty() {
            errors.push("Config key cannot be empty".to_string());
        }

        if value.is_empty() {
            errors.push(format!("Config value for '{}' cannot be empty", key));
        }

        if !errors.is_empty() {
            Validation::failure(errors)
        } else {
            Validation::success(ConfigEntry {
                key: key.to_string(),
                value: value.to_string(),
            })
        }
    }

    let config_pairs = vec![
        ("database.host", "localhost"),
        ("database.port", "5432"),
        ("api.timeout", "30"),
    ];

    println!("Validating config:");
    let result = traverse(config_pairs, |(k, v)| validate_config_entry(k, v));
    match result {
        Validation::Success(entries) => {
            println!("  All {} entries valid:", entries.len());
            for entry in entries {
                println!("    - {}: {}", entry.key, entry.value);
            }
        }
        Validation::Failure(errors) => {
            println!("  Validation failed:");
            for error in errors {
                println!("    - {}", error);
            }
        }
    }

    // With invalid entries
    let invalid_config = vec![("", "value"), ("key", ""), ("valid.key", "valid.value")];

    println!("\nValidating invalid config:");
    let result = traverse(invalid_config, |(k, v)| validate_config_entry(k, v));
    match result {
        Validation::Success(entries) => println!("  All entries valid: {:?}", entries),
        Validation::Failure(errors) => {
            println!("  {} validation errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Main ====================

fn main() {
    println!("Traverse and Sequence Examples");
    println!("===============================");

    example_basic_traverse();
    example_sequence();
    example_batch_user_validation();
    example_config_validation();

    println!("\n--- Async Examples ---");
    example_effect_traverse();
    example_batch_file_processing();
    example_sequence_effect();

    println!("\n=== All examples completed successfully! ===");
}
