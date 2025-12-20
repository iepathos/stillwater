//! TapEmit combinator - emit a derived value after success.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Emits a derived value after the inner effect succeeds.
///
/// If the inner effect succeeds, the function is called with a reference
/// to the output, and the result is emitted to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = into_sink::<_, _, String>(pure::<_, String, ()>(42))
///     .tap_emit(|n| format!("Result: {}", n));
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["Result: 42".to_string()]);
/// # });
/// ```
pub struct TapEmit<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for TapEmit<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TapEmit")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F> Effect for TapEmit<E, F>
where
    E: SinkEffect,
    E::Output: Clone + Send,
    F: FnOnce(&E::Output) -> E::Item + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, F> SinkEffect for TapEmit<E, F>
where
    E: SinkEffect,
    E::Output: Clone + Send,
    F: FnOnce(&E::Output) -> E::Item + Send,
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
        let value = self.inner.run_with_sink(env, &sink).await?;
        let derived = (self.f)(&value);
        sink(derived).await;
        Ok(value)
    }
}
