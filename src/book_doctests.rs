//! Cargo-backed doctest coverage for the README and mdBook.

#[doc = include_str!("../README.md")]
mod root_readme {}

#[doc = include_str!("../docs/README.md")]
mod docs_readme {}

#[doc = include_str!("../docs/guide/README.md")]
mod guide_readme {}

#[doc = include_str!("../docs/guide/01-semigroup.md")]
mod semigroup {}

#[doc = include_str!("../docs/guide/02-validation.md")]
mod validation {}

#[doc = include_str!("../docs/guide/03-effects.md")]
mod effects {}

#[doc = include_str!("../docs/guide/04-error-context.md")]
mod error_context {}

#[doc = include_str!("../docs/guide/05-io-module.md")]
mod io_module {}

#[doc = include_str!("../docs/guide/06-helper-combinators.md")]
mod helper_combinators {}

#[doc = include_str!("../docs/guide/07-try-trait.md")]
mod try_trait {}

#[doc = include_str!("../docs/guide/08-monoid.md")]
mod monoid {}

#[doc = include_str!("../docs/guide/09-reader-pattern.md")]
mod reader_pattern {}

#[doc = include_str!("../docs/guide/10-parallel-effects.md")]
mod parallel_effects {}

#[doc = include_str!("../docs/guide/11-traverse-patterns.md")]
mod traverse_patterns {}

#[doc = include_str!("../docs/guide/12-retry.md")]
mod retry {}

#[doc = include_str!("../docs/guide/13-homogeneous-validation.md")]
mod homogeneous_validation {}

#[doc = include_str!("../docs/guide/14-refined-types.md")]
mod refined_types {}

#[doc = include_str!("../docs/guide/15-testing.md")]
mod testing {}

#[doc = include_str!("../docs/guide/16-resource-tracking.md")]
mod resource_tracking {}

#[doc = include_str!("../docs/guide/17-api-tiers.md")]
mod api_tiers {}

#[doc = include_str!("../docs/PERFORMANCE.md")]
mod performance {}

#[doc = include_str!("../docs/PATTERNS.md")]
mod patterns {}

#[doc = include_str!("../docs/FAQ.md")]
mod faq {}

#[doc = include_str!("../docs/MIGRATION.md")]
mod migration {}

#[doc = include_str!("../docs/COMPARISON.md")]
mod comparison {}

#[doc = include_str!("../docs/validation-api-comparison.md")]
mod validation_api_comparison {}

#[doc = include_str!("../docs/async-design.md")]
mod async_design {}

#[doc = include_str!("../docs/io-api-analysis.md")]
mod io_api_analysis {}

#[doc = include_str!("../docs/improving-our-process.md")]
mod improving_our_process {}
