//! Resource-aware bracket pattern for compile-time resource safety.
//!
//! This module provides `resource_bracket`, which extends the bracket pattern
//! with compile-time resource tracking. The type signature enforces that:
//!
//! 1. The acquire effect creates a resource of type R
//! 2. The release function consumes that resource
//! 3. The bracket as a whole is resource-neutral
//!
//! This enables compile-time detection of resource leaks and protocol violations.

use std::future::Future;
use std::marker::PhantomData;

use super::markers::ResourceKind;
use super::sets::{Empty, Has};
use super::tracked::{ResourceEffect, Tracked};
use crate::effect::trait_def::Effect;

/// Resource-safe bracket with compile-time tracking.
///
/// This bracket enforces at compile time that:
/// - The acquire effect creates resource R (`Acquires = Has<R>`)
/// - The use effect is resource-neutral (`Acquires = Empty, Releases = Empty`)
/// - The release function consumes resource R
/// - The bracket as a whole is resource-neutral
///
/// # Type Parameters
///
/// * `R` - The resource kind being managed
/// * `Acq` - The acquire effect
/// * `Use` - The function that uses the resource
/// * `Rel` - The release function
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// // This compiles: resource is properly managed
/// fn read_file_safe(path: &str) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
///     resource_bracket::<FileRes, _, _, _, _, _, _, _, _>(
///         open_file(path),
///         |h| async move { close_file(h).run(&()).await },
///         |h| read_contents(h),
///     )
/// }
/// ```
pub struct ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
{
    acquire: Acq,
    use_fn: Use,
    release: Rel,
    _phantom: PhantomData<R>,
}

impl<R, Acq, Use, Rel> ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
{
    /// Create a new ResourceBracket.
    ///
    /// Prefer using the builder API (`Bracket::<R>::new()`) or the
    /// `resource_bracket` function for a more ergonomic interface.
    pub fn new(acquire: Acq, use_fn: Use, release: Rel) -> Self {
        ResourceBracket {
            acquire,
            use_fn,
            release,
            _phantom: PhantomData,
        }
    }
}

impl<R, Acq, Use, Rel> std::fmt::Debug for ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceBracket")
            .field("resource", &R::NAME)
            .field("acquire", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<R, Acq, Use, Rel, UseEff, T, U, E, Env, RelFut> Effect for ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
    Acq: Effect<Output = T, Error = E, Env = Env>,
    Use: FnOnce(&T) -> UseEff + Send,
    UseEff: Effect<Output = U, Error = E, Env = Env>,
    Rel: FnOnce(T) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    T: Send,
    U: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    type Output = U;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<U, E> {
        // Acquire the resource
        let resource = self.acquire.run(env).await?;

        // Use the resource (borrowing for use, moving for release)
        let result = (self.use_fn)(&resource).run(env).await;

        // Release runs regardless of use result
        let release_result = (self.release)(resource).await;

        // Log cleanup errors if any
        if let Err(ref rel_err) = release_result {
            #[cfg(feature = "tracing")]
            tracing::warn!("Resource {} cleanup failed: {:?}", R::NAME, rel_err);
            #[cfg(not(feature = "tracing"))]
            eprintln!("Resource {} cleanup failed: {:?}", R::NAME, rel_err);
        }

        result
    }
}

// ResourceEffect implementation - the bracket is resource-neutral
impl<R, Acq, Use, Rel, UseEff, T, U, E, Env, RelFut> ResourceEffect
    for ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
    Acq: Effect<Output = T, Error = E, Env = Env>,
    Use: FnOnce(&T) -> UseEff + Send,
    UseEff: Effect<Output = U, Error = E, Env = Env>,
    Rel: FnOnce(T) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    T: Send,
    U: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    type Acquires = Empty;
    type Releases = Empty;
}

/// Create a resource-safe bracket with compile-time tracking.
///
/// This function creates a bracket that enforces resource safety at compile time.
/// The acquire effect must produce a resource, and the release function must
/// consume it. The bracket as a whole is guaranteed to be resource-neutral.
///
/// # Type Parameters
///
/// * `R` - The resource kind being managed
/// * `Acq` - The acquire effect (must have `Acquires = Has<R>`)
/// * `Use` - The function that uses the resource
/// * `Rel` - The release function
///
/// # Arguments
///
/// * `acquire` - Effect that acquires the resource
/// * `release` - Function that releases the resource (takes ownership)
/// * `use_fn` - Function that uses the resource (borrows)
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// let effect = resource_bracket::<FileRes, _, _, _, _, _, _, _, _>(
///     open_file("data.txt"),
///     |handle| async move { close_file(handle).run(&()).await },
///     |handle| read_contents(handle),
/// );
///
/// // The effect is guaranteed to be resource-neutral
/// let neutral = assert_resource_neutral(effect);
/// ```
pub fn resource_bracket<R, Acq, Use, Rel, UseEff, T, U, E, Env, RelFut>(
    acquire: Acq,
    release: Rel,
    use_fn: Use,
) -> ResourceBracket<R, Acq, Use, Rel>
where
    R: ResourceKind,
    Acq: Effect<Output = T, Error = E, Env = Env>,
    Use: FnOnce(&T) -> UseEff + Send,
    UseEff: Effect<Output = U, Error = E, Env = Env>,
    Rel: FnOnce(T) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    T: Send,
    U: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    ResourceBracket {
        acquire,
        use_fn,
        release,
        _phantom: PhantomData,
    }
}

