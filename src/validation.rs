//! Validation type for accumulating errors
//!
//! This module provides the `Validation` type, which is similar to `Result` but designed
//! specifically for validation scenarios where we want to accumulate all errors rather than
//! short-circuiting on the first failure.
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```
//! use stillwater::Validation;
//!
//! // Create validations
//! let success = Validation::<_, Vec<&str>>::success(42);
//! let failure = Validation::<i32, _>::failure(vec!["error"]);
//!
//! assert!(success.is_success());
//! assert!(failure.is_failure());
//! ```
//!
//! ## Combining validations
//!
//! ```
//! use stillwater::Validation;
//!
//! let v1 = Validation::<_, Vec<&str>>::success(1);
//! let v2 = Validation::<_, Vec<&str>>::success(2);
//! let result = v1.and(v2);
//!
//! assert_eq!(result, Validation::Success((1, 2)));
//! ```
//!
//! ## Accumulating errors
//!
//! ```
//! use stillwater::Validation;
//!
//! let v1 = Validation::<i32, _>::failure(vec!["error1"]);
//! let v2 = Validation::<i32, _>::failure(vec!["error2"]);
//! let result = v1.and(v2);
//!
//! assert_eq!(result, Validation::Failure(vec!["error1", "error2"]));
//! ```
//!
//! ## Validating tuples
//!
//! ```
//! use stillwater::{Validation, validation::ValidateAll};
//!
//! let result = (
//!     Validation::<_, Vec<&str>>::success(1),
//!     Validation::<_, Vec<&str>>::success(2),
//!     Validation::<_, Vec<&str>>::success(3),
//! ).validate_all();
//!
//! assert_eq!(result, Validation::Success((1, 2, 3)));
//! ```

use crate::Semigroup;

/// A validation that either succeeds with a value or fails with accumulated errors
///
/// Unlike `Result`, `Validation` is designed to accumulate multiple errors when combining
/// validations. This makes it ideal for form validation, API input validation, and other
/// scenarios where you want to report all errors at once rather than failing on the first one.
///
/// # Type Parameters
///
/// * `T` - The type of the success value
/// * `E` - The type of the error value (must implement `Semigroup` for accumulation)
///
/// # Examples
///
/// ```
/// use stillwater::{Validation, Semigroup};
///
/// // Simple validation
/// let v = Validation::<_, Vec<&str>>::success(42);
/// assert_eq!(v.into_result(), Ok(42));
///
/// // Error accumulation
/// let v1 = Validation::<i32, _>::failure(vec!["error1"]);
/// let v2 = Validation::<i32, _>::failure(vec!["error2"]);
/// let combined = v1.and(v2);
/// assert_eq!(combined, Validation::Failure(vec!["error1", "error2"]));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validation<T, E> {
    /// Successful validation with a value
    Success(T),
    /// Failed validation with accumulated errors
    Failure(E),
}

impl<T, E> Validation<T, E> {
    /// Create a successful validation
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<i32, String>::success(42);
    /// assert!(v.is_success());
    /// ```
    #[inline]
    pub fn success(value: T) -> Self {
        Validation::Success(value)
    }

    /// Create a failed validation
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<i32, Vec<&str>>::failure(vec!["error"]);
    /// assert!(v.is_failure());
    /// ```
    #[inline]
    pub fn failure(error: E) -> Self {
        Validation::Failure(error)
    }

