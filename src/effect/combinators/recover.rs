//! Recover combinator for selective error recovery.

use crate::effect::Effect;
use crate::predicate::Predicate;
use std::marker::PhantomData;

/// Recovers from errors matching a predicate.
///
/// Zero-cost: no heap allocation. The struct stores only the inner effect,
/// predicate, and handler function.
///
/// # Examples
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// #[derive(Debug, PartialEq, Clone)]
/// enum Error {
///     CacheMiss,
///     NetworkError,
/// }
///
/// let effect = fetch_from_cache(id)
///     .recover(
///         |e: &Error| matches!(e, Error::CacheMiss),
///         |_| fetch_from_db(id)
///     );
/// ```
pub struct Recover<E, P, H, E2> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) handler: H,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, P, H, E2> std::fmt::Debug for Recover<E, P, H, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recover")
            .field("inner", &"<effect>")
            .field("predicate", &"<predicate>")
            .field("handler", &"<handler>")
            .finish()
    }
}

impl<E, P, H, E2> Recover<E, P, H, E2> {
    pub fn new(inner: E, predicate: P, handler: H) -> Self {
        Self {
            inner,
            predicate,
            handler,
            _marker: PhantomData,
        }
    }
}

impl<E, P, H, E2> Effect for Recover<E, P, H, E2>
where
    E: Effect,
    P: Predicate<E::Error>,
    H: FnOnce(E::Error) -> E2 + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                if self.predicate.check(&error) {
                    (self.handler)(error).run(env).await
                } else {
                    Err(error)
                }
            }
        }
    }
}
