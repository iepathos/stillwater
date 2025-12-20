//! Censor combinator - transform accumulated writes.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;

/// An effect that transforms accumulated writes.
///
/// This is useful for filtering, redacting, or otherwise transforming
/// the accumulated writes after the inner effect completes.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("debug: verbose".to_string())
///     .and_then(|_| tell_one("info: important".to_string()))
///     .censor(|logs| logs.into_iter().filter(|l| !l.starts_with("debug")).collect());
///
/// let (_, logs) = effect.run_writer(&()).await;
/// assert_eq!(logs, vec!["info: important".to_string()]);
/// # });
/// ```
pub struct Censor<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for Censor<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Censor")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F> Effect for Censor<E, F>
where
    E: WriterEffect,
    F: FnOnce(E::Writes) -> E::Writes + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, F> WriterEffect for Censor<E, F>
where
    E: WriterEffect,
    F: FnOnce(E::Writes) -> E::Writes + Send,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;
        let transformed = (self.f)(writes);
        (result, transformed)
    }
}
