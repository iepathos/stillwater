//! Lift regular Effects into WriterEffect with empty writes.

use std::marker::PhantomData;

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Monoid;

/// Lifts a regular Effect into a WriterEffect with empty writes.
///
/// This allows regular effects to be composed with WriterEffects
/// in chains and combinations.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let regular_effect = pure::<_, String, ()>(42);
/// let writer_effect = into_writer::<_, _, Vec<String>>(regular_effect);
///
/// let (result, logs) = writer_effect
///     .tap_tell(|n| vec![format!("Got: {}", n)])
///     .run_writer(&())
///     .await;
///
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["Got: 42".to_string()]);
/// # });
/// ```
pub struct IntoWriter<E, W> {
    inner: E,
    _phantom: PhantomData<fn() -> W>,
}

impl<E, W> std::fmt::Debug for IntoWriter<E, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntoWriter")
            .field("inner", &"<effect>")
            .finish()
    }
}

impl<E, W> Effect for IntoWriter<E, W>
where
    E: Effect,
    W: Monoid + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, W> WriterEffect for IntoWriter<E, W>
where
    E: Effect,
    W: Monoid + Send,
{
    type Writes = W;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let result = self.inner.run(env).await;
        (result, W::empty())
    }
}

/// Lift a regular Effect into a WriterEffect with empty writes.
///
/// This is the primary way to integrate existing effects that don't
/// have Writer capabilities into a WriterEffect chain.
///
/// # Type Parameters
///
/// * `E` - The effect type to lift
/// * `Env` - The environment type (inferred)
/// * `W` - The writes type (must be specified or inferred from context)
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Pure computation lifted into Writer context
/// let effect = into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(10))
///     .and_then(|n|
///         tell_one(format!("Got: {}", n))
///             .map(move |_| n * 2)
///     )
///     .tap_tell(|result| vec![format!("Final: {}", result)]);
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(20));
/// assert_eq!(logs, vec!["Got: 10".to_string(), "Final: 20".to_string()]);
/// # });
/// ```
///
/// # Integrating with Reader
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # #[derive(Clone)]
/// # struct Env { multiplier: i32 }
///
/// # tokio_test::block_on(async {
/// let effect = into_writer::<_, _, Vec<String>>(asks::<_, String, Env, _>(|env| env.multiplier))
///     .tap_tell(|m| vec![format!("Multiplier: {}", m)])
///     .map(|m| m * 10);
///
/// let env = Env { multiplier: 3 };
/// let (result, logs) = effect.run_writer(&env).await;
/// assert_eq!(result, Ok(30));
/// assert_eq!(logs, vec!["Multiplier: 3".to_string()]);
/// # });
/// ```
pub fn into_writer<E, Env, W>(effect: E) -> IntoWriter<E, W>
where
    E: Effect<Env = Env>,
    Env: Clone + Send + Sync,
    W: Monoid + Send,
{
    IntoWriter {
        inner: effect,
        _phantom: PhantomData,
    }
}
