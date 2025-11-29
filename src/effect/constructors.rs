//! Free function constructors for creating effects.
//!
//! This module provides standalone functions as an ergonomic alternative to
//! associated function syntax. Instead of writing `Effect::pure(x)`, you can
//! write simply `pure(x)`.
//!
//! # Quick Start
//!
//! ```rust
//! use stillwater::effect::prelude::*;
//!
//! # tokio_test::block_on(async {
//! // Free functions for concise, readable effect creation
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x * 2)
//!     .and_then(|x| pure(x + 1));
//!
//! let result = effect.execute(&()).await;
//! assert_eq!(result, Ok(85));
//! # });
//! ```
//!
//! # Available Free Functions
//!
//! ## Value Constructors
//! - [`pure`] - Create effect that succeeds with a value
//! - [`fail`] - Create effect that fails with an error
//!
//! ## Conversion Constructors
//! - [`from_fn`] - Create effect from synchronous function
//! - [`from_async`] - Create effect from async function
//! - [`from_result`] - Lift a `Result` into an effect
//! - [`from_option`] - Lift an `Option` into an effect
//! - [`from_validation`] - Convert `Validation` to effect
//!
//! ## Reader Operations
//! - [`ask`] - Get the entire environment
//! - [`asks`] - Query a value from environment
//! - [`local`] - Run effect with modified environment
//!
//! ## Combinators
//! - [`zip3`] through [`zip8`] - Combine multiple effects
//!
//! # Why Free Functions?
//!
//! Free functions provide several benefits:
//!
//! 1. **Conciseness**: Less visual noise in effect chains
//! 2. **FP Idiom**: Familiar to users of Haskell, Scala ZIO, etc.
//! 3. **Composability**: Functions compose naturally
//! 4. **Readability**: Focus on what, not which type
//!
//! ## Comparison
//!
//! | Associated Function Style | Free Function Style |
//! |--------------------------|---------------------|
//! | `Effect::pure(42)` | `pure(42)` |
//! | `Effect::fail(err)` | `fail(err)` |
//! | `Effect::asks(\|e\| ...)` | `asks(\|e\| ...)` |
//! | `Effect::from_fn(f)` | `from_fn(f)` |
//!
//! ## Before and After Example
//!
//! ```rust,ignore
//! // Associated function style (verbose)
//! Effect::asks(|env: &Env| env.db.clone())
//!     .and_then(|db| Effect::from_async(move |_| db.query()))
//!
//! // Free function style (concise)
//! asks(|env: &Env| env.db.clone())
//!     .and_then(|db| from_async(move |_| db.query()))
//! ```
//!
//! The free function style reduces boilerplate while maintaining the same
//! type safety and zero-cost abstractions.

use std::future::Future;

use crate::effect::combinators::{
    Fail, FromAsync, FromFn, FromResult, Pure, Zip3, Zip4, Zip5, Zip6, Zip7, Zip8,
};
use crate::effect::reader::{Ask, Asks, Local};
use crate::effect::trait_def::Effect;