    /// Create a validation from a Result
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::from_result(Ok::<_, String>(42));
    /// assert_eq!(v, Validation::Success(42));
    ///
    /// let v = Validation::from_result(Err::<i32, _>("error".to_string()));
    /// assert_eq!(v, Validation::Failure("error".to_string()));
    /// ```
    #[inline]
    pub fn from_result(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Validation::Success(value),
            Err(error) => Validation::Failure(error),
        }
    }

    /// Convert this validation to a Result
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(42);
    /// assert_eq!(v.into_result(), Ok(42));
    ///
    /// let v = Validation::<i32, _>::failure("error".to_string());
    /// assert_eq!(v.into_result(), Err("error".to_string()));
    /// ```
    #[inline]
    pub fn into_result(self) -> Result<T, E> {
        match self {
            Validation::Success(value) => Ok(value),
            Validation::Failure(error) => Err(error),
        }
    }

    /// Check if this validation is successful
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(42);
    /// assert!(v.is_success());
    /// ```
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, Validation::Success(_))
    }

    /// Check if this validation failed
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<i32, _>::failure("error".to_string());
    /// assert!(v.is_failure());
    /// ```
    #[inline]
    pub fn is_failure(&self) -> bool {
        matches!(self, Validation::Failure(_))
    }

    /// Transform the success value if present
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(5);
    /// let result = v.map(|x| x * 2);
    /// assert_eq!(result, Validation::Success(10));
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> Validation<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Validation::Success(value) => Validation::Success(f(value)),
            Validation::Failure(error) => Validation::Failure(error),
        }
    }

    /// Transform the error value if present
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<i32, _>::failure(vec!["error"]);
    /// let result = v.map_err(|errors| errors.len());
    /// assert_eq!(result, Validation::Failure(1));
    /// ```
    #[inline]
    pub fn map_err<E2, F>(self, f: F) -> Validation<T, E2>
    where
        F: FnOnce(E) -> E2,
    {
        match self {
            Validation::Success(value) => Validation::Success(value),
            Validation::Failure(error) => Validation::Failure(f(error)),
        }
    }
}

impl<T, E: Semigroup> Validation<T, E> {
    /// Combine two validations, accumulating errors using the Semigroup instance
    ///
    /// If both validations are successful, returns a success with a tuple of both values.
    /// If either or both fail, accumulates the errors using `Semigroup::combine`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// // Both successful
    /// let v1 = Validation::<_, Vec<&str>>::success(1);
    /// let v2 = Validation::<_, Vec<&str>>::success(2);
    /// assert_eq!(v1.and(v2), Validation::Success((1, 2)));
    ///
    /// // Both failed - errors accumulate
    /// let v1 = Validation::<i32, _>::failure(vec!["error1"]);
    /// let v2 = Validation::<i32, _>::failure(vec!["error2"]);
    /// assert_eq!(v1.and(v2), Validation::Failure(vec!["error1", "error2"]));
    /// ```
    pub fn and<U>(self, other: Validation<U, E>) -> Validation<(T, U), E> {
        match (self, other) {
            (Validation::Success(a), Validation::Success(b)) => Validation::Success((a, b)),
            (Validation::Failure(e1), Validation::Failure(e2)) => {
                Validation::Failure(e1.combine(e2))
            }
            (Validation::Failure(e), _) => Validation::Failure(e),
            (_, Validation::Failure(e)) => Validation::Failure(e),
        }
    }

    /// Chain a dependent validation
    ///
    /// Similar to `Result::and_then`, but for validations. The function is only called
    /// if the current validation is successful.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, Vec<&str>>::success(5);
    /// let result = v.and_then(|x| {
    ///     if x > 0 {
    ///         Validation::success(x * 2)
    ///     } else {
    ///         Validation::failure(vec!["must be positive"])
    ///     }
    /// });
    /// assert_eq!(result, Validation::Success(10));
    /// ```
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> Validation<U, E>
    where
        F: FnOnce(T) -> Validation<U, E>,
    {
        match self {
            Validation::Success(value) => f(value),
            Validation::Failure(error) => Validation::Failure(error),
        }
    }

    /// Combine all validations in a Vec
    ///
    /// Returns a success with a Vec of all success values if all validations succeed.
    /// Otherwise, accumulates all errors using `Semigroup::combine`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Validation;
    ///
    /// let validations = vec![
    ///     Validation::<_, Vec<&str>>::success(1),
    ///     Validation::<_, Vec<&str>>::success(2),
    ///     Validation::<_, Vec<&str>>::success(3),
    /// ];
    /// let result = Validation::all_vec(validations);
    /// assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    ///
    /// let validations = vec![
    ///     Validation::<i32, _>::failure(vec!["error1"]),
    ///     Validation::<i32, _>::failure(vec!["error2"]),
    /// ];
    /// let result = Validation::all_vec(validations);
    /// assert_eq!(result, Validation::Failure(vec!["error1", "error2"]));
    /// ```
    pub fn all_vec(validations: Vec<Validation<T, E>>) -> Validation<Vec<T>, E> {
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for validation in validations {
            match validation {
                Validation::Success(value) => successes.push(value),
                Validation::Failure(error) => failures.push(error),
            }
        }

        if failures.is_empty() {
            Validation::Success(successes)
        } else {
            Validation::Failure(
                failures
                    .into_iter()
                    .reduce(|acc, e| acc.combine(e))
                    .unwrap(),
            )
        }
    }
}

