//! Bracket pattern for safe resource management.
//!
//! The bracket pattern ensures resources are properly released even when
//! errors occur. This module provides:
//!
//! - [`bracket`] - Acquire/use/release with guaranteed cleanup
//! - [`bracket2`] - Two resources with LIFO cleanup
//! - [`bracket3`] - Three resources with LIFO cleanup
//! - [`bracket_full`] - Explicit error handling for both use and cleanup errors
//! - [`bracket_sync`] - Panic-safe variant with synchronous cleanup
//! - [`Resource`] - Encapsulated resource with reusable acquire/release
//! - [`Acquiring`] - Fluent builder for multiple resources
//! - [`BracketError`] - Error type for bracket operations
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//!
//! // Single resource
//! let result = bracket(
//!     open_connection(),
//!     |conn| async move { conn.close().await },
//!     |conn| fetch_user(conn, user_id),
//! ).run(&env).await;
//!
//! // Multiple resources with builder
//! let result = acquiring(
//!     open_connection(),
//!     |c| async move { c.close().await },
//! )
//! .and(open_file(path), |f| async move { f.close().await })
//! .with(|(conn, file)| process(conn, file))
//! .run(&env)
//! .await;
//! ```

use std::future::Future;
use std::marker::PhantomData;

use crate::effect::boxed::BoxFuture;
use crate::effect::trait_def::Effect;

// ============================================================================
// BracketError
// ============================================================================

/// Error type for bracket operations with explicit error handling.
///
/// This enum design ensures all states are valid - no invalid state possible.
/// Each variant clearly identifies which phase of the bracket operation failed.
///
/// # Variants
///
/// - `AcquireError` - Resource acquisition failed
/// - `UseError` - The use function failed, cleanup succeeded
/// - `CleanupError` - The use function succeeded, cleanup failed
/// - `Both` - Both use and cleanup failed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BracketError<E> {
    /// Resource acquisition failed - never got to use the resource.
    AcquireError(E),
    /// The use function failed, cleanup succeeded.
    UseError(E),
    /// The use function succeeded, cleanup failed.
    CleanupError(E),
    /// Both use and cleanup failed.
    Both {
        /// The error from the use function
        use_error: E,
        /// The error from the cleanup function
        cleanup_error: E,
    },
}

