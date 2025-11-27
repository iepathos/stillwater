//! Check combinator - fail if predicate is false.

use crate::effect::trait_def::Effect;

/// An effect that fails with an error if a predicate returns false.
///
/// Created by [`EffectExt::check`](crate::effect::ext::EffectExt::check).
#[derive(Debug)]
pub struct Check<E, P, F> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) error_fn: F,
}

impl<E, P, F> Effect for Check<E, P, F>
where
    E: Effect,
    P: FnOnce(&E::Output) -> bool + Send,
    F: FnOnce() -> E::Error + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        if (self.predicate)(&value) {
            Ok(value)
        } else {
            Err((self.error_fn)())
        }
    }
}