// Free function for combining validations in a tuple
impl<T, E> Validation<T, E> {
    /// Combine all validations in a tuple
    ///
    /// This is a convenience method that delegates to the `ValidateAll` trait.
    /// It works with tuples of validations up to size 12.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::{Validation, validation::ValidateAll};
    ///
    /// let result = (
    ///     Validation::<_, Vec<&str>>::success(1),
    ///     Validation::<_, Vec<&str>>::success(2),
    ///     Validation::<_, Vec<&str>>::success(3),
    /// ).validate_all();
    /// assert_eq!(result, Validation::Success((1, 2, 3)));
    /// ```
    pub fn all<V, E2>(validations: V) -> Validation<V::Output, E2>
    where
        E2: Semigroup,
        V: ValidateAll<E2>,
    {
        validations.validate_all()
    }
}

/// Trait for combining multiple validations in a tuple
///
/// This trait is implemented for tuples of validations, allowing the `Validation::all`
/// method to work with heterogeneous validation types.
pub trait ValidateAll<E: Semigroup> {
    /// The output type when all validations succeed
    type Output;

    /// Combine all validations, accumulating errors
    fn validate_all(self) -> Validation<Self::Output, E>;
}

// Macro to implement ValidateAll for tuples of different sizes
macro_rules! impl_validate_all {
    // Base case: single element
    ($T1:ident) => {
        impl<E: Semigroup, $T1> ValidateAll<E> for (Validation<$T1, E>,) {
            type Output = ($T1,);

            fn validate_all(self) -> Validation<Self::Output, E> {
                self.0.map(|v| (v,))
            }
        }
    };

    // Two elements
    ($T1:ident, $T2:ident) => {
        impl<E: Semigroup, $T1, $T2> ValidateAll<E> for (Validation<$T1, E>, Validation<$T2, E>) {
            type Output = ($T1, $T2);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2) = self;
                $T1.and($T2)
            }
        }
    };

    // Three elements
    ($T1:ident, $T2:ident, $T3:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3> ValidateAll<E>
            for (Validation<$T1, E>, Validation<$T2, E>, Validation<$T3, E>)
        {
            type Output = ($T1, $T2, $T3);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3) = self;
                $T1.and($T2)
                    .map(|(a, b)| (a, b))
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
            }
        }
    };

    // Four elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
            }
        }
    };

    // Five elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
            }
        }
    };

    // Six elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
            }
        }
    };

    // Seven elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6, $T7);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
            }
        }
    };

    // Eight elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
                Validation<$T8, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
                    .and($T8)
                    .map(|((a, b, c, d, e, f, g), h)| (a, b, c, d, e, f, g, h))
            }
        }
    };

    // Nine elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
                Validation<$T8, E>,
                Validation<$T9, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
                    .and($T8)
                    .map(|((a, b, c, d, e, f, g), h)| (a, b, c, d, e, f, g, h))
                    .and($T9)
                    .map(|((a, b, c, d, e, f, g, h), i)| (a, b, c, d, e, f, g, h, i))
            }
        }
    };

    // Ten elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
                Validation<$T8, E>,
                Validation<$T9, E>,
                Validation<$T10, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
                    .and($T8)
                    .map(|((a, b, c, d, e, f, g), h)| (a, b, c, d, e, f, g, h))
                    .and($T9)
                    .map(|((a, b, c, d, e, f, g, h), i)| (a, b, c, d, e, f, g, h, i))
                    .and($T10)
                    .map(|((a, b, c, d, e, f, g, h, i), j)| (a, b, c, d, e, f, g, h, i, j))
            }
        }
    };

    // Eleven elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident, $T11:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10, $T11> ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
                Validation<$T8, E>,
                Validation<$T9, E>,
                Validation<$T10, E>,
                Validation<$T11, E>,
            )
        {
            type Output = ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10, $T11);

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10, $T11) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
                    .and($T8)
                    .map(|((a, b, c, d, e, f, g), h)| (a, b, c, d, e, f, g, h))
                    .and($T9)
                    .map(|((a, b, c, d, e, f, g, h), i)| (a, b, c, d, e, f, g, h, i))
                    .and($T10)
                    .map(|((a, b, c, d, e, f, g, h, i), j)| (a, b, c, d, e, f, g, h, i, j))
                    .and($T11)
                    .map(|((a, b, c, d, e, f, g, h, i, j), k)| (a, b, c, d, e, f, g, h, i, j, k))
            }
        }
    };

    // Twelve elements
    ($T1:ident, $T2:ident, $T3:ident, $T4:ident, $T5:ident, $T6:ident, $T7:ident, $T8:ident, $T9:ident, $T10:ident, $T11:ident, $T12:ident) => {
        impl<E: Semigroup, $T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10, $T11, $T12>
            ValidateAll<E>
            for (
                Validation<$T1, E>,
                Validation<$T2, E>,
                Validation<$T3, E>,
                Validation<$T4, E>,
                Validation<$T5, E>,
                Validation<$T6, E>,
                Validation<$T7, E>,
                Validation<$T8, E>,
                Validation<$T9, E>,
                Validation<$T10, E>,
                Validation<$T11, E>,
                Validation<$T12, E>,
            )
        {
            type Output = (
                $T1,
                $T2,
                $T3,
                $T4,
                $T5,
                $T6,
                $T7,
                $T8,
                $T9,
                $T10,
                $T11,
                $T12,
            );

            #[allow(non_snake_case)]
            fn validate_all(self) -> Validation<Self::Output, E> {
                let ($T1, $T2, $T3, $T4, $T5, $T6, $T7, $T8, $T9, $T10, $T11, $T12) = self;
                $T1.and($T2)
                    .and($T3)
                    .map(|((a, b), c)| (a, b, c))
                    .and($T4)
                    .map(|((a, b, c), d)| (a, b, c, d))
                    .and($T5)
                    .map(|((a, b, c, d), e)| (a, b, c, d, e))
                    .and($T6)
                    .map(|((a, b, c, d, e), f)| (a, b, c, d, e, f))
                    .and($T7)
                    .map(|((a, b, c, d, e, f), g)| (a, b, c, d, e, f, g))
                    .and($T8)
                    .map(|((a, b, c, d, e, f, g), h)| (a, b, c, d, e, f, g, h))
                    .and($T9)
                    .map(|((a, b, c, d, e, f, g, h), i)| (a, b, c, d, e, f, g, h, i))
                    .and($T10)
                    .map(|((a, b, c, d, e, f, g, h, i), j)| (a, b, c, d, e, f, g, h, i, j))
                    .and($T11)
                    .map(|((a, b, c, d, e, f, g, h, i, j), k)| (a, b, c, d, e, f, g, h, i, j, k))
                    .and($T12)
                    .map(|((a, b, c, d, e, f, g, h, i, j, k), l)| {
                        (a, b, c, d, e, f, g, h, i, j, k, l)
                    })
            }
        }
    };
}

