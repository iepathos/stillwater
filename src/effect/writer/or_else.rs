//! WriterOrElse combinator - recover from errors while preserving writes.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Semigroup;

/// Recover from an error in a WriterEffect.
///
/// If this effect fails, apply the recovery function to produce a new effect.
/// If this effect succeeds, the value passes through unchanged.
///
/// Writes from both the original effect (up to the error) and the recovery
/// effect are accumulated.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Create a failing effect with some writes
/// let effect = tell_one::<_, String, ()>("before error".to_string())
///     .and_then(|_| into_writer::<_, _, Vec<String>>(fail::<(), String, ()>("boom".into())))
///     .or_else(|_| tell_one::<_, String, ()>("recovered".to_string()).map(|_| ()));
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec!["before error".to_string(), "recovered".to_string()]);
/// # });
/// ```
pub struct WriterOrElse<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for WriterOrElse<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterOrElse")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for WriterOrElse<E, F>
where
    E: WriterEffect,
    E2: WriterEffect<Output = E::Output, Env = E::Env, Writes = E::Writes>,
    F: FnOnce(E::Error) -> E2 + Send,
{
    type Output = E::Output;
    type Error = E2::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(e) => (self.f)(e).run(env).await,
        }
    }
}

impl<E, F, E2> WriterEffect for WriterOrElse<E, F>
where
    E: WriterEffect,
    E::Writes: Semigroup,
    E2: WriterEffect<Output = E::Output, Env = E::Env, Writes = E::Writes>,
    F: FnOnce(E::Error) -> E2 + Send,
{
    type Writes = E::Writes;

    async fn run_writer(self, env: &Self::Env) -> (Result<Self::Output, E2::Error>, Self::Writes) {
        let (result, writes1) = self.inner.run_writer(env).await;

        match result {
            Ok(value) => (Ok(value), writes1),
            Err(e) => {
                let (result2, writes2) = (self.f)(e).run_writer(env).await;
                (result2, writes1.combine(writes2))
            }
        }
    }
}
