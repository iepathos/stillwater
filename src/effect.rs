//! Effect type for composing async computations with dependencies
//!
//! This module provides the `Effect` type, which represents a composable, async computation
//! that depends on an environment and may fail. Effects enable the "pure core, imperative shell"
//! pattern by separating business logic from I/O operations.
//!
//! # Core Concepts
//!
//! - **Pure Core**: Business logic is pure and testable (no I/O, no side effects)
//! - **Imperative Shell**: I/O operations are pushed to the boundaries
//! - **Environment**: Dependencies are injected explicitly through the environment parameter
//! - **Composability**: Effects can be composed using `map`, `and_then`, etc.
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```
//! use stillwater::Effect;
//!
//! # tokio_test::block_on(async {
//! // Create a pure effect
//! let effect = Effect::<_, String, ()>::pure(42);
//! assert_eq!(effect.run(&()).await, Ok(42));
//!
//! // Create a failed effect
//! let effect = Effect::<i32, _, ()>::fail("error");
//! assert_eq!(effect.run(&()).await, Err("error"));
//! # });
//! ```
//!
//! ## Composing effects
//!
//! ```
//! use stillwater::Effect;
//!
//! # tokio_test::block_on(async {
//! let effect = Effect::<_, String, ()>::pure(5)
//!     .map(|x| x * 2)
//!     .and_then(|x| Effect::pure(x + 10));
//!
//! assert_eq!(effect.run(&()).await, Ok(20));
//! # });
//! ```
//!
//! ## Using environment
//!
//! ```
//! use stillwater::Effect;
//!
//! # tokio_test::block_on(async {
//! struct Env {
//!     multiplier: i32,
//! }
//!
//! let effect = Effect::from_fn(|env: &Env| {
//!     Ok::<_, String>(env.multiplier * 2)
//! });
//!
//! let env = Env { multiplier: 21 };
//! assert_eq!(effect.run(&env).await, Ok(42));
//! # });
//! ```
//!
//! ## Async operations
//!
//! ```
//! use stillwater::Effect;
//!
//! # tokio_test::block_on(async {
//! let effect = Effect::from_async(|_: &()| async {
//!     // Simulate async I/O
//!     Ok::<_, String>(42)
//! });
//!
//! assert_eq!(effect.run(&()).await, Ok(42));
//! # });
//! ```

use std::future::Future;
use std::pin::Pin;

use crate::{ContextError, Validation};

/// A boxed future that is Send
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Function type for Effect internals
type EffectFn<T, E, Env> = Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<T, E>> + Send>;

/// An effect that may perform I/O and depends on an environment
///
/// `Effect<T, E, Env>` represents an async computation that:
/// - Produces a value of type `T` on success
/// - Fails with an error of type `E`
/// - Depends on an environment of type `Env`
///
/// Effects are lazy - they don't execute until `run()` is called.
///
/// # Type Parameters
///
/// * `T` - The type of the success value
/// * `E` - The type of the error value (defaults to `std::convert::Infallible`)
/// * `Env` - The type of the environment (defaults to `()`)
///
/// # Examples
///
/// ```
/// use stillwater::Effect;
///
/// # tokio_test::block_on(async {
/// // Effect with no error type
/// let effect: Effect<_, String> = Effect::pure(42);
/// assert_eq!(effect.run(&()).await, Ok(42));
///
/// // Effect with error type
/// let effect: Effect<i32, String> = Effect::fail("error".to_string());
/// assert_eq!(effect.run(&()).await, Err("error".to_string()));
/// # });
/// ```
pub struct Effect<T, E = std::convert::Infallible, Env = ()> {
    run_fn: EffectFn<T, E, Env>,
}

// Manual Debug implementation since FnOnce is not Debug
impl<T, E, Env> std::fmt::Debug for Effect<T, E, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effect")
            .field("run_fn", &"<function>")
            .finish()
    }
}