// Generate implementations for tuples of size 1 through 12
impl_validate_all!(T1);
impl_validate_all!(T1, T2);
impl_validate_all!(T1, T2, T3);
impl_validate_all!(T1, T2, T3, T4);
impl_validate_all!(T1, T2, T3, T4, T5);
impl_validate_all!(T1, T2, T3, T4, T5, T6);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_validate_all!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

#[cfg(test)]
mod tests {
    use super::*;

    // Basic constructor tests
    #[test]
    fn test_success() {
        let v = Validation::<_, Vec<&str>>::success(42);
        assert!(v.is_success());
        assert!(!v.is_failure());
    }

    #[test]
    fn test_failure() {
        let v = Validation::<i32, _>::failure(vec!["error"]);
        assert!(v.is_failure());
        assert!(!v.is_success());
    }

    // Conversion tests
    #[test]
    fn test_from_result_ok() {
        let v = Validation::from_result(Ok::<_, Vec<&str>>(42));
        assert_eq!(v, Validation::Success(42));
    }

    #[test]
    fn test_from_result_err() {
        let v = Validation::from_result(Err::<i32, _>(vec!["error"]));
        assert_eq!(v, Validation::Failure(vec!["error"]));
    }

    #[test]
    fn test_into_result_success() {
        let v = Validation::<_, Vec<&str>>::success(42);
        assert_eq!(v.into_result(), Ok(42));
    }

