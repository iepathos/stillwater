//! Fallback combinator for providing default values on error.

use crate::effect::Effect;

/// Provides a default value on any error.
///
/// Zero-cost: no heap allocation. Stores only the inner effect
/// and the default value.
///
/// # Examples
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let count = get_count().fallback(0);
/// // Returns 0 on any error
/// ```
pub struct Fallback<E>
where
    E: Effect,
{
    pub(crate) inner: E,
    pub(crate) default: E::Output,
}

impl<E> std::fmt::Debug for Fallback<E>
where
    E: Effect,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fallback")
            .field("inner", &"<effect>")
            .field("default", &"<value>")
            .finish()
    }
}

impl<E> Fallback<E>
where
    E: Effect,
{
    /// Creates a new `Fallback` combinator.
    ///
    /// # Parameters
    /// - `inner`: The effect to execute
    /// - `default`: The default value to return if the effect fails
    pub fn new(inner: E, default: E::Output) -> Self {
        Self { inner, default }
    }
}

impl<E> Effect for Fallback<E>
where
    E: Effect,
    E::Output: Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(_) => Ok(self.default),
        }
    }
}
