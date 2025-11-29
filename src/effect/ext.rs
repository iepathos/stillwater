//! Extension trait providing combinator methods for all Effects.
//!
//! The `EffectExt` trait is automatically implemented for all types
//! that implement `Effect`. It provides ergonomic combinator methods
//! like `map`, `and_then`, `or_else`, and `boxed`.

use std::marker::PhantomData;

use crate::effect::boxed::BoxedEffect;
use crate::effect::combinators::{
    AndThen, AndThenAuto, AndThenRef, Check, Ensure, EnsurePred, EnsureWith, Fallback, FallbackTo,
    Map, MapErr, OrElse, Recover, RecoverSome, RecoverWith, Tap, Unless, With, Zip, ZipWith,
};
use crate::effect::reader::Local;
use crate::effect::trait_def::Effect;

/// Extension trait providing combinator methods for all Effects.
///
/// This trait is automatically implemented for all types that implement `Effect`.
/// You don't need to implement this trait yourself.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(21)
///     .map(|x| x * 2)
///     .and_then(|x| pure(x + 1))
///     .map_err(|e| format!("Error: {}", e));
///
/// assert_eq!(effect.execute(&()).await, Ok(43));
/// ```
pub trait EffectExt: Effect {
    /// Transform the success value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = pure::<_, String, ()>(21).map(|x| x * 2);
    /// assert_eq!(effect.execute(&()).await, Ok(42));
    /// ```
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Send,
        U: Send,
    {
        Map { inner: self, f }
    }

    /// Transform the error value.
    ///
    /// Useful for converting error types to enable chaining with `and_then`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = fail::<i32, _, ()>("error")
    ///     .map_err(|e| format!("wrapped: {}", e));
    /// assert_eq!(effect.execute(&()).await, Err("wrapped: error".to_string()));
    /// ```
    fn map_err<E2, F>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E2 + Send,
        E2: Send,
    {
        MapErr { inner: self, f }
    }

    /// Chain a dependent effect.
    ///
    /// If this effect succeeds, apply the function to produce the next effect.
    /// If this effect fails, propagate the error.
    ///
    /// # Note on Error Types
    ///
    /// The chained effect must have the same error type. Use `map_err`
    /// to convert error types before chaining:
    ///
    /// ```rust,ignore
    /// fetch_user(id)                           // Error = DbError
    ///     .map_err(AppError::from)             // Error = AppError
    ///     .and_then(|user| send_email(user))   // Error = AppError
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = pure::<_, String, ()>(21)
    ///     .and_then(|x| pure(x * 2));
    /// assert_eq!(effect.execute(&()).await, Ok(42));
    /// ```
    fn and_then<E2, F>(self, f: F) -> AndThen<Self, F>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        F: FnOnce(Self::Output) -> E2 + Send,
    {
        AndThen { inner: self, f }
    }

    /// Recover from an error.
    ///
    /// If this effect fails, apply the recovery function to produce a new effect.
    /// If this effect succeeds, the value passes through unchanged.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = fail::<i32, _, ()>("error")
    ///     .or_else(|_| pure(42));
    /// assert_eq!(effect.execute(&()).await, Ok(42));
    /// ```
    fn or_else<E2, F>(self, f: F) -> OrElse<Self, F>
    where
        E2: Effect<Output = Self::Output, Env = Self::Env>,
        F: FnOnce(Self::Error) -> E2 + Send,
    {
        OrElse { inner: self, f }
    }