impl<E> BracketError<E> {
    /// Returns the acquire error, if any.
    pub fn acquire_error(&self) -> Option<&E> {
        match self {
            BracketError::AcquireError(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the use error, if any.
    pub fn use_error(&self) -> Option<&E> {
        match self {
            BracketError::UseError(e) | BracketError::Both { use_error: e, .. } => Some(e),
            _ => None,
        }
    }

    /// Returns the cleanup error, if any.
    pub fn cleanup_error(&self) -> Option<&E> {
        match self {
            BracketError::CleanupError(e)
            | BracketError::Both {
                cleanup_error: e, ..
            } => Some(e),
            _ => None,
        }
    }

    /// Maps the error type using the provided function.
    pub fn map<F, E2>(self, f: F) -> BracketError<E2>
    where
        F: Fn(E) -> E2,
    {
        match self {
            BracketError::AcquireError(e) => BracketError::AcquireError(f(e)),
            BracketError::UseError(e) => BracketError::UseError(f(e)),
            BracketError::CleanupError(e) => BracketError::CleanupError(f(e)),
            BracketError::Both {
                use_error,
                cleanup_error,
            } => BracketError::Both {
                use_error: f(use_error),
                cleanup_error: f(cleanup_error),
            },
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for BracketError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BracketError::AcquireError(e) => write!(f, "acquire failed: {}", e),
            BracketError::UseError(e) => write!(f, "{}", e),
            BracketError::CleanupError(e) => write!(f, "cleanup failed: {}", e),
            BracketError::Both {
                use_error,
                cleanup_error,
            } => {
                write!(
                    f,
                    "use failed: {}; cleanup also failed: {}",
                    use_error, cleanup_error
                )
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for BracketError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BracketError::AcquireError(e) => Some(e),
            BracketError::UseError(e) => Some(e),
            BracketError::Both { use_error, .. } => Some(use_error),
            BracketError::CleanupError(e) => Some(e),
        }
    }
}

// ============================================================================
// Bracket - core pattern
// ============================================================================

/// Bracket combinator type for resource management.
///
/// The bracket pattern has three phases:
/// 1. **Acquire**: Obtain the resource
/// 2. **Use**: Use the resource to produce a result
/// 3. **Release**: Release the resource (always runs, even on error)
///
/// Release errors are logged and the use result is returned.
pub struct Bracket<Acquire, Use, Release> {
    acquire: Acquire,
    use_fn: Use,
    release: Release,
}

impl<Acquire, Use, Release> std::fmt::Debug for Bracket<Acquire, Use, Release> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bracket")
            .field("acquire", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<Acquire, Use, Release> Bracket<Acquire, Use, Release> {
    /// Create a new Bracket.
    pub fn new(acquire: Acquire, use_fn: Use, release: Release) -> Self {
        Bracket {
            acquire,
            use_fn,
            release,
        }
    }
}

impl<Acquire, Use, Release, UseEffect, R, T, E, Env, RelFut> Effect
    for Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    R: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        // Acquire the resource
        let resource = self.acquire.run(env).await?;

        // Use the resource (borrowing for use, moving for release)
        let result = (self.use_fn)(&resource).run(env).await;

        // Release runs regardless of use result
        let release_result = (self.release)(resource).await;

        // Log cleanup errors if any
        if let Err(ref rel_err) = release_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        result
    }
}

/// Bracket pattern for safe resource management.
///
/// Acquires a resource, uses it, and guarantees release even on error.
/// Release errors are logged and the use result is returned.
///
/// # Type Parameters
///
/// * `Acquire` - Effect that acquires the resource
/// * `Use` - Function that uses the resource (receives a reference)
/// * `Release` - Function that releases the resource (receives ownership)
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = bracket(
///     from_fn(|env: &AppEnv| env.db.get_connection()),
///     |conn| async move { conn.close().await },
///     |conn| from_fn(move |_| conn.execute("SELECT 1")),
/// );
/// ```
pub fn bracket<Acquire, Use, Release, UseEffect, R, T, E, Env, RelFut>(
    acquire: Acquire,
    release: Release,
    use_fn: Use,
) -> Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    R: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    Bracket::new(acquire, use_fn, release)
}

// ============================================================================
// BracketFull - explicit error handling
// ============================================================================

/// Bracket with explicit error handling - returns BracketError with all error info.
pub struct BracketFull<Acquire, Use, Release> {
    acquire: Acquire,
    use_fn: Use,
    release: Release,
}

impl<Acquire, Use, Release> std::fmt::Debug for BracketFull<Acquire, Use, Release> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BracketFull")
            .field("acquire", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<Acquire, Use, Release> BracketFull<Acquire, Use, Release> {
    /// Create a new BracketFull.
    pub fn new(acquire: Acquire, use_fn: Use, release: Release) -> Self {
        BracketFull {
            acquire,
            use_fn,
            release,
        }
    }
}

impl<Acquire, Use, Release, UseEffect, R, T, E, Env, RelFut> Effect
    for BracketFull<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    R: Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = BracketError<E>;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, BracketError<E>> {
        // Acquire - map error to AcquireError
        let resource = match self.acquire.run(env).await {
            Ok(r) => r,
            Err(e) => return Err(BracketError::AcquireError(e)),
        };

        // Use resource
        let use_result = (self.use_fn)(&resource).run(env).await;

        // Release resource
        let release_result = (self.release)(resource).await;

        // Combine results
        match (use_result, release_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Ok(_), Err(cleanup_err)) => Err(BracketError::CleanupError(cleanup_err)),
            (Err(use_err), Ok(())) => Err(BracketError::UseError(use_err)),
            (Err(use_err), Err(cleanup_err)) => Err(BracketError::Both {
                use_error: use_err,
                cleanup_error: cleanup_err,
            }),
        }
    }
}

/// Bracket with explicit error handling.
///
/// Unlike [`bracket`], this returns a [`BracketError`] that contains
/// information about both use and cleanup errors.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let result = bracket_full(
///     acquire_resource(),
///     |r| async move { r.close().await },
///     |r| use_resource(r),
/// ).run(&env).await;
///
/// match result {
///     Ok(value) => println!("Success: {:?}", value),
///     Err(BracketError::UseError(e)) => println!("Use failed: {:?}", e),
///     Err(BracketError::CleanupError(e)) => println!("Cleanup failed: {:?}", e),
///     Err(BracketError::Both { use_error, cleanup_error }) => {
///         println!("Both failed: {:?}, {:?}", use_error, cleanup_error);
///     }
///     Err(BracketError::AcquireError(e)) => println!("Acquire failed: {:?}", e),
/// }
/// ```
pub fn bracket_full<Acquire, Use, Release, UseEffect, R, T, E, Env, RelFut>(
    acquire: Acquire,
    release: Release,
    use_fn: Use,
) -> BracketFull<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    R: Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    BracketFull::new(acquire, use_fn, release)
}

// ============================================================================
// BracketSync - panic-safe with synchronous cleanup
// ============================================================================

/// Panic-safe bracket with synchronous cleanup.
pub struct BracketSync<Acquire, Use, Release> {
    acquire: Acquire,
    use_fn: Use,
    release: Release,
}

impl<Acquire, Use, Release> std::fmt::Debug for BracketSync<Acquire, Use, Release> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BracketSync")
            .field("acquire", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<Acquire, Use, Release> BracketSync<Acquire, Use, Release> {
    /// Create a new BracketSync.
    pub fn new(acquire: Acquire, use_fn: Use, release: Release) -> Self {
        BracketSync {
            acquire,
            use_fn,
            release,
        }
    }
}

impl<Acquire, Use, Release, UseEffect, R, T, E, Env> Effect for BracketSync<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send + std::panic::UnwindSafe,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> Result<(), E> + Send,
    R: Send + std::panic::UnwindSafe,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync + std::panic::RefUnwindSafe,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        // Acquire resource
        let resource = self.acquire.run(env).await?;

        // Use resource with panic catching
        let use_result = {
            let resource_ref = &resource;
            let env_for_use = env;
            let use_fn = self.use_fn;

            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // We use block_on to await inside catch_unwind
                futures::executor::block_on(use_fn(resource_ref).run(env_for_use))
            }))
        };

        // Release resource (always runs, even after panic)
        let release_result = (self.release)(resource);

        // Handle results
        match use_result {
            Ok(Ok(value)) => {
                if let Err(ref rel_err) = release_result {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                    #[cfg(not(feature = "tracing"))]
                    eprintln!("Resource cleanup failed: {:?}", rel_err);
                }
                Ok(value)
            }
            Ok(Err(use_err)) => {
                if let Err(ref rel_err) = release_result {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                    #[cfg(not(feature = "tracing"))]
                    eprintln!("Resource cleanup failed: {:?}", rel_err);
                }
                Err(use_err)
            }
            Err(panic_payload) => {
                // Log cleanup error if any, then re-panic
                if let Err(ref rel_err) = release_result {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Resource cleanup failed after panic: {:?}", rel_err);
                    #[cfg(not(feature = "tracing"))]
                    eprintln!("Resource cleanup failed after panic: {:?}", rel_err);
                }
                std::panic::resume_unwind(panic_payload)
            }
        }
    }
}

