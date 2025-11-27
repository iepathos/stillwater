//! With combinator - combine with another effect, returning tuple.

use std::marker::PhantomData;

use crate::effect::trait_def::Effect;

/// An effect that combines two effects, returning a tuple of their results.
///
/// Created by [`EffectExt::with`](crate::effect::ext::EffectExt::with).
#[derive(Debug)]
pub struct With<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> Effect for With<E, F, E2>
where
    E: Effect,
    E::Output: Clone,
    F: FnOnce(&E::Output) -> E2 + Send,
    E2: Effect<Error = E::Error, Env = E::Env>,
{
    type Output = (E::Output, E2::Output);
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        let value_clone = value.clone();
        let other = (self.f)(&value).run(env).await?;
        Ok((value_clone, other))
    }
}
