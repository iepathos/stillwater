//! SinkEffect trait definition.

use std::future::Future;

use crate::effect::Effect;

/// An effect that emits items to a sink during execution.
///
/// Unlike `WriterEffect` which accumulates all writes in memory, `SinkEffect`
/// streams items to a provided sink function as they occur, enabling constant
/// memory usage regardless of output volume.
///
/// # When to Use
///
/// - **SinkEffect**: High-volume output, real-time streaming, production logging
/// - **WriterEffect**: Testing, short chains, audit trails needing full history
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("starting".to_string())
///     .and_then(|_| emit("processing".to_string()))
///     .and_then(|_| emit("done".to_string()))
///     .map(|_| 42);
///
/// // Stream to console
/// let result = effect.run_with_sink(&(), |log| async move {
///     println!("{}", log);
/// }).await;
///
/// assert_eq!(result, Ok(42));
/// # });
/// ```
pub trait SinkEffect: Effect {
    /// The type of items emitted to the sink.
    type Item: Send;

    /// Execute this effect, emitting items to the sink as they occur.
    ///
    /// The sink function is called for each emitted item. Items are
    /// emitted in order, and the sink can be async for I/O operations.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment for this effect
    /// * `sink` - Function called for each emitted item
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("hello".to_string());
    ///
    /// let result = effect.run_with_sink(&(), |item| async move {
    ///     // Could send to logging service, write to file, etc.
    ///     println!("Received: {}", item);
    /// }).await;
    /// # });
    /// ```
    fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send;
}
