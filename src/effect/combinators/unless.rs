//! Unless combinator for inverse validation logic.

use crate::effect::Effect;

/// Fails if the predicate returns true (inverse of Ensure).
///
/// This combinator is the logical inverse of `Ensure`. It passes the value
/// through if the predicate returns false, and fails if the predicate returns true.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(5)
///     .unless(|x| *x < 0, "must not be negative".to_string());
///
/// assert_eq!(effect.execute(&()).await, Ok(5));
/// ```
pub struct Unless<E, P, Err> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) error: Err,
}

impl<E, P, Err> std::fmt::Debug for Unless<E, P, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Unless")
            .field("inner", &"<effect>")
            .field("predicate", &"<function>")
            .field("error", &"<error>")
            .finish()
    }
}

impl<E, P, Err> Unless<E, P, Err> {
    /// Create a new Unless combinator.
    pub fn new(inner: E, predicate: P, error: Err) -> Self {
        Unless {
            inner,
            predicate,
            error,
        }
    }
}

impl<E, P, Err> Effect for Unless<E, P, Err>
where
    E: Effect,
    P: FnOnce(&E::Output) -> bool + Send,
    Err: Into<E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        if !(self.predicate)(&value) {
            Ok(value)
        } else {
            Err(self.error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::effect::constructors::pure;
    use crate::effect::EffectExt;

    #[tokio::test]
    async fn test_unless_passes_on_false() {
        let effect =
            pure::<_, String, ()>(5).unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_unless_fails_on_true() {
        let effect =
            pure::<_, String, ()>(-5).unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(
            effect.execute(&()).await,
            Err("must not be negative".to_string())
        );
    }
}
