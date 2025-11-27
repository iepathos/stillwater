//! AndThenRef combinator - chain by borrowing value, return original.

use std::marker::PhantomData;

use crate::effect::trait_def::Effect;

/// An effect that chains by borrowing the value, returning the original.
///
/// Created by [`EffectExt::and_then_ref`](crate::effect::ext::EffectExt::and_then_ref).
#[derive(Debug)]
pub struct AndThenRef<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) f: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> Effect for AndThenRef<E, F, E2>
where
    E: Effect,
    E::Output: Clone,
    F: FnOnce(&E::Output) -> E2 + Send,
    E2: Effect<Error = E::Error, Env = E::Env>,
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
