//! Extended Semigroup Implementations Examples
//!
//! This example demonstrates Stillwater's Semigroup implementations for
//! standard Rust collection types (HashMap, HashSet, BTreeMap, BTreeSet, Option)
//! and wrapper types (First, Last, Intersection) for alternative combining semantics.
//!
//! Run with: cargo run --example extended_semigroup

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use stillwater::{First, Intersection, Last, Semigroup};

// Example 1: HashMap - Merge with Value Combining
//
// HashMap combines by merging entries. When keys conflict, values
// are combined using their Semigroup instance.
fn example_hashmap_combining() {
    println!("\n=== Example 1: HashMap - Merge with Value Combining ===\n");

    let mut errors1 = HashMap::new();
    errors1.insert("validation", vec!["Invalid email"]);
    errors1.insert("auth", vec!["Token expired"]);

    let mut errors2 = HashMap::new();
    errors2.insert("validation", vec!["Password too short"]);
    errors2.insert("permission", vec!["Access denied"]);

    let all_errors = errors1.combine(errors2);

    println!("Combined error map:");
    for (category, errors) in all_errors {
        println!("  {}: {:?}", category, errors);
    }

    println!("\nNote: 'validation' errors were combined, other keys merged in");
}

// Example 2: Configuration Merging with HashMap
//
// Practical use case: merging configuration from multiple sources.
fn example_config_merging() {
    println!("\n=== Example 2: Configuration Merging ===\n");

    #[derive(Debug, Clone)]
    struct Config {
        settings: HashMap<String, Vec<String>>,
    }

    impl Semigroup for Config {
        fn combine(self, other: Self) -> Self {
            Config {
                settings: self.settings.combine(other.settings),
            }
        }
    }

    let defaults = Config {
        settings: [
            ("log_level".into(), vec!["info".into()]),
            ("features".into(), vec!["basic".into()]),
        ]
        .into_iter()
        .collect(),
    };

    let env_config = Config {
        settings: [
            ("log_level".into(), vec!["debug".into()]),  // Override
            ("features".into(), vec!["premium".into()]), // Adds to existing
        ]
        .into_iter()
        .collect(),
    };

    let user_config = Config {
        settings: [("api_key".into(), vec!["secret".into()])]
            .into_iter()
            .collect(),
    };

    let final_config = defaults.combine(env_config).combine(user_config);

    println!("Final configuration:");
    for (key, values) in final_config.settings {
        println!("  {}: {:?}", key, values);
    }

    println!("\nNote: Configs are layered - values from same keys are combined");
}

// Example 3: HashSet - Union
//
// HashSet combines using union (all unique elements).
fn example_hashset_union() {
    println!("\n=== Example 3: HashSet - Union ===\n");

    let permissions1: HashSet<_> = ["read", "write"].iter().cloned().collect();
    let permissions2: HashSet<_> = ["write", "delete", "admin"].iter().cloned().collect();

    let all_permissions = permissions1.combine(permissions2);

    println!("Combined permissions: {:?}", all_permissions);
    println!("\nNote: Union keeps all unique elements");
}

