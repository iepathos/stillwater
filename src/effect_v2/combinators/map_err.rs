//! MapErr combinator - transforms the error value of an effect.

use crate::effect_v2::trait_def::Effect;

/// MapErr combinator - transforms the error value.
///
/// Zero-cost: no heap allocation. The `MapErr` struct stores only
/// the inner effect and the error transformation function.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error")
///     .map_err(|e: &str| format!("wrapped: {}", e));
/// assert_eq!(effect.execute(&()).await, Err("wrapped: error".to_string()));
/// ```
pub struct MapErr<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
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

    fn run(
        self,
        env: &Self::Env,
    ) -> impl std::future::Future<Output = Result<Self::Output, E2>> + Send {
        async move { self.inner.run(env).await.map_err(self.f) }
    }
}
