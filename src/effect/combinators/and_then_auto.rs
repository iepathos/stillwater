//! AndThenAuto combinator - chain with automatic error conversion.

use std::marker::PhantomData;

use crate::effect::trait_def::Effect;

/// An effect that chains with automatic error conversion.
///
/// Created by [`EffectExt::and_then_auto`](crate::effect::ext::EffectExt::and_then_auto).
#[derive(Debug)]
pub struct AndThenAuto<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> Effect for AndThenAuto<E, F, E2>
where
    E: Effect,
    F: FnOnce(E::Output) -> E2 + Send,
    E2: Effect<Env = E::Env>,
    E::Error: From<E2::Error>,
{
    type Output = E2::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        (self.f)(value).run(env).await.map_err(E::Error::from)
    }
}
