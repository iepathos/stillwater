//! Extension trait providing combinator methods for all SinkEffects.

use std::sync::{Arc, Mutex};

use crate::effect::sink::and_then::SinkAndThen;
use crate::effect::sink::boxed::BoxedSinkEffect;
use crate::effect::sink::map::SinkMap;
use crate::effect::sink::map_err::SinkMapErr;
use crate::effect::sink::or_else::SinkOrElse;
use crate::effect::sink::tap_emit::TapEmit;
use crate::effect::sink::zip::SinkZip;
use crate::effect::sink::SinkEffect;

/// Extension trait providing Sink-specific combinator methods for all SinkEffects.
///
/// This trait is automatically implemented for all types that implement `SinkEffect`.
/// You don't need to implement this trait yourself.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("step 1".to_string())
///     .and_then(|_| emit("step 2".to_string()))
///     .map(|_| 42)
///     .tap_emit(|n| format!("result: {}", n));
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["step 1", "step 2", "result: 42"]);
/// # });
/// ```
pub trait SinkEffectExt: SinkEffect {
    /// Chain a dependent SinkEffect.
    ///
    /// If this effect succeeds, the function is called with the output
    /// to produce the next effect. Items from both effects are streamed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("first".to_string())
    ///     .and_then(|_| emit("second".to_string()));
    ///
    /// let (result, logs) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(logs, vec!["first", "second"]);
    /// # });
    /// ```
    fn and_then<F, E2>(self, f: F) -> SinkAndThen<Self, F>
    where
        Self: Sized,
        E2: SinkEffect<Error = Self::Error, Env = Self::Env, Item = Self::Item>,
        F: FnOnce(Self::Output) -> E2 + Send,
    {
        SinkAndThen { inner: self, f }
    }

    /// Transform the output value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("log".to_string())
    ///     .map(|_| 42)
    ///     .map(|n| n * 2);
    ///
    /// let (result, _) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(84));
    /// # });
    /// ```
    fn map<F, U>(self, f: F) -> SinkMap<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> U + Send,
        U: Send,
    {
        SinkMap { inner: self, f }
    }

