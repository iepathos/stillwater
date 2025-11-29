//! Ensure combinator for validating effect outputs with closure predicates.

use crate::effect::Effect;

/// Validates the effect's output with a closure predicate.
///
/// If the predicate returns true, the value passes through.
/// If the predicate returns false, the effect fails with the provided error.
///
/// This is the closure-based version. For composable predicates from the
/// predicate module, use `ensure_pred` instead.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(25)
///     .ensure(|age| *age >= 18, "Must be 18 or older".to_string());
///
/// assert_eq!(effect.execute(&()).await, Ok(25));
/// ```
pub struct Ensure<E, P, Err> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) error: Err,
}

impl<E, P, Err> std::fmt::Debug for Ensure<E, P, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ensure")
            .field("inner", &"<effect>")
            .field("predicate", &"<function>")
            .field("error", &"<error>")
            .finish()
    }
}

impl<E, P, Err> Ensure<E, P, Err> {
    /// Create a new Ensure combinator.
    pub fn new(inner: E, predicate: P, error: Err) -> Self {
        Ensure {
            inner,
            predicate,
            error,
        }
    }
}

impl<E, P, Err> Effect for Ensure<E, P, Err>
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
        if (self.predicate)(&value) {
            Ok(value)
        } else {
            Err(self.error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::effect::constructors::{fail, pure};
    use crate::effect::EffectExt;

    #[tokio::test]
    async fn test_ensure_passes_when_true() {
        let effect = pure::<_, String, ()>(5).ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_ensure_fails_when_false() {
        let effect = pure::<_, String, ()>(-5).ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(
            effect.execute(&()).await,
            Err("must be positive".to_string())
        );
    }

    #[tokio::test]
    async fn test_ensure_short_circuits_on_prior_error() {
        let effect = fail::<i32, _, ()>("prior error".to_string())
            .ensure(|_| panic!("should not be called"), "other".to_string());
        assert_eq!(effect.execute(&()).await, Err("prior error".to_string()));
    }

    #[tokio::test]
    async fn test_chained_ensures() {
        let effect = pure::<_, String, ()>(50)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .ensure(|x| *x < 100, "must be less than 100".to_string())
            .ensure(|x| *x % 2 == 0, "must be even".to_string());
        assert_eq!(effect.execute(&()).await, Ok(50));
    }

    #[tokio::test]
    async fn test_chained_ensures_first_fails() {
        let effect = pure::<_, String, ()>(-5)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .ensure(|_| panic!("should not reach"), "other".to_string());
        assert_eq!(
            effect.execute(&()).await,
            Err("must be positive".to_string())
        );
    }

    #[tokio::test]
    async fn test_ensure_with_map() {
        let effect = pure::<_, String, ()>(5)
            .map(|x| x * 2)
            .ensure(|x| *x > 5, "must be greater than 5".to_string())
            .map(|x| x + 1);
        assert_eq!(effect.execute(&()).await, Ok(11));
    }

    #[tokio::test]
    async fn test_ensure_with_and_then() {
        let effect = pure::<_, String, ()>(5)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .and_then(|x| pure(x * 2));
        assert_eq!(effect.execute(&()).await, Ok(10));
    }
}
