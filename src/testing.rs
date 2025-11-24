//! Testing utilities and helpers for Stillwater
//!
//! This module provides ergonomic utilities for testing code that uses Stillwater's
//! types and effects. It includes mock environment builders, assertion macros, and
//! property-based testing support.
//!
//! # Examples
//!
//! ## MockEnv Builder
//!
//! ```rust
//! use stillwater::testing::MockEnv;
//!
//! struct Database {
//!     data: Vec<String>,
//! }
//!
//! let env = MockEnv::new()
//!     .with(|| Database { data: vec!["test".to_string()] })
//!     .build();
//! ```
//!
//! ## Assertion Macros
//!
//! ```rust
//! use stillwater::{Validation, assert_success, assert_failure};
//!
//! let success = Validation::<_, Vec<String>>::success(42);
//! assert_success!(success);
//!
//! let failure = Validation::<i32, _>::failure(vec!["error".to_string()]);
//! assert_failure!(failure);
//! ```

/// Builder for creating test environments.
///
/// `MockEnv` provides a composable way to build test environments by chaining
/// dependencies together. Each call to `with()` adds a new component to the
/// environment, creating a nested tuple structure.
///
/// # Example
///
/// ```rust
/// use stillwater::testing::MockEnv;
///
/// struct Config {
///     debug: bool,
/// }
///
/// struct Database {
///     url: String,
/// }
///
/// let env = MockEnv::new()
///     .with(|| Config { debug: true })
///     .with(|| Database { url: "test://localhost".to_string() })
///     .build();
///
/// // env is now (((), Config), Database)
/// let ((_, config), db) = env;
/// assert_eq!(config.debug, true);
/// assert_eq!(db.url, "test://localhost");
/// ```
#[derive(Debug)]
pub struct MockEnv<Env> {
    env: Env,
}

impl MockEnv<()> {
    /// Create a new empty mock environment.
    pub fn new() -> Self {
        Self { env: () }
    }
}

impl Default for MockEnv<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Env> MockEnv<Env> {
    /// Add a new component to the environment.
    ///
    /// The component is created by calling the provided function. This allows
    /// for lazy initialization and better composability.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::testing::MockEnv;
    ///
    /// struct Counter {
    ///     count: i32,
    /// }
    ///
    /// let env = MockEnv::new()
    ///     .with(|| Counter { count: 0 })
    ///     .build();
    /// ```
    pub fn with<F, T>(self, f: F) -> MockEnv<(Env, T)>
    where
        F: FnOnce() -> T,
    {
        MockEnv {
            env: (self.env, f()),
        }
    }

    /// Build the final environment.
    ///
    /// This consumes the builder and returns the constructed environment.
    pub fn build(self) -> Env {
        self.env
    }
}

/// Assert that a validation succeeds.
///
/// This macro will panic if the validation is a `Failure`.
///
/// # Example
///
/// ```rust
/// use stillwater::{Validation, assert_success};
///
/// let val = Validation::<_, Vec<String>>::success(42);
/// assert_success!(val);
/// ```
#[macro_export]
macro_rules! assert_success {
    ($validation:expr) => {
        match $validation {
            $crate::Validation::Success(_) => {}
            $crate::Validation::Failure(e) => {
                panic!("Expected Success, got Failure: {:?}", e);
            }
        }
    };
}

/// Assert that a validation fails.
///
/// This macro will panic if the validation is a `Success`.
///
/// # Example
///
/// ```rust
/// use stillwater::{Validation, assert_failure};
///
/// let val = Validation::<i32, _>::failure(vec!["error".to_string()]);
/// assert_failure!(val);
/// ```
#[macro_export]
macro_rules! assert_failure {
    ($validation:expr) => {
        match $validation {
            $crate::Validation::Failure(_) => {}
            $crate::Validation::Success(v) => {
                panic!("Expected Failure, got Success: {:?}", v);
            }
        }
    };
}

/// Assert that a validation fails with specific errors.
///
/// This macro will panic if the validation is a `Success` or if the errors
/// don't match the expected errors.
///
/// # Example
///
/// ```rust
/// use stillwater::{Validation, assert_validation_errors};
///
/// let val = Validation::<i32, _>::failure(vec!["error1", "error2"]);
/// assert_validation_errors!(val, vec!["error1", "error2"]);
/// ```
#[macro_export]
macro_rules! assert_validation_errors {
    ($validation:expr, $expected:expr) => {
        match $validation {
            $crate::Validation::Failure(errors) => {
                assert_eq!(errors, $expected);
            }
            $crate::Validation::Success(v) => {
                panic!(
                    "Expected Failure with errors {:?}, got Success: {:?}",
                    $expected, v
                );
            }
        }
    };
}

#[cfg(feature = "proptest")]
use proptest::prelude::*;

#[cfg(feature = "proptest")]
impl<T, E> Arbitrary for Validation<T, E>
where
    T: Arbitrary,
    E: Arbitrary,
{
    type Parameters = (T::Parameters, E::Parameters);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
        let (t_params, e_params) = args;
        prop_oneof![
            any_with::<T>(t_params).prop_map(Validation::success),
            any_with::<E>(e_params).prop_map(Validation::failure),
        ]
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Validation;

    #[test]
    fn mock_env_new() {
        let env = MockEnv::new().build();
        assert_eq!(env, ());
    }

    #[test]
    fn mock_env_with_single() {
        let env = MockEnv::new().with(|| 42).build();
        assert_eq!(env, ((), 42));
    }

    #[test]
    fn mock_env_with_multiple() {
        let env = MockEnv::new()
            .with(|| "hello")
            .with(|| 42)
            .with(|| true)
            .build();

        let (((_, s), _n), b) = env;
        assert_eq!(s, "hello");
        assert_eq!(b, true);
    }

    #[test]
    fn assert_success_macro() {
        let val = Validation::<_, Vec<String>>::success(42);
        assert_success!(val);
    }

    #[test]
    fn assert_failure_macro() {
        let val = Validation::<i32, _>::failure(vec!["error".to_string()]);
        assert_failure!(val);
    }

    #[test]
    fn assert_validation_errors_macro() {
        let val = Validation::<i32, _>::failure(vec!["error1", "error2"]);
        assert_validation_errors!(val, vec!["error1", "error2"]);
    }

    #[test]
    #[should_panic(expected = "Expected Success, got Failure")]
    fn assert_success_panics_on_failure() {
        let val = Validation::<i32, _>::failure(vec!["error".to_string()]);
        assert_success!(val);
    }

    #[test]
    #[should_panic(expected = "Expected Failure, got Success")]
    fn assert_failure_panics_on_success() {
        let val = Validation::<_, Vec<String>>::success(42);
        assert_failure!(val);
    }

    #[test]
    #[should_panic(expected = "Expected Failure with errors")]
    fn assert_validation_errors_panics_on_success() {
        let val = Validation::<_, Vec<String>>::success(42);
        assert_validation_errors!(val, vec!["error".to_string()]);
    }

    #[cfg(feature = "proptest")]
    mod proptest_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn validation_arbitrary_generates_valid_instances(
                val in any::<Validation<i32, Vec<String>>>()
            ) {
                match val {
                    Validation::Success(_) => assert!(val.is_success()),
                    Validation::Failure(_) => assert!(val.is_failure()),
                }
            }
        }
    }
}
