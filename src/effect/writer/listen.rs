//! Listen combinator - include accumulated writes in output.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;

/// An effect that includes accumulated writes in the output.
///
/// The output becomes a tuple of `(original_output, writes)`, allowing
/// the caller to inspect what was written during the computation.
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
///     .listen();
///
/// let (result, writes) = effect.run_writer(&()).await;
///
/// // Output now contains both value and writes
/// assert_eq!(result, Ok((42, vec!["logged".to_string()])));
/// // Writes are also returned separately
/// assert_eq!(writes, vec!["logged".to_string()]);
/// # });
/// ```
pub struct Listen<E> {
    pub(crate) inner: E,
}

impl<E> std::fmt::Debug for Listen<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listen")
            .field("inner", &"<effect>")
            .finish()
    }
}

impl<E> Effect for Listen<E>
where
    E: WriterEffect,
    E::Writes: Clone,
{
    type Output = (E::Output, E::Writes);
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, writes) = self.inner.run_writer(env).await;
        result.map(|output| (output, writes))
    }
}

impl<E> WriterEffect for Listen<E>
where
    E: WriterEffect,
    E::Writes: Clone,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;
        let new_result = result.map(|output| (output, writes.clone()));
        (new_result, writes)
    }
}
