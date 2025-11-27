//! Pure effect - wraps a value as an effect with no side effects.

use std::marker::PhantomData;

use crate::effect_v2::trait_def::Effect;

/// A pure value wrapped as an Effect.
///
/// This is zero-cost - no heap allocation occurs. The `Pure` struct
/// stores only the value itself plus phantom data for type parameters.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = pure::<_, String, ()>(42);
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
#[derive(Debug, Clone)]
pub struct Pure<T, E, Env> {
    value: T,
    _phantom: PhantomData<(E, Env)>,
}

impl<T, E, Env> Pure<T, E, Env> {
    /// Create a new Pure effect from a value.
    pub fn new(value: T) -> Self {
        Pure {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for Pure<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<T, E> {
        Ok(self.value)
    }
}
