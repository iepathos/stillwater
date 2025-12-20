//! Pass combinator - use output to determine how to transform writes.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;

/// An effect that uses part of its output to transform accumulated writes.
///
/// The inner effect must produce a tuple `(T, F)` where `F` is a function
/// that transforms the writes. The final output is just `T`.
///
/// This is useful when you want to compute both a value and a transformation
/// for the writes in a single computation.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell::<_, String, ()>(vec!["a".to_string(), "b".to_string(), "c".to_string()])
///     .map(|_| {
///         // Return value and a function to limit writes
///         (42, |logs: Vec<String>| logs.into_iter().take(2).collect())
///     })
///     .pass();
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["a".to_string(), "b".to_string()]);
/// # });
/// ```
pub struct Pass<E> {
    pub(crate) inner: E,
}

impl<E> std::fmt::Debug for Pass<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pass").field("inner", &"<effect>").finish()
    }
}

impl<E, T, F> Effect for Pass<E>
where
    E: WriterEffect<Output = (T, F)>,
    T: Send,
    F: FnOnce(E::Writes) -> E::Writes + Send,
{
    type Output = T;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, _writes) = self.inner.run_writer(env).await;
        result.map(|(value, _f)| value)
    }
}

impl<E, T, F> WriterEffect for Pass<E>
where
    E: WriterEffect<Output = (T, F)>,
    T: Send,
    F: FnOnce(E::Writes) -> E::Writes + Send,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;

        match result {
            Ok((value, f)) => {
                let transformed = f(writes);
                (Ok(value), transformed)
            }
            Err(e) => (Err(e), writes),
        }
    }
}
