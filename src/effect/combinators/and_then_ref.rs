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
    F: FnOnce(&E::Output) -> E2 + Send,
    E2: Effect<Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        (self.f)(&value).run(env).await?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::effect::compat::RunStandalone;
    use crate::effect::constructors::{fail, pure};
    use crate::effect::ext::EffectExt;

    #[derive(Debug, PartialEq)]
    struct NonCloneOutput(String);

    #[tokio::test]
    async fn preserves_non_clone_output() {
        let effect = pure::<_, String, ()>(NonCloneOutput("value".to_string()))
            .and_then_ref(|value| pure(value.0.len()));

        assert_eq!(
            effect.run_standalone().await,
            Ok(NonCloneOutput("value".to_string()))
        );
    }

    #[tokio::test]
    async fn propagates_failure_for_non_clone_output() {
        let effect = pure::<_, String, ()>(NonCloneOutput("value".to_string()))
            .and_then_ref(|_| fail::<(), _, ()>("dependent failure".to_string()));

        assert_eq!(
            effect.run_standalone().await,
            Err("dependent failure".to_string())
        );
    }
}
