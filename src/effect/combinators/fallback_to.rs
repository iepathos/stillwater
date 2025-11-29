//! FallbackTo combinator for alternative effects on error.

use crate::effect::Effect;

/// Tries an alternative effect on any error.
///
/// Zero-cost: no heap allocation. Stores only the primary and
/// alternative effects.
///
/// # Examples
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let data = fetch_primary()
///     .fallback_to(fetch_secondary());
/// ```
pub struct FallbackTo<E1, E2> {
    pub(crate) primary: E1,
    pub(crate) alternative: E2,
}

impl<E1, E2> std::fmt::Debug for FallbackTo<E1, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FallbackTo")
            .field("primary", &"<effect>")
            .field("alternative", &"<effect>")
            .finish()
    }
}

impl<E1, E2> FallbackTo<E1, E2> {
    /// Creates a new `FallbackTo` combinator.
    ///
    /// # Parameters
    /// - `primary`: The primary effect to try first
    /// - `alternative`: The alternative effect to try if the primary fails
    pub fn new(primary: E1, alternative: E2) -> Self {
        Self {
            primary,
            alternative,
        }
    }
}

impl<E1, E2> Effect for FallbackTo<E1, E2>
where
    E1: Effect,
    E2: Effect<Output = E1::Output, Error = E1::Error, Env = E1::Env>,
{
    type Output = E1::Output;
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.primary.run(env).await {
            Ok(value) => Ok(value),
            Err(_) => self.alternative.run(env).await,
        }
    }
}
