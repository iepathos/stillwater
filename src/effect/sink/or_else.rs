//! SinkOrElse combinator - error recovery for SinkEffect.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Error recovery for SinkEffect.
///
/// If the inner effect fails, the function is called with the error
/// to produce a recovery effect. Emissions from the recovery effect
/// are also streamed to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, String>("before error".to_string())
///     .and_then(|_| into_sink::<_, _, String>(fail::<i32, _, ()>("oops".to_string())))
///     .or_else(|_err| {
///         emit("recovered".to_string())
///             .map(|_| 42)
///     });
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["before error".to_string(), "recovered".to_string()]);
/// # });
/// ```
pub struct SinkOrElse<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for SinkOrElse<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkOrElse")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for SinkOrElse<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Output = E::Output, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Error) -> E2 + Send,
{
    type Output = E::Output;
    type Error = E2::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(err) => (self.f)(err).run(env).await,
        }
    }
}

impl<E, F, E2> SinkEffect for SinkOrElse<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Output = E::Output, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Error) -> E2 + Send,
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
        match self.inner.run_with_sink(env, &sink).await {
            Ok(value) => Ok(value),
            Err(err) => (self.f)(err).run_with_sink(env, sink).await,
        }
    }
}
