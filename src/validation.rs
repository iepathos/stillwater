//! Validation type for accumulating errors
//!
//! This module will contain the Validation type that uses Semigroup for error accumulation.
//! Implementation pending in future specs.

/// Placeholder for Validation type
#[derive(Debug)]
pub struct Validation<T, E> {
    _phantom: std::marker::PhantomData<(T, E)>,
}
