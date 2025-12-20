//! OrElse combinator - recovers from errors.

use crate::effect::trait_def::Effect;

/// OrElse combinator - recovers from errors.
///
/// Zero-cost: no heap allocation. The `OrElse` struct stores only
/// the inner effect and the recovery function.
///
/// If the inner effect succeeds, the value passes through unchanged.
/// If it fails, the recovery function is called with the error to
/// produce a new effect.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error")
///     .or_else(|_| pure(42));
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub struct OrElse<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
}

impl<Inner, F> std::fmt::Debug for OrElse<Inner, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrElse")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<Inner, F, E2> Effect for OrElse<Inner, F>
where
    Inner: Effect,
    E2: Effect<Output = Inner::Output, Env = Inner::Env>,
    F: FnOnce(Inner::Error) -> E2 + Send,
{
    type Output = Inner::Output;
    type Error = E2::Error;
    type Env = Inner::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(e) => (self.f)(e).run(env).await,
        }
    }
}

// WriterEffect implementation for OrElse - combines writes from original and recovery
impl<Inner, F, E2> crate::effect::writer::WriterEffect for OrElse<Inner, F>
where
    Inner: crate::effect::writer::WriterEffect,
    Inner::Writes: crate::Semigroup,
    E2: crate::effect::writer::WriterEffect<
        Output = Inner::Output,
        Env = Inner::Env,
        Writes = Inner::Writes,
    >,
    F: FnOnce(Inner::Error) -> E2 + Send,
{
    type Writes = Inner::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        use crate::Semigroup;

        let (result, writes1) = self.inner.run_writer(env).await;

        match result {
            Ok(value) => (Ok(value), writes1),
            Err(e) => {
                let (result2, writes2) = (self.f)(e).run_writer(env).await;
                (result2, writes1.combine(writes2))
            }
        }
    }
}