/// Panic-safe bracket with synchronous cleanup.
///
/// Unlike [`bracket`], this variant guarantees cleanup runs even if the use
/// function panics, provided the release function is synchronous.
///
/// # Panic Safety
///
/// - If `use_fn` panics, cleanup still runs, then the panic is re-raised
/// - If cleanup fails after a panic, the cleanup error is logged and panic re-raised
///
/// # Limitations
///
/// Uses `futures::executor::block_on` internally, which creates a nested runtime.
/// This may cause issues if the use function spawns tasks on the outer runtime.
///
/// # When to Use
///
/// - Your cleanup is synchronous (or can be made sync)
/// - You need guaranteed cleanup even on panic
/// - You're at an application boundary where panics might occur
pub fn bracket_sync<Acquire, Use, Release, UseEffect, R, T, E, Env>(
    acquire: Acquire,
    release: Release,
    use_fn: Use,
) -> BracketSync<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(&R) -> UseEffect + Send + std::panic::UnwindSafe,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> Result<(), E> + Send,
    R: Send + std::panic::UnwindSafe,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync + std::panic::RefUnwindSafe,
{
    BracketSync::new(acquire, use_fn, release)
}

// ============================================================================
// Bracket2 and Bracket3 - multiple resources
// ============================================================================

/// Bracket with two resources, released in reverse order (LIFO).
pub struct Bracket2<Acq1, Acq2, Use, Rel1, Rel2> {
    acquire1: Acq1,
    acquire2: Acq2,
    use_fn: Use,
    release1: Rel1,
    release2: Rel2,
}

impl<Acq1, Acq2, Use, Rel1, Rel2> std::fmt::Debug for Bracket2<Acq1, Acq2, Use, Rel1, Rel2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bracket2")
            .field("acquire1", &"<effect>")
            .field("acquire2", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release1", &"<function>")
            .field("release2", &"<function>")
            .finish()
    }
}

impl<Acq1, Acq2, Use, Rel1, Rel2, UseEffect, R1, R2, T, E, Env, RelFut1, RelFut2> Effect
    for Bracket2<Acq1, Acq2, Use, Rel1, Rel2>
where
    Acq1: Effect<Output = R1, Error = E, Env = Env>,
    Acq2: Effect<Output = R2, Error = E, Env = Env>,
    Use: FnOnce(&R1, &R2) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Rel1: FnOnce(R1) -> RelFut1 + Send,
    RelFut1: Future<Output = Result<(), E>> + Send,
    Rel2: FnOnce(R2) -> RelFut2 + Send,
    RelFut2: Future<Output = Result<(), E>> + Send,
    R1: Send,
    R2: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        // Acquire first resource
        let r1 = self.acquire1.run(env).await?;

        // Acquire second resource
        let r2 = match self.acquire2.run(env).await {
            Ok(r) => r,
            Err(e) => {
                // Release first resource on failure
                let release_result = (self.release1)(r1).await;
                if let Err(ref rel_err) = release_result {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Resource cleanup failed: {:?}", rel_err);
                    #[cfg(not(feature = "tracing"))]
                    eprintln!("Resource cleanup failed: {:?}", rel_err);
                }
                return Err(e);
            }
        };

        // Use both resources
        let result = (self.use_fn)(&r1, &r2).run(env).await;

        // Release in reverse order (LIFO)
        let rel2_result = (self.release2)(r2).await;
        if let Err(ref rel_err) = rel2_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        let rel1_result = (self.release1)(r1).await;
        if let Err(ref rel_err) = rel1_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        result
    }
}

/// Bracket with two resources, released in reverse order (LIFO).
///
/// Resources are acquired in order (first, then second) and released
/// in reverse order (second, then first).
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let result = bracket2(
///     open_connection(),       // Acquired first
///     open_file(path),         // Acquired second
///     |conn| async move { conn.close().await },  // Released second
///     |file| async move { file.close().await },  // Released first (LIFO)
///     |conn, file| process(conn, file),
/// ).run(&env).await;
/// ```
pub fn bracket2<Acq1, Acq2, Use, Rel1, Rel2, UseEffect, R1, R2, T, E, Env, RelFut1, RelFut2>(
    acquire1: Acq1,
    acquire2: Acq2,
    release1: Rel1,
    release2: Rel2,
    use_fn: Use,
) -> Bracket2<Acq1, Acq2, Use, Rel1, Rel2>
where
    Acq1: Effect<Output = R1, Error = E, Env = Env>,
    Acq2: Effect<Output = R2, Error = E, Env = Env>,
    Use: FnOnce(&R1, &R2) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Rel1: FnOnce(R1) -> RelFut1 + Send,
    RelFut1: Future<Output = Result<(), E>> + Send,
    Rel2: FnOnce(R2) -> RelFut2 + Send,
    RelFut2: Future<Output = Result<(), E>> + Send,
    R1: Send,
    R2: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    Bracket2 {
        acquire1,
        acquire2,
        use_fn,
        release1,
        release2,
    }
}

/// Bracket with three resources, released in reverse order (LIFO).
pub struct Bracket3<Acq1, Acq2, Acq3, Use, Rel1, Rel2, Rel3> {
    acquire1: Acq1,
    acquire2: Acq2,
    acquire3: Acq3,
    use_fn: Use,
    release1: Rel1,
    release2: Rel2,
    release3: Rel3,
}

impl<Acq1, Acq2, Acq3, Use, Rel1, Rel2, Rel3> std::fmt::Debug
    for Bracket3<Acq1, Acq2, Acq3, Use, Rel1, Rel2, Rel3>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bracket3")
            .field("acquire1", &"<effect>")
            .field("acquire2", &"<effect>")
            .field("acquire3", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release1", &"<function>")
            .field("release2", &"<function>")
            .field("release3", &"<function>")
            .finish()
    }
}