impl<T, E, Env> Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Create a pure value (no effects)
    ///
    /// This creates an effect that always succeeds with the given value,
    /// performing no I/O or side effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(42);
    /// assert_eq!(effect.run(&()).await, Ok(42));
    /// # });
    /// ```
    pub fn pure(value: T) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { Ok(value) })),
        }
    }

    /// Create a failing effect
    ///
    /// This creates an effect that always fails with the given error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<i32, _, ()>::fail("error");
    /// assert_eq!(effect.run(&()).await, Err("error"));
    /// # });
    /// ```
    pub fn fail(error: E) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { Err(error) })),
        }
    }

    /// Create from synchronous function
    ///
    /// Wraps a synchronous function that depends on the environment and returns a `Result`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::from_fn(|_: &()| Ok::<_, String>(42));
    /// assert_eq!(effect.run(&()).await, Ok(42));
    /// # });
    /// ```
    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnOnce(&Env) -> Result<T, E> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                let result = f(env);
                Box::pin(async move { result })
            }),
        }
    }

    /// Create from async function
    ///
    /// Wraps an async function that depends on the environment and returns a `Result`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::from_async(|_: &()| async {
    ///     Ok::<_, String>(42)
    /// });
    /// assert_eq!(effect.run(&()).await, Ok(42));
    /// # });
    /// ```
    pub fn from_async<F, Fut>(f: F) -> Self
    where
        F: FnOnce(&Env) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| Box::pin(f(env))),
        }
    }

    /// Create from Result
    ///
    /// Lifts a `Result` into an Effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::from_result(Ok(42));
    /// assert_eq!(effect.run(&()).await, Ok(42));
    ///
    /// let effect = Effect::<i32, _, ()>::from_result(Err("error"));
    /// assert_eq!(effect.run(&()).await, Err("error"));
    /// # });
    /// ```
    pub fn from_result(result: Result<T, E>) -> Self {
        Effect {
            run_fn: Box::new(move |_| Box::pin(async move { result })),
        }
    }

    /// Convert Validation to Effect
    ///
    /// Lifts a `Validation` into an Effect.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::{Effect, Validation};
    ///
    /// # tokio_test::block_on(async {
    /// let validation = Validation::<_, String>::success(42);
    /// let effect = Effect::from_validation(validation);
    /// assert_eq!(effect.run(&()).await, Ok(42));
    ///
    /// let validation = Validation::<i32, _>::failure("error");
    /// let effect = Effect::from_validation(validation);
    /// assert_eq!(effect.run(&()).await, Err("error"));
    /// # });
    /// ```
    pub fn from_validation(validation: Validation<T, E>) -> Self {
        match validation {
            Validation::Success(value) => Effect::pure(value),
            Validation::Failure(error) => Effect::fail(error),
        }
    }

    /// Chain effects
    ///
    /// If the current effect succeeds, apply the function to its result
    /// to produce the next effect. If it fails, propagate the error.
    ///
    /// This is similar to `Result::and_then`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(5)
    ///     .and_then(|x| Effect::pure(x * 2));
    /// assert_eq!(effect.run(&()).await, Ok(10));
    ///
    /// // Error propagation
    /// let effect = Effect::<_, String, ()>::fail("error".to_string())
    ///     .and_then(|x: i32| Effect::pure(x * 2));
    /// assert_eq!(effect.run(&()).await, Err("error".to_string()));
    /// # });
    /// ```
    pub fn and_then<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    let value = (self.run_fn)(env).await?;
                    let next = f(value);
                    (next.run_fn)(env).await
                })
            }),
        }
    }

    /// Transform success value
    ///
    /// Applies a function to the success value if the effect succeeds.
    /// This is similar to `Result::map`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(5)
    ///     .map(|x| x * 2);
    /// assert_eq!(effect.run(&()).await, Ok(10));
    /// # });
    /// ```
    pub fn map<U, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> U + Send + 'static,
        U: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| Box::pin(async move { (self.run_fn)(env).await.map(f) })),
        }
    }

    /// Transform error value
    ///
    /// Applies a function to the error value if the effect fails.
    /// This is similar to `Result::map_err`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<i32, _, ()>::fail("error")
    ///     .map_err(|e| format!("Failed: {}", e));
    /// assert_eq!(effect.run(&()).await, Err("Failed: error".to_string()));
    /// # });
    /// ```
    pub fn map_err<E2, F>(self, f: F) -> Effect<T, E2, Env>
    where
        F: FnOnce(E) -> E2 + Send + 'static,
        E2: Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move { (self.run_fn)(env).await.map_err(f) })
            }),
        }
    }

    /// Recover from errors
    ///
    /// If the effect fails, apply the recovery function to the error
    /// to produce a new effect. If it succeeds, return the value unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<i32, _, ()>::fail("error")
    ///     .or_else(|_| Effect::pure(42));
    /// assert_eq!(effect.run(&()).await, Ok(42));
    ///
    /// // No recovery needed for success
    /// let effect = Effect::<_, String, ()>::pure(100)
    ///     .or_else(|_| Effect::pure(42));
    /// assert_eq!(effect.run(&()).await, Ok(100));
    /// # });
    /// ```
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> Effect<T, E, Env> + Send + 'static,
    {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    match (self.run_fn)(env).await {
                        Ok(value) => Ok(value),
                        Err(err) => {
                            let recovery = f(err);
                            (recovery.run_fn)(env).await
                        }
                    }
                })
            }),
        }
    }

    /// Run the effect with the given environment
    ///
    /// This executes the effect and returns a Future that resolves to a `Result`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(42);
    /// let result = effect.run(&()).await;
    /// assert_eq!(result, Ok(42));
    /// # });
    /// ```
    pub async fn run(self, env: &Env) -> Result<T, E> {
        (self.run_fn)(env).await
    }

    /// Perform a side effect and return the original value
    ///
    /// Useful for logging, metrics, or other operations that don't
    /// affect the main computation. The side effect function receives
    /// a reference to the value and must return an Effect. If the side
    /// effect fails, the entire computation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(42)
    ///     .tap(|value| {
    ///         println!("Value: {}", value);
    ///         Effect::pure(())
    ///     });
    ///
    /// assert_eq!(effect.run(&()).await, Ok(42));
    /// # });
    /// ```
    #[inline]
    pub fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Effect<(), E, Env> + Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }

    /// Fail with error if predicate is false
    ///
    /// Provides a declarative way to express validation conditions.
    /// If the predicate returns true, the value passes through unchanged.
    /// If false, the error function is called to produce an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// // Success case
    /// let effect = Effect::<_, String, ()>::pure(25)
    ///     .check(|age| *age >= 18, || "too young".to_string());
    /// assert_eq!(effect.run(&()).await, Ok(25));
    ///
    /// // Failure case
    /// let effect = Effect::<_, String, ()>::pure(15)
    ///     .check(|age| *age >= 18, || "too young".to_string());
    /// assert_eq!(effect.run(&()).await, Err("too young".to_string()));
    /// # });
    /// ```
    #[inline]
    pub fn check<P, F>(self, predicate: P, error_fn: F) -> Self
    where
        P: FnOnce(&T) -> bool + Send + 'static,
        F: FnOnce() -> E + Send + 'static,
    {
        self.and_then(move |value| {
            if predicate(&value) {
                Effect::pure(value)
            } else {
                Effect::fail(error_fn())
            }
        })
    }

    /// Combine with another effect, returning both values
    ///
    /// Useful when you need results from multiple effects.
    /// The function receives a reference to the first value
    /// and returns an effect for the second value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(5)
    ///     .with(|value| Effect::pure(*value * 2))
    ///     .map(|(first, second)| first + second);
    ///
    /// assert_eq!(effect.run(&()).await, Ok(15));  // 5 + 10
    /// # });
    /// ```
    #[inline]
    pub fn with<U, F>(self, f: F) -> Effect<(T, U), E, Env>
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |other| (value_clone, other))
        })
    }

    /// Chain effect with automatic error conversion
    ///
    /// Eliminates manual `.map_err(E::from)` calls when error types differ.
    /// The error type from the chained effect must be convertible to the
    /// current error type via the `From` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// #[derive(Debug, PartialEq)]
    /// enum ValidationError {
    ///     Invalid,
    /// }
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum AppError {
    ///     Validation(ValidationError),
    /// }
    ///
    /// impl From<ValidationError> for AppError {
    ///     fn from(e: ValidationError) -> Self {
    ///         AppError::Validation(e)
    ///     }
    /// }
    ///
    /// let effect = Effect::<_, AppError, ()>::pure(42)
    ///     .and_then_auto(|_| Effect::<i32, ValidationError, ()>::pure(100));
    ///
    /// assert_eq!(effect.run(&()).await, Ok(100));
    /// # });
    /// ```
    #[inline]
    pub fn and_then_auto<U, E2, F>(self, f: F) -> Effect<U, E, Env>
    where
        F: FnOnce(T) -> Effect<U, E2, Env> + Send + 'static,
        U: Send + 'static,
        E2: Send + 'static,
        E: From<E2>,
    {
        self.and_then(move |value| f(value).map_err(E::from))
    }

    /// Chain effect by borrowing value, then returning it
    ///
    /// Avoids multiple clones when you need to use a value in multiple effects
    /// but only care about the final result. The function receives a reference
    /// to the value and returns an effect whose result is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<_, String, ()>::pure(42)
    ///     .and_then_ref(|value| {
    ///         assert_eq!(*value, 42);
    ///         Effect::pure("processed")
    ///     })
    ///     .and_then_ref(|value| {
    ///         assert_eq!(*value, 42);
    ///         Effect::pure("again")
    ///     });
    ///
    /// assert_eq!(effect.run(&()).await, Ok(42));
    /// # });
    /// ```
    #[inline]
    pub fn and_then_ref<U, F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> Effect<U, E, Env> + Send + 'static,
        U: Send + 'static,
        T: Clone,
    {
        self.and_then(move |value| {
            let value_clone = value.clone();
            f(&value).map(move |_| value_clone)
        })
    }
}

