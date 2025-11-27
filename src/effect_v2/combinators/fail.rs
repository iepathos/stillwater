//! Fail effect - represents a failed computation.

use std::marker::PhantomData;

use crate::effect_v2::trait_def::Effect;

/// A failure value wrapped as an Effect.
///
/// This is zero-cost - no heap allocation occurs. The `Fail` struct
/// stores only the error value plus phantom data for type parameters.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error".to_string());
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// ```
#[derive(Debug, Clone)]
pub struct Fail<T, E, Env> {
    error: E,
    _phantom: PhantomData<(T, Env)>,
}

impl<T, E, Env> Fail<T, E, Env> {
    /// Create a new Fail effect from an error.
    pub fn new(error: E) -> Self {
        Fail {
            error,
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for Fail<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<T, E> {
        Err(self.error)
    }
}
