//! TapTell combinator - emit a derived value after the inner effect succeeds.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Semigroup;

/// An effect that emits a derived value after the inner effect succeeds.
///
/// This is useful for logging the result of a computation without
/// changing the control flow. If the inner effect fails, the tap
/// is not executed.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("start".to_string())
///     .map(|_| 42)
///     .tap_tell(|n| vec![format!("result: {}", n)]);
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(42));
/// assert_eq!(logs, vec!["start".to_string(), "result: 42".to_string()]);
/// # });
/// ```
pub struct TapTell<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for TapTell<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TapTell")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, W2> Effect for TapTell<E, F>
where
    E: WriterEffect,
    E::Output: Clone + Send,
    F: FnOnce(&E::Output) -> W2 + Send,
    W2: Into<E::Writes>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, F, W2> WriterEffect for TapTell<E, F>
where
    E: WriterEffect,
    E::Output: Clone + Send,
    E::Writes: Semigroup,
    F: FnOnce(&E::Output) -> W2 + Send,
    W2: Into<E::Writes>,
{
    type Writes = E::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, writes) = self.inner.run_writer(env).await;

        match &result {
            Ok(value) => {
                let additional: E::Writes = (self.f)(value).into();
                (result, writes.combine(additional))
            }
            Err(_) => (result, writes),
        }
    }
}
