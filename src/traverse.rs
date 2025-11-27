//! Traverse and sequence utilities for working with collections of effects
//!
//! This module provides the fundamental operations for working with collections of `Validation`
//! and `Effect` types. These operations allow you to transform collections of effectful
//! computations in a composable, functional way.
//!
//! # Core Concepts
//!
//! - **`sequence`**: Convert a collection of effects into an effect of a collection
//!   - `Vec<Validation<T, E>>` → `Validation<Vec<T>, E>`
//!   - `Vec<Effect<T, E, Env>>` → `Effect<Vec<T>, E, Env>`
//!
//! - **`traverse`**: Map a function over a collection and sequence the results
//!   - Equivalent to `map(f).sequence()` but more efficient
//!
//! # Examples
//!
//! ## Validation
//!
//! ```
//! use stillwater::{Validation, traverse::traverse};
//!
//! fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
//!     s.parse()
//!         .map(Validation::success)
//!         .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid number: {}", s)]))
//! }
//!
//! let numbers = vec!["1", "2", "3"];
//! let result = traverse(numbers, parse_number);
//! assert_eq!(result, Validation::Success(vec![1, 2, 3]));
//!
//! let mixed = vec!["1", "invalid", "3"];
//! let result = traverse(mixed, parse_number);
//! assert!(result.is_failure());
//! ```
//!
//! ## Effect
//!
//! ```
//! use stillwater::{Effect, traverse::traverse_effect};
//!
//! # tokio_test::block_on(async {
//! fn process(x: i32) -> Effect<i32, String, ()> {
//!     Effect::pure(x * 2)
//! }
//!
//! let numbers = vec![1, 2, 3];
//! let effect = traverse_effect(numbers, process);
//! assert_eq!(effect.run_standalone().await, Ok(vec![2, 4, 6]));
//! # });
//! ```

use crate::{Effect, Semigroup, Validation};

/// Traverse a collection with a validation function.
///
/// Applies `f` to each element, accumulating all errors if any fail.
/// If all validations succeed, returns a success with a vector of all results.
///
/// # Type Parameters
///
/// * `T` - Input element type
/// * `U` - Output element type
/// * `E` - Error type (must implement `Semigroup` for error accumulation)
/// * `F` - Function type that transforms `T` into `Validation<U, E>`
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Validation, traverse::traverse};
///
/// fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
///     if x > 0 {
///         Validation::success(x)
///     } else {
///         Validation::failure(vec![format!("{} is not positive", x)])
///     }
/// }
///
/// let result = traverse(vec![1, 2, 3], validate_positive);
/// assert_eq!(result, Validation::Success(vec![1, 2, 3]));
///
/// let result = traverse(vec![1, -2, -3], validate_positive);
/// assert!(result.is_failure());
/// ```
pub fn traverse<T, U, E, F, I>(iter: I, f: F) -> Validation<Vec<U>, E>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Validation<U, E>,
    E: Semigroup,
{
    let validations: Vec<_> = iter.into_iter().map(f).collect();
    Validation::all_vec(validations)
}

/// Sequence a collection of validations.
///
/// Converts a collection of validations into a validation of a collection.
/// If all validations succeed, returns success with all values.
/// If any fail, accumulates all errors using `Semigroup`.
///
/// # Type Parameters
///
/// * `T` - Success value type
/// * `E` - Error type (must implement `Semigroup` for error accumulation)
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Validation, traverse::sequence};
///
/// let vals = vec![
///     Validation::<_, Vec<String>>::success(1),
///     Validation::success(2),
///     Validation::success(3),
/// ];
/// let result = sequence(vals);
/// assert_eq!(result, Validation::Success(vec![1, 2, 3]));
///
/// let vals = vec![
///     Validation::<i32, _>::failure(vec!["error1".to_string()]),
///     Validation::success(2),
///     Validation::failure(vec!["error2".to_string()]),
/// ];
/// let result = sequence(vals);
/// assert!(result.is_failure());
/// ```
pub fn sequence<T, E, I>(iter: I) -> Validation<Vec<T>, E>
where
    I: IntoIterator<Item = Validation<T, E>>,
    E: Semigroup,
{
    Validation::all_vec(iter.into_iter().collect())
}

/// Traverse a collection with an effect function.
///
/// Applies `f` to each element sequentially, collecting all results.
/// Uses fail-fast semantics - stops at the first error.
///
/// # Type Parameters
///
/// * `T` - Input element type
/// * `U` - Output element type
/// * `E` - Error type
/// * `Env` - Environment type
/// * `F` - Function type that transforms `T` into `Effect<U, E, Env>`
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Effect, traverse::traverse_effect};
///
/// # tokio_test::block_on(async {
/// fn double(x: i32) -> Effect<i32, String, ()> {
///     Effect::pure(x * 2)
/// }
///
/// let result = traverse_effect(vec![1, 2, 3], double);
/// assert_eq!(result.run_standalone().await, Ok(vec![2, 4, 6]));
/// # });
/// ```
pub fn traverse_effect<T, U, E, Env, F, I>(iter: I, f: F) -> Effect<Vec<U>, E, Env>
where
    I: IntoIterator<Item = T>,
    F: Fn(T) -> Effect<U, E, Env> + Clone + Send + 'static,
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    let items: Vec<_> = iter.into_iter().collect();
    let effects: Vec<_> = items.into_iter().map(f).collect();
    Effect::par_try_all(effects)
}

