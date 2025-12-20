//! Extension trait for adding resource tracking to effects.
//!
//! This module provides the `ResourceEffectExt` trait which adds
//! `.acquires()` and `.releases()` methods to all effects.
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//! use stillwater::effect::resource::*;
//!
//! let effect = pure(42)
//!     .acquires::<FileRes>()   // Mark resource acquisition
//!     .map(|x| x * 2);
//! ```

use super::markers::ResourceKind;
use super::sets::{Empty, Has, ResourceSet};
use super::tracked::{ResourceEffect, Tracked};
use crate::effect::trait_def::Effect;

/// Extension trait for adding resource tracking to effects.
///
/// This trait is automatically implemented for all types that implement `Effect`.
/// It provides ergonomic methods to mark resource acquisition and release.
pub trait ResourceEffectExt: Effect + Sized {
    /// Mark that this effect acquires resource R.
    ///
    /// This wraps the effect in a `Tracked` with `Acquires = Has<R>`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::resource::*;
    ///
    /// fn open_file(path: &str) -> impl ResourceEffect<Acquires = Has<FileRes>> {
    ///     pure(FileHandle::new(path)).acquires::<FileRes>()
    /// }
    /// ```
    fn acquires<R: ResourceKind>(self) -> Tracked<Self, Has<R>, Empty> {
        Tracked::new(self)
    }

    /// Mark that this effect releases resource R.
    ///
    /// This wraps the effect in a `Tracked` with `Releases = Has<R>`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use stillwater::effect::resource::*;
    ///
    /// fn close_file(handle: FileHandle) -> impl ResourceEffect<Releases = Has<FileRes>> {
    ///     pure(()).releases::<FileRes>()
    /// }
    /// ```
    fn releases<R: ResourceKind>(self) -> Tracked<Self, Empty, Has<R>> {
        Tracked::new(self)
    }

    /// Mark this effect as resource-neutral (no acquisitions or releases).
    ///
    /// This is useful for explicitly annotating effects that don't
    /// interact with tracked resources.
    fn neutral(self) -> Tracked<Self, Empty, Empty> {
        Tracked::new(self)
    }
}

// Blanket implementation for all Effect types
impl<E: Effect> ResourceEffectExt for E {}

/// Assert that an effect is resource-neutral (compile-time check).
///
/// This function accepts only effects with `Acquires = Empty` and `Releases = Empty`.
/// Use it to enforce that a function returns a resource-neutral effect.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::resource::*;
///
/// fn process_data() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
///     // Using resource_bracket ensures neutrality
///     let effect = resource_bracket::<FileRes, _, _, _, _, _, _, _, _>(
///         open_file("data.txt"),
///         |f| async move { close_file(f).run(&()).await },
///         |f| read_contents(f),
///     );
///     assert_resource_neutral(effect)
/// }
/// ```
pub fn assert_resource_neutral<Eff>(effect: Eff) -> Eff
where
    Eff: ResourceEffect<Acquires = Empty, Releases = Empty>,
{
    effect
}

/// Check if an effect is resource-neutral at compile time.
///
/// This is a type-level assertion that the effect doesn't acquire
/// or release any tracked resources.
pub trait IsResourceNeutral: ResourceEffect<Acquires = Empty, Releases = Empty> {}

impl<E: ResourceEffect<Acquires = Empty, Releases = Empty>> IsResourceNeutral for E {}

/// Extension trait for tracked effects to chain resource operations.
pub trait TrackedExt<Eff, Acq: ResourceSet, Rel: ResourceSet>: Sized {
    /// Add another acquired resource to the tracking.
    fn also_acquires<R: ResourceKind>(self) -> Tracked<Eff, Has<R, Acq>, Rel>;

    /// Add another released resource to the tracking.
    fn also_releases<R: ResourceKind>(self) -> Tracked<Eff, Acq, Has<R, Rel>>;
}

impl<Eff: Effect, Acq: ResourceSet, Rel: ResourceSet> TrackedExt<Eff, Acq, Rel>
    for Tracked<Eff, Acq, Rel>
{
    fn also_acquires<R: ResourceKind>(self) -> Tracked<Eff, Has<R, Acq>, Rel> {
        Tracked::new(self.into_inner())
    }

    fn also_releases<R: ResourceKind>(self) -> Tracked<Eff, Acq, Has<R, Rel>> {
        Tracked::new(self.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::pure;
    use crate::effect::resource::markers::{DbRes, FileRes};

    #[tokio::test]
    async fn acquires_marks_resource() {
        let effect = pure::<_, String, ()>(42).acquires::<FileRes>();
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn releases_marks_resource() {
        let effect = pure::<_, String, ()>(()).releases::<FileRes>();
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn neutral_marks_no_resources() {
        let effect = pure::<_, String, ()>(42).neutral();
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn also_acquires_adds_resource() {
        let effect = pure::<_, String, ()>(42)
            .acquires::<FileRes>()
            .also_acquires::<DbRes>();
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn also_releases_adds_resource() {
        let effect = pure::<_, String, ()>(())
            .releases::<FileRes>()
            .also_releases::<DbRes>();
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(()));
    }

    // Type-level tests
    fn _assert_acquires<T: ResourceEffect<Acquires = A>, A: ResourceSet>() {}
    fn _assert_releases<T: ResourceEffect<Releases = R>, R: ResourceSet>() {}
    fn _assert_neutral<T: IsResourceNeutral>() {}

    #[test]
    fn acquires_type_check() {
        type WithFile = Tracked<crate::effect::combinators::Pure<i32, String, ()>, Has<FileRes>>;
        _assert_acquires::<WithFile, Has<FileRes>>();
    }

    #[test]
    fn releases_type_check() {
        type WithFile =
            Tracked<crate::effect::combinators::Pure<(), String, ()>, Empty, Has<FileRes>>;
        _assert_releases::<WithFile, Has<FileRes>>();
    }

    #[test]
    fn neutral_type_check() {
        type Neutral = Tracked<crate::effect::combinators::Pure<i32, String, ()>, Empty, Empty>;
        _assert_neutral::<Neutral>();
    }

    #[test]
    fn assert_resource_neutral_compiles() {
        let effect = pure::<_, String, ()>(42).neutral();
        let _ = assert_resource_neutral(effect);
    }

    #[test]
    fn also_acquires_type_check() {
        type WithBoth =
            Tracked<crate::effect::combinators::Pure<i32, String, ()>, Has<DbRes, Has<FileRes>>>;
        _assert_acquires::<WithBoth, Has<DbRes, Has<FileRes>>>();
    }
}