impl<
        Acq1,
        Acq2,
        Acq3,
        Use,
        Rel1,
        Rel2,
        Rel3,
        UseEffect,
        R1,
        R2,
        R3,
        T,
        E,
        Env,
        RelFut1,
        RelFut2,
        RelFut3,
    > Effect for Bracket3<Acq1, Acq2, Acq3, Use, Rel1, Rel2, Rel3>
where
    Acq1: Effect<Output = R1, Error = E, Env = Env>,
    Acq2: Effect<Output = R2, Error = E, Env = Env>,
    Acq3: Effect<Output = R3, Error = E, Env = Env>,
    Use: FnOnce(&R1, &R2, &R3) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Rel1: FnOnce(R1) -> RelFut1 + Send,
    RelFut1: Future<Output = Result<(), E>> + Send,
    Rel2: FnOnce(R2) -> RelFut2 + Send,
    RelFut2: Future<Output = Result<(), E>> + Send,
    Rel3: FnOnce(R3) -> RelFut3 + Send,
    RelFut3: Future<Output = Result<(), E>> + Send,
    R1: Send,
    R2: Send,
    R3: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        // Acquire first resource
        let r1 = self.acquire1.run(env).await?;

        // Acquire second resource
        let r2 = match self.acquire2.run(env).await {
            Ok(r) => r,
            Err(e) => {
                let _ = (self.release1)(r1).await;
                return Err(e);
            }
        };

        // Acquire third resource
        let r3 = match self.acquire3.run(env).await {
            Ok(r) => r,
            Err(e) => {
                // Release in reverse order
                let _ = (self.release2)(r2).await;
                let _ = (self.release1)(r1).await;
                return Err(e);
            }
        };

        // Use all resources
        let result = (self.use_fn)(&r1, &r2, &r3).run(env).await;

        // Release in reverse order (LIFO)
        let rel3_result = (self.release3)(r3).await;
        if let Err(ref rel_err) = rel3_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        let rel2_result = (self.release2)(r2).await;
        if let Err(ref rel_err) = rel2_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        let rel1_result = (self.release1)(r1).await;
        if let Err(ref rel_err) = rel1_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        result
    }
}

/// Bracket with three resources, released in reverse order (LIFO).
pub fn bracket3<
    Acq1,
    Acq2,
    Acq3,
    Use,
    Rel1,
    Rel2,
    Rel3,
    UseEffect,
    R1,
    R2,
    R3,
    T,
    E,
    Env,
    RelFut1,
    RelFut2,
    RelFut3,
>(
    acquire1: Acq1,
    acquire2: Acq2,
    acquire3: Acq3,
    release1: Rel1,
    release2: Rel2,
    release3: Rel3,
    use_fn: Use,
) -> Bracket3<Acq1, Acq2, Acq3, Use, Rel1, Rel2, Rel3>
where
    Acq1: Effect<Output = R1, Error = E, Env = Env>,
    Acq2: Effect<Output = R2, Error = E, Env = Env>,
    Acq3: Effect<Output = R3, Error = E, Env = Env>,
    Use: FnOnce(&R1, &R2, &R3) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Rel1: FnOnce(R1) -> RelFut1 + Send,
    RelFut1: Future<Output = Result<(), E>> + Send,
    Rel2: FnOnce(R2) -> RelFut2 + Send,
    RelFut2: Future<Output = Result<(), E>> + Send,
    Rel3: FnOnce(R3) -> RelFut3 + Send,
    RelFut3: Future<Output = Result<(), E>> + Send,
    R1: Send,
    R2: Send,
    R3: Send,
    T: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    Bracket3 {
        acquire1,
        acquire2,
        acquire3,
        use_fn,
        release1,
        release2,
        release3,
    }
}

// ============================================================================
// Resource - reusable acquire/release pair
// ============================================================================

/// A resource that can be acquired and must be released.
///
/// `Resource` encapsulates the acquire/release pattern, making it
/// reusable and composable.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let db = Resource::new(
///     pure(DatabaseConnection::new()),
///     |conn| async move { conn.close().await }
/// );
///
/// // Use the resource
/// let result = db.with(|conn| {
///     pure(conn.query("SELECT 1"))
/// }).run(&()).await;
/// ```
pub struct Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    #[allow(clippy::type_complexity)]
    acquire: Box<dyn FnOnce(&Env) -> BoxFuture<'static, Result<T, E>> + Send>,
    release: Box<dyn FnOnce(T) -> BoxFuture<'static, Result<(), E>> + Send>,
}

