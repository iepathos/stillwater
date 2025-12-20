//! SinkMapErr combinator - transform the error of a SinkEffect.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// An effect that transforms the error of a SinkEffect.
///
/// Emissions pass through unchanged.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect: SinkMapErr<_, _> = emit::<String, i32, ()>("log".to_string())
///     .map_err(|e: i32| format!("Error code: {}", e));
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec!["log".to_string()]);
/// # });
/// ```
pub struct SinkMapErr<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for SinkMapErr<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkMapErr")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for SinkMapErr<E, F>
where
    E: SinkEffect,
    F: FnOnce(E::Error) -> E2 + Send,
    E2: Send,
{
    type Output = E::Output;
    type Error = E2;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await.map_err(self.f)
    }
}

impl<E, F, E2> SinkEffect for SinkMapErr<E, F>
where
    E: SinkEffect,
    F: FnOnce(E::Error) -> E2 + Send,
    E2: Send,
{
    type Item = E::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        self.inner.run_with_sink(env, sink).await.map_err(self.f)
    }
}
