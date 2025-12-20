//! Collection combinators for WriterEffect.

use crate::effect::writer::boxed::BoxedWriterEffect;
use crate::effect::writer::WriterEffect;
use crate::Monoid;

/// Traverse a collection, running a writer effect for each item and accumulating all writes.
///
/// This is the WriterEffect equivalent of `Iterator::map` combined with `fold`,
/// but preserves both the mapped results and all accumulated writes.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let items = vec![1, 2, 3];
/// let effect = traverse_writer(items, |n| {
///     tell_one::<_, String, ()>(format!("Processing {}", n))
///         .map(move |_| n * 10)
/// });
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(vec![10, 20, 30]));
/// assert_eq!(logs, vec![
///     "Processing 1".to_string(),
///     "Processing 2".to_string(),
///     "Processing 3".to_string(),
/// ]);
/// # });
/// ```
pub fn traverse_writer<T, U, E, Env, W, F, Eff>(
    items: Vec<T>,
    f: F,
) -> BoxedWriterEffect<Vec<U>, E, Env, W>
where
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = U, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    BoxedWriterEffect::new(TraverseWriter { items, f })
}

struct TraverseWriter<T, F> {
    items: Vec<T>,
    f: F,
}

impl<T, U, E, Env, W, F, Eff> crate::effect::Effect for TraverseWriter<T, F>
where
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = U, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    type Output = Vec<U>;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, _writes) = self.run_writer(env).await;
        result
    }
}

impl<T, U, E, Env, W, F, Eff> WriterEffect for TraverseWriter<T, F>
where
    T: Send + 'static,
    U: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = U, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    type Writes = W;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let mut results = Vec::with_capacity(self.items.len());
        let mut all_writes = W::empty();

        for item in self.items {
            let effect = (self.f)(item);
            let (result, writes) = effect.run_writer(env).await;
            all_writes = all_writes.combine(writes);

            match result {
                Ok(value) => results.push(value),
                Err(e) => return (Err(e), all_writes),
            }
        }

        (Ok(results), all_writes)
    }
}

/// Fold a collection with a writer effect, accumulating writes at each step.
///
/// This is the WriterEffect equivalent of `Iterator::fold`, but preserves
/// all accumulated writes from each step.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let items = vec![1, 2, 3, 4];
/// let effect = fold_writer(items, 0, |acc, n| {
///     tell_one::<_, String, ()>(format!("Adding {} to {}", n, acc))
///         .map(move |_| acc + n)
/// });
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(10));
/// assert_eq!(logs, vec![
///     "Adding 1 to 0".to_string(),
///     "Adding 2 to 1".to_string(),
///     "Adding 3 to 3".to_string(),
///     "Adding 4 to 6".to_string(),
/// ]);
/// # });
/// ```
pub fn fold_writer<T, A, E, Env, W, F, Eff>(
    items: Vec<T>,
    init: A,
    f: F,
) -> BoxedWriterEffect<A, E, Env, W>
where
    T: Send + 'static,
    A: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(A, T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = A, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    BoxedWriterEffect::new(FoldWriter { items, init, f })
}

struct FoldWriter<T, A, F> {
    items: Vec<T>,
    init: A,
    f: F,
}

impl<T, A, E, Env, W, F, Eff> crate::effect::Effect for FoldWriter<T, A, F>
where
    T: Send + 'static,
    A: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(A, T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = A, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    type Output = A;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, _writes) = self.run_writer(env).await;
        result
    }
}

impl<T, A, E, Env, W, F, Eff> WriterEffect for FoldWriter<T, A, F>
where
    T: Send + 'static,
    A: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
    F: Fn(A, T) -> Eff + Send + 'static,
    Eff: WriterEffect<Output = A, Error = E, Env = Env, Writes = W> + Send + 'static,
{
    type Writes = W;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let mut accumulator = self.init;
        let mut all_writes = W::empty();

        for item in self.items {
            let effect = (self.f)(accumulator, item);
            let (result, writes) = effect.run_writer(env).await;
            all_writes = all_writes.combine(writes);

            match result {
                Ok(value) => accumulator = value,
                Err(e) => return (Err(e), all_writes),
            }
        }

        (Ok(accumulator), all_writes)
    }
}
