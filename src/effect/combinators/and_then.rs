//! AndThen combinator - chains dependent effects.

use crate::effect::trait_def::Effect;

/// AndThen combinator - chains dependent effects.
///
/// Zero-cost: no heap allocation. The `AndThen` struct stores only
/// the inner effect and the function that produces the next effect.
///
/// The error type of the chained effect must match the error type
/// of the original effect. Use `map_err` to convert error types
/// before chaining:
///
/// ```rust,ignore
/// fetch_user(id)                           // Error = DbError
///     .map_err(AppError::from)             // Error = AppError
///     .and_then(|user| send_email(user))   // Error = AppError (via Into)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(21)
///     .and_then(|x| pure(x * 2));
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub struct AndThen<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
}

impl<Inner, F> std::fmt::Debug for AndThen<Inner, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndThen")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<Inner, F, E2> Effect for AndThen<Inner, F>
where
    Inner: Effect,
    E2: Effect<Error = Inner::Error, Env = Inner::Env>,
    F: FnOnce(Inner::Output) -> E2 + Send,
{
    type Output = E2::Output;
    type Error = Inner::Error;
    type Env = Inner::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        (self.f)(value).run(env).await
    }
}

// WriterEffect implementation for AndThen - combines writes from both effects
impl<Inner, F, E2> crate::effect::writer::WriterEffect for AndThen<Inner, F>
where
    Inner: crate::effect::writer::WriterEffect,
    Inner::Writes: crate::Semigroup,
    E2: crate::effect::writer::WriterEffect<
        Error = Inner::Error,
        Env = Inner::Env,
        Writes = Inner::Writes,
    >,
    F: FnOnce(Inner::Output) -> E2 + Send,
{
    type Writes = Inner::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        use crate::Semigroup;

        let (result1, writes1) = self.inner.run_writer(env).await;

        match result1 {
            Ok(value) => {
                let next_effect = (self.f)(value);
                let (result2, writes2) = next_effect.run_writer(env).await;
                (result2, writes1.combine(writes2))
            }
            Err(e) => (Err(e), writes1),
        }
    }
}
