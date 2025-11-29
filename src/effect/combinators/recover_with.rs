//! RecoverWith combinator for Result-returning recovery.

use crate::effect::Effect;
use crate::predicate::Predicate;

/// Recovers from errors with a Result-returning function.
///
/// Zero-cost: no heap allocation. Useful when recovery doesn't need
/// to run an effect, just return a value or transform the error.
///
/// # Examples
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = parse_config()
///     .recover_with(
///         |e: &ConfigError| e.is_missing_field(),
///         |_| Ok(Config::default())
///     );
/// ```
pub struct RecoverWith<E, P, F> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) handler: F,
}

impl<E, P, F> std::fmt::Debug for RecoverWith<E, P, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoverWith")
            .field("inner", &"<effect>")
            .field("predicate", &"<predicate>")
            .field("handler", &"<handler>")
            .finish()
    }
}

impl<E, P, F> RecoverWith<E, P, F> {
    /// Creates a new `RecoverWith` combinator.
    ///
    /// # Parameters
    /// - `inner`: The effect to execute
    /// - `predicate`: A predicate to check if an error should be recovered
    /// - `handler`: A function that handles matching errors and returns a Result
    pub fn new(inner: E, predicate: P, handler: F) -> Self {
        Self {
            inner,
            predicate,
            handler,
        }
    }
}

impl<E, P, F> Effect for RecoverWith<E, P, F>
where
    E: Effect,
    P: Predicate<E::Error>,
    F: FnOnce(E::Error) -> Result<E::Output, E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                if self.predicate.check(&error) {
                    (self.handler)(error)
                } else {
                    Err(error)
                }
            }
        }
    }
}
