//! Builder pattern for resource brackets.
//!
//! Provides a fluent API that avoids the turbofish with many underscores:
//!
//! ```rust,ignore
//! // Before: 10 type parameters, 9 underscores
//! resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
//!     open_file(path),
//!     |h| async move { close_file(h).run(&()).await },
//!     |h| read_contents(h),
//! )
//!
//! // After: single type parameter, fluent chain
//! Bracket::<FileRes>::new()
//!     .acquire(open_file(path))
//!     .release(|h| async move { close_file(h).run(&()).await })
//!     .use_fn(|h| read_contents(h))
//! ```

use std::marker::PhantomData;

use super::bracket::ResourceBracket;
use super::markers::ResourceKind;

/// Builder for resource brackets with a fluent API.
///
/// This avoids the turbofish with 9 underscores by capturing the resource
/// type once, then inferring everything else from method arguments.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// fn read_file_safe(path: &str) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
///     Bracket::<FileRes>::new()
///         .acquire(open_file(path))
///         .release(|h| async move { close_file(h).run(&()).await })
///         .use_fn(|h| read_contents(h))
/// }
/// ```
pub struct Bracket<R: ResourceKind> {
    _phantom: PhantomData<R>,
}

impl<R: ResourceKind> Bracket<R> {
    /// Start building a resource bracket for resource type R.
    ///
    /// This is the only place you need to specify the resource type.
    pub fn new() -> Self {
        Bracket {
            _phantom: PhantomData,
        }
    }

    /// Specify the acquire effect.
    ///
    /// The effect's output type becomes the resource handle passed to
    /// `release` and `use_fn`.
    pub fn acquire<Acq>(self, acquire: Acq) -> BracketWithAcquire<R, Acq> {
        BracketWithAcquire {
            acquire,
            _phantom: PhantomData,
        }
    }
}

impl<R: ResourceKind> Default for Bracket<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: ResourceKind> std::fmt::Debug for Bracket<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bracket")
            .field("resource", &R::NAME)
            .finish()
    }
}

/// Builder state after acquire is specified.
pub struct BracketWithAcquire<R: ResourceKind, Acq> {
    acquire: Acq,
    _phantom: PhantomData<R>,
}

impl<R: ResourceKind, Acq> BracketWithAcquire<R, Acq> {
    /// Specify the release function.
    ///
    /// This function receives ownership of the resource handle and must
    /// return a future that produces `Result<(), E>`.
    pub fn release<Rel>(self, release: Rel) -> BracketWithRelease<R, Acq, Rel> {
        BracketWithRelease {
            acquire: self.acquire,
            release,
            _phantom: PhantomData,
        }
    }
}

impl<R: ResourceKind, Acq> std::fmt::Debug for BracketWithAcquire<R, Acq> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BracketWithAcquire")
            .field("resource", &R::NAME)
            .field("acquire", &"<effect>")
            .finish()
    }
}

/// Builder state after acquire and release are specified.
pub struct BracketWithRelease<R: ResourceKind, Acq, Rel> {
    acquire: Acq,
    release: Rel,
    _phantom: PhantomData<R>,
}

impl<R: ResourceKind, Acq, Rel> BracketWithRelease<R, Acq, Rel> {
    /// Specify the use function and build the final bracket.
    ///
    /// This function receives a reference to the resource handle and
    /// returns an effect. The bracket guarantees the release function
    /// runs even if the use function fails.
    pub fn use_fn<Use>(self, use_fn: Use) -> ResourceBracket<R, Acq, Use, Rel> {
        ResourceBracket::new(self.acquire, use_fn, self.release)
    }
}

impl<R: ResourceKind, Acq, Rel> std::fmt::Debug for BracketWithRelease<R, Acq, Rel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BracketWithRelease")
            .field("resource", &R::NAME)
            .field("acquire", &"<effect>")
            .field("release", &"<function>")
            .finish()
    }
}

/// Convenience function to start building a bracket.
///
/// Equivalent to `Bracket::<R>::new()`.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// fn read_file(path: &str) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
///     bracket::<FileRes>()
///         .acquire(open_file(path))
///         .release(|h| async move { close_file(h).run(&()).await })
///         .use_fn(|h| read_contents(h))
/// }
/// ```
pub fn bracket<R: ResourceKind>() -> Bracket<R> {
    Bracket::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, pure};
    use crate::effect::resource::markers::FileRes;
    use crate::effect::trait_def::Effect;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn builder_happy_path() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Bracket::<FileRes>::new()
            .acquire(pure::<_, String, ()>(42))
            .release(move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            })
            .use_fn(|val: &i32| pure::<_, String, ()>(*val * 2))
            .run(&())
            .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn builder_with_function() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = bracket::<FileRes>()
            .acquire(pure::<_, String, ()>(42))
            .release(move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            })
            .use_fn(|val: &i32| pure::<_, String, ()>(*val * 2))
            .run(&())
            .await;

        assert_eq!(result, Ok(84));
        assert!(released.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn builder_releases_on_use_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Bracket::<FileRes>::new()
            .acquire(pure::<_, String, ()>(42))
            .release(move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            })
            .use_fn(|_: &i32| fail::<i32, String, ()>("use failed".to_string()))
            .run(&())
            .await;

        assert_eq!(result, Err("use failed".to_string()));
        assert!(
            released.load(Ordering::SeqCst),
            "cleanup must run on failure"
        );
    }

    #[tokio::test]
    async fn builder_no_release_on_acquire_failure() {
        let released = Arc::new(AtomicBool::new(false));
        let released_clone = released.clone();

        let result = Bracket::<FileRes>::new()
            .acquire(fail::<i32, String, ()>("acquire failed".to_string()))
            .release(move |_: i32| {
                released_clone.store(true, Ordering::SeqCst);
                async { Ok(()) }
            })
            .use_fn(|val: &i32| pure::<_, String, ()>(*val * 2))
            .run(&())
            .await;

        assert_eq!(result, Err("acquire failed".to_string()));
        assert!(
            !released.load(Ordering::SeqCst),
            "cleanup must NOT run when acquire fails"
        );
    }

    #[test]
    fn builder_debug_impls() {
        let b1 = Bracket::<FileRes>::new();
        assert!(format!("{:?}", b1).contains("Bracket"));

        let b2 = b1.acquire(pure::<_, String, ()>(42));
        assert!(format!("{:?}", b2).contains("BracketWithAcquire"));

        let b3 = b2.release(|_: i32| async { Ok::<(), String>(()) });
        assert!(format!("{:?}", b3).contains("BracketWithRelease"));
    }

    #[test]
    fn builder_default() {
        let b: Bracket<FileRes> = Default::default();
        assert!(format!("{:?}", b).contains("File"));
    }

    // Type-level test: verify the result is resource-neutral
    use super::super::sets::Empty;
    use super::super::tracked::ResourceEffect;
    fn _assert_neutral<T: ResourceEffect<Acquires = Empty, Releases = Empty>>() {}

    #[test]
    fn builder_produces_neutral_bracket() {
        // This compiles only if the builder produces a resource-neutral effect
        fn check() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
            Bracket::<FileRes>::new()
                .acquire(pure::<_, String, ()>(42))
                .release(|_: i32| async { Ok::<(), String>(()) })
                .use_fn(|val: &i32| pure::<_, String, ()>(*val))
        }
        let _ = check; // Silence unused warning
    }
}