// Example 4: Feature Flags with HashSet
//
// Practical use case: combining feature flags from different sources.
fn example_feature_flags() {
    println!("\n=== Example 4: Feature Flags ===\n");

    #[derive(Debug, Clone)]
    struct Features {
        enabled: HashSet<String>,
    }

    impl Semigroup for Features {
        fn combine(self, other: Self) -> Self {
            Features {
                enabled: self.enabled.combine(other.enabled),
            }
        }
    }

    let base_features = Features {
        enabled: ["logging", "metrics"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    let tier1_features = Features {
        enabled: ["api_access"].iter().map(|s| s.to_string()).collect(),
    };

    let tier2_features = Features {
        enabled: ["advanced_analytics", "priority_support"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    let user_features = base_features
        .combine(tier1_features)
        .combine(tier2_features);

    println!("Enabled features:");
    for feature in user_features.enabled {
        println!("  - {}", feature);
    }

    println!("\nNote: All features are accumulated via union");
}

// Example 5: BTreeMap and BTreeSet - Ordered Collections
//
// BTreeMap and BTreeSet have same combining semantics as their hash variants,
// but maintain sorted order.
fn example_btree_collections() {
    println!("\n=== Example 5: BTreeMap and BTreeSet - Ordered Collections ===\n");

    let mut map1 = BTreeMap::new();
    map1.insert("zebra", vec![1]);
    map1.insert("apple", vec![2]);

    let mut map2 = BTreeMap::new();
    map2.insert("zebra", vec![3]);
    map2.insert("banana", vec![4]);

    let combined_map = map1.combine(map2);

    println!("BTreeMap (sorted keys):");
    for (key, values) in combined_map {
        println!("  {}: {:?}", key, values);
    }

    let set1: BTreeSet<_> = [5, 1, 3].iter().cloned().collect();
    let set2: BTreeSet<_> = [4, 2, 1].iter().cloned().collect();

    let combined_set = set1.combine(set2);

    println!("\nBTreeSet (sorted elements): {:?}", combined_set);
    println!("\nNote: Elements/keys are in sorted order");
}

// Example 6: Option<T: Semigroup> - Lifting Semigroups
//
// Option lifts a Semigroup operation, combining inner values when both are Some.
fn example_option_semigroup() {
    println!("\n=== Example 6: Option<T: Semigroup> - Lifting Semigroups ===\n");

    let opt1 = Some(vec![1, 2, 3]);
    let opt2 = Some(vec![4, 5, 6]);
    println!("Some + Some: {:?}", opt1.combine(opt2));

    let some = Some(vec![1, 2, 3]);
    let none: Option<Vec<i32>> = None;
    println!("Some + None: {:?}", some.clone().combine(none.clone()));
    println!("None + Some: {:?}", none.combine(some));

    let none1: Option<Vec<i32>> = None;
    let none2: Option<Vec<i32>> = None;
    println!("None + None: {:?}", none1.combine(none2));

    println!("\nNote: Option preserves Some values, combines when both are Some");
}

// Example 7: Optional Error Accumulation
//
// Practical use case: accumulating optional validation errors.
fn example_optional_errors() {
    println!("\n=== Example 7: Optional Error Accumulation ===\n");

    fn validate_email(email: &str) -> Option<Vec<String>> {
        if email.contains('@') {
            None // No errors
        } else {
            Some(vec!["Invalid email format".to_string()])
        }
    }

    fn validate_age(age: u8) -> Option<Vec<String>> {
        if age >= 18 {
            None
        } else {
            Some(vec!["Must be 18 or older".to_string()])
        }
    }

    fn validate_password(password: &str) -> Option<Vec<String>> {
        if password.len() >= 8 {
            None
        } else {
            Some(vec!["Password must be at least 8 characters".to_string()])
        }
    }

    let email_errors = validate_email("invalid");
    let age_errors = validate_age(15);
    let password_errors = validate_password("short");

    let all_errors = email_errors.combine(age_errors).combine(password_errors);

    match all_errors {
        Some(errors) => {
            println!("Validation failed with {} errors:", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
        None => println!("All validations passed"),
    }

    println!("\nNote: Only accumulates errors, None means success");
}

// Example 8: First<T> - Keep First Value
//
// First wrapper keeps the first (left) value, ignoring subsequent values.
fn example_first_wrapper() {
    println!("\n=== Example 8: First<T> - Keep First Value ===\n");

    let first = First(42).combine(First(100));
    println!("First(42).combine(First(100)) = First({})", first.0);

    let first_str = First("default").combine(First("override"));
    println!(
        "First(\"default\").combine(First(\"override\")) = First(\"{}\")",
        first_str.0
    );

    println!("\nNote: Always keeps the first value - useful for defaults");
}

// Example 9: Default Configuration with First
//
// Practical use case: providing default values that aren't overridden.
fn example_defaults_with_first() {
    println!("\n=== Example 9: Default Configuration with First ===\n");

    let defaults: HashMap<String, First<i32>> = [
        ("timeout".into(), First(30)),
        ("retries".into(), First(3)),
        ("max_connections".into(), First(100)),
    ]
    .into_iter()
    .collect();

    let user_config: HashMap<String, First<i32>> =
        [("timeout".into(), First(60))].into_iter().collect();

    // User config is first, so it wins where keys overlap
    let final_config = user_config.combine(defaults);

    println!("Final configuration:");
    for (key, First(value)) in final_config {
        println!("  {}: {}", key, value);
    }

    println!("\nNote: User's timeout (60) kept, other values from defaults");
}

// Example 10: Last<T> - Keep Last Value
//
// Last wrapper keeps the last (right) value, for override semantics.
fn example_last_wrapper() {
    println!("\n=== Example 10: Last<T> - Keep Last Value ===\n");

    let last = Last(42).combine(Last(100));
    println!("Last(42).combine(Last(100)) = Last({})", last.0);

    let last_str = Last("default").combine(Last("override"));
    println!(
        "Last(\"default\").combine(Last(\"override\")) = Last(\"{}\")",
        last_str.0
    );

    println!("\nNote: Always keeps the last value - useful for overrides");
}

// Example 11: Layered Configuration with Last
//
// Practical use case: building config from layers where later layers override.
fn example_layered_config() {
    println!("\n=== Example 11: Layered Configuration with Last ===\n");

    let base_config: HashMap<String, Last<String>> = [
        ("env".into(), Last("development".into())),
        ("debug".into(), Last("false".into())),
        ("log_level".into(), Last("info".into())),
    ]
    .into_iter()
    .collect();

    let env_config: HashMap<String, Last<String>> = [
        ("env".into(), Last("production".into())),
        ("debug".into(), Last("false".into())),
    ]
    .into_iter()
    .collect();

    let runtime_config: HashMap<String, Last<String>> = [("debug".into(), Last("true".into()))]
        .into_iter()
        .collect();

    let final_config = base_config.combine(env_config).combine(runtime_config);

    println!("Layered configuration (base -> env -> runtime):");
    for (key, Last(value)) in final_config {
        println!("  {}: {}", key, value);
    }

    println!("\nNote: Later layers override earlier ones");
}

// Example 12: Intersection<Set> - Set Intersection
//
// Intersection wrapper provides intersection semantics instead of union.
fn example_intersection() {
    println!("\n=== Example 12: Intersection<Set> - Set Intersection ===\n");

    let set1: HashSet<_> = [1, 2, 3, 4].iter().cloned().collect();
    let set2: HashSet<_> = [3, 4, 5, 6].iter().cloned().collect();

    let intersection_result = Intersection(set1).combine(Intersection(set2));

    println!(
        "Intersection of {{1,2,3,4}} and {{3,4,5,6}}: {:?}",
        intersection_result.0
    );

    println!("\nNote: Only common elements remain");
}

// Example 13: Required Permissions with Intersection
//
// Practical use case: finding permissions user actually has.
fn example_required_permissions() {
    println!("\n=== Example 13: Required Permissions with Intersection ===\n");

    let required_perms: HashSet<_> = ["read", "write", "delete", "admin"]
        .iter()
        .cloned()
        .collect();

    let user_perms: HashSet<_> = ["read", "write", "delete"].iter().cloned().collect();

    let effective_perms = Intersection(required_perms).combine(Intersection(user_perms));

    println!("Required permissions: read, write, delete, admin");
    println!("User has: read, write, delete");
    println!("Effective permissions: {:?}", effective_perms.0);

    println!("\nNote: Intersection finds what the user actually has from what's required");
}

// Example 14: Error Aggregation by Type
//
// Comprehensive example: aggregating errors by category using HashMap.
fn example_error_aggregation() {
    println!("\n=== Example 14: Error Aggregation by Type ===\n");

    type ErrorsByType = HashMap<String, Vec<String>>;

    fn validate_user(username: &str) -> ErrorsByType {
        let mut errors = HashMap::new();
        if username.is_empty() {
            errors.insert("validation".into(), vec!["Username required".into()]);
        }
        errors
    }

    fn check_permissions(user_id: i32) -> ErrorsByType {
        let mut errors = HashMap::new();
        if user_id < 0 {
            errors.insert("permission".into(), vec!["Invalid user ID".into()]);
        }
        errors
    }

    fn check_rate_limit(requests: u32) -> ErrorsByType {
        let mut errors = HashMap::new();
        if requests > 100 {
            errors.insert("rate_limit".into(), vec!["Too many requests".into()]);
        }
        errors
    }

    let validation_errors = validate_user("");
    let permission_errors = check_permissions(-1);
    let rate_errors = check_rate_limit(150);

    let all_errors = validation_errors
        .combine(permission_errors)
        .combine(rate_errors);

    println!("Aggregated errors by type:");
    for (error_type, errors) in all_errors {
        println!("  {}:", error_type);
        for error in errors {
            println!("    - {}", error);
        }
    }

    println!("\nNote: Errors grouped by category for better reporting");
}

fn main() {
    println!("==============================================");
    println!("  Extended Semigroup Implementations");
    println!("==============================================");

    println!("\nDemonstrates Semigroup for collections:");
    println!("- HashMap, BTreeMap (merge with value combining)");
    println!("- HashSet, BTreeSet (union)");
    println!("- Option (lift semigroup to optional)");
    println!("- First, Last (alternative semantics)");
    println!("- Intersection (set intersection)");

    example_hashmap_combining();
    example_config_merging();
    example_hashset_union();
    example_feature_flags();
    example_btree_collections();
    example_option_semigroup();
    example_optional_errors();
    example_first_wrapper();
    example_defaults_with_first();
    example_last_wrapper();
    example_layered_config();
    example_intersection();
    example_required_permissions();
    example_error_aggregation();

    println!("\n==============================================");
    println!("         All examples completed!");
    println!("==============================================");
}
