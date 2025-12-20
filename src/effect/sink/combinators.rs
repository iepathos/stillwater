//! Collection combinators for SinkEffect.

use std::future::Future;
use std::marker::PhantomData;

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Traverse a collection, running a sink effect for each item and streaming output.
///
/// This is the SinkEffect equivalent of `Iterator::map` combined with `collect`,
/// but streams items to the sink immediately instead of accumulating.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let items = vec![1, 2, 3];
/// let effect = traverse_sink(items, |n| {
///     emit::<_, String, ()>(format!("Processing {}", n))
///         .map(move |_| n * 10)
/// });
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(vec![10, 20, 30]));
/// assert_eq!(logs, vec![
///     "Processing 1".to_string(),
///     "Processing 2".to_string(),
///     "Processing 3".to_string(),
/// ]);
/// # });
/// ```
pub fn traverse_sink<I, F, Eff>(items: I, f: F) -> TraverseSink<I::Item, F, Eff>
where
    I: IntoIterator,
    I::IntoIter: Send,
    I::Item: Send,
    F: Fn(I::Item) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    TraverseSink {
        items: items.into_iter().collect(),
        f,
        _phantom: PhantomData,
    }
}

/// The traverse_sink combinator type.
pub struct TraverseSink<T, F, Eff> {
    items: Vec<T>,
    f: F,
    _phantom: PhantomData<fn() -> Eff>,
}

impl<T, F, Eff> std::fmt::Debug for TraverseSink<T, F, Eff>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraverseSink")
            .field("items", &self.items)
            .field("f", &"<function>")
            .finish()
    }
}

impl<T, F, Eff> Effect for TraverseSink<T, F, Eff>
where
    T: Send,
    F: Fn(T) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    type Output = Vec<Eff::Output>;
    type Error = Eff::Error;
    type Env = Eff::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::with_capacity(self.items.len());
        for item in self.items {
            results.push((self.f)(item).run(env).await?);
        }
        Ok(results)
    }
}

impl<T, F, Eff> SinkEffect for TraverseSink<T, F, Eff>
where
    T: Send,
    F: Fn(T) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    type Item = Eff::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        let mut results = Vec::with_capacity(self.items.len());
        for item in self.items {
            results.push((self.f)(item).run_with_sink(env, &sink).await?);
        }
        Ok(results)
    }
}

/// Fold a collection with streaming output.
///
/// This is the SinkEffect equivalent of `Iterator::fold`, but streams
/// items to the sink immediately instead of accumulating.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let items = vec![1, 2, 3, 4];
/// let effect = fold_sink(items, 0, |acc, n| {
///     emit::<_, String, ()>(format!("Adding {} to {}", n, acc))
///         .map(move |_| acc + n)
/// });
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(10));
/// assert_eq!(logs, vec![
///     "Adding 1 to 0".to_string(),
///     "Adding 2 to 1".to_string(),
///     "Adding 3 to 3".to_string(),
///     "Adding 4 to 6".to_string(),
/// ]);
/// # });
/// ```
pub fn fold_sink<I, F, Eff, Acc>(items: I, init: Acc, f: F) -> FoldSink<I::Item, F, Acc, Eff>
where
    I: IntoIterator,
    I::IntoIter: Send,
    I::Item: Send,
    Acc: Send,
    F: Fn(Acc, I::Item) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    FoldSink {
        items: items.into_iter().collect(),
        init,
        f,
        _phantom: PhantomData,
    }
}

/// The fold_sink combinator type.
pub struct FoldSink<T, F, Acc, Eff> {
    items: Vec<T>,
    init: Acc,
    f: F,
    _phantom: PhantomData<fn() -> Eff>,
}

impl<T, F, Acc, Eff> std::fmt::Debug for FoldSink<T, F, Acc, Eff>
where
    T: std::fmt::Debug,
    Acc: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FoldSink")
            .field("items", &self.items)
            .field("init", &self.init)
            .field("f", &"<function>")
            .finish()
    }
}

impl<T, F, Acc, Eff> Effect for FoldSink<T, F, Acc, Eff>
where
    T: Send,
    Acc: Send,
    F: Fn(Acc, T) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    type Output = Acc;
    type Error = Eff::Error;
    type Env = Eff::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let mut acc = self.init;
        for item in self.items {
            acc = (self.f)(acc, item).run(env).await?;
        }
        Ok(acc)
    }
}

impl<T, F, Acc, Eff> SinkEffect for FoldSink<T, F, Acc, Eff>
where
    T: Send,
    Acc: Send,
    F: Fn(Acc, T) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    type Item = Eff::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        let mut acc = self.init;
        for item in self.items {
            acc = (self.f)(acc, item).run_with_sink(env, &sink).await?;
        }
        Ok(acc)
    }
}
