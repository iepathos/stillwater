//! SinkZip combinator - combine two SinkEffects.

use std::future::Future;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Combines two SinkEffects, streaming from both left-to-right.
///
/// Both effects are run sequentially. If either fails, the combined
/// effect fails with that error. Items from both effects are streamed
/// in order (left effect's items first, then right effect's items).
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let left = emit::<_, String, ()>("left".to_string()).map(|_| 1);
/// let right = emit::<_, String, ()>("right".to_string()).map(|_| 2);
///
/// let effect = left.zip(right);
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok((1, 2)));
/// assert_eq!(logs, vec!["left".to_string(), "right".to_string()]);
/// # });
/// ```
pub struct SinkZip<E1, E2> {
    pub(crate) left: E1,
    pub(crate) right: E2,
}

impl<E1, E2> std::fmt::Debug for SinkZip<E1, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkZip")
            .field("left", &"<effect>")
            .field("right", &"<effect>")
            .finish()
    }
}

impl<E1, E2> Effect for SinkZip<E1, E2>
where
    E1: SinkEffect,
    E2: SinkEffect<Error = E1::Error, Env = E1::Env, Item = E1::Item>,
{
    type Output = (E1::Output, E2::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let left_value = self.left.run(env).await?;
        let right_value = self.right.run(env).await?;
        Ok((left_value, right_value))
    }
}

impl<E1, E2> SinkEffect for SinkZip<E1, E2>
where
    E1: SinkEffect,
    E2: SinkEffect<Error = E1::Error, Env = E1::Env, Item = E1::Item>,
{
    type Item = E1::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        // Run left effect, streaming to sink
        let left_value = self.left.run_with_sink(env, &sink).await?;

        // Run right effect, streaming to same sink
        let right_value = self.right.run_with_sink(env, sink).await?;

        Ok((left_value, right_value))
    }
}
