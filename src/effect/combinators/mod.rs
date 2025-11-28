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
mod and_then_auto;
mod and_then_ref;
mod check;
mod fail;
mod from_async;
mod from_fn;
mod from_result;
mod map;
mod map_err;
mod or_else;
mod pure;
mod tap;
mod with;
mod zip;
mod zip_with;

pub use and_then::AndThen;
pub use and_then_auto::AndThenAuto;
pub use and_then_ref::AndThenRef;
pub use check::Check;
pub use fail::Fail;
pub use from_async::FromAsync;
pub use from_fn::FromFn;
pub use from_result::FromResult;
pub use map::Map;
pub use map_err::MapErr;
pub use or_else::OrElse;
pub use pure::Pure;
pub use tap::Tap;
pub use with::With;
pub use zip::{Zip, Zip3, Zip4, Zip5, Zip6, Zip7, Zip8};
pub use zip_with::ZipWith;
