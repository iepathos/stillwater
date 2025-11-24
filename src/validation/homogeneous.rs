//! Homogeneous validation utilities for ensuring type consistency in collections
//!
//! This module provides utilities for validating that all items in a collection have the same
//! discriminant (type variant), which is useful for enums where each variant forms a Semigroup
//! but different variants cannot be combined.
//!
//! # The Problem
//!
//! Many Rust programs use discriminated unions (enums) where each variant forms a Semigroup,
//! but combining across variants is a type error:
//!
//! ```
//! # use stillwater::Semigroup;
//! #[derive(Clone, Debug, PartialEq)]
//! enum Aggregate {
//!     Sum(f64),      // Sum + Sum = Sum (valid)
//!     Count(usize),  // Count + Count = Count (valid)
//!     // But: Sum + Count = ??? (type error!)
//! }
//!
//! impl Semigroup for Aggregate {
//!     fn combine(self, other: Self) -> Self {
//!         match (self, other) {
//!             (Aggregate::Sum(a), Aggregate::Sum(b)) => Aggregate::Sum(a + b),
//!             (Aggregate::Count(a), Aggregate::Count(b)) => Aggregate::Count(a + b),
//!             _ => panic!("Type mismatch!"), // 💥 Crash!
//!         }
//!     }
//! }
//! ```
//!
//! # The Solution
//!
//! Following Stillwater's philosophy of **"pure core, imperative shell"**, we validate
//! homogeneity at boundaries before combining:
//!
//! ```
//! # use stillwater::Semigroup;
//! # use stillwater::validation::homogeneous::validate_homogeneous;
//! # use stillwater::Validation;
//! # use std::mem::discriminant;
//! # #[derive(Clone, Debug, PartialEq)]
//! # enum Aggregate {
//! #     Sum(f64),
//! #     Count(usize),
//! # }
//! # impl Semigroup for Aggregate {
//! #     fn combine(self, other: Self) -> Self {
//! #         match (self, other) {
//! #             (Aggregate::Sum(a), Aggregate::Sum(b)) => Aggregate::Sum(a + b),
//! #             (Aggregate::Count(a), Aggregate::Count(b)) => Aggregate::Count(a + b),
//! #             _ => unreachable!("Validated before combining"),
//! #         }
//! #     }
//! # }
//! let items = vec![
//!     Aggregate::Count(5),
//!     Aggregate::Sum(10.0),    // Wrong type!
//!     Aggregate::Count(3),
//! ];
//!
//! let result = validate_homogeneous(
//!     items,
//!     |a| discriminant(a),
//!     |idx, _got, _expected| format!("Type mismatch at index {}", idx),
//! );
//!
//! match result {
//!     Validation::Success(items) => {
//!         // All same type, safe to combine
//!         # let _ = items;
//!     }
//!     Validation::Failure(errors) => {
//!         // errors = ["Type mismatch at index 1"]
//!         assert_eq!(errors.len(), 1); // ALL errors reported!
//!     }
//! }
//! ```

use crate::{Semigroup, Validation};

/// Validate that all items in a collection have the same discriminant.
///
/// This is useful for enums where each variant forms a Semigroup, but
/// different variants cannot be combined. Validates homogeneity before
/// combining with the Semigroup trait.
///
/// # Arguments
///
/// * `items` - Collection to validate
/// * `discriminant` - Function to extract discriminant for comparison
/// * `make_error` - Function to create error for type mismatch
///
/// # Returns
///
/// - `Validation::Success(items)` if all items have same discriminant
/// - `Validation::Failure(errors)` with ALL mismatches if heterogeneous
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use stillwater::validation::homogeneous::validate_homogeneous;
/// use stillwater::Validation;
/// use std::mem::discriminant;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Sum(10.0),    // Wrong type!
///     Aggregate::Count(3),
///     Aggregate::Sum(20.0),    // Also wrong!
/// ];
///
/// let result = validate_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, _got, _expected| format!("Index {}: type mismatch", idx),
/// );
///
/// match result {
///     Validation::Success(_items) => {
///         // All same type, safe to combine
///     }
///     Validation::Failure(errors) => {
///         // errors = [
///         //   "Index 1: type mismatch",
///         //   "Index 3: type mismatch"
///         // ]
///         assert_eq!(errors.len(), 2); // ALL errors reported!
///     }
/// }
/// ```
///
/// ## Empty collections
///
/// ```
/// # use stillwater::validation::homogeneous::validate_homogeneous;
/// # use stillwater::Validation;
/// # use std::mem::discriminant;
/// # #[derive(Clone, PartialEq, Debug)]
/// # enum Aggregate { Count(usize) }
/// let items: Vec<Aggregate> = vec![];
///
/// let result = validate_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, _got, _expected| format!("Error at {}", idx),
/// );
///
/// assert!(result.is_success()); // Empty collections always validate
/// ```
pub fn validate_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<Vec<T>, Vec<E>>
where
    D: Eq,
{
    if items.is_empty() {
        return Validation::success(items);
    }

    let errors: Vec<E> = items
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, item)| discriminant(item) != discriminant(&items[0]))
        .map(|(idx, item)| make_error(idx, item, &items[0]))
        .collect();

    if errors.is_empty() {
        Validation::success(items)
    } else {
        Validation::failure(errors)
    }
}

