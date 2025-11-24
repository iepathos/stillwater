//! Monoid and Semigroup Example
//!
//! Demonstrates the Monoid trait for types with identity elements.
//! Shows practical patterns including:
//! - Semigroup trait for combining values
//! - Monoid trait extending Semigroup with identity
//! - Numeric monoids (Sum, Product)
//! - Collection monoids (Vec, String, Option)
//! - Using fold_all and reduce for aggregation
//! - Real-world use cases

use stillwater::monoid::{fold_all, reduce, Max, Min, Product, Sum};
use stillwater::{Monoid, Semigroup};

// ==================== Basic Semigroup ====================

/// Example 1: Semigroup - Combining values
///
/// Demonstrates the foundational Semigroup trait for associative combination.
fn example_semigroup() {
    println!("\n=== Example 1: Semigroup - Combining Values ===");

    // Vec is a semigroup - concatenation
    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5];
    let combined = v1.combine(v2);
    println!("Vec combine: {:?}", combined);

    // String is a semigroup - concatenation
    let s1 = "Hello, ".to_string();
    let s2 = "World!".to_string();
    let greeting = s1.combine(s2);
    println!("String combine: {}", greeting);

    // Tuples combine element-wise
    let t1 = (vec![1], "foo".to_string());
    let t2 = (vec![2], "bar".to_string());
    let t_combined = t1.combine(t2);
    println!("Tuple combine: {:?}", t_combined);
}

// ==================== Monoid Identity ====================

/// Example 2: Monoid - Identity elements
///
/// Demonstrates how Monoid extends Semigroup with an identity element.
fn example_monoid_identity() {
    println!("\n=== Example 2: Monoid - Identity Elements ===");

    // Vec monoid - empty vector is identity
    let v = vec![1, 2, 3];
    let empty: Vec<i32> = Monoid::empty();
    println!("Vec: {:?}", v);
    println!("Empty: {:?}", empty);
    println!("v + empty = {:?}", v.clone().combine(empty.clone()));
    println!("empty + v = {:?}", empty.combine(v.clone()));

    // String monoid - empty string is identity
    let s = "hello".to_string();
    let empty_str: String = Monoid::empty();
    println!("\nString: {:?}", s);
    println!("s + empty = {:?}", s.clone().combine(empty_str.clone()));
    println!("empty + s = {:?}", empty_str.combine(s));

    // Option monoid - None is identity
    let some_vec = Some(vec![1, 2, 3]);
    let none: Option<Vec<i32>> = Monoid::empty();
    println!("\nOption: {:?}", some_vec);
    println!("Some + None = {:?}", some_vec.clone().combine(none));
}

// ==================== Numeric Monoids ====================

/// Example 3: Sum monoid
///
/// Demonstrates using Sum wrapper for addition.
fn example_sum_monoid() {
    println!("\n=== Example 3: Sum Monoid (Addition) ===");

    // Basic combination
    let a = Sum(5);
    let b = Sum(10);
    println!("{:?} + {:?} = {:?}", a, b, a.combine(b));

    // Identity element (0)
    let x = Sum(42);
    let zero: Sum<i32> = Monoid::empty();
    println!("\nSum identity (0): {:?}", zero);
    println!("{:?} + 0 = {:?}", x, x.combine(zero));

    // Folding multiple values
    let numbers = vec![Sum(1), Sum(2), Sum(3), Sum(4), Sum(5)];
    let total = fold_all(numbers);
    println!("\nSum of [1,2,3,4,5] = {:?}", total);

    // Real-world: calculating total price
    let prices = vec![Sum(10.50), Sum(25.00), Sum(7.99), Sum(15.25)];
    let total_price = fold_all(prices);
    println!("Total price: ${:.2}", total_price.0);
}

/// Example 4: Product monoid
///
/// Demonstrates using Product wrapper for multiplication.
fn example_product_monoid() {
    println!("\n=== Example 4: Product Monoid (Multiplication) ===");

    // Basic combination
    let a = Product(3);
    let b = Product(7);
    println!("{:?} × {:?} = {:?}", a, b, a.combine(b));

    // Identity element (1)
    let x = Product(42);
    let one: Product<i32> = Monoid::empty();
    println!("\nProduct identity (1): {:?}", one);
    println!("{:?} × 1 = {:?}", x, x.combine(one));

    // Folding multiple values
    let numbers = vec![Product(2), Product(3), Product(4)];
    let result = fold_all(numbers);
    println!("\nProduct of [2,3,4] = {:?}", result);

    // Real-world: compound interest calculation
    let growth_rates = vec![Product(1.05), Product(1.08), Product(1.03)];
    let compound = fold_all(growth_rates);
    println!("Compound growth: {:.4}x", compound.0);
}