    #[test]
    fn test_into_result_failure() {
        let v = Validation::<i32, _>::failure(vec!["error"]);
        assert_eq!(v.into_result(), Err(vec!["error"]));
    }

    // map tests
    #[test]
    fn test_map_on_success() {
        let v = Validation::<_, Vec<&str>>::success(5);
        let result = v.map(|x| x * 2);
        assert_eq!(result, Validation::Success(10));
    }

    #[test]
    fn test_map_on_failure() {
        let v = Validation::<i32, _>::failure(vec!["error"]);
        let result = v.map(|x| x * 2);
        assert_eq!(result, Validation::Failure(vec!["error"]));
    }

    #[test]
    fn test_map_err_on_success() {
        let v = Validation::<_, Vec<&str>>::success(42);
        let result = v.map_err(|errors| errors.len());
        assert_eq!(result, Validation::Success(42));
    }

    #[test]
    fn test_map_err_on_failure() {
        let v = Validation::<i32, _>::failure(vec!["error1", "error2"]);
        let result = v.map_err(|errors| errors.len());
        assert_eq!(result, Validation::Failure(2));
    }

    // and tests
    #[test]
    fn test_and_both_success() {
        let v1 = Validation::<_, Vec<&str>>::success(1);
        let v2 = Validation::<_, Vec<&str>>::success(2);
        assert_eq!(v1.and(v2), Validation::Success((1, 2)));
    }

    #[test]
    fn test_and_both_failure() {
        let v1 = Validation::<i32, _>::failure(vec!["error1"]);
        let v2 = Validation::<i32, _>::failure(vec!["error2"]);
        assert_eq!(v1.and(v2), Validation::Failure(vec!["error1", "error2"]));
    }

    #[test]
    fn test_and_first_failure() {
        let v1 = Validation::<i32, _>::failure(vec!["error"]);
        let v2 = Validation::<_, Vec<&str>>::success(2);
        assert_eq!(v1.and(v2), Validation::Failure(vec!["error"]));
    }

    #[test]
    fn test_and_second_failure() {
        let v1 = Validation::<_, Vec<&str>>::success(1);
        let v2 = Validation::<i32, _>::failure(vec!["error"]);
        assert_eq!(v1.and(v2), Validation::Failure(vec!["error"]));
    }

    // and_then tests
    #[test]
    fn test_and_then_success() {
        let v = Validation::<_, Vec<&str>>::success(5);
        let result = v.and_then(|x| Validation::success(x * 2));
        assert_eq!(result, Validation::Success(10));
    }

    #[test]
    fn test_and_then_failure() {
        let v = Validation::<i32, _>::failure(vec!["error"]);
        let result = v.and_then(|x| Validation::success(x * 2));
        assert_eq!(result, Validation::Failure(vec!["error"]));
    }

    #[test]
    fn test_and_then_chain_failure() {
        let v = Validation::<_, Vec<&str>>::success(5);
        let result: Validation<i32, Vec<&str>> = v.and_then(|_| Validation::failure(vec!["new error"]));
        assert_eq!(result, Validation::Failure(vec!["new error"]));
    }

    // all tests with tuples
    #[test]
    fn test_all_single_success() {
        use crate::validation::ValidateAll;
        let result = (
            Validation::<_, Vec<&str>>::success(1),
        ).validate_all();
        assert_eq!(result, Validation::Success((1,)));
    }

    #[test]
    fn test_all_two_success() {
        use crate::validation::ValidateAll;
        let result = (
            Validation::<_, Vec<&str>>::success(1),
            Validation::<_, Vec<&str>>::success(2),
        ).validate_all();
        assert_eq!(result, Validation::Success((1, 2)));
    }

