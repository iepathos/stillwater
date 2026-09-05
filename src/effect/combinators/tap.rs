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
    F: FnOnce(&E::Output) -> E2 + Send,
    E2: Effect<Output = (), Error = E::Error, Env = E::Env>,
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
        let effect = pure::<_, String, ()>(NonCloneOutput("value".to_string())).tap(|value| {
            assert_eq!(value.0, "value");
            pure(())
        });

        assert_eq!(
            effect.run_standalone().await,
            Ok(NonCloneOutput("value".to_string()))
        );
    }

    #[tokio::test]
    async fn propagates_failure_for_non_clone_output() {
        let effect = pure::<_, String, ()>(NonCloneOutput("value".to_string()))
            .tap(|_| fail::<(), _, ()>("tap failure".to_string()));

        assert_eq!(
            effect.run_standalone().await,
            Err("tap failure".to_string())
        );
    }
}
