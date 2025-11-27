//! Effect trait definition - the core abstraction for zero-cost effects.
//!
//! This module defines the `Effect` trait, which represents a computation that:
//! - Produces a value of type `Output` on success
//! - May fail with an error of type `Error`
//! - Depends on an environment of type `Env`
//!
//! # Design Philosophy
//!
//! This trait follows the same pattern as `Future` and `Iterator`:
//! - Combinators return concrete types (zero-cost abstractions)
//! - Use `.boxed()` when you need type erasure
//!
//! # Environment Cloning
//!
//! The `Env` type requires `Clone` to enable boxing. When an effect is boxed,
//! the environment must be cloned into the boxed future to achieve `'static`
//! lifetime. This is typically cheap when environments contain `Arc`-wrapped
//! resources:
//!
//! ```rust,ignore
//! #[derive(Clone)]
//! struct AppEnv {
//!     db: Arc<DatabasePool>,
//!     config: Arc<Config>,
//!     http: Arc<HttpClient>,
//! }
//! ```

use std::future::Future;

/// The core Effect trait - represents a computation that may perform effects.
///
/// This trait is the foundation of Stillwater's zero-cost effect system.
/// Unlike the boxed `Effect` struct, implementing types can be zero-sized
/// when possible, and combinators return concrete types rather than boxed
/// trait objects.
///
/// # Type Parameters
///
/// * `Output` - The success type produced by this effect (must be `Send`)
/// * `Error` - The error type that may be produced (must be `Send`)
/// * `Env` - The environment type required to run this effect (must be `Clone + Send + Sync`)
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// fn fetch_user(id: i32) -> impl Effect<Output = User, Error = DbError, Env = AppEnv> {
///     asks(|env: &AppEnv| env.db.clone())
///         .and_then(move |db| from_async(move |_| db.fetch_user(id)))
/// }
/// ```
pub trait Effect: Sized + Send {
    /// The success type produced by this effect.
    type Output: Send;

    /// The error type that may be produced.
    type Error: Send;

    /// The environment type required to run this effect.
    ///
    /// Must be `Clone` to support boxing (cloning is deferred until boxing).
    type Env: Clone + Send + Sync;

    /// Execute this effect with the given environment.
    ///
    /// This is the core method that runs the effect. The returned future
    /// produces a `Result<Output, Error>`.
    ///
    /// # Arguments
    ///
    /// * `env` - Reference to the environment containing dependencies
    ///
    /// # Returns
    ///
    /// A future that resolves to `Ok(output)` on success or `Err(error)` on failure.
    fn run(self, env: &Self::Env)
        -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}