/// Validate homogeneity and combine in one step.
///
/// This is a convenience function that validates all items have the same
/// discriminant, then combines them using their Semigroup instance.
///
/// # Examples
///
/// ```
/// use stillwater::validation::homogeneous::combine_homogeneous;
/// use stillwater::{Semigroup, Validation};
/// use std::mem::discriminant;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// impl Semigroup for Aggregate {
///     fn combine(self, other: Self) -> Self {
///         match (self, other) {
///             (Aggregate::Count(a), Aggregate::Count(b)) => {
///                 Aggregate::Count(a + b)
///             }
///             (Aggregate::Sum(a), Aggregate::Sum(b)) => {
///                 Aggregate::Sum(a + b)
///             }
///             // Safe to panic - only called after validation
///             _ => unreachable!("Validated before combining"),
///         }
///     }
/// }
///
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Count(3),
///     Aggregate::Count(2),
/// ];
///
/// let result = combine_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, _got, _expected| format!("Type mismatch at index {}", idx),
/// );
///
/// match result {
///     Validation::Success(combined) => {
///         assert_eq!(combined, Aggregate::Count(10));
///     }
///     Validation::Failure(_errors) => {
///         panic!("Should not fail");
///     }
/// }
/// ```
///
/// ## Handling validation failures
///
/// ```
/// # use stillwater::validation::homogeneous::combine_homogeneous;
/// # use stillwater::{Semigroup, Validation};
/// # use std::mem::discriminant;
/// # #[derive(Clone, PartialEq, Debug)]
/// # enum Aggregate {
/// #     Count(usize),
/// #     Sum(f64),
/// # }
/// # impl Semigroup for Aggregate {
/// #     fn combine(self, other: Self) -> Self {
/// #         match (self, other) {
/// #             (Aggregate::Count(a), Aggregate::Count(b)) => {
/// #                 Aggregate::Count(a + b)
/// #             }
/// #             (Aggregate::Sum(a), Aggregate::Sum(b)) => {
/// #                 Aggregate::Sum(a + b)
/// #             }
/// #             _ => unreachable!("Validated before combining"),
/// #         }
/// #     }
/// # }
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Sum(3.0),  // Wrong type!
///     Aggregate::Count(2),
/// ];
///
/// let result = combine_homogeneous(
///     items,
///     |a| discriminant(a),
///     |idx, _got, _expected| format!("Type mismatch at index {}", idx),
/// );
///
/// match result {
///     Validation::Success(_) => {
///         panic!("Should fail validation");
///     }
///     Validation::Failure(errors) => {
///         assert_eq!(errors.len(), 1);
///         assert_eq!(errors[0], "Type mismatch at index 1");
///     }
/// }
/// ```
pub fn combine_homogeneous<T, D, E>(
    items: Vec<T>,
    discriminant: impl Fn(&T) -> D,
    make_error: impl Fn(usize, &T, &T) -> E,
) -> Validation<T, Vec<E>>
where
    T: Semigroup,
    D: Eq,
{
    validate_homogeneous(items, discriminant, make_error).map(|items| {
        items
            .into_iter()
            .reduce(|a, b| a.combine(b))
            .expect("Validated non-empty")
    })
}

/// Helper trait for types that can provide their discriminant name.
///
/// This is useful for generating helpful error messages.
///
/// # Examples
///
/// ```
/// use stillwater::validation::homogeneous::DiscriminantName;
///
/// #[derive(Clone, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// impl DiscriminantName for Aggregate {
///     fn discriminant_name(&self) -> &'static str {
///         match self {
///             Aggregate::Count(_) => "Count",
///             Aggregate::Sum(_) => "Sum",
///         }
///     }
/// }
///
/// let aggregate = Aggregate::Count(5);
/// assert_eq!(aggregate.discriminant_name(), "Count");
/// ```
pub trait DiscriminantName {
    /// Returns the discriminant name for this value
    fn discriminant_name(&self) -> &'static str;
}

/// A standardized error type for type mismatches in homogeneous validation.
///
/// This error type provides structured information about where a type mismatch
/// occurred and what types were involved.
///
/// # Examples
///
/// ```
/// use stillwater::validation::homogeneous::{
///     validate_homogeneous, TypeMismatchError, DiscriminantName
/// };
/// use std::mem::discriminant;
///
/// #[derive(Clone, Debug)]
/// enum Aggregate {
///     Count(usize),
///     Sum(f64),
/// }
///
/// impl DiscriminantName for Aggregate {
///     fn discriminant_name(&self) -> &'static str {
///         match self {
///             Aggregate::Count(_) => "Count",
///             Aggregate::Sum(_) => "Sum",
///         }
///     }
/// }
///
/// let items = vec![
///     Aggregate::Count(5),
///     Aggregate::Sum(3.0),
/// ];
///
/// let result = validate_homogeneous(
///     items,
///     |a| discriminant(a),
///     TypeMismatchError::new,
/// );
///
/// match result {
///     stillwater::Validation::Failure(errors) => {
///         assert_eq!(errors[0].index, 1);
///         assert_eq!(errors[0].expected, "Count");
///         assert_eq!(errors[0].got, "Sum");
///     }
///     _ => panic!("Expected failure"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMismatchError {
    /// The index where the mismatch occurred
    pub index: usize,
    /// The expected discriminant name
    pub expected: String,
    /// The actual discriminant name
    pub got: String,
}

