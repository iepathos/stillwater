//! Zero-cost combinator types for effect composition.
//!
//! This module contains concrete types returned by effect combinators.
//! Unlike boxed effects, these types are zero-cost - they don't allocate
//! on the heap and can be optimized by the compiler.
//!
//! Most users won't need to work with these types directly. Instead,
//! use the combinator methods on `EffectExt` which return these types
//! behind `impl Effect<...>`.

mod and_then;
mod fail;
mod from_async;
mod from_fn;
mod from_result;
mod map;
mod map_err;
mod or_else;
mod pure;

pub use and_then::AndThen;
pub use fail::Fail;
pub use from_async::FromAsync;
pub use from_fn::FromFn;
pub use from_result::FromResult;
pub use map::Map;
pub use map_err::MapErr;
pub use or_else::OrElse;
pub use pure::Pure;
