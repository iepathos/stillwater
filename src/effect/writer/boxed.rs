//! BoxedWriterEffect for type erasure.

use std::future::Future;
use std::pin::Pin;

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Monoid;

// Type alias for the boxed writer effect inner function type
type BoxedWriterFn<T, E, Env, W> = Box<
    dyn FnOnce(Env) -> Pin<Box<dyn Future<Output = (Result<T, E>, W)> + Send + 'static>>
        + Send
        + 'static,
>;

/// A type-erased WriterEffect for use in collections, match arms, or recursive functions.
///
/// Similar to `BoxedEffect`, this enables:
/// - Storing different writer effect types in `Vec` or `HashMap`
/// - Returning different effect types from match arms
/// - Breaking infinite types in recursive functions
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Store heterogeneous writer effects
/// let effects: Vec<BoxedWriterEffect<i32, String, (), Vec<String>>> = vec![
///     tell_one("a".to_string()).map(|_| 1).boxed_writer(),
///     tell_one("b".to_string()).map(|_| 2).boxed_writer(),
/// ];
///
/// let mut results = Vec::new();
/// let mut all_logs = Vec::new();
///
/// for effect in effects {
///     let (result, logs) = effect.run_writer(&()).await;
///     results.push(result.unwrap());
///     all_logs.extend(logs);
/// }
///
/// assert_eq!(results, vec![1, 2]);
/// assert_eq!(all_logs, vec!["a".to_string(), "b".to_string()]);
/// # });
/// ```
///
/// # Match Arms with Different Types
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// fn conditional_log(flag: bool) -> BoxedWriterEffect<i32, String, (), Vec<String>> {
///     if flag {
///         tell_one("enabled".to_string()).map(|_| 1).boxed_writer()
///     } else {
///         into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(0)).boxed_writer()
///     }
/// }
///
/// # tokio_test::block_on(async {
/// let (result, logs) = conditional_log(true).run_writer(&()).await;
/// assert_eq!(result, Ok(1));
/// assert_eq!(logs, vec!["enabled".to_string()]);
/// # });
/// ```
pub struct BoxedWriterEffect<T, E, Env, W>
where
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
{
    inner: BoxedWriterFn<T, E, Env, W>,
}

impl<T, E, Env, W> std::fmt::Debug for BoxedWriterEffect<T, E, Env, W>
where
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedWriterEffect").finish_non_exhaustive()
    }
}

impl<T, E, Env, W> BoxedWriterEffect<T, E, Env, W>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
{
    /// Create a new BoxedWriterEffect from a WriterEffect.
    pub fn new<Eff>(effect: Eff) -> Self
    where
        Eff: WriterEffect<Output = T, Error = E, Env = Env, Writes = W> + Send + 'static,
    {
        BoxedWriterEffect {
            inner: Box::new(move |env: Env| Box::pin(async move { effect.run_writer(&env).await })),
        }
    }
}

impl<T, E, Env, W> Effect for BoxedWriterEffect<T, E, Env, W>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, _writes) = (self.inner)(env.clone()).await;
        result
    }
}

impl<T, E, Env, W> WriterEffect for BoxedWriterEffect<T, E, Env, W>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    W: Monoid + Send + 'static,
{
    type Writes = W;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        (self.inner)(env.clone()).await
    }
}
