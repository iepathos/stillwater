//! Tracked wrapper and ResourceEffect trait for compile-time resource tracking.
//!
//! This module provides:
//! - `ResourceEffect` trait - extends Effect with resource acquisition/release tracking
//! - `Tracked` wrapper - adds resource annotations to any effect
//!
//! All tracking is compile-time only, with zero runtime overhead.

use std::marker::PhantomData;

use super::sets::{Empty, ResourceSet};
use crate::effect::trait_def::Effect;

/// An effect with compile-time resource tracking.
///
/// This trait extends `Effect` with associated types for tracking
/// which resources are acquired and released by the effect.
///
/// # Type Parameters
///
/// * `Acquires` - Resources this effect creates/acquires
/// * `Releases` - Resources this effect consumes/releases
///
/// # Default Behavior
///
/// For backward compatibility, effects that don't implement this trait
/// can be wrapped with `Tracked` to add resource tracking.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// fn open_file(path: &str) -> impl ResourceEffect<
///     Output = FileHandle,
///     Acquires = Has<FileRes>,
///     Releases = Empty,
/// > {
///     pure(FileHandle::new(path)).acquires::<FileRes>()
/// }
/// ```
pub trait ResourceEffect: Effect {
    /// Resources this effect acquires (creates).
    type Acquires: ResourceSet;

    /// Resources this effect releases (consumes).
    type Releases: ResourceSet;
}

/// Wrapper that adds resource tracking to any effect.
///
/// This is a zero-cost wrapper - it has the same runtime behavior
/// as the inner effect, with resource tracking at the type level only.
///
/// # Type Parameters
///
/// * `Eff` - The inner effect
/// * `Acq` - Resources acquired (defaults to `Empty`)
/// * `Rel` - Resources released (defaults to `Empty`)
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// // Wrap an effect to track that it acquires a file resource
/// let tracked: Tracked<_, Has<FileRes>, Empty> = Tracked::new(open_file_effect);
/// ```
pub struct Tracked<Eff, Acq: ResourceSet = Empty, Rel: ResourceSet = Empty> {
    inner: Eff,
    _phantom: PhantomData<(Acq, Rel)>,
}

impl<Eff, Acq: ResourceSet, Rel: ResourceSet> Tracked<Eff, Acq, Rel> {
    /// Create a new Tracked wrapper around an effect.
    pub fn new(inner: Eff) -> Self {
        Tracked {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the inner effect.
    pub fn inner(&self) -> &Eff {
        &self.inner
    }

    /// Unwrap the inner effect.
    pub fn into_inner(self) -> Eff {
        self.inner
    }
}

impl<Eff: Clone, Acq: ResourceSet, Rel: ResourceSet> Clone for Tracked<Eff, Acq, Rel> {
    fn clone(&self) -> Self {
        Tracked {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Eff, Acq: ResourceSet, Rel: ResourceSet> std::fmt::Debug for Tracked<Eff, Acq, Rel> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tracked")
            .field("inner", &"<effect>")
            .field("acquires", &std::any::type_name::<Acq>())
            .field("releases", &std::any::type_name::<Rel>())
            .finish()
    }
}

// Effect implementation - delegates everything to inner
impl<Eff: Effect, Acq: ResourceSet, Rel: ResourceSet> Effect for Tracked<Eff, Acq, Rel> {
    type Output = Eff::Output;
    type Error = Eff::Error;
    type Env = Eff::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

// ResourceEffect implementation - uses the type parameters
impl<Eff: Effect, Acq: ResourceSet, Rel: ResourceSet> ResourceEffect for Tracked<Eff, Acq, Rel> {
    type Acquires = Acq;
    type Releases = Rel;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::pure;
    use crate::effect::resource::markers::FileRes;
    use crate::effect::resource::sets::Has;

    #[tokio::test]
    async fn tracked_delegates_to_inner() {
        let inner = pure::<_, String, ()>(42);
        let tracked: Tracked<_, Empty, Empty> = Tracked::new(inner);

        let result = tracked.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn tracked_with_resources_still_works() {
        let inner = pure::<_, String, ()>(42);
        let tracked: Tracked<_, Has<FileRes>, Empty> = Tracked::new(inner);

        let result = tracked.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn tracked_is_zero_sized_overhead() {
        // Tracked only adds PhantomData, which is zero-sized
        use std::mem::size_of;

        // The phantom data for resource sets is zero-sized
        assert_eq!(size_of::<PhantomData<(Empty, Empty)>>(), 0);
    }

    #[test]
    fn tracked_debug_impl() {
        let inner = pure::<_, String, ()>(42);
        let tracked: Tracked<_, Has<FileRes>, Empty> = Tracked::new(inner);

        let debug_str = format!("{:?}", tracked);
        assert!(debug_str.contains("Tracked"));
        assert!(debug_str.contains("<effect>"));
    }

    #[test]
    fn tracked_inner_access() {
        let inner = pure::<_, String, ()>(42);
        let tracked: Tracked<_, Empty, Empty> = Tracked::new(inner);

        let _ = tracked.inner();
        let _ = tracked.into_inner();
    }

    // Type-level tests
    fn _assert_resource_effect<T: ResourceEffect>() {}
    fn _assert_acquires<T: ResourceEffect<Acquires = A>, A: ResourceSet>() {}
    fn _assert_releases<T: ResourceEffect<Releases = R>, R: ResourceSet>() {}

    #[test]
    fn tracked_implements_resource_effect() {
        type TrackedPure = Tracked<crate::effect::combinators::Pure<i32, String, ()>, Empty, Empty>;
        _assert_resource_effect::<TrackedPure>();
    }

    #[test]
    fn tracked_acquires_type() {
        type TrackedWithAcq =
            Tracked<crate::effect::combinators::Pure<i32, String, ()>, Has<FileRes>, Empty>;
        _assert_acquires::<TrackedWithAcq, Has<FileRes>>();
    }

    #[test]
    fn tracked_releases_type() {
        type TrackedWithRel =
            Tracked<crate::effect::combinators::Pure<i32, String, ()>, Empty, Has<FileRes>>;
        _assert_releases::<TrackedWithRel, Has<FileRes>>();
    }
}