    /// Recover from errors matching a predicate.
    ///
    /// If the effect fails and the predicate returns true for the error,
    /// the handler is called to produce a recovery effect. If the predicate
    /// returns false, the error propagates unchanged.
    ///
    /// Uses the `Predicate<Error>` trait, which supports both closures
    /// and composable predicate combinators.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    /// use stillwater::predicate::*;
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// enum Error {
    ///     CacheMiss,
    ///     NetworkError(String),
    /// }
    ///
    /// // Using a closure (works via blanket impl)
    /// let effect = fetch_from_cache(id)
    ///     .recover(
    ///         |e: &Error| matches!(e, Error::CacheMiss),
    ///         |_| fetch_from_db(id)
    ///     );
    ///
    /// // Or using composable predicates
    /// let is_cache_miss = |e: &Error| matches!(e, Error::CacheMiss);
    /// let is_timeout = |e: &Error| matches!(e, Error::NetworkError(s) if s.contains("timeout"));
    /// let recoverable = is_cache_miss.or(is_timeout);
    ///
    /// let effect = fetch_from_cache(id)
    ///     .recover(recoverable, |_| fetch_from_db(id));
    /// ```
    fn recover<P, H, E2>(self, predicate: P, handler: H) -> Recover<Self, P, H, E2>
    where
        P: crate::predicate::Predicate<Self::Error>,
        H: FnOnce(Self::Error) -> E2 + Send,
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        Recover::new(self, predicate, handler)
    }

    /// Recover from errors with a Result-returning function.
    ///
    /// Similar to `recover`, but the handler returns a Result directly
    /// instead of an Effect. Useful when recovery doesn't need environment.
    ///
    /// Uses the `Predicate<Error>` trait for consistency with `recover`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    ///
    /// let effect = parse_config()
    ///     .recover_with(
    ///         |e: &ConfigError| e.is_missing_field(),
    ///         |_| Ok(Config::default())
    ///     );
    /// ```
    fn recover_with<P, F>(self, predicate: P, f: F) -> RecoverWith<Self, P, F>
    where
        P: crate::predicate::Predicate<Self::Error>,
        F: FnOnce(Self::Error) -> Result<Self::Output, Self::Error> + Send,
    {
        RecoverWith::new(self, predicate, f)
    }

    /// Recover using a partial function.
    ///
    /// The function returns `Some(effect)` to recover, or `None` to let
    /// the error propagate. This is useful for pattern-matching on errors.
    ///
    /// Requires `Error: Clone` because the error must be cloned before
    /// being passed to the partial function, so it can be returned if
    /// `None` is produced.
    ///
    /// For non-Clone errors, use `recover` or `recover_with` instead.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    ///
    /// #[derive(Debug, Clone)]
    /// enum Error {
    ///     Timeout,
    ///     NotFound,
    ///     Fatal(String),
    /// }
    ///
    /// let effect = risky_operation()
    ///     .recover_some(|e| match e {
    ///         Error::Timeout => Some(pure(default_value())),
    ///         Error::NotFound => Some(create_new()),
    ///         _ => None, // Other errors propagate
    ///     });
    /// ```
    fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F, E2>
    where
        Self::Error: Clone,
        F: FnOnce(Self::Error) -> Option<E2> + Send,
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        RecoverSome::new(self, f)
    }

    /// Provide a default value on any error.
    ///
    /// Returns the default value directly on any error without wrapping
    /// in an effect. The default value is moved into the combinator.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    ///
    /// let count = get_count().fallback(0);
    /// // Returns 0 on any error
    /// ```
    fn fallback(self, default: Self::Output) -> Fallback<Self>
    where
        Self::Output: Send,
    {
        Fallback::new(self, default)
    }

    /// Try an alternative effect on any error.
    ///
    /// This is a shorthand for `or_else(|_| alternative)`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let data = fetch_primary()
    ///     .fallback_to(fetch_secondary());
    /// ```
    fn fallback_to<E2>(self, alternative: E2) -> FallbackTo<Self, E2>
    where
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        FallbackTo::new(self, alternative)
    }

    /// Run this effect with a modified environment.
    ///
    /// The transformation function converts from the outer environment
    /// to the inner environment required by this effect.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Clone)]
    /// struct OuterEnv { multiplier: i32 }
    /// #[derive(Clone)]
    /// struct InnerEnv { value: i32 }
    ///
    /// let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
    /// let effect = inner_effect.local(|outer: &OuterEnv| InnerEnv { value: 21 * outer.multiplier });
    ///
    /// assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
    /// ```
    fn local<F, Env2>(self, f: F) -> Local<Self, F, Env2>
    where
        F: FnOnce(&Env2) -> Self::Env + Send,
        Env2: Clone + Send + Sync,
    {
        Local::new(self, f)
    }

    /// Convert to a boxed effect for type erasure.
    ///
    /// Use this when you need to:
    /// - Store effects in collections
    /// - Return different effect types from match arms
    /// - Create recursive effects
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    ///     pure(1).boxed(),
    ///     pure(2).map(|x| x * 2).boxed(),
    /// ];
    /// ```
    fn boxed(self) -> BoxedEffect<Self::Output, Self::Error, Self::Env>
    where
        Self: 'static,
    {
        BoxedEffect::new(self)
    }

    /// Perform a side effect and return the original value.
    ///
    /// Useful for logging, metrics, or other operations that don't
    /// affect the main computation. The side effect function receives
    /// a reference to the value and must return an Effect. If the side
    /// effect fails, the entire computation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = pure::<_, String, ()>(42)
    ///     .tap(|value| {
    ///         println!("Value: {}", value);
    ///         pure(())
    ///     });
    ///
    /// assert_eq!(effect.execute(&()).await, Ok(42));
    /// ```
    fn tap<E2, F>(self, f: F) -> Tap<Self, F, E2>
    where
        Self::Output: Clone,
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Output = (), Error = Self::Error, Env = Self::Env>,
    {
        Tap {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Fail with error if predicate returns false.
    ///
    /// Provides a declarative way to express validation conditions.
    /// If the predicate returns true, the value passes through unchanged.
    /// If false, the error function is called to produce an error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Success case
    /// let effect = pure::<_, String, ()>(25)
    ///     .check(|age| *age >= 18, || "too young".to_string());
    /// assert_eq!(effect.execute(&()).await, Ok(25));
    ///
    /// // Failure case
    /// let effect = pure::<_, String, ()>(15)
    ///     .check(|age| *age >= 18, || "too young".to_string());
    /// assert_eq!(effect.execute(&()).await, Err("too young".to_string()));
    /// ```
    fn check<P, F>(self, predicate: P, error_fn: F) -> Check<Self, P, F>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        F: FnOnce() -> Self::Error + Send,
    {
        Check {
            inner: self,
            predicate,
            error_fn,
        }
    }

    /// Combine with another effect, returning both values as a tuple.
    ///
    /// Useful when you need results from multiple effects.
    /// The function receives a reference to the first value
    /// and returns an effect for the second value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = pure::<_, String, ()>(5)
    ///     .with(|value| pure(*value * 2))
    ///     .map(|(first, second)| first + second);
    ///
    /// assert_eq!(effect.execute(&()).await, Ok(15));  // 5 + 10
    /// ```
    fn with<E2, F>(self, f: F) -> With<Self, F, E2>
    where
        Self::Output: Clone,
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Error = Self::Error, Env = Self::Env>,
    {
        With {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Chain effect with automatic error conversion.
    ///
    /// Eliminates manual `.map_err(E::from)` calls when error types differ.
    /// The error type from the chained effect must be convertible to the
    /// current error type via the `From` trait.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[derive(Debug, PartialEq)]
    /// enum ValidationError { Invalid }
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum AppError { Validation(ValidationError) }
    ///
    /// impl From<ValidationError> for AppError {
    ///     fn from(e: ValidationError) -> Self {
    ///         AppError::Validation(e)
    ///     }
    /// }
    ///
    /// let effect = pure::<_, AppError, ()>(42)
    ///     .and_then_auto(|_| pure::<i32, ValidationError, ()>(100));
    ///
    /// assert_eq!(effect.execute(&()).await, Ok(100));
    /// ```
    fn and_then_auto<E2, F>(self, f: F) -> AndThenAuto<Self, F, E2>
    where
        F: FnOnce(Self::Output) -> E2 + Send,
        E2: Effect<Env = Self::Env>,
        Self::Error: From<E2::Error>,
    {
        AndThenAuto {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Chain effect by borrowing value, then return original.
    ///
    /// Avoids multiple clones when you need to use a value in multiple effects
    /// but only care about the final result. The function receives a reference
    /// to the value and returns an effect whose result is discarded.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = pure::<_, String, ()>(42)
    ///     .and_then_ref(|value| {
    ///         assert_eq!(*value, 42);
    ///         pure("processed")
    ///     })
    ///     .and_then_ref(|value| {
    ///         assert_eq!(*value, 42);
    ///         pure("again")
    ///     });
    ///
    /// assert_eq!(effect.execute(&()).await, Ok(42));
    /// ```
    fn and_then_ref<E2, F>(self, f: F) -> AndThenRef<Self, F, E2>
    where
        Self::Output: Clone,
        F: FnOnce(&Self::Output) -> E2 + Send,
        E2: Effect<Error = Self::Error, Env = Self::Env>,
    {
        AndThenRef {
            inner: self,
            f,
            _marker: PhantomData,
        }
    }

    /// Run and await the effect.
    ///
    /// Convenience method combining run + await.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = pure::<_, String, ()>(42).execute(&()).await;
    /// assert_eq!(result, Ok(42));
    /// ```
    #[allow(async_fn_in_trait)]
    async fn execute(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.run(env).await
    }

    /// Combine this effect with another, returning both results as a tuple.
    ///
    /// `zip` is useful when you have two independent effects and need both results.
    /// Unlike `and_then`, which expresses sequential dependency, `zip` expresses
    /// that both effects are independent and can potentially run in parallel.
    ///
    /// # Execution Order
    ///
    /// The current implementation runs effects sequentially for simplicity.
    /// Use parallel combinators (`par2`, `par3`, etc.) for concurrent execution.
    ///
    /// # Error Handling
    ///
    /// Uses fail-fast semantics: if either effect fails, the combined effect
    /// fails with that error. Errors are not accumulated.
    ///
    /// For error accumulation, use `Validation::all()` instead.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    ///
    /// // Independent effects - order doesn't matter
    /// let effect = fetch_user(id)
    ///     .zip(fetch_settings(id))
    ///     .map(|(user, settings)| UserProfile { user, settings });
    ///
    /// // Chain multiple zips
    /// let effect = fetch_a()
    ///     .zip(fetch_b())
    ///     .zip(fetch_c())
    ///     .map(|((a, b), c)| combine(a, b, c));
    ///
    /// // Or use zip3 for cleaner syntax
    /// let effect = zip3(fetch_a(), fetch_b(), fetch_c())
    ///     .map(|(a, b, c)| combine(a, b, c));
    /// ```
    ///
    /// # See Also
    ///
    /// - `zip_with` - combine with a function directly
    /// - `zip3`, `zip4`, etc. - combine multiple effects
    /// - `and_then` - for dependent/sequential effects
    /// - `par2`, `par3`, etc. - for parallel execution
    fn zip<E2>(self, other: E2) -> Zip<Self, E2>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
    {
        Zip::new(self, other)
    }

    /// Combine this effect with another using a function.
    ///
    /// More efficient than `zip().map()` as it's a single combinator.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    ///
    /// let effect = pure::<_, String, ()>(2)
    ///     .zip_with(pure(3), |a, b| a * b);
    /// assert_eq!(effect.execute(&()).await, Ok(6));
    ///
    /// // Equivalent to but more efficient than:
    /// let effect = pure::<_, String, ()>(2)
    ///     .zip(pure(3))
    ///     .map(|(a, b)| a * b);
    /// ```
    fn zip_with<E2, R, F>(self, other: E2, f: F) -> ZipWith<Self, E2, F>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        F: FnOnce(Self::Output, E2::Output) -> R + Send,
        R: Send,
    {
        ZipWith::new(self, other, f)
    }

    /// Ensure the output satisfies a closure predicate, failing with the given error otherwise.
    ///
    /// This is useful for adding validation to effect chains without
    /// verbose `and_then` boilerplate.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = fetch_user(id)
    ///     .ensure(|u| u.age >= 18, Error::TooYoung)
    ///     .ensure(|u| u.is_active, Error::InactiveUser);
    /// ```
    ///
    /// # See Also
    ///
    /// - `ensure_with` - for errors that need the value
    /// - `ensure_pred` - for composable predicates from predicate module
    /// - `unless` - inverse of ensure (fail if TRUE)
    /// - `filter_or` - alias for ensure
    fn ensure<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        Ensure::new(self, predicate, error)
    }

    /// Ensure with a lazily-computed error.
    ///
    /// The error function is only called if the predicate fails,
    /// and receives a reference to the value.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = fetch_user(id)
    ///     .ensure_with(
    ///         |u| u.age >= 18,
    ///         |u| Error::TooYoung { actual_age: u.age }
    ///     );
    /// ```
    fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> EnsureWith<Self, P, F>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        F: FnOnce(&Self::Output) -> Self::Error + Send,
        Self: Sized,
    {
        EnsureWith::new(self, predicate, error_fn)
    }

    /// Ensure using a Predicate from the predicate module.
    ///
    /// This enables composable, reusable predicates.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::predicate::*;
    ///
    /// let valid_age = between(18, 120);
    /// let effect = fetch_age()
    ///     .ensure_pred(valid_age, Error::InvalidAge);
    /// ```
    fn ensure_pred<P, Err>(self, predicate: P, error: Err) -> EnsurePred<Self, P, Err>
    where
        P: crate::predicate::Predicate<Self::Output> + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        EnsurePred::new(self, predicate, error)
    }

    /// Alias for `ensure` - filter with a fallback error.
    ///
    /// Named to match common FP convention.
    fn filter_or<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        self.ensure(predicate, error)
    }

    /// Ensure the output does NOT satisfy a predicate.
    ///
    /// Inverse of `ensure`: fails if predicate is TRUE.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let effect = fetch_user(id)
    ///     .unless(|u| u.is_banned, Error::UserBanned);
    /// ```
    fn unless<P, Err>(self, predicate: P, error: Err) -> Unless<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        Unless::new(self, predicate, error)
    }
}

// Blanket implementation for all Effect types
impl<E: Effect> EffectExt for E {}