impl<T, E, Env> std::fmt::Debug for Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("acquire", &"<effect>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<T, E, Env> Resource<T, E, Env>
where
    T: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Create a new resource with acquire effect and release function.
    pub fn new<Acq, Rel, RelFut>(acquire: Acq, release: Rel) -> Self
    where
        Acq: Effect<Output = T, Error = E, Env = Env> + 'static,
        Rel: FnOnce(T) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Resource {
            acquire: Box::new(move |env: &Env| {
                let env = env.clone();
                Box::pin(async move { acquire.run(&env).await })
            }),
            release: Box::new(move |t| Box::pin(release(t))),
        }
    }

    /// Use this resource with a function, guaranteeing cleanup.
    ///
    /// This is equivalent to `bracket` but with the acquire/release
    /// already encapsulated in the Resource.
    pub fn with<U, F, UseEffect>(self, f: F) -> ResourceWith<T, U, E, Env, F>
    where
        U: Send + 'static,
        F: FnOnce(&T) -> UseEffect + Send + 'static,
        UseEffect: Effect<Output = U, Error = E, Env = Env>,
    {
        ResourceWith {
            resource: self,
            use_fn: f,
            _marker: PhantomData,
        }
    }

    /// Combine two resources into one.
    ///
    /// The combined resource acquires both resources and releases them
    /// in reverse order (LIFO). If the second acquisition fails, the
    /// first resource is released and the error is logged.
    pub fn both<T2>(self, other: Resource<T2, E, Env>) -> Resource<(T, T2), E, Env>
    where
        T2: Send + 'static,
    {
        use std::sync::Arc;

        let acquire1 = self.acquire;
        let release1 = Arc::new(std::sync::Mutex::new(Some(self.release)));
        let acquire2 = other.acquire;
        let release2 = Arc::new(std::sync::Mutex::new(Some(other.release)));

        let release1_for_acquire = release1.clone();
        let release1_for_release = release1;
        let release2_for_release = release2;

        Resource {
            acquire: Box::new(move |env: &Env| {
                let env = env.clone();
                Box::pin(async move {
                    let t1 = acquire1(&env).await?;
                    match acquire2(&env).await {
                        Ok(t2) => Ok((t1, t2)),
                        Err(acquire_err) => {
                            // Release t1 if t2 acquisition fails
                            let release1 = release1_for_acquire
                                .lock()
                                .unwrap()
                                .take()
                                .expect("release1 already taken");
                            if let Err(cleanup_err) = release1(t1).await {
                                #[cfg(feature = "tracing")]
                                tracing::warn!(
                                    "Cleanup failed during partial acquisition rollback: {:?}",
                                    cleanup_err
                                );
                                #[cfg(not(feature = "tracing"))]
                                eprintln!(
                                    "Cleanup failed during partial acquisition rollback: {:?}",
                                    cleanup_err
                                );
                            }
                            Err(acquire_err)
                        }
                    }
                })
            }),
            release: Box::new(move |(t1, t2): (T, T2)| {
                Box::pin(async move {
                    // Release in reverse order
                    let release2 = release2_for_release
                        .lock()
                        .unwrap()
                        .take()
                        .expect("release2 already taken");
                    let release1 = release1_for_release
                        .lock()
                        .unwrap()
                        .take()
                        .expect("release1 already taken");
                    let r2 = release2(t2).await;
                    let r1 = release1(t1).await;
                    // Return first error if any
                    r2?;
                    r1?;
                    Ok(())
                })
            }),
        }
    }
}

/// Effect type for Resource::with
pub struct ResourceWith<T, U, E, Env, F>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    resource: Resource<T, E, Env>,
    use_fn: F,
    _marker: PhantomData<U>,
}

impl<T, U, E, Env, F> std::fmt::Debug for ResourceWith<T, U, E, Env, F>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceWith")
            .field("resource", &"<resource>")
            .field("use_fn", &"<function>")
            .finish()
    }
}

impl<T, U, E, Env, F, UseEffect> Effect for ResourceWith<T, U, E, Env, F>
where
    T: Send + 'static,
    U: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
    F: FnOnce(&T) -> UseEffect + Send + 'static,
    UseEffect: Effect<Output = U, Error = E, Env = Env>,
{
    type Output = U;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<U, E> {
        // Acquire
        let resource = (self.resource.acquire)(env).await?;

        // Use
        let result = (self.use_fn)(&resource).run(env).await;

        // Release
        let release_result = (self.resource.release)(resource).await;
        if let Err(ref rel_err) = release_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource cleanup failed: {:?}", rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource cleanup failed: {:?}", rel_err);
        }

        result
    }
}

// ============================================================================
// Acquiring - fluent builder pattern
// ============================================================================

/// Builder for acquiring multiple resources with guaranteed cleanup.
///
/// This provides a fluent API that avoids deeply nested brackets while
/// generating the same efficient code structure internally.
///
/// # Tuple Nesting
///
/// When chaining multiple `.and()` calls, resources are nested as left-associated tuples:
///
/// | Resources | Type | Destructure |
/// |-----------|------|-------------|
/// | 1 | `T` | `\|a\|` |
/// | 2 | `(T1, T2)` | `\|(a, b)\|` |
/// | 3 | `((T1, T2), T3)` | `\|((a, b), c)\|` |
/// | 4 | `(((T1, T2), T3), T4)` | `\|(((a, b), c), d)\|` |
///
/// Use `with_flat2`, `with_flat3`, `with_flat4` for ergonomic flat parameter access.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let result = acquiring(
///     open_conn(),
///     |c| async move { c.close().await },
/// )
/// .and(acquire_lock(), |l| async move { l.release().await })
/// .and(open_file(), |f| async move { f.close().await })
/// .with(|((conn, lock), file)| process(conn, lock, file))
/// .run(&env)
/// .await;
/// ```
pub struct Acquiring<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    resource: Resource<T, E, Env>,
}

impl<T, E, Env> std::fmt::Debug for Acquiring<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acquiring")
            .field("resource", &"<resource>")
            .finish()
    }
}

/// Start building a resource acquisition chain.
///
/// This is the entry point for the fluent builder API. Chain multiple
/// resources with `.and()` and finalize with `.with()` or `.with_flatN()`.
///
/// # Example
///
/// ```rust,ignore
/// acquiring(open_database(), |db| async move { db.close().await })
///     .and(open_file(), |f| async move { f.close().await })
///     .with(|(db, file)| do_work(db, file))
/// ```
pub fn acquiring<R, E, Env, Acq, Rel, RelFut>(acquire: Acq, release: Rel) -> Acquiring<R, E, Env>
where
    R: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
    Acq: Effect<Output = R, Error = E, Env = Env> + 'static,
    Rel: FnOnce(R) -> RelFut + Send + 'static,
    RelFut: Future<Output = Result<(), E>> + Send + 'static,
{
    Acquiring {
        resource: Resource::new(acquire, release),
    }
}

