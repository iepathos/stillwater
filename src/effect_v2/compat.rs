//! Backward compatibility module for migration from old Effect API.
//!
//! This module provides type aliases and helper methods to ease migration
//! from the boxed `Effect` struct to the new trait-based system.
//!
//! # Migration Guide
//!
//! ## Old API vs New API
//!
//! | Old API | New API |
//! |---------|---------|
//! | `Effect<T, E, Env>` struct | `impl Effect<Output=T, Error=E, Env=Env>` |
//! | `Effect::pure(x)` | `pure::<_, E, Env>(x)` |
//! | `Effect::fail(e)` | `fail::<T, _, Env>(e)` |
//! | `.run(&env).await` | `.execute(&env).await` or `.run(&env).await` |
//!
//! ## Migration Steps
//!
//! 1. **Update imports**: Change `use stillwater::Effect` to `use stillwater::effect_v2::prelude::*`
//!
//! 2. **Update function signatures**: Change return types from `Effect<T, E, Env>` to
//!    `impl Effect<Output = T, Error = E, Env = Env>` for zero-cost, or `BoxedEffect<T, E, Env>`
//!    when you need type erasure.
//!
//! 3. **Update constructors**: Change `Effect::pure(x)` to `pure(x)` (type inference usually works)
//!
//! 4. **Add `.boxed()` where needed**: For collections, recursive effects, or match arms
//!    with different effect types.
//!
//! ## Example Migration
//!
//! ```rust,ignore
//! // Old code
//! fn old_workflow() -> Effect<User, AppError, AppEnv> {
//!     Effect::pure(user_id)
//!         .and_then(|id| fetch_user(id))
//!         .map(|user| user.name)
//! }
//!
//! // New code - zero cost
//! fn new_workflow() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
//!     pure(user_id)
//!         .and_then(|id| fetch_user(id))
//!         .map(|user| user.name)
//! }
//!
//! // New code - when you need concrete type
//! fn new_workflow_boxed() -> BoxedEffect<String, AppError, AppEnv> {
//!     pure(user_id)
//!         .and_then(|id| fetch_user(id))
//!         .map(|user| user.name)
//!         .boxed()
//! }
//! ```

use crate::effect_v2::boxed::BoxedEffect;
use crate::effect_v2::ext::EffectExt;

/// Type alias for backward compatibility.
///
/// This provides the old `Effect<T, E, Env>` API as a type alias to `BoxedEffect`.
/// Use this during migration, then update to the new zero-cost API.
#[deprecated(
    since = "0.11.0",
    note = "Use `impl Effect<...>` for zero-cost or `BoxedEffect` for type erasure"
)]
pub type LegacyEffect<T, E, Env> = BoxedEffect<T, E, Env>;

/// Helper trait adding legacy constructor methods to BoxedEffect.
///
/// This provides the `BoxedEffect::pure()` and `BoxedEffect::fail()` associated
/// functions that existed on the old `Effect` struct.
#[deprecated(since = "0.11.0", note = "Use `pure()` and `fail()` functions instead")]
pub trait LegacyConstructors<T, E, Env>: Sized {
    /// Create a pure effect (boxed).
    ///
    /// Deprecated: Use `pure(value).boxed()` instead.
    fn legacy_pure(value: T) -> BoxedEffect<T, E, Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Clone + Send + Sync + 'static;

    /// Create a failing effect (boxed).
    ///
    /// Deprecated: Use `fail(error).boxed()` instead.
    fn legacy_fail(error: E) -> BoxedEffect<T, E, Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Clone + Send + Sync + 'static;
}

#[allow(deprecated)]
impl<T, E, Env> LegacyConstructors<T, E, Env> for BoxedEffect<T, E, Env> {
    fn legacy_pure(value: T) -> BoxedEffect<T, E, Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Clone + Send + Sync + 'static,
    {
        crate::effect_v2::constructors::pure(value).boxed()
    }

    fn legacy_fail(error: E) -> BoxedEffect<T, E, Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Clone + Send + Sync + 'static,
    {
        crate::effect_v2::constructors::fail(error).boxed()
    }
}

/// Extension trait for running effects with unit environment.
#[allow(async_fn_in_trait)]
pub trait RunStandalone: crate::effect_v2::trait_def::Effect<Env = ()> {
    /// Run an effect that doesn't require an environment.
    ///
    /// This is a convenience method for effects with `Env = ()` (unit type).
    async fn run_standalone(self) -> Result<Self::Output, Self::Error>;
}

impl<E: crate::effect_v2::trait_def::Effect<Env = ()>> RunStandalone for E {
    async fn run_standalone(self) -> Result<Self::Output, Self::Error> {
        self.run(&()).await
    }
}
