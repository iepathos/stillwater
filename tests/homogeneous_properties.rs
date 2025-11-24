//! Property-based tests for homogeneous validation

use proptest::prelude::*;
use std::mem::discriminant;
use stillwater::validation::homogeneous::{combine_homogeneous, validate_homogeneous};
use stillwater::{Semigroup, Validation};

#[derive(Clone, Debug, PartialEq)]
enum TestEnum {
    A(i32),
    B(String),
}

impl Semigroup for TestEnum {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (TestEnum::A(x), TestEnum::A(y)) => TestEnum::A(x.saturating_add(y)),
            (TestEnum::B(x), TestEnum::B(y)) => TestEnum::B(x + &y),
            _ => unreachable!("Should be validated before combining"),
        }
    }
}

proptest! {
    #[test]
    fn prop_homogeneous_always_validates(
        values in prop::collection::vec(any::<i32>(), 1..100)
    ) {
        let items: Vec<TestEnum> = values
            .into_iter()
            .map(TestEnum::A)
            .collect();

        let result = validate_homogeneous(
            items,
            discriminant,
            |idx, _, _| format!("Error at {}", idx),
        );

        prop_assert!(result.is_success());
    }

    #[test]
    fn prop_single_item_always_validates(value in any::<i32>()) {
        let items = vec![TestEnum::A(value)];

        let result = validate_homogeneous(
            items,
            discriminant,
            |idx, _, _| format!("Error at {}", idx),
        );

        prop_assert!(result.is_success());
    }

    #[test]
    fn prop_heterogeneous_finds_all_mismatches(
        a_count in 1usize..10,
        b_count in 1usize..5
    ) {
        let mut items = vec![];

        // Add a_count A variants
        for i in 0..a_count {
            items.push(TestEnum::A(i as i32));
        }

        // Add b_count B variants (these are mismatches)
        for i in 0..b_count {
            items.push(TestEnum::B(format!("b{}", i)));
        }

        let result = validate_homogeneous(
            items,
            discriminant,
            |idx, _, _| idx,
        );

        match result {
            Validation::Failure(errors) => {
                // Should find all B variants (they're mismatches)
                prop_assert_eq!(errors.len(), b_count);
            }
            Validation::Success(_) => {
                panic!("Should fail with heterogeneous items");
            }
        }
    }

    #[test]
    fn prop_combine_equals_fold(values in prop::collection::vec(any::<i32>(), 1..50)) {
        let items1: Vec<TestEnum> = values.iter().map(|&v| TestEnum::A(v)).collect();
        let items2: Vec<TestEnum> = values.iter().map(|&v| TestEnum::A(v)).collect();

        let combined = combine_homogeneous(
            items1,
            discriminant,
            |idx, _, _| format!("Error at {}", idx),
        );

        let folded = items2
            .into_iter()
            .reduce(|a, b| a.combine(b))
            .unwrap();

        match combined {
            Validation::Success(result) => {
                prop_assert_eq!(result, folded);
            }
            _ => {
                panic!("Should succeed with homogeneous items");
            }
        }
    }

    #[test]
    #[allow(clippy::let_unit_value)]
    fn prop_empty_validates(unit in any::<()>()) {
        let _ = unit; // Use the unit to satisfy proptest
        let items: Vec<TestEnum> = vec![];

        let result = validate_homogeneous(
            items,
            discriminant,
            |idx, _, _| format!("Error at {}", idx),
        );

        prop_assert!(result.is_success());
    }

    #[test]
    fn prop_error_count_matches_mismatch_count(
        first_count in 1usize..20,
        second_count in 1usize..20
    ) {
        let mut items = vec![];

        // Add first_count A variants
        for i in 0..first_count {
            items.push(TestEnum::A(i as i32));
        }

        // Add second_count B variants (these are mismatches)
        for i in 0..second_count {
            items.push(TestEnum::B(format!("b{}", i)));
        }

        let result = validate_homogeneous(
            items,
            discriminant,
            |idx, _, _| idx,
        );

        match result {
            Validation::Failure(errors) => {
                // Should find exactly second_count errors
                prop_assert_eq!(errors.len(), second_count);
            }
            _ => {
                panic!("Should fail with heterogeneous items");
            }
        }
    }
}
