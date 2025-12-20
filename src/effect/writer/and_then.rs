//! WriterAndThen combinator - chains dependent WriterEffects.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Semigroup;

/// Chains dependent WriterEffects, combining their writes.
///
/// Zero-cost: no heap allocation. The `WriterAndThen` struct stores only
/// the inner effect and the function that produces the next effect.
///
/// Writes from both effects are combined using `Monoid::combine`, ensuring
/// they accumulate in left-to-right order.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("step 1".to_string())
///     .and_then(|_| tell_one("step 2".to_string()))
///     .and_then(|_| tell_one("step 3".to_string()));
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec![
///     "step 1".to_string(),
///     "step 2".to_string(),
///     "step 3".to_string(),
/// ]);
/// # });
/// ```
pub struct WriterAndThen<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for WriterAndThen<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriterAndThen")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for WriterAndThen<E, F>
where
    E: WriterEffect,
    E2: WriterEffect<Error = E::Error, Env = E::Env, Writes = E::Writes>,
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

impl<E, F, E2> WriterEffect for WriterAndThen<E, F>
where
    E: WriterEffect,
    E::Writes: Semigroup,
    E2: WriterEffect<Error = E::Error, Env = E::Env, Writes = E::Writes>,
    F: FnOnce(E::Output) -> E2 + Send,
{
    type Writes = E::Writes;

    async fn run_writer(self, env: &Self::Env) -> (Result<E2::Output, E::Error>, Self::Writes) {
        let (result1, writes1) = self.inner.run_writer(env).await;

        match result1 {
            Ok(value) => {
                let next_effect = (self.f)(value);
                let (result2, writes2) = next_effect.run_writer(env).await;
                (result2, writes1.combine(writes2))
            }
            Err(e) => (Err(e), writes1),
        }
    }
}