// Extension trait for adding context - workaround for lack of specialization
/// Extension trait for adding context to Effect errors
pub trait EffectContext<T, E, Env> {
    /// Add context to errors from this effect
    fn context(self, msg: impl Into<String> + Send + 'static) -> Effect<T, ContextError<E>, Env>;
}

// Implementation for general errors - wraps E in ContextError
impl<T, E, Env> EffectContext<T, E, Env> for Effect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Add context to errors from this effect
    ///
    /// Wraps any error from this effect in a `ContextError` with the given context message.
    /// This enables building up a trail of context as errors propagate through the call stack.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    /// use stillwater::effect::EffectContext;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<i32, _, ()>::fail("connection refused")
    ///     .context("connecting to database");
    ///
    /// match effect.run(&()).await {
    ///     Err(ctx_err) => {
    ///         assert_eq!(ctx_err.inner(), &"connection refused");
    ///         assert_eq!(ctx_err.context_trail(), &["connecting to database"]);
    ///     }
    ///     Ok(_) => panic!("Expected error"),
    /// }
    /// # });
    /// ```
    fn context(self, msg: impl Into<String> + Send + 'static) -> Effect<T, ContextError<E>, Env> {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move {
                    (self.run_fn)(env)
                        .await
                        .map_err(|err| ContextError::new(err).context(msg))
                })
            }),
        }
    }
}

