//! Convenience support that remains compatible with unit-environment effects.

/// Extension trait for running effects with unit environment.
#[allow(async_fn_in_trait)]
pub trait RunStandalone: crate::effect::trait_def::Effect<Env = ()> {
    /// Run an effect that doesn't require an environment.
    ///
    /// This is a convenience method for effects with `Env = ()` (unit type).
    async fn run_standalone(self) -> Result<Self::Output, Self::Error>;
}

impl<E: crate::effect::trait_def::Effect<Env = ()>> RunStandalone for E {
    async fn run_standalone(self) -> Result<Self::Output, Self::Error> {
        self.run(&()).await
    }
}
