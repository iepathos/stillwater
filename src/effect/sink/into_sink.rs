//! Lift regular Effects into SinkEffect with no emissions.

use std::future::Future;
use std::marker::PhantomData;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Lifts a regular Effect into a SinkEffect with no emissions.
///
/// This allows regular effects to be composed with SinkEffects
/// in chains and combinations.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let regular_effect = pure::<_, String, ()>(42);
/// let sink_effect = into_sink::<_, _, String>(regular_effect);
///
/// let (result, collected) = sink_effect.run_collecting(&()).await;
///
/// assert_eq!(result, Ok(42));
/// assert!(collected.is_empty());
/// # });
/// ```
pub struct IntoSink<E, T> {
    inner: E,
    _phantom: PhantomData<fn() -> T>,
}

impl<E, T> std::fmt::Debug for IntoSink<E, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntoSink")
            .field("inner", &"<effect>")
            .finish()
    }
}

impl<E, T> Effect for IntoSink<E, T>
where
    E: Effect,
    T: Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, T> SinkEffect for IntoSink<E, T>
where
    E: Effect,
    T: Send,
{
    type Item = T;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        _sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        self.inner.run(env).await
    }
}

/// Lift a regular Effect into a SinkEffect with no emissions.
///
/// This is the primary way to integrate existing effects that don't
/// have Sink capabilities into a SinkEffect chain.
///
/// # Type Parameters
///
/// * `E` - The effect type to lift
/// * `Env` - The environment type (inferred)
/// * `T` - The item type (must be specified or inferred from context)
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Pure computation lifted into Sink context
/// let effect = into_sink::<_, _, String>(pure::<_, String, ()>(10))
///     .and_then(|n|
///         emit(format!("Got: {}", n))
///             .map(move |_| n * 2)
///     )
///     .tap_emit(|result| format!("Final: {}", result));
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(20));
/// assert_eq!(logs, vec!["Got: 10".to_string(), "Final: 20".to_string()]);
/// # });
/// ```
///
/// # Integrating with Reader
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # #[derive(Clone)]
/// # struct Env { multiplier: i32 }
///
/// # tokio_test::block_on(async {
/// let effect = into_sink::<_, _, String>(asks::<_, String, Env, _>(|env| env.multiplier))
///     .tap_emit(|m| format!("Multiplier: {}", m))
///     .map(|m| m * 10);
///
/// let env = Env { multiplier: 3 };
/// let (result, logs) = effect.run_collecting(&env).await;
/// assert_eq!(result, Ok(30));
/// assert_eq!(logs, vec!["Multiplier: 3".to_string()]);
/// # });
/// ```
pub fn into_sink<E, Env, T>(effect: E) -> IntoSink<E, T>
where
    E: Effect<Env = Env>,
    Env: Clone + Send + Sync,
    T: Send,
{
    IntoSink {
        inner: effect,
        _phantom: PhantomData,
    }
}
