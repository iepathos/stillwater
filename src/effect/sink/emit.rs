//! Emit combinator - emits a single item to the sink.

use std::future::Future;
use std::marker::PhantomData;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// An effect that emits a single item to the sink.
///
/// This is the fundamental Sink operation - it emits an item and
/// produces `()` as output.
///
/// # Type Parameters
///
/// * `T` - The type of item to emit.
/// * `E` - The error type (generic to match chained effects).
/// * `Env` - The environment type (generic to work with any environment).
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("hello".to_string());
///
/// let (result, collected) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(collected, vec!["hello".to_string()]);
/// # });
/// ```
#[derive(Debug)]
pub struct Emit<T, E, Env> {
    item: T,
    _phantom: PhantomData<fn() -> (E, Env)>,
}

impl<T, E, Env> Clone for Emit<T, E, Env>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = ();
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<Self::Output, Self::Error> {
        // When run as a plain Effect, emission is a no-op
        Ok(())
    }
}

impl<T, E, Env> SinkEffect for Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Item = T;

    async fn run_with_sink<S, Fut>(self, _env: &Self::Env, sink: S) -> Result<(), E>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        sink(self.item).await;
        Ok(())
    }
}

/// Emit a single item to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("log message".to_string());
///
/// effect.run_with_sink(&(), |log| async move {
///     println!("{}", log);
/// }).await;
/// # });
/// ```
pub fn emit<T, E, Env>(item: T) -> Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Emit {
        item,
        _phantom: PhantomData,
    }
}

/// An effect that emits multiple items to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit_many::<_, _, String, ()>(vec!["a", "b", "c"]);
///
/// let (result, collected) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(collected, vec!["a", "b", "c"]);
/// # });
/// ```
#[derive(Debug)]
pub struct EmitMany<I, T, E, Env> {
    items: I,
    _phantom: PhantomData<fn() -> (T, E, Env)>,
}

impl<I, T, E, Env> Clone for EmitMany<I, T, E, Env>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I, T, E, Env> Effect for EmitMany<I, T, E, Env>
where
    I: IntoIterator<Item = T> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = ();
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<Self::Output, Self::Error> {
        // When run as a plain Effect, emission is a no-op
        Ok(())
    }
}

impl<I, T, E, Env> SinkEffect for EmitMany<I, T, E, Env>
where
    I: IntoIterator<Item = T> + Send,
    I::IntoIter: Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Item = T;

    async fn run_with_sink<S, Fut>(self, _env: &Self::Env, sink: S) -> Result<(), E>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        for item in self.items {
            sink(item).await;
        }
        Ok(())
    }
}

/// Emit multiple items to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit_many::<_, _, String, ()>(vec!["step 1", "step 2", "step 3"]);
///
/// effect.run_with_sink(&(), |log| async move {
///     println!("{}", log);
/// }).await;
/// # });
/// ```
pub fn emit_many<I, T, E, Env>(items: I) -> EmitMany<I, T, E, Env>
where
    I: IntoIterator<Item = T> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    EmitMany {
        items,
        _phantom: PhantomData,
    }
}