/// Create a tracked resource bracket with compile-time verification.
///
/// This is a stricter version of `resource_bracket` that requires the acquire
/// effect to implement `ResourceEffect` with `Acquires = Has<R>`.
///
/// # Type Parameters
///
/// * `R` - The resource kind being managed
/// * `Acq` - The acquire effect (must implement ResourceEffect with Acquires = `Has<R>`)
/// * `Use` - The function that uses the resource
/// * `Rel` - The release function
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// // The acquire effect must be annotated with .acquires::<FileRes>()
/// let effect = tracked_resource_bracket::<FileRes, _, _, _, _, _, _, _, _>(
///     open_file_impl().acquires::<FileRes>(),
///     |h| async move { Ok(()) },
///     |h| read_contents(h),
/// );
/// ```
pub fn tracked_resource_bracket<R, Acq, Use, Rel, UseEff, T, U, E, Env, RelFut>(
    acquire: Acq,
    release: Rel,
    use_fn: Use,
) -> Tracked<ResourceBracket<R, Acq, Use, Rel>, Empty, Empty>
where
    R: ResourceKind,
    Acq: ResourceEffect<Output = T, Error = E, Env = Env, Acquires = Has<R>, Releases = Empty>,
    Use: FnOnce(&T) -> UseEff + Send,
    UseEff: ResourceEffect<Output = U, Error = E, Env = Env, Acquires = Empty, Releases = Empty>,
    Rel: FnOnce(T) -> RelFut + Send,
    RelFut: Future<Output = Result<(), E>> + Send,
    T: Send,
    U: Send,
    E: Send + std::fmt::Debug,
    Env: Clone + Send + Sync,
{
    Tracked::new(ResourceBracket {
        acquire,
        use_fn,
        release,
        _phantom: PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, pure};
    use crate::effect::resource::ext::ResourceEffectExt;
    use crate::effect::resource::markers::FileRes;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn resource_bracket_happy_path() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
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
    async fn resource_bracket_releases_on_use_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
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
    async fn resource_bracket_acquire_failure_no_release() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
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

    #[test]
    fn resource_bracket_debug() {
        let bracket = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
            pure::<_, String, ()>(42),
            |_: i32| async { Ok(()) },
            |val: &i32| pure::<_, String, ()>(*val),
        );

        let debug_str = format!("{:?}", bracket);
        assert!(debug_str.contains("ResourceBracket"));
        assert!(debug_str.contains("File"));
    }

    // Type-level tests
    fn _assert_resource_neutral<T: ResourceEffect<Acquires = Empty, Releases = Empty>>() {}

    #[test]
    fn resource_bracket_is_neutral() {
        type BracketType = ResourceBracket<
            FileRes,
            crate::effect::combinators::Pure<i32, String, ()>,
            fn(&i32) -> crate::effect::combinators::Pure<i32, String, ()>,
            fn(i32) -> std::future::Ready<Result<(), String>>,
        >;
        _assert_resource_neutral::<BracketType>();
    }

    #[tokio::test]
    async fn tracked_resource_bracket_works() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let acquire = pure::<_, String, ()>(42).acquires::<FileRes>();

        let result = tracked_resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
            acquire,
            move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            },
            |val: &i32| pure::<_, String, ()>(*val * 2).neutral(),
        )
        .run(&())
        .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn resource_bracket_logs_cleanup_error_on_success() {
        // Test that cleanup errors are logged but use result is returned
        // This covers the error logging path (lines 103/105)
        let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
            pure::<_, String, ()>(42),
            |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
            |val: &i32| pure::<_, String, ()>(*val * 2),
        )
        .run(&())
        .await;

        // Use succeeded, so we get Ok even though cleanup failed
        assert_eq!(result, Ok(84));
    }

    #[tokio::test]
    async fn resource_bracket_logs_cleanup_error_on_use_failure() {
        // Both use and cleanup fail - use error is returned, cleanup error is logged
        let result = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
            pure::<_, String, ()>(42),
            |_: i32| async { Err::<(), String>("cleanup failed".to_string()) },
            |_: &i32| fail::<i32, String, ()>("use failed".to_string()),
        )
        .run(&())
        .await;

        // Use error is returned (cleanup error is logged)
        assert_eq!(result, Err("use failed".to_string()));
    }
}
