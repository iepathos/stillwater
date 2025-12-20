//! ResourceEffect implementations for existing combinators.
//!
//! This module provides `ResourceEffect` implementations for the core
//! combinators, enabling resource tracking to compose correctly through
//! effect chains.
//!
//! # Resource Propagation Rules
//!
//! - `Pure`, `Fail` - Resource-neutral (Empty, Empty)
//! - `Map`, `MapErr` - Preserves inner effect's resources
//! - `AndThen` - Unions resources from both effects
//! - `Tracked` - Uses its explicit type parameters

use super::sets::{Empty, Union};
use super::tracked::ResourceEffect;
use crate::effect::combinators::{AndThen, Fail, Map, MapErr, Pure};

// =============================================================================
// Pure is resource-neutral
// =============================================================================

impl<T, E, Env> ResourceEffect for Pure<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Acquires = Empty;
    type Releases = Empty;
}

// =============================================================================
// Fail is resource-neutral
// =============================================================================

impl<T, E, Env> ResourceEffect for Fail<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Acquires = Empty;
    type Releases = Empty;
}

// =============================================================================
// Map preserves inner effect's resources
// =============================================================================

impl<Inner, F, U> ResourceEffect for Map<Inner, F>
where
    Inner: ResourceEffect,
    F: FnOnce(Inner::Output) -> U + Send,
    U: Send,
{
    type Acquires = Inner::Acquires;
    type Releases = Inner::Releases;
}

// =============================================================================
// MapErr preserves inner effect's resources
// =============================================================================

impl<Inner, F, E2> ResourceEffect for MapErr<Inner, F>
where
    Inner: ResourceEffect,
    F: FnOnce(Inner::Error) -> E2 + Send,
    E2: Send,
{
    type Acquires = Inner::Acquires;
    type Releases = Inner::Releases;
}

// =============================================================================
// AndThen unions resources from both effects
// =============================================================================

impl<Inner, F, E2> ResourceEffect for AndThen<Inner, F>
where
    Inner: ResourceEffect,
    E2: ResourceEffect<Error = Inner::Error, Env = Inner::Env>,
    F: FnOnce(Inner::Output) -> E2 + Send,
    Inner::Acquires: Union<E2::Acquires>,
    Inner::Releases: Union<E2::Releases>,
{
    type Acquires = <Inner::Acquires as Union<E2::Acquires>>::Output;
    type Releases = <Inner::Releases as Union<E2::Releases>>::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::constructors::{fail, pure};
    use crate::effect::resource::ext::ResourceEffectExt;
    use crate::effect::resource::markers::FileRes;
    use crate::effect::resource::sets::{Has, ResourceSet};
    use crate::effect::trait_def::Effect;
    use crate::effect::EffectExt;

    // Type-level assertion helpers
    fn _assert_acquires<T: ResourceEffect<Acquires = A>, A: ResourceSet>() {}
    #[allow(dead_code)]
    fn _assert_releases<T: ResourceEffect<Releases = R>, R: ResourceSet>() {}
    fn _assert_neutral<T: ResourceEffect<Acquires = Empty, Releases = Empty>>() {}

    #[test]
    fn pure_is_neutral() {
        type PureType = Pure<i32, String, ()>;
        _assert_neutral::<PureType>();
    }

    #[test]
    fn fail_is_neutral() {
        type FailType = Fail<i32, String, ()>;
        _assert_neutral::<FailType>();
    }

    #[test]
    fn map_preserves_resources() {
        // Create a tracked effect with acquisitions
        let effect = pure::<_, String, ()>(42).acquires::<FileRes>();

        // Map should preserve the tracked resources
        let mapped = effect.map(|x| x * 2);

        // Type-level check: mapped still has FileRes as Acquires
        fn check<T: ResourceEffect<Acquires = Has<FileRes>>>(_: T) {}
        check(mapped);
    }

    #[test]
    fn map_err_preserves_resources() {
        let effect = pure::<_, i32, ()>(42).acquires::<FileRes>();
        let mapped = effect.map_err(|e| e.to_string());

        fn check<T: ResourceEffect<Acquires = Has<FileRes>>>(_: T) {}
        check(mapped);
    }

    // Note: AndThen with ResourceEffect requires both effects to implement ResourceEffect
    // For tracked effects, this works automatically

    #[tokio::test]
    async fn pure_runs_correctly() {
        let effect = pure::<_, String, ()>(42);
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn fail_runs_correctly() {
        let effect = fail::<i32, _, ()>("error".to_string());
        let result = effect.run(&()).await;
        assert_eq!(result, Err("error".to_string()));
    }

    #[tokio::test]
    async fn map_with_tracked_runs_correctly() {
        let effect = pure::<_, String, ()>(42)
            .acquires::<FileRes>()
            .map(|x| x * 2);
        let result = effect.run(&()).await;
        assert_eq!(result, Ok(84));
    }
}
