//! Extension trait providing combinator methods for all Effects.
//!
//! The `EffectExt` trait is automatically implemented for all types
//! that implement `Effect`. It provides ergonomic combinator methods
//! like `map`, `and_then`, `or_else`, and `boxed`.

use crate::effect_v2::boxed::BoxedEffect;
use crate::effect_v2::combinators::{AndThen, Map, MapErr, OrElse};
use crate::effect_v2::reader::Local;
use crate::effect_v2::trait_def::Effect;

/// Extension trait providing combinator methods for all Effects.
///
/// This trait is automatically implemented for all types that implement `Effect`.
/// You don't need to implement this trait yourself.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
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
    async fn execute(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.run(env).await
    }
}

// Blanket implementation for all Effect types
impl<E: Effect> EffectExt for E {}
