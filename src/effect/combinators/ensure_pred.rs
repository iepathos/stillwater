//! EnsurePred combinator for validation using Predicate trait.

use crate::effect::Effect;
use crate::predicate::Predicate;

/// Validates using a Predicate from the predicate module.
///
/// This enables composable, reusable predicates like:
/// - `between(18, 120)`
/// - `gt(0).and(lt(100))`
/// - `len_min(3).and(len_max(20))`
///
/// For inline validation with closures, use `ensure` instead.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::predicate::*;
///
/// let valid_age = between(18, 120);
/// let effect = pure::<_, String, ()>(25)
///     .ensure_pred(valid_age, "invalid age".to_string());
///
/// assert_eq!(effect.execute(&()).await, Ok(25));
/// ```
pub struct EnsurePred<E, P, Err> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) error: Err,
}

impl<E, P, Err> std::fmt::Debug for EnsurePred<E, P, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnsurePred")
            .field("inner", &"<effect>")
            .field("predicate", &"<predicate>")
            .field("error", &"<error>")
            .finish()
    }
}

impl<E, P, Err> EnsurePred<E, P, Err> {
    /// Create a new EnsurePred combinator.
    pub fn new(inner: E, predicate: P, error: Err) -> Self {
        EnsurePred {
            inner,
            predicate,
            error,
        }
    }
}

impl<E, P, Err> Effect for EnsurePred<E, P, Err>
where
    E: Effect,
    P: Predicate<E::Output> + Send,
    Err: Into<E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        if self.predicate.check(&value) {
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
    use crate::predicate::*;

    #[tokio::test]
    async fn test_ensure_pred_with_predicate() {
        let effect =
            pure::<_, String, ()>(25).ensure_pred(between(18, 120), "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_ensure_pred_fails() {
        let effect =
            pure::<_, String, ()>(150).ensure_pred(between(18, 120), "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Err("invalid age".to_string()));
    }

    #[tokio::test]
    async fn test_ensure_pred_with_composed_predicate() {
        let valid_age = gt(0).and(lt(150));

        let effect = pure::<_, String, ()>(25).ensure_pred(valid_age, "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_mixing_closure_and_predicate() {
        let effect = pure::<_, String, ()>(25)
            .ensure(|x| *x > 0, "must be positive".to_string()) // Closure
            .ensure_pred(between(0, 150), "out of range".to_string()); // Predicate
        assert_eq!(effect.execute(&()).await, Ok(25));
    }
}
