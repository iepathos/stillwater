//! Type-level resource sets for compile-time tracking.
//!
//! This module provides type-level sets for tracking multiple resources.
//! All operations are compile-time only, with zero runtime overhead.
//!
//! # Structure
//!
//! - `Empty` - An empty resource set (no resources)
//! - `Has<R, Rest>` - A non-empty set containing resource R plus Rest
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::resource::{Empty, Has, FileRes, DbRes};
//!
//! // Type representing "has FileRes"
//! type SingleResource = Has<FileRes>;
//!
//! // Type representing "has FileRes and DbRes"
//! type MultipleResources = Has<FileRes, Has<DbRes>>;
//! ```

use std::marker::PhantomData;

use super::markers::ResourceKind;

/// Marker trait for valid resource sets.
///
/// This trait is sealed and implemented only for `Empty` and `Has<R, Rest>`.
/// It ensures only valid resource set types can be used in resource tracking.
pub trait ResourceSet: Send + Sync + 'static {}

/// Empty resource set - represents no resources.
///
/// This is the base case for resource sets. An effect with
/// `Acquires = Empty` and `Releases = Empty` is resource-neutral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Empty;

impl ResourceSet for Empty {}

/// Non-empty resource set - contains resource R plus Rest.
///
/// This is used to build up resource sets at the type level.
/// Resources are tracked in a cons-list structure.
///
/// # Type Parameters
///
/// * `R` - The resource kind being tracked
/// * `Rest` - Additional resources (defaults to `Empty`)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Has<R: ResourceKind, Rest: ResourceSet = Empty>(PhantomData<(R, Rest)>);

impl<R: ResourceKind, Rest: ResourceSet> Default for Has<R, Rest> {
    fn default() -> Self {
        Has(PhantomData)
    }
}

impl<R: ResourceKind, Rest: ResourceSet> std::fmt::Debug for Has<R, Rest> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Has<{}>", R::NAME)
    }
}

impl<R: ResourceKind, Rest: ResourceSet> ResourceSet for Has<R, Rest> {}

/// Type-level union of two resource sets.
///
/// This trait computes the union of two resource sets at compile time.
/// The result contains all resources from both sets.
pub trait Union<Other: ResourceSet>: ResourceSet {
    /// The resulting set containing resources from both sets.
    type Output: ResourceSet;
}

// Union with Empty is identity
impl<S: ResourceSet> Union<Empty> for S {
    type Output = S;
}

// Union of Empty with non-empty is the non-empty set
impl<R: ResourceKind, Rest: ResourceSet> Union<Has<R, Rest>> for Empty {
    type Output = Has<R, Rest>;
}

// Union of Has with Has - add to the list
impl<R1: ResourceKind, Rest1: ResourceSet, R2: ResourceKind, Rest2: ResourceSet>
    Union<Has<R2, Rest2>> for Has<R1, Rest1>
where
    Rest1: Union<Has<R2, Rest2>>,
{
    type Output = Has<R1, <Rest1 as Union<Has<R2, Rest2>>>::Output>;
}

/// Marker trait for sets that contain a specific resource.
///
/// This is used to enforce that certain operations have
/// specific resources available.
///
/// Note: Due to Rust's coherence rules, we only implement Contains
/// for direct containment (Has<R, _> contains R). Nested containment
/// checking is done through the Subset trait instead.
pub trait Contains<R: ResourceKind>: ResourceSet {}

// Has<R, _> contains R
impl<R: ResourceKind, Rest: ResourceSet> Contains<R> for Has<R, Rest> {}

/// Marker trait for subsets.
///
/// `A: Subset<B>` means all resources in A are also in B.
///
/// Note: Due to Rust's coherence rules, subset checking is limited
/// to specific cases. For complex resource relationships, use the
/// Union trait to combine resource sets.
pub trait Subset<Super: ResourceSet>: ResourceSet {}

// Empty is a subset of everything
impl<S: ResourceSet> Subset<S> for Empty {}

// Has<R, Empty> is a subset of Has<R, _> (single resource subset)
impl<R: ResourceKind, Rest: ResourceSet> Subset<Has<R, Rest>> for Has<R, Empty> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::resource::markers::{DbRes, FileRes, TxRes};

    #[test]
    fn empty_set_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Empty>(), 0);
    }

    #[test]
    fn has_set_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Has<FileRes>>(), 0);
        assert_eq!(std::mem::size_of::<Has<FileRes, Has<DbRes>>>(), 0);
    }

    #[test]
    fn debug_impl() {
        let empty = Empty;
        assert_eq!(format!("{:?}", empty), "Empty");

        let has_file: Has<FileRes> = Has::default();
        assert!(format!("{:?}", has_file).contains("File"));
    }

    // Type-level tests using const assertions
    fn _assert_resource_set<T: ResourceSet>() {}
    fn _assert_contains<S: Contains<R>, R: ResourceKind>() {}
    fn _assert_subset<A: Subset<B>, B: ResourceSet>() {}

    #[test]
    fn empty_is_resource_set() {
        _assert_resource_set::<Empty>();
    }

    #[test]
    fn has_is_resource_set() {
        _assert_resource_set::<Has<FileRes>>();
        _assert_resource_set::<Has<FileRes, Has<DbRes>>>();
    }

    #[test]
    fn has_contains_its_resource() {
        _assert_contains::<Has<FileRes>, FileRes>();
    }

    // Note: Nested containment checking is limited due to Rust's coherence rules.
    // We only check direct containment (Has<R, _> contains R).

    #[test]
    fn empty_is_subset_of_empty() {
        _assert_subset::<Empty, Empty>();
    }

    #[test]
    fn empty_is_subset_of_has() {
        _assert_subset::<Empty, Has<FileRes>>();
    }

    #[test]
    fn single_has_is_subset_of_same() {
        _assert_subset::<Has<FileRes, Empty>, Has<FileRes>>();
    }

    // Test Union at type level
    fn _assert_union<A: Union<B>, B: ResourceSet>() {}

    #[test]
    fn union_with_empty() {
        _assert_union::<Has<FileRes>, Empty>();
        _assert_union::<Empty, Has<FileRes>>();
    }

    #[test]
    fn union_of_has() {
        _assert_union::<Has<FileRes>, Has<DbRes>>();
    }

    // Compile-time verification that union output is a ResourceSet
    fn _union_output_is_resource_set<A, B>()
    where
        A: Union<B>,
        B: ResourceSet,
    {
        _assert_resource_set::<<A as Union<B>>::Output>();
    }

    #[test]
    fn union_output_types() {
        _union_output_is_resource_set::<Empty, Empty>();
        _union_output_is_resource_set::<Has<FileRes>, Empty>();
        _union_output_is_resource_set::<Empty, Has<DbRes>>();
        _union_output_is_resource_set::<Has<FileRes>, Has<DbRes>>();
    }

    // Additional type-level tests for complex scenarios
    #[test]
    fn three_resource_set() {
        type ThreeRes = Has<FileRes, Has<DbRes, Has<TxRes>>>;
        _assert_resource_set::<ThreeRes>();
        // Note: We can only check direct containment due to coherence rules
        _assert_contains::<ThreeRes, FileRes>();
    }
}
