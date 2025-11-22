//! Effect type for composing computations
//!
//! This module will contain the Effect type for effect composition.
//! Implementation pending in future specs.

/// Placeholder for Effect type
#[derive(Debug)]
pub struct Effect<T, E, Env> {
    _phantom: std::marker::PhantomData<(T, E, Env)>,
}
