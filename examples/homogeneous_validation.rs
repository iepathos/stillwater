//! Homogeneous Validation Example
//!
//! Demonstrates type-safe validation for discriminated unions before combining.
//! Shows how to prevent runtime panics when combining enum variants that cannot
//! be safely mixed.
//!
//! Includes:
//! - Basic homogeneous validation
//! - MapReduce aggregation pattern
//! - JSON-like configuration merging
//! - Database query result validation
//! - Error accumulation with custom error types

use std::mem::discriminant;
use stillwater::validation::homogeneous::{
    combine_homogeneous, validate_homogeneous, DiscriminantName, TypeMismatchError,
};
use stillwater::{Semigroup, Validation};

// ==================== Example 1: Basic Aggregation ====================

/// Example 1: Basic homogeneous validation with aggregates
///
/// Demonstrates the core problem: enum variants that can combine within
/// their own type but not across types.
fn example_basic_aggregation() {
    println!("\n=== Example 1: Basic Aggregation ===");

    #[derive(Clone, Debug, PartialEq)]
    enum Aggregate {
        Sum(f64),
        Count(usize),
    }

    impl Semigroup for Aggregate {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (Aggregate::Sum(a), Aggregate::Sum(b)) => Aggregate::Sum(a + b),
                (Aggregate::Count(a), Aggregate::Count(b)) => Aggregate::Count(a + b),
                _ => unreachable!("Call validate_homogeneous first"),
            }
        }
    }

    impl DiscriminantName for Aggregate {
        fn discriminant_name(&self) -> &'static str {
            match self {
                Aggregate::Sum(_) => "Sum",
                Aggregate::Count(_) => "Count",
            }
        }
    }

    // Success case: all same type
    let homogeneous = vec![
        Aggregate::Count(5),
        Aggregate::Count(3),
        Aggregate::Count(2),
    ];

    let result = combine_homogeneous(homogeneous, discriminant, TypeMismatchError::new);

    match result {
        Validation::Success(combined) => {
            println!("✓ Combined successfully: {:?}", combined);
            assert_eq!(combined, Aggregate::Count(10));
        }
        Validation::Failure(errors) => {
            println!("✗ Validation failed: {:?}", errors);
        }
    }

    // Failure case: mixed types
    let heterogeneous = vec![
        Aggregate::Count(5),
        Aggregate::Sum(10.0), // Wrong type!
        Aggregate::Count(3),
    ];

    let result = combine_homogeneous(heterogeneous, discriminant, TypeMismatchError::new);

    match result {
        Validation::Success(_) => {
            println!("✗ Should have failed validation");
        }
        Validation::Failure(errors) => {
            println!("✓ Validation caught type mismatch:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Example 2: MapReduce Pattern ====================

/// Example 2: MapReduce aggregation with worker results
///
/// Demonstrates validating results from parallel workers before aggregating.
fn example_mapreduce_aggregation() {
    println!("\n=== Example 2: MapReduce Aggregation ===");

    #[derive(Clone, Debug, PartialEq)]
    enum WorkerResult {
        Count(usize),
    }

    impl Semigroup for WorkerResult {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (WorkerResult::Count(a), WorkerResult::Count(b)) => WorkerResult::Count(a + b),
            }
        }
    }

    impl DiscriminantName for WorkerResult {
        fn discriminant_name(&self) -> &'static str {
            match self {
                WorkerResult::Count(_) => "Count",
            }
        }
    }

    // Simulate results from 5 parallel workers
    let worker_results = vec![
        WorkerResult::Count(100),
        WorkerResult::Count(150),
        WorkerResult::Count(200),
        WorkerResult::Count(175),
        WorkerResult::Count(125),
    ];

    println!("Workers returned: {:?}", worker_results);

    let aggregated = combine_homogeneous(worker_results, discriminant, |idx, got, expected| {
        format!(
            "Worker {} returned {}, expected {}",
            idx,
            got.discriminant_name(),
            expected.discriminant_name()
        )
    });

    match aggregated {
        Validation::Success(result) => {
            println!("✓ Aggregated result: {:?}", result);
            assert_eq!(result, WorkerResult::Count(750));
        }
        Validation::Failure(errors) => {
            println!("✗ Aggregation failed:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Example 3: Configuration Merging ====================

/// Example 3: JSON-like configuration validation
///
/// Demonstrates validating that all config sources have the same type
/// before merging them together.
fn example_config_merging() {
    println!("\n=== Example 3: Configuration Merging ===");

    #[derive(Clone, Debug, PartialEq)]
    enum ConfigValue {
        Object(Vec<(String, String)>),
        String(String),
    }

    impl Semigroup for ConfigValue {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (ConfigValue::Object(mut a), ConfigValue::Object(b)) => {
                    a.extend(b);
                    ConfigValue::Object(a)
                }
                (ConfigValue::String(a), ConfigValue::String(b)) => {
                    ConfigValue::String(format!("{}{}", a, b))
                }
                _ => unreachable!("Validated before combining"),
            }
        }
    }

    impl DiscriminantName for ConfigValue {
        fn discriminant_name(&self) -> &'static str {
            match self {
                ConfigValue::Object(_) => "Object",
                ConfigValue::String(_) => "String",
            }
        }
    }

    // Success case: all configs are objects
    let configs = vec![
        ConfigValue::Object(vec![("host".into(), "localhost".into())]),
        ConfigValue::Object(vec![("port".into(), "8080".into())]),
        ConfigValue::Object(vec![("debug".into(), "true".into())]),
    ];

    println!("Merging {} config sources...", configs.len());

    let result = combine_homogeneous(configs, discriminant, |idx, got, expected| {
        format!(
            "Config source {} is {}, expected {}",
            idx,
            got.discriminant_name(),
            expected.discriminant_name()
        )
    });

    match result {
        Validation::Success(merged) => {
            println!("✓ Merged config: {:?}", merged);
        }
        Validation::Failure(errors) => {
            println!("✗ Config merge failed:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Failure case: mixed config types
    let bad_configs = vec![
        ConfigValue::Object(vec![("host".into(), "localhost".into())]),
        ConfigValue::String("8080".into()), // Wrong type!
        ConfigValue::Object(vec![("debug".into(), "true".into())]),
    ];

    println!(
        "\nMerging {} config sources (with type error)...",
        bad_configs.len()
    );

    let result = combine_homogeneous(bad_configs, discriminant, TypeMismatchError::new);

    match result {
        Validation::Success(_) => {
            println!("✗ Should have failed validation");
        }
        Validation::Failure(errors) => {
            println!("✓ Validation caught config type mismatch:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Example 4: Database Query Results ====================

/// Example 4: Database query result validation
///
/// Demonstrates validating that all shards return the same result type.
fn example_database_sharding() {
    println!("\n=== Example 4: Database Sharding ===");

    #[derive(Clone, Debug, PartialEq)]
    enum QueryResult {
        Rows(Vec<String>),
    }

    impl Semigroup for QueryResult {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (QueryResult::Rows(mut a), QueryResult::Rows(b)) => {
                    a.extend(b);
                    QueryResult::Rows(a)
                }
            }
        }
    }

    impl DiscriminantName for QueryResult {
        fn discriminant_name(&self) -> &'static str {
            match self {
                QueryResult::Rows(_) => "Rows",
            }
        }
    }

    // Simulate query results from 3 database shards
    let shard_results = vec![
        QueryResult::Rows(vec!["row1".into(), "row2".into()]),
        QueryResult::Rows(vec!["row3".into(), "row4".into()]),
        QueryResult::Rows(vec!["row5".into()]),
    ];

    println!("Query returned results from {} shards", shard_results.len());

    let combined = combine_homogeneous(shard_results, discriminant, |idx, got, expected| {
        format!(
            "Shard {} returned {}, expected {}",
            idx,
            got.discriminant_name(),
            expected.discriminant_name()
        )
    });

    match combined {
        Validation::Success(result) => {
            println!("✓ Combined query result: {:?}", result);
            let QueryResult::Rows(rows) = result;
            println!("  Total rows: {}", rows.len());
        }
        Validation::Failure(errors) => {
            println!("✗ Query combination failed:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Example 5: Error Accumulation ====================

/// Example 5: Error accumulation with multiple mismatches
///
/// Demonstrates that validation reports ALL type mismatches, not just the first.
fn example_error_accumulation() {
    println!("\n=== Example 5: Error Accumulation ===");

    #[derive(Clone, Debug, PartialEq)]
    enum Value {
        Int(i64),
        Float(f64),
        Text(String),
    }

    impl DiscriminantName for Value {
        fn discriminant_name(&self) -> &'static str {
            match self {
                Value::Int(_) => "Int",
                Value::Float(_) => "Float",
                Value::Text(_) => "Text",
            }
        }
    }

    // Collection with multiple type mismatches
    let mixed_values = vec![
        Value::Int(1),
        Value::Float(2.0), // Error at index 1
        Value::Int(3),
        Value::Text("x".into()), // Error at index 3
        Value::Int(5),
        Value::Float(6.0), // Error at index 5
    ];

    println!("Validating {} values...", mixed_values.len());

    let result = validate_homogeneous(mixed_values, discriminant, TypeMismatchError::new);

    match result {
        Validation::Success(_) => {
            println!("✗ Should have failed validation");
        }
        Validation::Failure(errors) => {
            println!("✓ Found {} type mismatches:", errors.len());
            for error in &errors {
                println!("  - {}", error);
            }
            assert_eq!(errors.len(), 3); // All 3 mismatches reported!
        }
    }
}

// ==================== Main ====================

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║         Stillwater Homogeneous Validation Examples           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    example_basic_aggregation();
    example_mapreduce_aggregation();
    example_config_merging();
    example_database_sharding();
    example_error_accumulation();

    println!("\n✓ All examples completed!");
}
