//! RecoverSome combinator for Option-based partial recovery.

use crate::effect::Effect;
use std::marker::PhantomData;

/// Recovers using an Option-returning partial function.
///
/// Requires Error: Clone to preserve the error when None is returned.
/// The error is cloned before being passed to the partial function,
/// so the original can be returned if recovery is not possible.
///
/// # Examples
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// #[derive(Debug, Clone)]
/// enum Error {
///     Timeout,
///     NotFound,
///     Fatal(String),
/// }
///
/// let effect = risky_operation()
///     .recover_some(|e| match e {
///         Error::Timeout => Some(pure(default_value())),
///         Error::NotFound => Some(create_new()),
///         _ => None, // Other errors propagate
///     });
/// ```
pub struct RecoverSome<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) partial_fn: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> std::fmt::Debug for RecoverSome<E, F, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoverSome")
            .field("inner", &"<effect>")
            .field("partial_fn", &"<function>")
            .finish()
    }
}

impl<E, F, E2> RecoverSome<E, F, E2> {
    pub fn new(inner: E, partial_fn: F) -> Self {
        Self {
            inner,
            partial_fn,
            _marker: PhantomData,
        }
    }
}

impl<E, F, E2> Effect for RecoverSome<E, F, E2>
where
    E: Effect,
    E::Error: Clone,
    F: FnOnce(E::Error) -> Option<E2> + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                // Clone error before passing to partial_fn so we can
                // return the original if None is returned
                let error_clone = error.clone();
                match (self.partial_fn)(error_clone) {
                    Some(recovery_effect) => recovery_effect.run(env).await,
                    None => Err(error), // Use original error
                }
            }
        }
    }
}
