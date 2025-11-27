//! FromFn - effect from a synchronous function.

use std::marker::PhantomData;

use crate::effect_v2::trait_def::Effect;

/// Effect from a synchronous function.
///
/// Zero-cost: no heap allocation. The function is stored directly
/// in the struct and invoked when the effect is run.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// let effect = from_fn::<_, String, _, _>(|env: &Env| Ok(env.value * 2));
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// ```
pub struct FromFn<F, Env> {
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<Env>,
}

impl<F, Env> std::fmt::Debug for FromFn<F, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromFn").field("f", &"<function>").finish()
    }
}

impl<F, Env> FromFn<F, Env> {
    /// Create a new FromFn effect.
    pub fn new(f: F) -> Self {
        FromFn {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, T, E, Env> Effect for FromFn<F, Env>
where
    F: FnOnce(&Env) -> Result<T, E> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Env) -> Result<T, E> {
        (self.f)(env)
    }
}