/// Sequence a collection of effects.
///
/// Converts a collection of effects into an effect of a collection.
/// Executes effects sequentially with fail-fast semantics.
///
/// # Type Parameters
///
/// * `T` - Success value type
/// * `E` - Error type
/// * `Env` - Environment type
/// * `I` - Input iterator type
///
/// # Examples
///
/// ```
/// use stillwater::{Effect, traverse::sequence_effect};
///
/// # tokio_test::block_on(async {
/// let effects = vec![
///     Effect::<_, String, ()>::pure(1),
///     Effect::pure(2),
///     Effect::pure(3),
/// ];
/// let result = sequence_effect(effects);
/// assert_eq!(result.run_standalone().await, Ok(vec![1, 2, 3]));
/// # });
/// ```
pub fn sequence_effect<T, E, Env, I>(iter: I) -> Effect<Vec<T>, E, Env>
where
    I: IntoIterator<Item = Effect<T, E, Env>> + Send + 'static,
    I::IntoIter: Send,
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    Effect::par_try_all(iter)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validation traverse tests
    #[test]
    fn test_traverse_all_success() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(vec![1, 2, 3], validate_positive);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    }

    #[test]
    fn test_traverse_with_failures() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(vec![1, -2, -3], validate_positive);
        assert!(result.is_failure());

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_traverse_empty() {
        fn validate_positive(x: i32) -> Validation<i32, Vec<String>> {
            if x > 0 {
                Validation::success(x)
            } else {
                Validation::failure(vec![format!("{} is not positive", x)])
            }
        }

        let result = traverse(Vec::<i32>::new(), validate_positive);
        assert_eq!(result, Validation::Success(vec![]));
    }

    // Validation sequence tests
    #[test]
    fn test_sequence_all_success() {
        let vals = vec![
            Validation::<_, Vec<String>>::success(1),
            Validation::success(2),
            Validation::success(3),
        ];
        let result = sequence(vals);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));
    }

    #[test]
    fn test_sequence_with_failures() {
        let vals = vec![
            Validation::<i32, _>::failure(vec!["error1".to_string()]),
            Validation::success(2),
            Validation::failure(vec!["error2".to_string()]),
        ];
        let result = sequence(vals);
        assert!(result.is_failure());
    }

    #[test]
    fn test_sequence_empty() {
        let vals: Vec<Validation<i32, Vec<String>>> = vec![];
        let result = sequence(vals);
        assert_eq!(result, Validation::Success(vec![]));
    }

    // Effect traverse tests
    #[tokio::test]
    async fn test_traverse_effect_all_success() {
        fn double(x: i32) -> Effect<i32, String, ()> {
            Effect::pure(x * 2)
        }

        let result = traverse_effect(vec![1, 2, 3], double);
        assert_eq!(result.run_standalone().await, Ok(vec![2, 4, 6]));
    }

    #[tokio::test]
    async fn test_traverse_effect_with_failure() {
        fn check_positive(x: i32) -> Effect<i32, String, ()> {
            if x > 0 {
                Effect::pure(x)
            } else {
                Effect::fail(format!("{} is not positive", x))
            }
        }

        let result = traverse_effect(vec![1, -2, 3], check_positive);
        assert!(result.run_standalone().await.is_err());
    }

    #[tokio::test]
    async fn test_traverse_effect_empty() {
        fn double(x: i32) -> Effect<i32, String, ()> {
            Effect::pure(x * 2)
        }

        let result = traverse_effect(Vec::<i32>::new(), double);
        assert_eq!(result.run_standalone().await, Ok(vec![]));
    }

    // Effect sequence tests
    #[tokio::test]
    async fn test_sequence_effect_all_success() {
        let effects = vec![
            Effect::<_, String, ()>::pure(1),
            Effect::pure(2),
            Effect::pure(3),
        ];
        let result = sequence_effect(effects);
        assert_eq!(result.run_standalone().await, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn test_sequence_effect_with_failure() {
        let effects = vec![
            Effect::<_, String, ()>::pure(1),
            Effect::fail("error".to_string()),
            Effect::pure(3),
        ];
        let result = sequence_effect(effects);
        assert!(result.run_standalone().await.is_err());
    }

    #[tokio::test]
    async fn test_sequence_effect_empty() {
        let effects: Vec<Effect<i32, String, ()>> = vec![];
        let result = sequence_effect(effects);
        assert_eq!(result.run_standalone().await, Ok(vec![]));
    }

    // Integration tests
    #[test]
    fn test_traverse_with_parse() {
        fn parse_number(s: &str) -> Validation<i32, Vec<String>> {
            s.parse()
                .map(Validation::success)
                .unwrap_or_else(|_| Validation::failure(vec![format!("Invalid number: {}", s)]))
        }

        let numbers = vec!["1", "2", "3"];
        let result = traverse(numbers, parse_number);
        assert_eq!(result, Validation::Success(vec![1, 2, 3]));

        let mixed = vec!["1", "invalid", "3"];
        let result = traverse(mixed, parse_number);
        assert!(result.is_failure());
    }

    #[tokio::test]
    async fn test_traverse_effect_with_env() {
        struct Env {
            multiplier: i32,
        }

        fn multiply(x: i32) -> Effect<i32, String, Env> {
            Effect::<i32, String, Env>::asks(move |env: &Env| x * env.multiplier)
        }

        let env = Env { multiplier: 3 };
        let result = traverse_effect(vec![1, 2, 3], multiply);
        assert_eq!(result.run(&env).await, Ok(vec![3, 6, 9]));
    }
}
