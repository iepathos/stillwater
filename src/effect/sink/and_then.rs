//! SinkAndThen combinator - chains dependent SinkEffects.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Chains dependent SinkEffects, streaming from both.
///
/// When the first effect completes successfully, its output is passed
/// to the function to produce the next effect. Items from both effects
/// are streamed to the sink in order.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("step 1".to_string())
///     .and_then(|_| emit("step 2".to_string()))
///     .and_then(|_| emit("step 3".to_string()));
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec![
///     "step 1".to_string(),
///     "step 2".to_string(),
///     "step 3".to_string(),
/// ]);
/// # });
/// ```
pub struct SinkAndThen<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for SinkAndThen<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkAndThen")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for SinkAndThen<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Error = E::Error, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Output) -> E2 + Send,
{
    type Output = E2::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        (self.f)(value).run(env).await
    }
}

impl<E, F, E2> SinkEffect for SinkAndThen<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Error = E::Error, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Output) -> E2 + Send,
{
    type Item = E::Item;

    async fn run_with_sink<S, Fut>(self, env: &Self::Env, sink: S) -> Result<E2::Output, E::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        // Run first effect, streaming to sink
        let value = self.inner.run_with_sink(env, &sink).await?;

        // Run second effect with same sink
        let next_effect = (self.f)(value);
        next_effect.run_with_sink(env, sink).await
    }
}
