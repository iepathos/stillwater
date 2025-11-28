//! ZipWith combinator - combines two effects with a function.

use crate::effect::trait_def::Effect;

/// Combines two effects with a function.
///
/// More efficient than `zip().map()` as it's a single combinator struct
/// with no intermediate tuple allocation.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(2)
///     .zip_with(pure(3), |a, b| a * b);
/// assert_eq!(effect.execute(&()).await, Ok(6));
/// ```
#[derive(Debug)]
pub struct ZipWith<E1, E2, F> {
    pub(crate) first: E1,
    pub(crate) second: E2,
    pub(crate) f: F,
}

impl<E1, E2, F> ZipWith<E1, E2, F> {
    /// Create a new ZipWith combinator.
    pub fn new(first: E1, second: E2, f: F) -> Self {
        ZipWith { first, second, f }
    }
}

impl<E1, E2, F, R> Effect for ZipWith<E1, E2, F>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    F: FnOnce(E1::Output, E2::Output) -> R + Send,
    R: Send,
{
    type Output = R;
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<R, Self::Error> {
        let first_result = self.first.run(env).await?;
        let second_result = self.second.run(env).await?;
        Ok((self.f)(first_result, second_result))
    }
}
