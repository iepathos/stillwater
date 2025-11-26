//! Retry and resilience patterns for Effect-based computations.
//!
//! This module provides retry capabilities for stillwater effects, following the
//! "pure core, imperative shell" philosophy:
//!
//! - **Pure Core**: `RetryPolicy` is just data—no side effects, easily testable
//! - **Composable**: Policies can be combined and transformed
//! - **Declarative**: Describe *what* retry behavior you want, not *how* to implement it
//!
//! # Quick Start
//!
//! ```rust
//! use stillwater::{Effect, RetryPolicy};
//! use std::time::Duration;
//!
//! # tokio_test::block_on(async {
//! // Create a retry policy with exponential backoff
//! let policy = RetryPolicy::exponential(Duration::from_millis(100))
//!     .with_max_retries(3);
//!
//! // Retry an effect using a factory function
//! let effect = Effect::retry(
//!     || Effect::<_, String, ()>::pure(42),
//!     policy
//! );
//!
//! assert_eq!(effect.run(&()).await.unwrap().into_value(), 42);
//! # });
//! ```
//!
//! # Retry Strategies
//!
//! - **Constant**: Fixed delay between retries
//! - **Linear**: Delay increases linearly (100ms, 200ms, 300ms, ...)
//! - **Exponential**: Delay doubles each retry (100ms, 200ms, 400ms, ...)
//! - **Fibonacci**: Delay follows Fibonacci sequence
//!
//! # Jitter Support
//!
//! Jitter adds randomness to delays to prevent thundering herd problems.
//! Enable the `jitter` feature to use jitter:
//!
//! ```toml
//! stillwater = { version = "...", features = ["jitter"] }
//! ```
//!
//! ```rust,ignore
//! use stillwater::RetryPolicy;
//! use std::time::Duration;
//!
//! // Add ±25% randomness to delays
//! let policy = RetryPolicy::exponential(Duration::from_millis(100))
//!     .with_jitter(0.25)
//!     .with_max_retries(5);
//! ```
//!
//! # Error Types
//!
//! - [`RetryExhausted`]: Returned when all retries fail, contains the final error and metadata
//! - [`TimeoutError`]: Returned when an effect times out

mod error;
mod policy;

pub use error::{RetryExhausted, TimeoutError};
pub use policy::{JitterStrategy, RetryEvent, RetryPolicy, RetryStrategy};

#[cfg(test)]
mod tests;
