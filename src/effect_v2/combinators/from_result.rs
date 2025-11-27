//! FromResult - effect from a Result value.

use std::marker::PhantomData;

use crate::effect_v2::trait_def::Effect;

/// Effect from a Result value.
///
/// Zero-cost: no heap allocation. The Result is stored directly
/// in the struct and returned when the effect is run.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = from_result::<_, String, ()>(Ok(42));
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let effect = from_result::<i32, _, ()>(Err("error".to_string()));
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// ```
pub struct FromResult<T, E, Env> {
    pub(crate) result: Result<T, E>,
    pub(crate) _phantom: PhantomData<Env>,
}

impl<T, E, Env> std::fmt::Debug for FromResult<T, E, Env>
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromResult")
            .field("result", &self.result)
            .finish()
    }
}

impl<T, E, Env> FromResult<T, E, Env> {
    /// Create a new FromResult effect.
    pub fn new(result: Result<T, E>) -> Self {
        FromResult {
            result,
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Env) -> Result<T, E> {
        self.result
    }
}