impl<T, E, Env> Acquiring<T, E, Env>
where
    T: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Add another resource to the acquisition chain.
    ///
    /// Resources are acquired in order and released in reverse order (LIFO).
    pub fn and<T2, Acq, Rel, RelFut>(self, acquire: Acq, release: Rel) -> Acquiring<(T, T2), E, Env>
    where
        T2: Send + 'static,
        Acq: Effect<Output = T2, Error = E, Env = Env> + 'static,
        Rel: FnOnce(T2) -> RelFut + Send + 'static,
        RelFut: Future<Output = Result<(), E>> + Send + 'static,
    {
        Acquiring {
            resource: self.resource.both(Resource::new(acquire, release)),
        }
    }

    /// Use the acquired resources with a function, guaranteeing cleanup.
    ///
    /// This finalizes the builder and returns an Effect that will:
    /// 1. Acquire all resources in order
    /// 2. Run the provided function with references to all resources
    /// 3. Release all resources in reverse order (even on error)
    pub fn with<U, F, UseEffect>(self, f: F) -> ResourceWith<T, U, E, Env, F>
    where
        U: Send + 'static,
        F: FnOnce(&T) -> UseEffect + Send + 'static,
        UseEffect: Effect<Output = U, Error = E, Env = Env>,
    {
        self.resource.with(f)
    }
}

// Implement with_flat variants for 2, 3, and 4 resources
// Note: These use distinct method names (with_flat2, with_flat3, with_flat4) to avoid
// overlapping impl conflicts. E.g., (A, B) could match (A, (X, Y)) when B = (X, Y).

impl<A, B, E, Env> Acquiring<(A, B), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Use with flattened parameter access for two resources.
    ///
    /// Instead of `|(a, b)|`, you can use `|a, b|`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// acquiring(resource1, release1)
    ///     .and(resource2, release2)
    ///     .with_flat2(|a, b| use_resources(a, b))
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn with_flat2<U, F, UseEffect>(
        self,
        f: F,
    ) -> ResourceWith<(A, B), U, E, Env, impl FnOnce(&(A, B)) -> UseEffect + Send + 'static>
    where
        U: Send + 'static,
        F: FnOnce(&A, &B) -> UseEffect + Send + 'static,
        UseEffect: Effect<Output = U, Error = E, Env = Env>,
    {
        self.resource.with(move |(a, b)| f(a, b))
    }
}

impl<A, B, C, E, Env> Acquiring<((A, B), C), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    C: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Use with flattened parameter access for three resources.
    ///
    /// Instead of `|((a, b), c)|`, you can use `|a, b, c|`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// acquiring(resource1, release1)
    ///     .and(resource2, release2)
    ///     .and(resource3, release3)
    ///     .with_flat3(|a, b, c| use_resources(a, b, c))
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn with_flat3<U, F, UseEffect>(
        self,
        f: F,
    ) -> ResourceWith<((A, B), C), U, E, Env, impl FnOnce(&((A, B), C)) -> UseEffect + Send + 'static>
    where
        U: Send + 'static,
        F: FnOnce(&A, &B, &C) -> UseEffect + Send + 'static,
        UseEffect: Effect<Output = U, Error = E, Env = Env>,
    {
        self.resource.with(move |((a, b), c)| f(a, b, c))
    }
}

impl<A, B, C, D, E, Env> Acquiring<(((A, B), C), D), E, Env>
where
    A: Send + 'static,
    B: Send + 'static,
    C: Send + 'static,
    D: Send + 'static,
    E: Send + std::fmt::Debug + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Use with flattened parameter access for four resources.
    ///
    /// Instead of `|(((a, b), c), d)|`, you can use `|a, b, c, d|`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// acquiring(resource1, release1)
    ///     .and(resource2, release2)
    ///     .and(resource3, release3)
    ///     .and(resource4, release4)
    ///     .with_flat4(|a, b, c, d| use_resources(a, b, c, d))
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn with_flat4<U, F, UseEffect>(
        self,
        f: F,
    ) -> ResourceWith<
        (((A, B), C), D),
        U,
        E,
        Env,
        impl FnOnce(&(((A, B), C), D)) -> UseEffect + Send + 'static,
    >
    where
        U: Send + 'static,
        F: FnOnce(&A, &B, &C, &D) -> UseEffect + Send + 'static,
        UseEffect: Effect<Output = U, Error = E, Env = Env>,
    {
        self.resource.with(move |(((a, b), c), d)| f(a, b, c, d))
    }
}

// ============================================================================
// Legacy bracket_simple (kept for backwards compatibility)
// ============================================================================

/// Simplified bracket that uses a closure for release.
///
/// This variant is for cases where release doesn't need to be async or return a Result.
#[deprecated(
    since = "0.12.0",
    note = "Use `bracket` with async release function instead"
)]
pub fn bracket_simple<Acquire, Use, ReleaseFn, UseEffect, R, T, E, Env>(
    acquire: Acquire,
    use_fn: Use,
    release_fn: ReleaseFn,
) -> impl Effect<Output = T, Error = E, Env = Env>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    ReleaseFn: FnOnce(R) + Send,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    BracketSimple {
        acquire,
        use_fn,
        release_fn,
    }
}

/// Simple bracket that uses a closure for release instead of an effect.
struct BracketSimple<Acquire, Use, ReleaseFn> {
    acquire: Acquire,
    use_fn: Use,
    release_fn: ReleaseFn,
}

