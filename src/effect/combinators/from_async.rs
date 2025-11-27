//! FromAsync - effect from an async function.

use std::future::Future;
use std::marker::PhantomData;

use crate::effect::trait_def::Effect;

/// Effect from an async function.
///
/// Zero-cost: no heap allocation (beyond the future itself).
/// The async function is stored directly in the struct and
/// invoked when the effect is run.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = from_async::<_, String, (), _, _>(|_| async { Ok(42) });
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub struct FromAsync<F, Env> {
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<Env>,
}

impl<F, Env> std::fmt::Debug for FromAsync<F, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromAsync")
            .field("f", &"<function>")
            .finish()
    }
}

impl<F, Env> FromAsync<F, Env> {
    /// Create a new FromAsync effect.
    pub fn new(f: F) -> Self {
        FromAsync {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, Fut, T, E, Env> Effect for FromAsync<F, Env>
where
    F: FnOnce(&Env) -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        (self.f)(env)
    }
}