    #[test]
    fn test_all_three_success() {
        use crate::validation::ValidateAll;
        let result = (
            Validation::<_, Vec<&str>>::success(1),
            Validation::<_, Vec<&str>>::success(2),
            Validation::<_, Vec<&str>>::success(3),
        ).validate_all();
        assert_eq!(result, Validation::Success((1, 2, 3)));
    }

    #[test]
    fn test_all_with_failures() {
        use crate::validation::ValidateAll;
        let result = (
            Validation::<i32, _>::failure(vec!["error1"]),
            Validation::<i32, _>::failure(vec!["error2"]),
            Validation::<i32, _>::failure(vec!["error3"]),
        ).validate_all();
        assert_eq!(
            result,
            Validation::Failure(vec!["error1", "error2", "error3"])
        );
    }

    #[test]
    fn test_all_mixed() {
        use crate::validation::ValidateAll;
        let result = (
            Validation::<_, Vec<&str>>::success(1),
            Validation::<i32, _>::failure(vec!["error1"]),
            Validation::<i32, _>::failure(vec!["error2"]),
        ).validate_all();
        assert_eq!(result, Validation::Failure(vec!["error1", "error2"]));
    }

    // all_vec tests
    #[test]
    fn test_all_vec_empty() {
        let validations: Vec<Validation<i32, Vec<&str>>> = vec![];
        let result = Validation::all_vec(validations);
        assert_eq!(result, Validation::Success(vec![]));
    }

    #[test]
    fn test_all_vec_all_success() {
        let validations = vec![
            Validation::<_, Vec<&str>>::success(1),
            Validation::<_, Vec<&str>>::success(2),
            Validation::<_, Vec<&str>>::success(3),
        ];
        let result = Validation::all_vec(validations);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    }

    #[test]
    fn test_all_vec_all_failures() {
        let validations = vec![
            Validation::<i32, _>::failure(vec!["error1"]),
            Validation::<i32, _>::failure(vec!["error2"]),
            Validation::<i32, _>::failure(vec!["error3"]),
        ];
        let result = Validation::all_vec(validations);
        assert_eq!(
            result,
            Validation::Failure(vec!["error1", "error2", "error3"])
        );
    }

    #[test]
    fn test_all_vec_mixed() {
        let validations = vec![
            Validation::<_, Vec<&str>>::success(1),
            Validation::failure(vec!["error1"]),
            Validation::<_, Vec<&str>>::success(2),
            Validation::failure(vec!["error2"]),
        ];
        let result = Validation::all_vec(validations);
        assert_eq!(result, Validation::Failure(vec!["error1", "error2"]));
    }

    // Integration test: form validation
    #[test]
    fn test_form_validation() {
        #[derive(Debug, PartialEq)]
        enum ValidationError {
            InvalidEmail,
            PasswordTooShort,
            AgeTooYoung,
        }

        fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
            if email.contains('@') {
                Validation::success(email.to_string())
            } else {
                Validation::failure(vec![ValidationError::InvalidEmail])
            }
        }

        fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
            if pwd.len() >= 8 {
                Validation::success(pwd.to_string())
            } else {
                Validation::failure(vec![ValidationError::PasswordTooShort])
            }
        }

        fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
            if age >= 18 {
                Validation::success(age)
            } else {
                Validation::failure(vec![ValidationError::AgeTooYoung])
            }
        }

        // All valid
        use crate::validation::ValidateAll;
        let result = (
            validate_email("test@example.com"),
            validate_password("secure123"),
            validate_age(25),
        ).validate_all();
        assert!(result.is_success());

        // All invalid - should accumulate all 3 errors
        let result = (
            validate_email("invalid"),
            validate_password("short"),
            validate_age(15),
        ).validate_all();
        assert_eq!(
            result,
            Validation::Failure(vec![
                ValidationError::InvalidEmail,
                ValidationError::PasswordTooShort,
                ValidationError::AgeTooYoung,
            ])
        );

        // Partial failures
        let result = (
            validate_email("test@example.com"),
            validate_password("short"),
            validate_age(15),
        ).validate_all();
        assert_eq!(
            result,
            Validation::Failure(vec![
                ValidationError::PasswordTooShort,
                ValidationError::AgeTooYoung,
            ])
        );
    }
}
