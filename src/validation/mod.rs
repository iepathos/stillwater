//! Validation module for accumulating errors and ensuring data consistency
//!
//! This module provides:
//! - The core `Validation` type for error accumulation
//! - Homogeneous validation utilities for ensuring collections are type-consistent

pub mod core;
pub mod homogeneous;

// Re-export core validation types
pub use core::*;
