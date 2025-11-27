//! BoxedEffect - type-erased effect for opt-in boxing.
//!
//! Use `BoxedEffect` when you need to:
//! - Store different effect types in a collection
//! - Return different effects from match arms
//! - Create recursive effect functions
//!
//! Boxing clones the environment to achieve `'static` lifetime.
//! This is cheap when `Env` contains `Arc`-wrapped resources.

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use crate::effect_v2::trait_def::Effect;

/// A boxed future that is Send + 'static
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A type-erased effect.
///
/// Use `BoxedEffect` when you need type erasure for:
/// - Storing different effect types in a collection
/// - Returning different effects from match arms
/// - Creating recursive effect functions
///
/// **Note**: Boxing clones the environment to achieve `'static` lifetime.
/// This is cheap when `Env` contains `Arc`-wrapped resources.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// // Store different effects in a Vec
/// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
///     pure(1).boxed(),
///     pure(2).and_then(|x| pure(x * 2)).boxed(),
/// ];
///
/// // Recursive effect
/// fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
///     if n <= 0 {
///         pure(0).boxed()
///     } else {
///         pure(n)
///             .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
///             .boxed()
///     }
/// }
/// ```
pub struct BoxedEffect<T, E, Env> {
    // Takes OWNED Env (cloned from reference at run time)
    run_fn: Box<dyn FnOnce(Env) -> BoxFuture<'static, Result<T, E>> + Send>,
    _phantom: PhantomData<Env>,
}

impl<T, E, Env> std::fmt::Debug for BoxedEffect<T, E, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedEffect")
            .field("run_fn", &"<function>")
            .finish()
    }
}

impl<T, E, Env> BoxedEffect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Create a boxed effect from any effect.
    ///
    /// The environment will be cloned when the effect is run.
    pub fn new<Eff>(effect: Eff) -> Self
    where
        Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
    {
        BoxedEffect {
            run_fn: Box::new(move |env: Env| {
                // env is now owned, so the async block is 'static
                Box::pin(async move { effect.run(&env).await })
            }),
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for BoxedEffect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        let env_owned = env.clone(); // Clone here for 'static lifetime
        (self.run_fn)(env_owned)
    }
}

/// A type-erased effect that is not Send (for non-Send futures).
///
/// Use `BoxedLocalEffect` when your effect contains non-Send types
/// and you don't need to run it across threads.
pub struct BoxedLocalEffect<T, E, Env> {
    run_fn: Box<dyn FnOnce(Env) -> Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>>,
    _phantom: PhantomData<Env>,
}

impl<T, E, Env> std::fmt::Debug for BoxedLocalEffect<T, E, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedLocalEffect")
            .field("run_fn", &"<function>")
            .finish()
    }
}

impl<T, E, Env> BoxedLocalEffect<T, E, Env>
where
    T: 'static,
    E: 'static,
    Env: Clone + 'static,
{
    /// Create a boxed local effect from any effect.
    ///
    /// The environment will be cloned when the effect is run.
    pub fn new<Eff>(effect: Eff) -> Self
    where
        Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
        Eff::Output: 'static,
        Eff::Error: 'static,
    {
        BoxedLocalEffect {
            run_fn: Box::new(move |env: Env| Box::pin(async move { effect.run(&env).await })),
            _phantom: PhantomData,
        }
    }

    /// Run the boxed local effect with the given environment.
    pub fn run_local(self, env: &Env) -> Pin<Box<dyn Future<Output = Result<T, E>> + 'static>> {
        let env_owned = env.clone();
        (self.run_fn)(env_owned)
    }
}