// Add inherent method for ContextError chaining
impl<T, E, Env> Effect<T, ContextError<E>, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Add another layer of context
    ///
    /// Adds a new context message to an effect that already has a `ContextError`.
    /// This allows building up deep context trails as errors propagate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Effect;
    /// use stillwater::effect::EffectContext;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = Effect::<i32, _, ()>::fail("file not found")
    ///     .context("reading config file")
    ///     .context("initializing application");
    ///
    /// match effect.run(&()).await {
    ///     Err(ctx_err) => {
    ///         assert_eq!(ctx_err.inner(), &"file not found");
    ///         assert_eq!(ctx_err.context_trail().len(), 2);
    ///     }
    ///     Ok(_) => panic!("Expected error"),
    /// }
    /// # });
    /// ```
    pub fn context(self, msg: impl Into<String> + Send + 'static) -> Self {
        Effect {
            run_fn: Box::new(move |env| {
                Box::pin(async move { (self.run_fn)(env).await.map_err(|err| err.context(msg)) })
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic constructor tests
    #[tokio::test]
    async fn test_pure() {
        let effect = Effect::<_, String, ()>::pure(42);
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_fail() {
        let effect = Effect::<i32, _, ()>::fail("error");
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    // Conversion tests
    #[tokio::test]
    async fn test_from_result_ok() {
        let effect = Effect::<_, String, ()>::from_result(Ok(42));
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_from_result_err() {
        let effect = Effect::<i32, _, ()>::from_result(Err("error"));
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_from_fn_sync() {
        let effect = Effect::from_fn(|_: &()| Ok::<_, String>(42));
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_from_fn_sync_error() {
        let effect = Effect::from_fn(|_: &()| Err::<i32, _>("error"));
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_from_async() {
        let effect = Effect::from_async(|_: &()| async { Ok::<_, String>(42) });
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_from_async_error() {
        let effect = Effect::from_async(|_: &()| async { Err::<i32, _>("error") });
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_from_validation_success() {
        let validation = Validation::<_, String>::success(42);
        let effect = Effect::from_validation(validation);
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_from_validation_failure() {
        let validation = Validation::<i32, _>::failure("error");
        let effect = Effect::from_validation(validation);
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    // Combinator tests
    #[tokio::test]
    async fn test_map_success() {
        let effect = Effect::<_, String, ()>::pure(5).map(|x| x * 2);
        assert_eq!(effect.run(&()).await, Ok(10));
    }

    #[tokio::test]
    async fn test_map_failure() {
        let effect = Effect::<i32, _, ()>::fail("error").map(|x| x * 2);
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_map_err_success() {
        let effect = Effect::<_, String, ()>::pure(42).map_err(|e| format!("Failed: {}", e));
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_map_err_failure() {
        let effect = Effect::<i32, _, ()>::fail("error").map_err(|e| format!("Failed: {}", e));
        assert_eq!(effect.run(&()).await, Err("Failed: error".to_string()));
    }

    #[tokio::test]
    async fn test_and_then_success() {
        let effect = Effect::<_, String, ()>::pure(5).and_then(|x| Effect::pure(x * 2));
        assert_eq!(effect.run(&()).await, Ok(10));
    }

    #[tokio::test]
    async fn test_and_then_failure() {
        let effect = Effect::<i32, _, ()>::fail("error").and_then(|x| Effect::pure(x * 2));
        assert_eq!(effect.run(&()).await, Err("error"));
    }

    #[tokio::test]
    async fn test_and_then_chain_failure() {
        let effect = Effect::<_, String, ()>::pure(5)
            .and_then(|_| Effect::fail("error".to_string()))
            .map(|x: i32| x * 2); // This shouldn't run
        assert_eq!(effect.run(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_or_else_success() {
        let effect = Effect::<_, String, ()>::pure(100).or_else(|_| Effect::pure(42));
        assert_eq!(effect.run(&()).await, Ok(100));
    }

    #[tokio::test]
    async fn test_or_else_failure_recovery() {
        let effect = Effect::<i32, _, ()>::fail("error").or_else(|_| Effect::pure(42));
        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_or_else_failure_no_recovery() {
        let effect = Effect::<i32, _, ()>::fail("error1").or_else(|_| Effect::fail("error2"));
        assert_eq!(effect.run(&()).await, Err("error2"));
    }

    // Composition tests
    #[tokio::test]
    async fn test_mix_sync_and_async() {
        let effect = Effect::from_fn(|_: &()| Ok::<_, String>(5))
            .and_then(|x| Effect::from_async(move |_| async move { Ok(x * 2) }));
        assert_eq!(effect.run(&()).await, Ok(10));
    }

    #[tokio::test]
    async fn test_complex_chain() {
        let effect = Effect::<_, String, ()>::pure(2)
            .map(|x| x * 3) // 6
            .and_then(|x| Effect::pure(x + 4)) // 10
            .map(|x| x * 2) // 20
            .and_then(|x| Effect::pure(x / 2)); // 10
        assert_eq!(effect.run(&()).await, Ok(10));
    }

    // Environment tests
    #[tokio::test]
    async fn test_with_environment() {
        struct Env {
            value: i32,
        }

        let effect = Effect::from_fn(|env: &Env| Ok::<_, String>(env.value * 2));

        let env = Env { value: 21 };
        assert_eq!(effect.run(&env).await, Ok(42));
    }

    #[tokio::test]
    async fn test_with_environment_chained() {
        struct Env {
            multiplier: i32,
            adder: i32,
        }

        let effect = Effect::from_fn(|env: &Env| Ok::<_, String>(10 * env.multiplier))
            .and_then(|x| Effect::from_fn(move |env: &Env| Ok(x + env.adder)));

        let env = Env {
            multiplier: 3,
            adder: 12,
        };
        assert_eq!(effect.run(&env).await, Ok(42));
    }

    // Error propagation tests
    #[tokio::test]
    async fn test_error_propagation() {
        let effect = Effect::<_, String, ()>::pure(5)
            .and_then(|_| Effect::fail("error".to_string()))
            .map(|x: i32| x * 2); // This shouldn't run

        assert_eq!(effect.run(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_error_transformation() {
        let effect = Effect::<i32, _, ()>::fail(42)
            .map_err(|x| format!("Error code: {}", x))
            .or_else(|e| Effect::fail(format!("{} - recovered", e)));

        assert_eq!(
            effect.run(&()).await,
            Err("Error code: 42 - recovered".to_string())
        );
    }

    // Context error tests
    #[tokio::test]
    async fn test_effect_context() {
        let effect = Effect::<i32, _, ()>::fail("base error").context("operation failed");

        match effect.run(&()).await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"base error");
                assert_eq!(ctx_err.context_trail(), &["operation failed"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_multiple_contexts() {
        let effect = Effect::<i32, _, ()>::fail("base error")
            .context("step 1")
            .context("step 2")
            .context("step 3");

        match effect.run(&()).await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"base error");
                assert_eq!(ctx_err.context_trail(), &["step 1", "step 2", "step 3"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_context_success() {
        let effect = Effect::<_, String, ()>::pure(42).context("this context won't be used");

        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_effect_context_with_combinators() {
        let effect = Effect::<_, String, ()>::pure(5)
            .map(|x| x * 2)
            .and_then(|_| Effect::fail("error".to_string()))
            .context("step 1")
            .map(|x: i32| x + 10)
            .context("step 2");

        match effect.run(&()).await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"error".to_string());
                assert_eq!(ctx_err.context_trail(), &["step 1", "step 2"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_context_with_environment() {
        struct Env {
            database_url: String,
        }

        let effect = Effect::from_fn(|env: &Env| {
            Err::<i32, _>(format!("Connection failed: {}", env.database_url))
        })
        .context("connecting to database")
        .context("initializing application");

        let env = Env {
            database_url: "localhost:5432".to_string(),
        };

        match effect.run(&env).await {
            Err(ctx_err) => {
                assert!(ctx_err.inner().contains("Connection failed"));
                assert_eq!(ctx_err.context_trail().len(), 2);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_context_with_string_types() {
        let effect = Effect::<i32, _, ()>::fail(String::from("error"))
            .context(String::from("owned context"))
            .context("borrowed context");

        match effect.run(&()).await {
            Err(ctx_err) => {
                assert_eq!(
                    ctx_err.context_trail(),
                    &["owned context", "borrowed context"]
                );
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    // Helper combinator tests
    #[tokio::test]
    async fn test_tap_returns_original_value() {
        let effect = Effect::<_, String, ()>::pure(42).tap(|_value| Effect::pure(()));

        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_tap_side_effect_executes() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let effect = Effect::<_, String, ()>::pure(42).tap(move |value| {
            *called_clone.lock().unwrap() = true;
            assert_eq!(*value, 42);
            Effect::pure(())
        });

        assert_eq!(effect.run(&()).await, Ok(42));
        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_tap_propagates_side_effect_failure() {
        let effect =
            Effect::<_, String, ()>::pure(42).tap(|_value| Effect::fail("tap failed".to_string()));

        assert_eq!(effect.run(&()).await, Err("tap failed".to_string()));
    }

    #[tokio::test]
    async fn test_tap_on_failure_doesnt_execute() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let effect = Effect::<i32, _, ()>::fail("error".to_string()).tap(move |_value| {
            *called_clone.lock().unwrap() = true;
            Effect::pure(())
        });

        assert_eq!(effect.run(&()).await, Err("error".to_string()));
        assert!(!*called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_check_success() {
        let effect =
            Effect::<_, String, ()>::pure(25).check(|age| *age >= 18, || "too young".to_string());

        assert_eq!(effect.run(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_check_failure() {
        let effect =
            Effect::<_, String, ()>::pure(15).check(|age| *age >= 18, || "too young".to_string());

        assert_eq!(effect.run(&()).await, Err("too young".to_string()));
    }

    #[tokio::test]
    async fn test_check_multiple_conditions() {
        let effect = Effect::<_, String, ()>::pure(25)
            .check(|age| *age >= 18, || "too young".to_string())
            .check(|age| *age <= 65, || "too old".to_string());

        assert_eq!(effect.run(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_check_on_failure_doesnt_execute() {
        let effect = Effect::<i32, _, ()>::fail("error".to_string())
            .check(|age| *age >= 18, || "too young".to_string());

        assert_eq!(effect.run(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_with_returns_tuple() {
        let effect = Effect::<_, String, ()>::pure(5).with(|value| Effect::pure(*value * 2));

        assert_eq!(effect.run(&()).await, Ok((5, 10)));
    }

    #[tokio::test]
    async fn test_with_can_map_tuple() {
        let effect = Effect::<_, String, ()>::pure(5)
            .with(|value| Effect::pure(*value * 2))
            .map(|(first, second)| first + second);

        assert_eq!(effect.run(&()).await, Ok(15));
    }

    #[tokio::test]
    async fn test_with_propagates_first_failure() {
        let effect =
            Effect::<i32, _, ()>::fail("error".to_string()).with(|value| Effect::pure(*value * 2));

        assert_eq!(effect.run(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn test_with_propagates_second_failure() {
        let effect = Effect::<_, String, ()>::pure(5)
            .with(|_value| Effect::<i32, String, ()>::fail("second failed".to_string()));

        assert_eq!(effect.run(&()).await, Err("second failed".to_string()));
    }

    #[tokio::test]
    async fn test_and_then_auto_converts_error() {
        #[derive(Debug, PartialEq)]
        enum Error1 {
            Fail,
        }

        #[derive(Debug, PartialEq)]
        enum Error2 {
            Other(Error1),
        }

        impl From<Error1> for Error2 {
            fn from(e: Error1) -> Self {
                Error2::Other(e)
            }
        }

        let effect = Effect::<_, Error2, ()>::pure(42)
            .and_then_auto(|_| Effect::<i32, Error1, ()>::fail(Error1::Fail));

        assert_eq!(effect.run(&()).await, Err(Error2::Other(Error1::Fail)));
    }

    #[tokio::test]
    async fn test_and_then_auto_success() {
        #[derive(Debug, PartialEq)]
        enum Error1 {
            Fail,
        }

        #[derive(Debug, PartialEq)]
        enum Error2 {
            Other(Error1),
        }

        impl From<Error1> for Error2 {
            fn from(e: Error1) -> Self {
                Error2::Other(e)
            }
        }

        let effect =
            Effect::<_, Error2, ()>::pure(42).and_then_auto(|_| Effect::<_, Error1, ()>::pure(100));

        assert_eq!(effect.run(&()).await, Ok(100));
    }

    #[tokio::test]
    async fn test_and_then_auto_chain() {
        #[derive(Debug, PartialEq)]
        enum ValidationError {
            Invalid,
        }

        #[derive(Debug, PartialEq)]
        enum DbError {
            NotFound,
        }

        #[derive(Debug, PartialEq)]
        enum AppError {
            Validation(ValidationError),
            Database(DbError),
        }

        impl From<ValidationError> for AppError {
            fn from(e: ValidationError) -> Self {
                AppError::Validation(e)
            }
        }

        impl From<DbError> for AppError {
            fn from(e: DbError) -> Self {
                AppError::Database(e)
            }
        }

        let effect = Effect::<_, AppError, ()>::pure(42)
            .and_then_auto(|_| Effect::<i32, ValidationError, ()>::pure(100))
            .and_then_auto(|_| Effect::<i32, DbError, ()>::pure(200));

        assert_eq!(effect.run(&()).await, Ok(200));
    }

    #[tokio::test]
    async fn test_and_then_ref_returns_original() {
        let effect = Effect::<_, String, ()>::pure(42).and_then_ref(|value| {
            assert_eq!(*value, 42);
            Effect::pure("processed")
        });

        assert_eq!(effect.run(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_and_then_ref_multiple_calls() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let count = Arc::new(Mutex::new(0));
        let count1 = count.clone();
        let count2 = count.clone();

        let effect = Effect::<_, String, ()>::pure(42)
            .and_then_ref(move |value| {
                *count1.lock().unwrap() += 1;
                assert_eq!(*value, 42);
                Effect::pure("first")
            })
            .and_then_ref(move |value| {
                *count2.lock().unwrap() += 1;
                assert_eq!(*value, 42);
                Effect::pure("second")
            });

        assert_eq!(effect.run(&()).await, Ok(42));
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_and_then_ref_propagates_failure() {
        let effect = Effect::<_, String, ()>::pure(42)
            .and_then_ref(|_value| Effect::<(), String, ()>::fail("ref failed".to_string()));

        assert_eq!(effect.run(&()).await, Err("ref failed".to_string()));
    }

    #[tokio::test]
    async fn test_and_then_ref_on_failure_doesnt_execute() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        let effect = Effect::<i32, _, ()>::fail("error".to_string()).and_then_ref(move |_value| {
            *called_clone.lock().unwrap() = true;
            Effect::pure("processed")
        });

        assert_eq!(effect.run(&()).await, Err("error".to_string()));
        assert!(!*called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_composition_all_helpers() {
        let effect = Effect::<_, String, ()>::pure(20)
            .check(|age| *age >= 18, || "too young".to_string())
            .tap(|_age| Effect::pure(()))
            .with(|age| Effect::pure(*age * 2))
            .map(|(age, double)| age + double);

        assert_eq!(effect.run(&()).await, Ok(60)); // 20 + 40
    }

    #[tokio::test]
    async fn test_composition_with_and_then_ref() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let log = Arc::new(Mutex::new(Vec::new()));
        let log1 = log.clone();
        let log2 = log.clone();

        let effect = Effect::<_, String, ()>::pure(42)
            .and_then_ref(move |value| {
                log1.lock().unwrap().push(format!("step1: {}", value));
                Effect::pure(())
            })
            .and_then_ref(move |value| {
                log2.lock().unwrap().push(format!("step2: {}", value));
                Effect::pure(())
            })
            .map(|value| value * 2);

        assert_eq!(effect.run(&()).await, Ok(84));
        assert_eq!(
            *log.lock().unwrap(),
            vec!["step1: 42".to_string(), "step2: 42".to_string()]
        );
    }
}