    /// Transform the error value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect: SinkMapErr<_, _> = emit::<String, i32, ()>("log".to_string())
    ///     .map_err(|e: i32| format!("Error: {}", e));
    ///
    /// let (result, _) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(()));
    /// # });
    /// ```
    fn map_err<F, E2>(self, f: F) -> SinkMapErr<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Error) -> E2 + Send,
        E2: Send,
    {
        SinkMapErr { inner: self, f }
    }

    /// Recover from an error.
    ///
    /// If this effect fails, the function is called with the error
    /// to produce a recovery effect.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    /// use stillwater::effect::prelude::fail;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = into_sink::<_, _, String>(fail::<i32, String, ()>("error".to_string()))
    ///     .or_else(|_| emit::<String, String, ()>("recovered".to_string()).map(|_| 42));
    ///
    /// let (result, logs) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(logs, vec!["recovered".to_string()]);
    /// # });
    /// ```
    fn or_else<F, E2>(self, f: F) -> SinkOrElse<Self, F>
    where
        Self: Sized,
        E2: SinkEffect<Output = Self::Output, Env = Self::Env, Item = Self::Item>,
        F: FnOnce(Self::Error) -> E2 + Send,
    {
        SinkOrElse { inner: self, f }
    }

    /// Combine with another SinkEffect.
    ///
    /// Both effects are run sequentially, with their outputs combined
    /// into a tuple. Items from both are streamed in order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let left = emit::<_, String, ()>("left".to_string()).map(|_| 1);
    /// let right = emit::<_, String, ()>("right".to_string()).map(|_| 2);
    ///
    /// let (result, logs) = left.zip(right).run_collecting(&()).await;
    /// assert_eq!(result, Ok((1, 2)));
    /// assert_eq!(logs, vec!["left", "right"]);
    /// # });
    /// ```
    fn zip<E2>(self, other: E2) -> SinkZip<Self, E2>
    where
        Self: Sized,
        E2: SinkEffect<Error = Self::Error, Env = Self::Env, Item = Self::Item>,
    {
        SinkZip {
            left: self,
            right: other,
        }
    }

    /// Emit a derived value after success.
    ///
    /// If this effect succeeds, the function is called with a reference
    /// to the output, and the result is emitted to the sink.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    /// use stillwater::effect::prelude::pure;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = into_sink::<_, _, String>(pure::<_, String, ()>(42))
    ///     .tap_emit(|n| format!("Result: {}", n));
    ///
    /// let (result, logs) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(logs, vec!["Result: 42".to_string()]);
    /// # });
    /// ```
    fn tap_emit<F>(self, f: F) -> TapEmit<Self, F>
    where
        Self: Sized,
        Self::Output: Clone + Send,
        F: FnOnce(&Self::Output) -> Self::Item + Send,
    {
        TapEmit { inner: self, f }
    }

    /// Execute and collect all emissions (for testing).
    ///
    /// This bridges SinkEffect to WriterEffect-like semantics,
    /// collecting all emitted items into a Vec for assertions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("a".to_string())
    ///     .and_then(|_| emit("b".to_string()))
    ///     .and_then(|_| emit("c".to_string()))
    ///     .map(|_| 42);
    ///
    /// let (result, collected) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(collected, vec!["a", "b", "c"]);
    /// # });
    /// ```
    #[allow(async_fn_in_trait)]
    async fn run_collecting(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Vec<Self::Item>)
    where
        Self: Sized,
        Self::Item: Send + 'static,
    {
        let collected: Arc<Mutex<Vec<Self::Item>>> = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = Arc::clone(&collected);

        let result = self
            .run_with_sink(env, move |item| {
                let collected = Arc::clone(&collected_clone);
                async move {
                    collected.lock().expect("mutex poisoned").push(item);
                }
            })
            .await;

        let items = Arc::try_unwrap(collected)
            .ok()
            .expect("sink should be dropped")
            .into_inner()
            .expect("mutex poisoned");

        (result, items)
    }

    /// Execute, discarding all emissions.
    ///
    /// Useful when you only care about the result, not the output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let result = emit::<_, String, ()>("hello".to_string())
    ///     .map(|_| 42)
    ///     .run_ignore_emissions(&())
    ///     .await;
    ///
    /// assert_eq!(result, Ok(42));
    /// # });
    /// ```
    #[allow(async_fn_in_trait)]
    async fn run_ignore_emissions(self, env: &Self::Env) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        self.run_with_sink(env, |_| async {}).await
    }

    /// Convert to a boxed SinkEffect for type erasure.
    ///
    /// Use this when you need to:
    /// - Store effects in collections
    /// - Return different effect types from match arms
    /// - Create recursive effects
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effects: Vec<BoxedSinkEffect<i32, std::convert::Infallible, (), String>> = vec![
    ///     emit::<_, std::convert::Infallible, ()>("a".to_string()).map(|_| 1).boxed_sink(),
    ///     emit::<_, std::convert::Infallible, ()>("b".to_string()).map(|_| 2).boxed_sink(),
    /// ];
    ///
    /// let mut results = Vec::new();
    /// for effect in effects {
    ///     let (result, _) = effect.run_collecting(&()).await;
    ///     results.push(result.unwrap());
    /// }
    /// assert_eq!(results, vec![1, 2]);
    /// # });
    /// ```
    fn boxed_sink(self) -> BoxedSinkEffect<Self::Output, Self::Error, Self::Env, Self::Item>
    where
        Self: Sized + Send + 'static,
        Self::Output: Send + 'static,
        Self::Error: Send + 'static,
        Self::Env: Clone + Send + Sync + 'static,
        Self::Item: Send + 'static,
    {
        BoxedSinkEffect::new(self)
    }
}

// Blanket implementation for all SinkEffect types
impl<E: SinkEffect> SinkEffectExt for E {}
