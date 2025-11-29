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
mod ensure;
mod ensure_pred;
mod ensure_with;
mod fail;
mod fallback;
mod fallback_to;
mod from_async;
mod from_fn;
mod from_result;
mod map;
mod map_err;
mod or_else;
mod pure;
mod recover;
mod recover_some;
mod recover_with;
mod tap;
mod unless;
mod with;
mod zip;
mod zip_with;

pub use and_then::AndThen;
pub use and_then_auto::AndThenAuto;
pub use and_then_ref::AndThenRef;
pub use check::Check;
pub use ensure::Ensure;
pub use ensure_pred::EnsurePred;
pub use ensure_with::EnsureWith;
pub use fail::Fail;
pub use fallback::Fallback;
pub use fallback_to::FallbackTo;
pub use from_async::FromAsync;
pub use from_fn::FromFn;
pub use from_result::FromResult;
pub use map::Map;
pub use map_err::MapErr;
pub use or_else::OrElse;
pub use pure::Pure;
pub use recover::Recover;
pub use recover_some::RecoverSome;
pub use recover_with::RecoverWith;
pub use tap::Tap;
pub use unless::Unless;
pub use with::With;
pub use zip::{Zip, Zip3, Zip4, Zip5, Zip6, Zip7, Zip8};
pub use zip_with::ZipWith;

#[cfg(test)]
mod recover_tests;

#[cfg(test)]
mod zero_cost_tests;
