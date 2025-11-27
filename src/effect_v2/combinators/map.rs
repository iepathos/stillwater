//! Map combinator - transforms the success value of an effect.

use crate::effect_v2::trait_def::Effect;

/// Map combinator - transforms the success value.
///
/// Zero-cost: no heap allocation. The `Map` struct stores only
/// the inner effect and the transformation function.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = pure::<_, String, ()>(21).map(|x| x * 2);
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub struct Map<Inner, F> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
}

impl<Inner, F> std::fmt::Debug for Map<Inner, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<Inner, F, U> Effect for Map<Inner, F>
where
    Inner: Effect,
    F: FnOnce(Inner::Output) -> U + Send,
    U: Send,
{
    type Output = U;
    type Error = Inner::Error;
    type Env = Inner::Env;

    async fn run(self, env: &Self::Env) -> Result<U, Self::Error> {
        let value = self.inner.run(env).await?;
        Ok((self.f)(value))
    }
}
