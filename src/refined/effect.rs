//! Effect integration for refined types
//!
//! This module provides integration between refined types and stillwater's
//! Effect system for validation at effect boundaries.
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//! use stillwater::refined::{refine, NonEmptyString, PositiveI32};
//!
//! // Validate data in effect chains
//! let effect = pure::<_, String, ()>("hello".to_string())
//!     .and_then(|s| refine::<_, NonEmpty, ()>(s).map_err(|e| e.to_string()));
//! ```

use super::{Predicate, Refined};

impl<T, P> Refined<T, P>
where
    T: Send + 'static,
    P: Predicate<T>,
    P::Error: Send + 'static,
{
    /// Create an effect that validates a value.
    ///
    /// Useful for integrating validation into effect chains.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// type PositiveI32 = Refined<i32, Positive>;
    ///
    /// let effect = PositiveI32::validate_effect::<()>(42);
    /// ```
    pub fn validate_effect<Env>(
        value: T,
    ) -> crate::effect::combinators::FromFn<impl FnOnce(&Env) -> Result<Self, P::Error> + Send, Env>
    where
        Env: Clone + Send + Sync,
    {
        crate::from_fn(move |_env: &Env| Self::new(value))
    }
}

/// Lift a refined type constructor into an effect.
///
/// This enables validating values fetched from effects.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::refined::{refine, NonEmpty, Refined};
///
/// type NonEmptyString = Refined<String, NonEmpty>;
///
/// let effect = pure::<_, String, ()>("hello".to_string())
///     .and_then(|s| refine::<_, NonEmpty, ()>(s));
/// ```
pub fn refine<T, P, Env>(
    value: T,
) -> crate::effect::combinators::FromFn<
    impl FnOnce(&Env) -> Result<Refined<T, P>, P::Error> + Send,
    Env,
>
where
    T: Send + 'static,
    P: Predicate<T>,
    P::Error: Send + 'static,
    Env: Clone + Send + Sync,
{
    Refined::validate_effect(value)
}

/// Create a pure effect containing a refined value.
///
/// This is useful when you have an already-validated refined value
/// and want to lift it into an effect.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::refined::{pure_refined, Refined, Positive};
///
/// type PositiveI32 = Refined<i32, Positive>;
///
/// let n = PositiveI32::new(42).unwrap();
/// let effect = pure_refined::<_, _, String, ()>(n);
/// ```
pub fn pure_refined<T, P, E, Env>(
    refined: Refined<T, P>,
) -> crate::effect::combinators::Pure<Refined<T, P>, E, Env>
where
    T: Send + 'static,
    P: Predicate<T>,
    E: Send + 'static,
    Env: Clone + Send + Sync,
{
    crate::pure(refined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::prelude::*;
    use crate::refined::predicates::numeric::Positive;
    use crate::refined::predicates::string::NonEmpty;

    type NonEmptyString = Refined<String, NonEmpty>;
    type PositiveI32 = Refined<i32, Positive>;

    #[tokio::test]
    async fn test_validate_effect_success() {
        let effect = PositiveI32::validate_effect::<()>(42);
        let result = effect.run(&()).await;
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().get(), 42);
    }

    #[tokio::test]
    async fn test_validate_effect_failure() {
        let effect = PositiveI32::validate_effect::<()>(-5);
        let result = effect.run(&()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refine_success() {
        let effect = refine::<_, NonEmpty, ()>("hello".to_string());
        let result = effect.run(&()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_refine_failure() {
        let effect = refine::<_, NonEmpty, ()>("".to_string());
        let result = effect.run(&()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pure_refined() {
        let n = PositiveI32::new(42).unwrap();
        let effect = pure_refined::<_, _, String, ()>(n);
        let result = effect.run(&()).await;
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().get(), 42);
    }

    #[tokio::test]
    async fn test_effect_chain() {
        // Chain: pure value -> validate -> map
        let effect = pure::<_, &str, ()>("hello".to_string())
            .and_then(|s| refine::<_, NonEmpty, ()>(s))
            .map(|refined| refined.get().len());

        let result = effect.run(&()).await;
        assert_eq!(result, Ok(5));
    }

    #[tokio::test]
    async fn test_effect_chain_failure() {
        let effect = pure::<_, &str, ()>("".to_string())
            .and_then(|s| refine::<_, NonEmpty, ()>(s))
            .map(|refined| refined.get().len());

        let result = effect.run(&()).await;
        assert!(result.is_err());
    }
}
