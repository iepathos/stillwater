//! MapErr combinator - transforms the error value of an effect.

use crate::effect::trait_def::Effect;

/// MapErr combinator - transforms the error value.
///
/// Zero-cost: no heap allocation. The `MapErr` struct stores only
/// the inner effect and the error transformation function.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error")
///     .map_err(|e: &str| format!("wrapped: {}", e));
/// assert_eq!(effect.execute(&()).await, Err("wrapped: error".to_string()));
/// ```
pub struct MapErr<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
}

impl<Inner, F> std::fmt::Debug for MapErr<Inner, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapErr")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<Inner, F, E2> Effect for MapErr<Inner, F>
where
    Inner: Effect,
    F: FnOnce(Inner::Error) -> E2 + Send,
    E2: Send,
{
    type Output = Inner::Output;
    type Error = E2;
    type Env = Inner::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, E2> {
        self.inner.run(env).await.map_err(self.f)
    }
}

// WriterEffect implementation for MapErr - passes writes through unchanged
impl<Inner, F, E2> crate::effect::writer::WriterEffect for MapErr<Inner, F>
where
    Inner: crate::effect::writer::WriterEffect,
    F: FnOnce(Inner::Error) -> E2 + Send,
    E2: Send,
{
    type Writes = Inner::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;
        (result.map_err(self.f), writes)
    }
}