/// Create a pure effect that succeeds with the given value.
///
/// Zero-cost: no heap allocation.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = pure::<_, String, ()>(42);
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = fail::<i32, _, ()>("error".to_string());
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// # tokio_test::block_on(async {
/// let effect = from_fn(|env: &Env| Ok::<_, String>(env.value * 2));
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = from_async(|_: &()| async { Ok::<_, String>(42) });
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = from_result::<_, String, ()>(Ok(42));
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let effect = from_result::<i32, _, ()>(Err("error".to_string()));
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = from_option::<_, _, ()>(Some(42), || "missing".to_string());
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let effect = from_option::<i32, _, ()>(None, || "missing".to_string());
/// assert_eq!(effect.execute(&()).await, Err("missing".to_string()));
/// # });
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct Env { value: i32 }
///
/// # tokio_test::block_on(async {
/// let effect = ask::<String, Env>();
/// assert_eq!(effect.execute(&Env { value: 42 }).await, Ok(Env { value: 42 }));
/// # });
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
/// This is the `asks` operation from the Reader monad. This is one of the most
/// commonly used functions for accessing dependencies from the environment.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone)]
/// struct Env { value: i32 }
///
/// # tokio_test::block_on(async {
/// let effect = asks::<_, String, _, _>(|env: &Env| env.value * 2);
/// assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
/// # });
/// ```
///
/// See also: [`ask`], [`local`]
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
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Clone)]
/// struct OuterEnv { multiplier: i32 }
/// #[derive(Clone)]
/// struct InnerEnv { value: i32 }
///
/// # tokio_test::block_on(async {
/// let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
/// let effect = local(
///     |outer: &OuterEnv| InnerEnv { value: 21 * outer.multiplier },
///     inner_effect,
/// );
///
/// assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
/// # });
/// ```
///
/// See also: [`ask`], [`asks`]
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
/// ```rust
/// use stillwater::effect::prelude::*;
/// use stillwater::Validation;
///
/// # tokio_test::block_on(async {
/// let validation = Validation::<_, String>::success(42);
/// let effect = from_validation::<_, _, ()>(validation);
/// assert_eq!(effect.execute(&()).await, Ok(42));
///
/// let validation = Validation::<i32, _>::failure("error".to_string());
/// let effect = from_validation::<_, _, ()>(validation);
/// assert_eq!(effect.execute(&()).await, Err("error".to_string()));
/// # });
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

/// Combine three effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip3` type, no heap allocation.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = zip3(
///     fetch_user(id),
///     fetch_orders(id),
///     fetch_preferences(id),
/// );
/// // Returns Zip3<...> with Output = (User, Vec<Order>, Preferences)
/// ```
pub fn zip3<E1, E2, E3>(e1: E1, e2: E2, e3: E3) -> Zip3<E1, E2, E3>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip3::new(e1, e2, e3)
}

/// Combine four effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip4` type, no heap allocation.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = zip4(fetch_a(), fetch_b(), fetch_c(), fetch_d());
/// // Returns Zip4<...> with Output = (A, B, C, D)
/// ```
pub fn zip4<E1, E2, E3, E4>(e1: E1, e2: E2, e3: E3, e4: E4) -> Zip4<E1, E2, E3, E4>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip4::new(e1, e2, e3, e4)
}

/// Combine five effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip5` type, no heap allocation.
pub fn zip5<E1, E2, E3, E4, E5>(e1: E1, e2: E2, e3: E3, e4: E4, e5: E5) -> Zip5<E1, E2, E3, E4, E5>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip5::new(e1, e2, e3, e4, e5)
}

/// Combine six effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip6` type, no heap allocation.
pub fn zip6<E1, E2, E3, E4, E5, E6>(
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
) -> Zip6<E1, E2, E3, E4, E5, E6>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip6::new(e1, e2, e3, e4, e5, e6)
}

/// Combine seven effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip7` type, no heap allocation.
pub fn zip7<E1, E2, E3, E4, E5, E6, E7>(
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
    e7: E7,
) -> Zip7<E1, E2, E3, E4, E5, E6, E7>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
    E7: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip7::new(e1, e2, e3, e4, e5, e6, e7)
}

/// Combine eight effects into a flat tuple.
///
/// Zero-cost: returns a concrete `Zip8` type, no heap allocation.
#[allow(clippy::too_many_arguments)]
pub fn zip8<E1, E2, E3, E4, E5, E6, E7, E8>(
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
    e7: E7,
    e8: E8,
) -> Zip8<E1, E2, E3, E4, E5, E6, E7, E8>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
    E7: Effect<Error = E1::Error, Env = E1::Env>,
    E8: Effect<Error = E1::Error, Env = E1::Env>,
{
    Zip8::new(e1, e2, e3, e4, e5, e6, e7, e8)
}
