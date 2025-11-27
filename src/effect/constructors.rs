//! Constructor functions for creating effects.
//!
//! These functions provide ergonomic ways to create effects without
//! directly constructing the combinator types.

use std::future::Future;

use crate::effect::combinators::{Fail, FromAsync, FromFn, FromResult, Pure};
use crate::effect::reader::{Ask, Asks, Local};
use crate::effect::trait_def::Effect;

/// Create a pure effect that succeeds with the given value.
///
/// Zero-cost: no heap allocation.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(42);
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub fn pure<T, E, Env>(value: T) -> Pure<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Pure::new(value)
}

/// Create an effect that fails with the given error.
///
/// Zero-cost: no heap allocation.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = fail::<i32, _, ()>("error".to_string());
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// ```
pub fn fail<T, E, Env>(error: E) -> Fail<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Fail::new(error)
}

/// Create an effect from a synchronous function.
///
/// The function receives a reference to the environment and returns a `Result`.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// let effect = from_fn(|env: &Env| Ok::<_, String>(env.value * 2));
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// ```
pub fn from_fn<T, E, Env, F>(f: F) -> FromFn<F, Env>
where
    F: FnOnce(&Env) -> Result<T, E> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    FromFn::new(f)
}

/// Create an effect from an async function.
///
/// The function receives a reference to the environment and returns a Future.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = from_async(|_: &()| async { Ok::<_, String>(42) });
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub fn from_async<T, E, Env, F, Fut>(f: F) -> FromAsync<F, Env>
where
    F: FnOnce(&Env) -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    FromAsync::new(f)
}

/// Create an effect from a Result.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = from_result::<_, String, ()>(Ok(42));
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let effect = from_result::<i32, _, ()>(Err("error".to_string()));
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// ```
pub fn from_result<T, E, Env>(result: Result<T, E>) -> FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    FromResult::new(result)
}

/// Create an effect from an Option, using a default error if None.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = from_option::<_, _, ()>(Some(42), || "missing".to_string());
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let effect = from_option::<i32, _, ()>(None, || "missing".to_string());
/// assert_eq!(effect.execute(&()).await, Err("missing".to_string()));
/// ```
pub fn from_option<T, E, Env>(
    option: Option<T>,
    error_fn: impl FnOnce() -> E,
) -> FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    FromResult::new(option.ok_or_else(error_fn))
}

/// Get the entire environment (cloned).
///
/// This is the `ask` operation from the Reader monad.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Env { value: i32 }
///
/// let effect = ask::<String, Env>();
/// assert_eq!(effect.execute(&Env { value: 42 }).await, Ok(Env { value: 42 }));
/// ```
pub fn ask<E, Env>() -> Ask<E, Env>
where
    Env: Clone + Send + Sync,
    E: Send,
{
    Ask::new()
}

/// Query a value from the environment.
///
/// This is the `asks` operation from the Reader monad.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// let effect = asks::<_, String, _, _>(|env: &Env| env.value * 2);
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// ```
pub fn asks<U, E, Env, F>(f: F) -> Asks<F, E, Env>
where
    F: FnOnce(&Env) -> U + Send,
    U: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Asks::new(f)
}

/// Run an effect with a modified environment.
///
/// This is the `local` operation from the Reader monad.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
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
pub fn local<Inner, F, Env2>(f: F, inner: Inner) -> Local<Inner, F, Env2>
where
    Inner: Effect,
    F: FnOnce(&Env2) -> Inner::Env + Send,
    Env2: Clone + Send + Sync,
{
    Local::new(inner, f)
}

/// Create an effect from a Validation.
///
/// Converts a `Validation<T, E>` into an effect. Success becomes `Ok(value)`,
/// Failure becomes `Err(error)`.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
/// use stillwater::Validation;
///
/// let validation = Validation::<_, String>::success(42);
/// let effect = from_validation::<_, _, ()>(validation);
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let validation = Validation::<i32, _>::failure("error".to_string());
/// let effect = from_validation::<_, _, ()>(validation);
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// ```
pub fn from_validation<T, E, Env>(validation: crate::Validation<T, E>) -> FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    let result = match validation {
        crate::Validation::Success(value) => Ok(value),
        crate::Validation::Failure(error) => Err(error),
    };
    FromResult::new(result)
}
