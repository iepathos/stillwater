//! Reader pattern types for environment access.
//!
//! This module provides the Reader monad pattern for accessing and
//! modifying the environment in effects:
//!
//! - `Ask` - Get the entire environment (cloned)
//! - `Asks` - Query a value from the environment
//! - `Local` - Run an effect with a modified environment

use std::marker::PhantomData;

use crate::effect_v2::trait_def::Effect;

/// Get the entire environment (cloned).
///
/// Zero-cost struct, but clones `Env` at runtime.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Env { value: i32 }
///
/// let effect = ask::<String, Env>();
/// assert_eq!(effect.execute(&Env { value: 42 }).await, Ok(Env { value: 42 }));
/// ```
pub struct Ask<E, Env> {
    _phantom: PhantomData<(E, Env)>,
}

impl<E, Env> std::fmt::Debug for Ask<E, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ask").finish()
    }
}

impl<E, Env> Ask<E, Env> {
    /// Create a new Ask effect.
    pub fn new() -> Self {
        Ask {
            _phantom: PhantomData,
        }
    }
}

impl<E, Env> Default for Ask<E, Env> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, Env> Effect for Ask<E, Env>
where
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = Env;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl std::future::Future<Output = Result<Env, E>> + Send {
        let env_clone = env.clone();
        async move { Ok(env_clone) }
    }
}

/// Query a value from the environment.
///
/// Zero-cost: no heap allocation. The query function is stored
/// directly in the struct and invoked when the effect is run.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// let effect = asks::<_, String, _, _>(|env: &Env| env.value * 2);
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// ```
pub struct Asks<F, E, Env> {
    pub(crate) f: F,
    _phantom: PhantomData<(E, Env)>,
}

impl<F, E, Env> std::fmt::Debug for Asks<F, E, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Asks")
            .field("f", &"<function>")
            .finish()
    }
}

impl<F, E, Env> Asks<F, E, Env> {
    /// Create a new Asks effect.
    pub fn new(f: F) -> Self {
        Asks {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, U, E, Env> Effect for Asks<F, E, Env>
where
    F: FnOnce(&Env) -> U + Send,
    U: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = U;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Env) -> Result<U, E> { Ok((self.f)(env)) }
}

/// Run an effect with a modified environment.
///
/// Zero-cost: no heap allocation. The environment transformation
/// function and inner effect are stored directly in the struct.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// #[derive(Clone)]
/// struct OuterEnv { multiplier: i32 }
/// #[derive(Clone)]
/// struct InnerEnv { value: i32 }
///
/// let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
/// let effect = local(
///     |outer: &OuterEnv| InnerEnv { value: 21 * outer.multiplier },
///     inner_effect,
/// );
///
/// assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
/// ```
pub struct Local<Inner, F, Env2> {
    pub(crate) inner: Inner,
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<Env2>,
}

impl<Inner, F, Env2> std::fmt::Debug for Local<Inner, F, Env2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Local")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<Inner, F, Env2> Local<Inner, F, Env2> {
    /// Create a new Local effect.
    pub fn new(inner: Inner, f: F) -> Self {
        Local {
            inner,
            f,
            _phantom: PhantomData,
        }
    }
}

impl<Inner, F, Env2> Effect for Local<Inner, F, Env2>
where
    Inner: Effect,
    F: FnOnce(&Env2) -> Inner::Env + Send,
    Env2: Clone + Send + Sync,
{
    type Output = Inner::Output;
    type Error = Inner::Error;
    type Env = Env2;

    fn run(
        self,
        env: &Env2,
    ) -> impl std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
        let inner_env = (self.f)(env);
        async move { self.inner.run(&inner_env).await }
    }
}
