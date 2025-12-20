//! Extension trait providing combinator methods for all WriterEffects.

use crate::effect::writer::boxed::BoxedWriterEffect;
use crate::effect::writer::censor::Censor;
use crate::effect::writer::listen::Listen;
use crate::effect::writer::pass::Pass;
use crate::effect::writer::tap_tell::TapTell;
use crate::effect::writer::WriterEffect;
use crate::Monoid;
use crate::Semigroup;

/// Extension trait providing Writer-specific combinator methods for all WriterEffects.
///
/// This trait is automatically implemented for all types that implement `WriterEffect`.
/// You don't need to implement this trait yourself.
///
/// Note: Common combinators like `map`, `and_then`, `or_else`, and `zip` are provided
/// by `EffectExt` and work automatically with WriterEffect because `WriterEffect`
/// extends `Effect`. The existing combinators (`Map`, `AndThen`, etc.) implement
/// `WriterEffect` when their inner effect does.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("step 1".to_string())
///     .map(|_| 42)  // Uses EffectExt::map, works with WriterEffect
///     .tap_tell(|n| vec![format!("result: {}", n)])  // Uses WriterEffectExt::tap_tell
///     .censor(|logs| logs.into_iter().filter(|l| l.contains("result")).collect());
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["result: 42".to_string()]);
/// # });
/// ```
pub trait WriterEffectExt: WriterEffect {
    /// Emit a derived value after success.
    ///
    /// If this effect succeeds, the function is called with a reference to
    /// the output, and the result is added to the accumulated writes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(42))
    ///     .tap_tell(|n| vec![format!("Result: {}", n)]);
    ///
    /// let (result, logs) = effect.run_writer(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(logs, vec!["Result: 42".to_string()]);
    /// # });
    /// ```
    fn tap_tell<F, W2>(self, f: F) -> TapTell<Self, F>
    where
        Self: Sized,
        Self::Output: Clone + Send,
        Self::Writes: Semigroup,
        F: FnOnce(&Self::Output) -> W2 + Send,
        W2: Into<Self::Writes>,
    {
        TapTell { inner: self, f }
    }

    /// Transform accumulated writes.
    ///
    /// The function receives all accumulated writes and can filter,
    /// transform, or otherwise modify them.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = tell_one::<_, String, ()>("debug: verbose".to_string())
    ///     .and_then(|_| tell_one("info: important".to_string()))
    ///     .censor(|logs| logs.into_iter().filter(|l| !l.starts_with("debug")).collect());
    ///
    /// let (_, logs) = effect.run_writer(&()).await;
    /// assert_eq!(logs, vec!["info: important".to_string()]);
    /// # });
    /// ```
    fn censor<F>(self, f: F) -> Censor<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Writes) -> Self::Writes + Send,
    {
        Censor { inner: self, f }
    }

    /// Include writes in output.
    ///
    /// The output becomes a tuple of `(original_output, writes)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = tell_one::<_, String, ()>("logged".to_string())
    ///     .map(|_| 42)
    ///     .listen();
    ///
    /// let (result, writes) = effect.run_writer(&()).await;
    /// assert_eq!(result, Ok((42, vec!["logged".to_string()])));
    /// assert_eq!(writes, vec!["logged".to_string()]);
    /// # });
    /// ```
    fn listen(self) -> Listen<Self>
    where
        Self: Sized,
        Self::Writes: Clone,
    {
        Listen { inner: self }
    }

    /// Use output to determine how to transform writes.
    ///
    /// The inner effect must produce a tuple `(T, F)` where `F` is a function
    /// that transforms the writes. The final output is just `T`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = tell::<_, String, ()>(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    ///     .map(|_| (42, |logs: Vec<String>| logs.into_iter().take(2).collect()))
    ///     .pass();
    ///
    /// let (result, logs) = effect.run_writer(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(logs, vec!["a".to_string(), "b".to_string()]);
    /// # });
    /// ```
    fn pass<T, F>(self) -> Pass<Self>
    where
        Self: WriterEffect<Output = (T, F)> + Sized,
        T: Send,
        F: FnOnce(Self::Writes) -> Self::Writes + Send,
    {
        Pass { inner: self }
    }

    /// Execute the effect, discarding accumulated writes.
    ///
    /// Useful for testing or when writes aren't needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let result = tell_one::<_, String, ()>("hello".to_string())
    ///     .map(|_| 42)
    ///     .run_ignore_writes(&())
    ///     .await;
    ///
    /// assert_eq!(result, Ok(42));
    /// # });
    /// ```
    #[allow(async_fn_in_trait)]
    async fn run_ignore_writes(self, env: &Self::Env) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let (result, _writes) = WriterEffect::run_writer(self, env).await;
        result
    }

    /// Convert to a boxed WriterEffect for type erasure.
    ///
    /// Use this when you need to:
    /// - Store effects in collections
    /// - Return different effect types from match arms
    /// - Create recursive effects
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effects: Vec<BoxedWriterEffect<i32, std::convert::Infallible, (), Vec<String>>> = vec![
    ///     tell_one::<_, std::convert::Infallible, ()>("a".to_string()).map(|_| 1).boxed_writer(),
    ///     tell_one::<_, std::convert::Infallible, ()>("b".to_string()).map(|_| 2).boxed_writer(),
    /// ];
    ///
    /// let mut results = Vec::new();
    /// for effect in effects {
    ///     let (result, _) = effect.run_writer(&()).await;
    ///     results.push(result.unwrap());
    /// }
    /// assert_eq!(results, vec![1, 2]);
    /// # });
    /// ```
    fn boxed_writer(self) -> BoxedWriterEffect<Self::Output, Self::Error, Self::Env, Self::Writes>
    where
        Self: Sized + Send + 'static,
        Self::Output: Send + 'static,
        Self::Error: Send + 'static,
        Self::Env: Clone + Send + Sync + 'static,
        Self::Writes: Monoid + Send + 'static,
    {
        BoxedWriterEffect::new(self)
    }
}

// Blanket implementation for all WriterEffect types
impl<E: WriterEffect> WriterEffectExt for E {}
