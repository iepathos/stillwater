//! SinkMap combinator - transform the output of a SinkEffect.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// An effect that transforms the output of a SinkEffect.
///
/// Emissions pass through unchanged.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("logged".to_string())
///     .map(|_| 42)
///     .map(|n| n * 2);
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(84));
/// assert_eq!(logs, vec!["logged".to_string()]);
/// # });
/// ```
pub struct SinkMap<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for SinkMap<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkMap")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, U> Effect for SinkMap<E, F>
where
    E: SinkEffect,
    F: FnOnce(E::Output) -> U + Send,
    U: Send,
{
    type Output = U;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let result = self.inner.run(env).await?;
        Ok((self.f)(result))
    }
}

impl<E, F, U> SinkEffect for SinkMap<E, F>
where
    E: SinkEffect,
    F: FnOnce(E::Output) -> U + Send,
    U: Send,
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
        let result = self.inner.run_with_sink(env, sink).await?;
        Ok((self.f)(result))
    }
}
