//! Context error support for zero-cost effects.
//!
//! This module provides the `EffectContext` trait for adding context to errors
//! as they propagate through effect chains.

use crate::context::ContextError;
use crate::effect::combinators::MapErr;
use crate::effect::ext::EffectExt;
use crate::effect::trait_def::Effect;

/// Extension trait for adding context to Effect errors.
///
/// This trait provides a `.context()` method that wraps errors in `ContextError`,
/// enabling the accumulation of context information as errors propagate.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::effect::context::EffectContext;
///
/// let effect = fail::<i32, _, ()>("connection refused")
///     .context("connecting to database");
///
/// match effect.execute(&()).await {
///     Err(ctx_err) => {
///         assert_eq!(ctx_err.inner(), &"connection refused");
///         assert_eq!(ctx_err.context_trail(), &["connecting to database"]);
///     }
///     Ok(_) => panic!("Expected error"),
/// }
/// ```
pub trait EffectContext: Effect {
    /// Add context to errors from this effect.
    ///
    /// Wraps any error from this effect in a `ContextError` with the given
    /// context message. This enables building up a trail of context as errors
    /// propagate through the call stack.
    fn context(
        self,
        msg: impl Into<String> + Send + 'static,
    ) -> MapErr<Self, impl FnOnce(Self::Error) -> ContextError<Self::Error> + Send>
    where
        Self::Error: Send + 'static,
    {
        let msg = msg.into();
        self.map_err(move |err| ContextError::new(err).context(msg))
    }
}

// Blanket implementation for all Effect types
impl<E: Effect> EffectContext for E {}

/// Extension trait for chaining context on effects that already have ContextError.
///
/// This enables chaining multiple `.context()` calls on an effect that already
/// has a `ContextError` error type.
pub trait EffectContextChain<T, E, Env>:
    Effect<Output = T, Error = ContextError<E>, Env = Env>
{
    /// Add another layer of context.
    ///
    /// Adds a new context message to an effect that already has a `ContextError`.
    /// This allows building up deep context trails as errors propagate.
    fn context_chain(
        self,
        msg: impl Into<String> + Send + 'static,
    ) -> MapErr<Self, impl FnOnce(ContextError<E>) -> ContextError<E> + Send>
    where
        E: Send + 'static,
    {
        let msg = msg.into();
        self.map_err(move |err| err.context(msg))
    }
}

// Blanket implementation for effects with ContextError
impl<Eff, T, E, Env> EffectContextChain<T, E, Env> for Eff where
    Eff: Effect<Output = T, Error = ContextError<E>, Env = Env>
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::compat::RunStandalone;
    use crate::effect::constructors::{fail, pure};

    #[tokio::test]
    async fn test_effect_context() {
        let effect = fail::<i32, _, ()>("base error").context("operation failed");

        match effect.run_standalone().await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"base error");
                assert_eq!(ctx_err.context_trail(), &["operation failed"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_multiple_contexts() {
        let effect = fail::<i32, _, ()>("base error")
            .context("step 1")
            .context_chain("step 2")
            .context_chain("step 3");

        match effect.run_standalone().await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"base error");
                assert_eq!(ctx_err.context_trail(), &["step 1", "step 2", "step 3"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_context_success() {
        let effect = pure::<_, String, ()>(42).context("this context won't be used");

        assert_eq!(effect.run_standalone().await, Ok(42));
    }

    #[tokio::test]
    async fn test_effect_context_with_combinators() {
        let effect = pure::<_, String, ()>(5)
            .map(|x| x * 2)
            .and_then(|_| fail("error".to_string()))
            .context("step 1")
            .map(|x: i32| x + 10)
            .context_chain("step 2");

        match effect.run_standalone().await {
            Err(ctx_err) => {
                assert_eq!(ctx_err.inner(), &"error".to_string());
                assert_eq!(ctx_err.context_trail(), &["step 1", "step 2"]);
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_effect_context_with_string_types() {
        let effect = fail::<i32, _, ()>(String::from("error"))
            .context(String::from("owned context"))
            .context_chain("borrowed context");

        match effect.run_standalone().await {
            Err(ctx_err) => {
                assert_eq!(
                    ctx_err.context_trail(),
                    &["owned context", "borrowed context"]
                );
            }
            Ok(_) => panic!("Expected error"),
        }
    }
}