// ==================== Max and Min (Semigroups) ====================

/// Example 5: Max and Min semigroups
///
/// Demonstrates finding maximum and minimum values.
fn example_max_min() {
    println!("\n=== Example 5: Max and Min (Semigroups) ===");

    // Max - keeps larger value
    let m1 = Max(5);
    let m2 = Max(10);
    println!("Max({}, {}) = {:?}", m1.0, m2.0, m1.combine(m2));

    // Min - keeps smaller value
    let n1 = Min(5);
    let n2 = Min(10);
    println!("Min({}, {}) = {:?}", n1.0, n2.0, n1.combine(n2));

    // Note: Max and Min are only Semigroups, not Monoids
    // (no identity without bounded types)
    // For full Monoid, use Option<Max<T>> or Option<Min<T>>
    let max_with_identity: Option<Max<i32>> = None;
    let max_val = Some(Max(42));
    println!(
        "\nOption<Max> identity: {:?}",
        max_with_identity.combine(max_val)
    );
}

// ==================== Option Monoid ====================

/// Example 6: Option monoid
///
/// Demonstrates how Option lifts any Semigroup into a Monoid.
fn example_option_monoid() {
    println!("\n=== Example 6: Option Monoid ===");

    // None is the identity
    let none: Option<Vec<i32>> = Monoid::empty();
    println!("Identity (None): {:?}", none);

    // Some values combine their contents
    let v1 = Some(vec![1, 2]);
    let v2 = Some(vec![3, 4]);
    println!("Some + Some = {:?}", v1.combine(v2));

    // Some + None = Some
    let v3 = Some(vec![1, 2]);
    let v4: Option<Vec<i32>> = None;
    println!("Some + None = {:?}", v3.combine(v4));

    // None + Some = Some
    let v5: Option<Vec<i32>> = None;
    let v6 = Some(vec![3, 4]);
    println!("None + Some = {:?}", v5.combine(v6));

    // Real-world: optional configurations
    let default_config: Option<Vec<String>> = None;
    let user_config = Some(vec!["dark_mode".to_string()]);
    let final_config = default_config.combine(user_config);
    println!("\nConfig merge: {:?}", final_config);
}

// ==================== Fold and Reduce ====================

/// Example 7: fold_all and reduce
///
/// Demonstrates using utility functions for aggregation.
fn example_fold_and_reduce() {
    println!("\n=== Example 7: fold_all and reduce ===");

    // fold_all - combines all values starting with empty()
    let vecs = vec![vec![1, 2], vec![3, 4], vec![5]];
    let result: Vec<i32> = fold_all(vecs);
    println!("fold_all vecs: {:?}", result);

    // reduce is an alias for fold_all
    let strings = vec!["Hello".to_string(), " ".to_string(), "World".to_string()];
    let greeting: String = reduce(strings);
    println!("reduce strings: {}", greeting);

    // Empty collection returns identity
    let empty_vecs: Vec<Vec<i32>> = vec![];
    let result: Vec<i32> = fold_all(empty_vecs);
    println!("fold_all empty: {:?}", result);

    // Works with any Monoid
    let sums = vec![Sum(10), Sum(20), Sum(30)];
    let total = fold_all(sums);
    println!("fold_all sums: {:?}", total);
}

// ==================== Real-World Use Cases ====================

