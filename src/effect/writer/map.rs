//! WriterMap combinator - transform the output of a WriterEffect.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;

/// An effect that transforms the output of a WriterEffect.
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
/// let effect = tell_one::<_, String, ()>("logged".to_string())
///     .map(|_| 42)
///     .map(|n| n * 2);
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(84));
/// assert_eq!(logs, vec!["logged".to_string()]);
/// # });
/// ```
pub struct WriterMap<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for WriterMap<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterMap")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, U> Effect for WriterMap<E, F>
where
    E: WriterEffect,
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

impl<E, F, U> WriterEffect for WriterMap<E, F>
where
    E: WriterEffect,
    F: FnOnce(E::Output) -> U + Send,
    U: Send,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;
        (result.map(self.f), writes)
    }
}
