//! BoxedSinkEffect for type erasure.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::effect::sink::SinkEffect;
use crate::effect::Effect;

/// Type alias for the boxed sink effect inner function type.
///
/// This captures the result and any emitted items for later replay.
type BoxedSinkFn<T, E, Env, Item> = Box<
    dyn FnOnce(Env) -> Pin<Box<dyn Future<Output = (Result<T, E>, Vec<Item>)> + Send + 'static>>
        + Send
        + 'static,
>;

/// A type-erased SinkEffect for use in collections, match arms, or recursive functions.
///
/// Similar to `BoxedEffect`, this enables:
/// - Storing different sink effect types in `Vec` or `HashMap`
/// - Returning different effect types from match arms
/// - Breaking infinite types in recursive functions
///
/// # Implementation Note
///
/// Unlike the zero-cost SinkEffect implementations, BoxedSinkEffect internally
/// collects items and then replays them to the sink. This means it has O(n)
/// memory for the items during execution. For truly constant-memory streaming
/// of boxed effects, consider restructuring to avoid boxing.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// // Store heterogeneous sink effects
/// let effects: Vec<BoxedSinkEffect<i32, String, (), String>> = vec![
///     emit("a".to_string()).map(|_| 1).boxed_sink(),
///     emit("b".to_string()).map(|_| 2).boxed_sink(),
/// ];
///
/// let mut results = Vec::new();
/// let mut all_logs = Vec::new();
///
/// for effect in effects {
///     let (result, logs) = effect.run_collecting(&()).await;
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
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// fn conditional_log(flag: bool) -> BoxedSinkEffect<i32, String, (), String> {
///     if flag {
///         emit("enabled".to_string()).map(|_| 1).boxed_sink()
///     } else {
///         into_sink::<_, _, String>(pure::<_, String, ()>(0)).boxed_sink()
///     }
/// }
///
/// # tokio_test::block_on(async {
/// let (result, logs) = conditional_log(true).run_collecting(&()).await;
/// assert_eq!(result, Ok(1));
/// assert_eq!(logs, vec!["enabled".to_string()]);
/// # });
/// ```
pub struct BoxedSinkEffect<T, E, Env, Item>
where
    Env: Clone + Send + Sync + 'static,
    Item: Send + 'static,
{
    inner: BoxedSinkFn<T, E, Env, Item>,
}

impl<T, E, Env, Item> std::fmt::Debug for BoxedSinkEffect<T, E, Env, Item>
where
    Env: Clone + Send + Sync + 'static,
    Item: Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedSinkEffect").finish_non_exhaustive()
    }
}

impl<T, E, Env, Item> BoxedSinkEffect<T, E, Env, Item>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    Item: Send + 'static,
{
    /// Create a new BoxedSinkEffect from a SinkEffect.
    pub fn new<Eff>(effect: Eff) -> Self
    where
        Eff: SinkEffect<Output = T, Error = E, Env = Env, Item = Item> + Send + 'static,
    {
        BoxedSinkEffect {
            inner: Box::new(move |env: Env| {
                Box::pin(async move {
                    // Collect all items during execution
                    let collected: Arc<Mutex<Vec<Item>>> = Arc::new(Mutex::new(Vec::new()));
                    let collected_clone = Arc::clone(&collected);

                    let result = effect
                        .run_with_sink(&env, move |item| {
                            let collected = Arc::clone(&collected_clone);
                            async move {
                                collected.lock().expect("mutex poisoned").push(item);
                            }
                        })
                        .await;

                    let items = Arc::try_unwrap(collected)
                        .ok()
                        .expect("collected should be unique")
                        .into_inner()
                        .expect("mutex poisoned");

                    (result, items)
                })
            }),
        }
    }
}

impl<T, E, Env, Item> Effect for BoxedSinkEffect<T, E, Env, Item>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    Item: Send + 'static,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let (result, _items) = (self.inner)(env.clone()).await;
        result
    }
}

impl<T, E, Env, Item> SinkEffect for BoxedSinkEffect<T, E, Env, Item>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
    Item: Send + 'static,
{
    type Item = Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        // Execute and collect, then replay to actual sink
        let (result, items) = (self.inner)(env.clone()).await;

        for item in items {
            sink(item).await;
        }

        result
    }
}
