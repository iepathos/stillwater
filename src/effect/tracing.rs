//! Tracing support for zero-cost effects.
//!
//! This module provides the `Instrument` combinator and `instrument` method
//! for wrapping effects in tracing spans. Feature-gated behind `#[cfg(feature = "tracing")]`.

use crate::effect::trait_def::Effect;

/// An effect wrapped in a tracing span.
///
/// Created by [`EffectTracingExt::instrument`].
#[cfg(feature = "tracing")]
#[derive(Debug)]
pub struct Instrument<E> {
    pub(crate) inner: E,
    pub(crate) span: tracing::Span,
}

#[cfg(feature = "tracing")]
impl<E> Effect for Instrument<E>
where
    E: Effect,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        use tracing::Instrument as _;
        self.inner.run(env).instrument(self.span).await
    }
}

/// Extension trait for adding tracing instrumentation to effects.
///
/// This trait is only available when the `tracing` feature is enabled.
#[cfg(feature = "tracing")]
pub trait EffectTracingExt: Effect {
    /// Wrap this effect in a tracing span.
    ///
    /// The span is entered when the effect executes and exited when it completes.
    /// This follows the standard `tracing::Instrument` pattern for async code.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::prelude::*;
    /// use stillwater::effect::tracing::EffectTracingExt;
    /// use tracing::{info_span, debug_span};
    ///
    /// // Basic instrumentation
    /// let effect = pure::<_, String, ()>(42)
    ///     .instrument(info_span!("my_operation"));
    ///
    /// // With business context (recommended)
    /// fn fetch_order(order_id: String) -> impl Effect<Output = Order, Error = String, Env = Env> {
    ///     let span_id = order_id.clone();
    ///     from_fn(move |env| { /* ... */ })
    ///         .instrument(debug_span!("fetch_order", order_id = %span_id))
    /// }
    /// ```
    fn instrument(self, span: tracing::Span) -> Instrument<Self> {
        Instrument { inner: self, span }
    }
}

#[cfg(feature = "tracing")]
impl<E: Effect> EffectTracingExt for E {}

#[cfg(all(test, feature = "tracing"))]
mod tests {
    use super::*;
    use crate::effect::compat::RunStandalone;
    use crate::effect::constructors::{fail, pure};
    use crate::effect::ext::EffectExt;

    #[tokio::test]
    async fn test_instrument_returns_value() {
        let effect = pure::<_, String, ()>(42).instrument(tracing::info_span!("test_span"));

        let result = effect.run_standalone().await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_error_in_span_propagates() {
        let effect =
            fail::<i32, _, ()>("oops".to_string()).instrument(tracing::info_span!("failing"));

        let result = effect.run_standalone().await;
        assert_eq!(result, Err("oops".to_string()));
    }

    #[tokio::test]
    async fn test_nested_spans() {
        let inner = pure::<_, String, ()>(1).instrument(tracing::debug_span!("inner_op"));
        let outer = inner.and_then(|x| pure(x + 1).instrument(tracing::debug_span!("outer_op")));

        let result = outer.run_standalone().await;
        assert_eq!(result, Ok(2));
    }

    #[tokio::test]
    async fn test_composition_with_instrument() {
        let effect = pure::<_, String, ()>(5)
            .instrument(tracing::debug_span!("step1"))
            .map(|x| x * 2)
            .instrument(tracing::debug_span!("step2"))
            .and_then(|x| pure(x + 10).instrument(tracing::debug_span!("step3")));

        let result = effect.run_standalone().await;
        assert_eq!(result, Ok(20));
    }
}
