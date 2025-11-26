//! Integration tests for homogeneous validation with Effect and other Stillwater types

use std::mem::discriminant;
use stillwater::validation::homogeneous::{
    combine_homogeneous, validate_homogeneous, DiscriminantName, TypeMismatchError,
};
use stillwater::{Semigroup, Validation};

#[derive(Clone, Debug, PartialEq)]
enum AggregateResult {
    Count(usize),
    Sum(f64),
    Average(f64, usize),
}

impl Semigroup for AggregateResult {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (AggregateResult::Count(a), AggregateResult::Count(b)) => AggregateResult::Count(a + b),
            (AggregateResult::Sum(a), AggregateResult::Sum(b)) => AggregateResult::Sum(a + b),
            (AggregateResult::Average(s1, c1), AggregateResult::Average(s2, c2)) => {
                AggregateResult::Average(s1 + s2, c1 + c2)
            }
            _ => unreachable!("Call validate_homogeneous first"),
        }
    }
}

impl DiscriminantName for AggregateResult {
    fn discriminant_name(&self) -> &'static str {
        match self {
            AggregateResult::Count(_) => "Count",
            AggregateResult::Sum(_) => "Sum",
            AggregateResult::Average(_, _) => "Average",
        }
    }
}

#[test]
fn test_validation_with_successful_aggregation() {
    let results = vec![
        AggregateResult::Count(5),
        AggregateResult::Count(3),
        AggregateResult::Count(2),
    ];

    let combined = combine_homogeneous(results, discriminant, |idx, _, _| {
        format!("Worker {} type mismatch", idx)
    });

    match combined {
        Validation::Success(result) => {
            assert_eq!(result, AggregateResult::Count(10));
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_validation_with_failed_aggregation() {
    let results = vec![
        AggregateResult::Count(5),
        AggregateResult::Sum(3.0), // Wrong type!
        AggregateResult::Count(2),
    ];

    let combined = combine_homogeneous(results, discriminant, |idx, _, _| {
        format!("Worker {} type mismatch", idx)
    });

    match combined {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0], "Worker 1 type mismatch");
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_type_mismatch_error_with_discriminant_name() {
    let items = vec![
        AggregateResult::Count(5),
        AggregateResult::Sum(3.0),
        AggregateResult::Average(10.0, 2),
    ];

    let result = validate_homogeneous(items, discriminant, TypeMismatchError::new);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 2);

            assert_eq!(errors[0].index, 1);
            assert_eq!(errors[0].expected, "Count");
            assert_eq!(errors[0].got, "Sum");

            assert_eq!(errors[1].index, 2);
            assert_eq!(errors[1].expected, "Count");
            assert_eq!(errors[1].got, "Average");
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_validation_composition() {
    // Simulate validating multiple batches of results
    let batch1 = vec![AggregateResult::Count(5), AggregateResult::Count(3)];

    let batch2 = vec![
        AggregateResult::Count(2),
        AggregateResult::Sum(10.0), // Wrong type!
    ];

    let result1 = validate_homogeneous(batch1, discriminant, |idx, _, _| {
        format!("Batch 1, index {}", idx)
    });

    let result2 = validate_homogeneous(batch2, discriminant, |idx, _, _| {
        format!("Batch 2, index {}", idx)
    });

    // Compose validations
    let combined = result1.and(result2);

    match combined {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 1);
            assert!(errors[0].contains("Batch 2"));
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_json_like_enum_validation() {
    #[derive(Clone, Debug, PartialEq)]
    #[allow(dead_code)]
    enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Semigroup for Value {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (Value::Array(mut a), Value::Array(b)) => {
                    a.extend(b);
                    Value::Array(a)
                }
                (Value::Object(mut a), Value::Object(b)) => {
                    a.extend(b);
                    Value::Object(a)
                }
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                (Value::String(mut a), Value::String(b)) => {
                    a.push_str(&b);
                    Value::String(a)
                }
                _ => unreachable!("Validated before combining"),
            }
        }
    }

    impl DiscriminantName for Value {
        fn discriminant_name(&self) -> &'static str {
            match self {
                Value::Null => "Null",
                Value::Bool(_) => "Bool",
                Value::Number(_) => "Number",
                Value::String(_) => "String",
                Value::Array(_) => "Array",
                Value::Object(_) => "Object",
            }
        }
    }

    // Test mixing JSON types
    let mixed_values = vec![
        Value::Object(vec![]),
        Value::Array(vec![]), // Wrong type!
        Value::Object(vec![]),
    ];

    let result = validate_homogeneous(mixed_values, discriminant, TypeMismatchError::new);

    match result {
        Validation::Failure(errors) => {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].expected, "Object");
            assert_eq!(errors[0].got, "Array");
        }
        _ => panic!("Expected failure"),
    }

    // Test combining same types
    let arrays = vec![
        Value::Array(vec![Value::Number(1.0)]),
        Value::Array(vec![Value::Number(2.0)]),
        Value::Array(vec![Value::Number(3.0)]),
    ];

    let result = combine_homogeneous(arrays, discriminant, TypeMismatchError::new);

    match result {
        Validation::Success(combined) => {
            if let Value::Array(items) = combined {
                assert_eq!(items.len(), 3);
            } else {
                panic!("Expected array");
            }
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_mapreduce_aggregation_pattern() {
    // Simulate MapReduce aggregation where workers return results
    struct Worker {
        result: AggregateResult,
    }

    let workers = vec![
        Worker {
            result: AggregateResult::Sum(10.0),
        },
        Worker {
            result: AggregateResult::Sum(20.0),
        },
        Worker {
            result: AggregateResult::Sum(30.0),
        },
    ];

    let results: Vec<AggregateResult> = workers.into_iter().map(|w| w.result).collect();

    let aggregated = combine_homogeneous(results, discriminant, |idx, got, expected| {
        format!(
            "Worker {} returned {}, expected {}",
            idx,
            got.discriminant_name(),
            expected.discriminant_name()
        )
    });

    match aggregated {
        Validation::Success(result) => {
            assert_eq!(result, AggregateResult::Sum(60.0));
        }
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_error_accumulation_with_many_mismatches() {
    let mut items = vec![AggregateResult::Count(1)];

    // Add 50 mismatches
    for i in 0..50 {
        if i % 2 == 0 {
            items.push(AggregateResult::Sum(i as f64));
        } else {
            items.push(AggregateResult::Average(i as f64, 1));
        }
    }

    let result = validate_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

    match result {
        Validation::Failure(errors) => {
            // Should accumulate ALL 50 errors
            assert_eq!(errors.len(), 50);
        }
        _ => panic!("Expected failure"),
    }
}
