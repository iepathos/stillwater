//! WriterMapErr combinator - transform the error of a WriterEffect.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;

/// An effect that transforms the error of a WriterEffect.
///
/// Writes pass through unchanged.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Map error from String to a custom error type
/// let effect = tell_one::<_, String, ()>("logged".to_string())
///     .map(|_| 42)
///     .map_err(|e: String| format!("wrapped: {}", e));
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["logged".to_string()]);
/// # });
/// ```
pub struct WriterMapErr<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for WriterMapErr<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterMapErr")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for WriterMapErr<E, F>
where
    E: WriterEffect,
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

impl<E, F, E2> WriterEffect for WriterMapErr<E, F>
where
    E: WriterEffect,
    F: FnOnce(E::Error) -> E2 + Send,
    E2: Send,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;
        (result.map_err(self.f), writes)
    }
}
