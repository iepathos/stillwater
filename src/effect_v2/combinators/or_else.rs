//! OrElse combinator - recovers from errors.

use crate::effect_v2::trait_def::Effect;

/// OrElse combinator - recovers from errors.
///
/// Zero-cost: no heap allocation. The `OrElse` struct stores only
/// the inner effect and the recovery function.
///
/// If the inner effect succeeds, the value passes through unchanged.
/// If it fails, the recovery function is called with the error to
/// produce a new effect.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error")
///     .or_else(|_| pure(42));
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub struct OrElse<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
}

impl<Inner, F, E2> Effect for OrElse<Inner, F>
where
    Inner: Effect,
    E2: Effect<Output = Inner::Output, Env = Inner::Env>,
    F: FnOnce(Inner::Error) -> E2 + Send,
{
    type Output = Inner::Output;
    type Error = E2::Error;
    type Env = Inner::Env;

    fn run(
        self,
        env: &Self::Env,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(e) => (self.f)(e).run(env).await,
            }
        }
    }
}