/// Example 8: Log aggregation
///
/// Real-world example: combining log entries from multiple sources.
fn example_log_aggregation() {
    println!("\n=== Example 8: Real-World - Log Aggregation ===");

    #[derive(Debug, Clone)]
    struct LogEntry {
        message: String,
        level: String,
    }

    // Logs from different sources
    let service_a_logs = vec![
        LogEntry {
            message: "Request received".to_string(),
            level: "INFO".to_string(),
        },
        LogEntry {
            message: "Processing...".to_string(),
            level: "DEBUG".to_string(),
        },
    ];

    let service_b_logs = vec![
        LogEntry {
            message: "Database connected".to_string(),
            level: "INFO".to_string(),
        },
        LogEntry {
            message: "Query executed".to_string(),
            level: "DEBUG".to_string(),
        },
    ];

    let service_c_logs = vec![LogEntry {
        message: "Response sent".to_string(),
        level: "INFO".to_string(),
    }];

    // Combine all logs using Vec monoid
    let all_logs = vec![service_a_logs, service_b_logs, service_c_logs];
    let combined: Vec<LogEntry> = fold_all(all_logs);

    println!("Combined {} log entries:", combined.len());
    for (i, entry) in combined.iter().enumerate() {
        println!("  {}: [{}] {}", i + 1, entry.level, entry.message);
    }
}

/// Example 9: Statistics calculation
///
/// Real-world example: calculating statistics across datasets.
fn example_statistics() {
    println!("\n=== Example 9: Real-World - Statistics ===");

    // Calculate sum, count, and product in one pass
    let dataset1 = [2, 3, 5];
    let dataset2 = [7, 11];
    let dataset3 = [13];

    // Sum using Sum monoid
    let all_values_sum: Vec<Vec<Sum<i32>>> = vec![
        dataset1.iter().map(|&x| Sum(x)).collect(),
        dataset2.iter().map(|&x| Sum(x)).collect(),
        dataset3.iter().map(|&x| Sum(x)).collect(),
    ];
    let total_sum: Vec<Sum<i32>> = fold_all(all_values_sum);
    let sum: Sum<i32> = fold_all(total_sum);
    println!("Total sum: {:?}", sum);

    // Count using Sum monoid with 1s
    let counts = vec![
        vec![Sum(1); dataset1.len()],
        vec![Sum(1); dataset2.len()],
        vec![Sum(1); dataset3.len()],
    ];
    let total_count_vec: Vec<Sum<i32>> = fold_all(counts);
    let total_count = fold_all(total_count_vec);
    println!("Total count: {:?}", total_count);

    // Product
    let all_values_prod: Vec<Vec<Product<i32>>> = vec![
        dataset1.iter().map(|&x| Product(x)).collect(),
        dataset2.iter().map(|&x| Product(x)).collect(),
        dataset3.iter().map(|&x| Product(x)).collect(),
    ];
    let total_product_vec: Vec<Product<i32>> = fold_all(all_values_prod);
    let total_product = fold_all(total_product_vec);
    println!("Total product: {:?}", total_product);
}

/// Example 10: Configuration merging
///
/// Real-world example: merging configurations from multiple sources.
fn example_config_merge() {
    println!("\n=== Example 10: Real-World - Configuration Merging ===");

    // Configurations as Vec of key-value pairs
    let default_config = vec![
        ("timeout".to_string(), "30".to_string()),
        ("retries".to_string(), "3".to_string()),
    ];

    let env_config = vec![
        ("timeout".to_string(), "60".to_string()), // Override
        ("debug".to_string(), "true".to_string()), // New
    ];

    let user_config = vec![("theme".to_string(), "dark".to_string())];

    // Combine all configs (later values override earlier ones in real impl)
    let configs = vec![default_config, env_config, user_config];
    let merged: Vec<(String, String)> = fold_all(configs);

    println!("Merged configuration:");
    for (key, value) in &merged {
        println!("  {} = {}", key, value);
    }
    println!("Total settings: {}", merged.len());
}

// ==================== Main ====================

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          Stillwater - Monoid and Semigroup Examples          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    example_semigroup();
    example_monoid_identity();
    example_sum_monoid();
    example_product_monoid();
    example_max_min();
    example_option_monoid();
    example_fold_and_reduce();
    example_log_aggregation();
    example_statistics();
    example_config_merge();

    println!("\n✓ All examples completed successfully!");
    println!("\nKey Takeaways:");
    println!("  • Semigroup provides associative combine operation");
    println!("  • Monoid extends Semigroup with identity element (empty)");
    println!("  • Wrapper types (Sum, Product) enable multiple monoid instances");
    println!("  • fold_all/reduce aggregate collections using monoid laws");
    println!("  • Option lifts any Semigroup into a Monoid with None as identity");
    println!("  • Real-world uses: logs, configs, stats, data aggregation");
}
