//! Tap combinator - perform side effect and return original value.

use std::marker::PhantomData;

use crate::effect::trait_def::Effect;

/// An effect that performs a side effect and returns the original value.
///
/// Created by [`EffectExt::tap`](crate::effect::ext::EffectExt::tap).
#[derive(Debug)]
pub struct Tap<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> Effect for Tap<E, F, E2>
where
    E: Effect,
    E::Output: Clone,
    F: FnOnce(&E::Output) -> E2 + Send,
    E2: Effect<Output = (), Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        let value_clone = value.clone();
        (self.f)(&value).run(env).await?;
        Ok(value_clone)
    }
}
