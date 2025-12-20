//! Tell combinator - emits a value to be accumulated.

use std::marker::PhantomData;

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Monoid;

/// An effect that only emits a value, producing unit as output.
///
/// This is the fundamental Writer operation. It produces `()` as output
/// but emits a value that will be accumulated with other writes.
///
/// # Type Parameters
///
/// * `W` - The type to accumulate. Must implement `Monoid`.
/// * `E` - The error type (generic to match chained effects).
/// * `Env` - The environment type (generic to work with any environment).
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell::<_, String, ()>(vec!["hello".to_string()]);
/// let (result, writes) = effect.run_writer(&()).await;
///
/// assert_eq!(result, Ok(()));
/// assert_eq!(writes, vec!["hello".to_string()]);
/// # });
/// ```
#[derive(Debug)]
pub struct Tell<W, E, Env> {
    writes: W,
    _phantom: PhantomData<fn() -> (E, Env)>,
}

impl<W, E, Env> Clone for Tell<W, E, Env>
where
    W: Clone,
{
    fn clone(&self) -> Self {
        Self {
            writes: self.writes.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<W, E, Env> Effect for Tell<W, E, Env>
where
    W: Monoid + Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = ();
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

impl<W, E, Env> WriterEffect for Tell<W, E, Env>
where
    W: Monoid + Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Writes = W;

    async fn run_writer(self, _env: &Self::Env) -> (Result<(), E>, W) {
        (Ok(()), self.writes)
    }
}

/// Emit a value to be accumulated.
///
/// Error-generic and environment-generic: works with any `E` and `Env` types.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Simple logging - specify error type explicitly
/// let effect = tell::<_, String, ()>(vec!["Starting".to_string()]);
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec!["Starting".to_string()]);
/// # });
/// ```
///
/// # Chaining Writes
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell::<_, String, ()>(vec!["Step 1".to_string()])
///     .and_then(|_| tell(vec!["Step 2".to_string()]));
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec!["Step 1".to_string(), "Step 2".to_string()]);
/// # });
/// ```
///
/// # With Different Monoids
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
/// use stillwater::monoid::Sum;
///
/// # tokio_test::block_on(async {
/// // Count operations
/// let effect = tell::<_, String, ()>(Sum(1))
///     .and_then(|_| tell(Sum(1)))
///     .and_then(|_| tell(Sum(1)));
/// let (result, Sum(count)) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(count, 3);
/// # });
/// ```
pub fn tell<W, E, Env>(w: W) -> Tell<W, E, Env>
where
    W: Monoid + Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Tell {
        writes: w,
        _phantom: PhantomData,
    }
}

/// Emit a single item to a Vec accumulator.
///
/// Convenience function for the common case of accumulating items into a Vec.
/// Error-generic and environment-generic: works with any `E` and `Env` types.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("hello".to_string());
/// let (result, writes) = effect.run_writer(&()).await;
///
/// assert_eq!(result, Ok(()));
/// assert_eq!(writes, vec!["hello".to_string()]);
/// # });
/// ```
pub fn tell_one<T, E, Env>(item: T) -> Tell<Vec<T>, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Tell {
        writes: vec![item],
        _phantom: PhantomData,
    }
}