impl TypeMismatchError {
    /// Create a new type mismatch error from types implementing `DiscriminantName`
    ///
    /// # Examples
    ///
    /// ```
    /// # use stillwater::validation::homogeneous::{TypeMismatchError, DiscriminantName};
    /// # #[derive(Clone, Debug)]
    /// # enum Aggregate { Count(usize), Sum(f64) }
    /// # impl DiscriminantName for Aggregate {
    /// #     fn discriminant_name(&self) -> &'static str {
    /// #         match self {
    /// #             Aggregate::Count(_) => "Count",
    /// #             Aggregate::Sum(_) => "Sum",
    /// #         }
    /// #     }
    /// # }
    /// let error = TypeMismatchError::new(
    ///     1,
    ///     &Aggregate::Sum(3.0),
    ///     &Aggregate::Count(5),
    /// );
    ///
    /// assert_eq!(error.index, 1);
    /// assert_eq!(error.expected, "Count");
    /// assert_eq!(error.got, "Sum");
    /// ```
    pub fn new<T: DiscriminantName>(index: usize, got: &T, expected: &T) -> Self {
        TypeMismatchError {
            index,
            expected: expected.discriminant_name().to_string(),
            got: got.discriminant_name().to_string(),
        }
    }
}

impl std::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Type mismatch at index {}: expected {}, got {}",
            self.index, self.expected, self.got
        )
    }
}

impl std::error::Error for TypeMismatchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::discriminant;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEnum {
        A(i32),
        B(String),
    }

    impl Semigroup for TestEnum {
        fn combine(self, other: Self) -> Self {
            match (self, other) {
                (TestEnum::A(x), TestEnum::A(y)) => TestEnum::A(x + y),
                (TestEnum::B(x), TestEnum::B(y)) => TestEnum::B(x + &y),
                _ => panic!("Should be validated before combining"),
            }
        }
    }

    impl DiscriminantName for TestEnum {
        fn discriminant_name(&self) -> &'static str {
            match self {
                TestEnum::A(_) => "A",
                TestEnum::B(_) => "B",
            }
        }
    }

    #[test]
    fn test_homogeneous_validates_successfully() {
        let items = vec![TestEnum::A(1), TestEnum::A(2), TestEnum::A(3)];

        let result =
            validate_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        assert!(result.is_success());
    }

    #[test]
    fn test_heterogeneous_accumulates_all_errors() {
        let items = vec![
            TestEnum::A(1),
            TestEnum::B("wrong1".into()),
            TestEnum::A(2),
            TestEnum::B("wrong2".into()),
        ];

        let result =
            validate_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
                assert_eq!(errors[0], "Error at 1");
                assert_eq!(errors[1], "Error at 3");
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_combine_homogeneous_validates_and_combines() {
        let items = vec![TestEnum::A(1), TestEnum::A(2), TestEnum::A(3)];

        let result =
            combine_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        match result {
            Validation::Success(combined) => {
                assert_eq!(combined, TestEnum::A(6));
            }
            _ => panic!("Expected success"),
        }
    }

    #[test]
    fn test_empty_collection_validates() {
        let items: Vec<TestEnum> = vec![];

        let result =
            validate_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        assert!(result.is_success());
    }

    #[test]
    fn test_single_item_validates() {
        let items = vec![TestEnum::A(42)];

        let result =
            validate_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        assert!(result.is_success());
    }

    #[test]
    fn test_type_mismatch_error_creation() {
        let items = vec![TestEnum::A(1), TestEnum::B("wrong".into())];

        let result = validate_homogeneous(items, discriminant, TypeMismatchError::new);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].index, 1);
                assert_eq!(errors[0].expected, "A");
                assert_eq!(errors[0].got, "B");
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_type_mismatch_error_display() {
        let error = TypeMismatchError {
            index: 5,
            expected: "Count".to_string(),
            got: "Sum".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Type mismatch at index 5: expected Count, got Sum"
        );
    }

    #[test]
    fn test_combine_homogeneous_fails_on_mismatch() {
        let items = vec![TestEnum::A(1), TestEnum::B("wrong".into()), TestEnum::A(3)];

        let result =
            combine_homogeneous(items, discriminant, |idx, _, _| format!("Error at {}", idx));

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "Error at 1");
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_discriminant_name_trait() {
        let a = TestEnum::A(42);
        let b = TestEnum::B("test".into());

        assert_eq!(a.discriminant_name(), "A");
        assert_eq!(b.discriminant_name(), "B");
    }
}