impl<Acquire, Use, ReleaseFn, UseEffect, R, T, E, Env> Effect
    for BracketSimple<Acquire, Use, ReleaseFn>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    ReleaseFn: FnOnce(R) + Send,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        let resource = self.acquire.run(env).await?;
        let result = (self.use_fn)(resource.clone()).run(env).await;
        (self.release_fn)(resource);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, pure};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn bracket_returns_error_on_acquire_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket(
            fail::<i32, String, ()>("acquire failed".to_string()),
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |val: &i32| pure::<_, String, ()>(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("acquire failed".to_string()));
        assert!(
            !released.load(Ordering::SeqCst),
            "cleanup must NOT run when acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_releases_on_success() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket(
            pure::<_, String, ()>(42),
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |val: &i32| pure::<_, String, ()>(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bracket_releases_on_use_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket(
            pure::<_, String, ()>(42),
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("use failed".to_string()));
        assert!(
            released.load(Ordering::SeqCst),
            "cleanup must run on failure"
        );
    }

    #[tokio::test]
    async fn bracket_logs_cleanup_error_returns_use_result() {
        // Cleanup fails, but use succeeds - should return use result
        let result = bracket(
            pure::<_, String, ()>(42),
            |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
            |val: &i32| pure::<_, String, ()>(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(
            result,
            Ok(84),
            "use result returned despite cleanup failure"
        );
    }

    #[tokio::test]
    async fn bracket2_releases_in_lifo_order() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();

        let result = bracket2(
            pure::<_, String, ()>("first"),
            pure::<_, String, ()>("second"),
            move |_: &str| {
                order1.lock().unwrap().push("release_first");
                async { Ok(()) }
            },
            move |_: &str| {
                order2.lock().unwrap().push("release_second");
                async { Ok(()) }
            },
            |_: &&str, _: &&str| pure::<_, String, ()>("done"),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok("done"));
        let releases = order.lock().unwrap();
        assert_eq!(*releases, vec!["release_second", "release_first"]);
    }

    #[tokio::test]
    async fn bracket2_releases_first_if_second_acquire_fails() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket2(
            pure::<_, String, ()>("first"),
            fail::<&str, String, ()>("acquire2 failed".to_string()),
            move |_: &str| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |_: &str| async { Ok(()) },
            |_: &&str, _: &&str| pure::<_, String, ()>("done"),
        )
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "first resource must be released when second acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_full_returns_both_errors() {
        let result = bracket_full(
            pure::<_, String, ()>(42),
            |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
            |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::Both {
                use_error,
                cleanup_error,
            } => {
                assert_eq!(use_error, "use failed");
                assert_eq!(cleanup_error, "cleanup failed");
            }
            _ => panic!("expected BracketError::Both"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_use_error_only() {
        let result = bracket_full(
            pure::<_, String, ()>(42),
            |_: i32| async { Ok(()) },
            |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::UseError(e) => assert_eq!(e, "use failed"),
            _ => panic!("expected BracketError::UseError"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_cleanup_error_only() {
        let result = bracket_full(
            pure::<_, String, ()>(42),
            |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
            |_: &i32| pure::<i32, String, ()>(84),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::CleanupError(e) => assert_eq!(e, "cleanup failed"),
            _ => panic!("expected BracketError::CleanupError"),
        }
    }

    #[tokio::test]
    async fn bracket_full_returns_acquire_error() {
        let result = bracket_full(
            fail::<i32, String, ()>("acquire failed".to_string()),
            |_: i32| async { Ok(()) },
            |_: &i32| pure::<i32, String, ()>(42),
        )
        .run(&())
        .await;

        let err = result.unwrap_err();
        match err {
            BracketError::AcquireError(e) => assert_eq!(e, "acquire failed"),
            _ => panic!("expected BracketError::AcquireError"),
        }
    }

    #[tokio::test]
    async fn resource_use_guarantees_cleanup() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let resource = Resource::new(pure::<_, String, ()>(42), move |_: i32| {
            released_clone.store(true, Ordering::SeqCst);
            async { Ok(()) }
        });

        let result = resource
            .with(|val: &i32| pure::<_, String, ()>(*val * 2))
            .run(&())
            .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn acquiring_builder_single_resource() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = acquiring(pure::<_, String, ()>(42), move |_: i32| {
            released_clone.store(true, Ordering::SeqCst);
            async { Ok(()) }
        })
        .with(|val: &i32| pure::<_, String, ()>(*val * 2))
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn acquiring_builder_multiple_resources() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();
        let order3 = order.clone();

        let result = acquiring(pure::<_, String, ()>("first"), move |_: &str| {
            order1.lock().unwrap().push("release_first");
            async { Ok(()) }
        })
        .and(pure::<_, String, ()>("second"), move |_: &str| {
            order2.lock().unwrap().push("release_second");
            async { Ok(()) }
        })
        .and(pure::<_, String, ()>("third"), move |_: &str| {
            order3.lock().unwrap().push("release_third");
            async { Ok(()) }
        })
        .with(|((first, second), third): &((&str, &str), &str)| {
            // Verify we have all resources
            assert_eq!(*first, "first");
            assert_eq!(*second, "second");
            assert_eq!(*third, "third");
            pure::<_, String, ()>("done")
        })
        .run(&())
        .await;

        assert_eq!(result, Ok("done"));

        // Verify LIFO cleanup order
        let releases = order.lock().unwrap();
        assert_eq!(
            *releases,
            vec!["release_third", "release_second", "release_first"]
        );
    }

    #[tokio::test]
    async fn acquiring_builder_with_flat2_two_resources() {
        let result = acquiring(pure::<_, String, ()>(10), |_: i32| async { Ok(()) })
            .and(pure::<_, String, ()>(20), |_: i32| async { Ok(()) })
            .with_flat2(|a: &i32, b: &i32| pure::<_, String, ()>(*a + *b))
            .run(&())
            .await;

        assert_eq!(result, Ok(30));
    }

    #[tokio::test]
    async fn acquiring_builder_with_flat3_three_resources() {
        let result = acquiring(pure::<_, String, ()>(1), |_: i32| async { Ok(()) })
            .and(pure::<_, String, ()>(2), |_: i32| async { Ok(()) })
            .and(pure::<_, String, ()>(3), |_: i32| async { Ok(()) })
            .with_flat3(|a: &i32, b: &i32, c: &i32| pure::<_, String, ()>(a + b + c))
            .run(&())
            .await;

        assert_eq!(result, Ok(6));
    }

    #[tokio::test]
    async fn acquiring_builder_releases_on_partial_acquire_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = acquiring(pure::<_, String, ()>("first"), move |_: &str| {
            released_clone.store(true, Ordering::SeqCst);
            async { Ok(()) }
        })
        .and(
            fail::<&str, String, ()>("second acquire failed".to_string()),
            |_: &str| async { Ok(()) },
        )
        .with(|(first, second): &(&str, &str)| {
            pure::<_, String, ()>(format!("{} {}", first, second))
        })
        .run(&())
        .await;

        assert!(result.is_err());
        assert!(
            released.load(Ordering::SeqCst),
            "first resource must be released when second acquire fails"
        );
    }

    #[tokio::test]
    async fn bracket_sync_releases_on_success() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket_sync(
            pure::<_, String, ()>(42),
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                Ok(())
            },
            |val: &i32| pure::<_, String, ()>(*val * 2),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn bracket_sync_releases_on_use_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket_sync(
            pure::<_, String, ()>(42),
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                Ok(())
            },
            |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
        )
        .run(&())
        .await;

        assert_eq!(result, Err("use failed".to_string()));
        assert!(
            released.load(Ordering::SeqCst),
            "cleanup must run on failure"
        );
    }

    #[tokio::test]
    async fn bracket3_releases_in_lifo_order() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();
        let order3 = order.clone();

        let result = bracket3(
            pure::<_, String, ()>("first"),
            pure::<_, String, ()>("second"),
            pure::<_, String, ()>("third"),
            move |_: &str| {
                order1.lock().unwrap().push("release_first");
                async { Ok(()) }
            },
            move |_: &str| {
                order2.lock().unwrap().push("release_second");
                async { Ok(()) }
            },
            move |_: &str| {
                order3.lock().unwrap().push("release_third");
                async { Ok(()) }
            },
            |_: &&str, _: &&str, _: &&str| pure::<_, String, ()>("done"),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok("done"));
        let releases = order.lock().unwrap();
        assert_eq!(
            *releases,
            vec!["release_third", "release_second", "release_first"]
        );
    }

    #[tokio::test]
    async fn bracket_error_display() {
        let acquire_err: BracketError<&str> = BracketError::AcquireError("failed");
        assert_eq!(format!("{}", acquire_err), "acquire failed: failed");

        let use_err: BracketError<&str> = BracketError::UseError("failed");
        assert_eq!(format!("{}", use_err), "failed");

        let cleanup_err: BracketError<&str> = BracketError::CleanupError("failed");
        assert_eq!(format!("{}", cleanup_err), "cleanup failed: failed");

        let both_err: BracketError<&str> = BracketError::Both {
            use_error: "use failed",
            cleanup_error: "cleanup failed",
        };
        assert_eq!(
            format!("{}", both_err),
            "use failed: use failed; cleanup also failed: cleanup failed"
        );
    }

    #[tokio::test]
    async fn bracket_error_accessors() {
        let acquire_err: BracketError<&str> = BracketError::AcquireError("failed");
        assert_eq!(acquire_err.acquire_error(), Some(&"failed"));
        assert_eq!(acquire_err.use_error(), None);
        assert_eq!(acquire_err.cleanup_error(), None);

        let use_err: BracketError<&str> = BracketError::UseError("failed");
        assert_eq!(use_err.acquire_error(), None);
        assert_eq!(use_err.use_error(), Some(&"failed"));
        assert_eq!(use_err.cleanup_error(), None);

        let cleanup_err: BracketError<&str> = BracketError::CleanupError("failed");
        assert_eq!(cleanup_err.acquire_error(), None);
        assert_eq!(cleanup_err.use_error(), None);
        assert_eq!(cleanup_err.cleanup_error(), Some(&"failed"));

        let both_err: BracketError<&str> = BracketError::Both {
            use_error: "use",
            cleanup_error: "cleanup",
        };
        assert_eq!(both_err.acquire_error(), None);
        assert_eq!(both_err.use_error(), Some(&"use"));
        assert_eq!(both_err.cleanup_error(), Some(&"cleanup"));
    }

    #[tokio::test]
    async fn bracket_error_map() {
        let err: BracketError<i32> = BracketError::UseError(42);
        let mapped = err.map(|x| x.to_string());
        assert_eq!(mapped, BracketError::UseError("42".to_string()));
    }

    #[tokio::test]
    async fn resource_both_combines_correctly() {
        let order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let order1 = order.clone();
        let order2 = order.clone();

        let r1 = Resource::new(pure::<_, String, ()>(1), move |_: i32| {
            order1.lock().unwrap().push("release_1");
            async { Ok(()) }
        });
        let r2 = Resource::new(pure::<_, String, ()>(2), move |_: i32| {
            order2.lock().unwrap().push("release_2");
            async { Ok(()) }
        });

        let result = r1
            .both(r2)
            .with(|(a, b): &(i32, i32)| pure::<_, String, ()>(*a + *b))
            .run(&())
            .await;

        assert_eq!(result, Ok(3));

        // Verify LIFO cleanup
        let releases = order.lock().unwrap();
        assert_eq!(*releases, vec!["release_2", "release_1"]);
    }
}
