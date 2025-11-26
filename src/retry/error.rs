//! Error types for retry operations.

use std::time::Duration;

/// Error returned when all retry attempts are exhausted.
///
/// Contains the final error along with metadata about the retry sequence.
///
/// # Examples
///
/// ```rust
/// use stillwater::{Effect, RetryPolicy, RetryExhausted};
/// use std::time::Duration;
///
/// # tokio_test::block_on(async {
/// let policy = RetryPolicy::constant(Duration::from_millis(1))
///     .with_max_retries(2);
///
/// let effect = Effect::retry(
///     || Effect::<(), _, ()>::fail("always fails"),
///     policy
/// );
///
/// match effect.run(&()).await {
///     Err(exhausted) => {
///         assert_eq!(exhausted.final_error, "always fails");
///         assert_eq!(exhausted.attempts, 3); // 1 initial + 2 retries
///     }
///     Ok(_) => panic!("Expected failure"),
/// }
/// # });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryExhausted<E> {
    /// The error from the final attempt.
    pub final_error: E,
    /// Total number of attempts made (initial + retries).
    pub attempts: u32,
    /// Total time spent retrying.
    pub total_duration: Duration,
}

impl<E> RetryExhausted<E> {
    /// Create a new RetryExhausted error.
    pub fn new(final_error: E, attempts: u32, total_duration: Duration) -> Self {
        Self {
            final_error,
            attempts,
            total_duration,
        }
    }

    /// Extract the final error, discarding metadata.
    pub fn into_error(self) -> E {
        self.final_error
    }

    /// Get a reference to the final error.
    pub fn error(&self) -> &E {
        &self.final_error
    }

    /// Extract the success value (for symmetry with Result).
    ///
    /// Since RetryExhausted is always an error, this returns the inner error.
    pub fn into_value(self) -> E {
        self.final_error
    }
}

impl<E: std::fmt::Display> std::fmt::Display for RetryExhausted<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "retry exhausted after {} attempts ({:?}): {}",
            self.attempts, self.total_duration, self.final_error
        )
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RetryExhausted<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.final_error)
    }
}

/// Error returned when an effect times out.
///
/// Can wrap either a timeout or an inner error from the effect.
///
/// # Examples
///
/// ```rust
/// use stillwater::{Effect, TimeoutError};
/// use std::time::Duration;
///
/// # tokio_test::block_on(async {
/// let effect = Effect::from_async(|_: &()| async {
///     tokio::time::sleep(Duration::from_secs(10)).await;
///     Ok::<_, String>(42)
/// })
/// .with_timeout(Duration::from_millis(10));
///
/// match effect.run(&()).await {
///     Err(TimeoutError::Timeout { duration }) => {
///         assert_eq!(duration, Duration::from_millis(10));
///     }
///     _ => panic!("Expected timeout"),
/// }
/// # });
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeoutError<E> {
    /// The operation timed out.
    Timeout {
        /// The timeout duration that was exceeded.
        duration: Duration,
    },
    /// An inner error occurred before timeout.
    Inner(E),
}

impl<E> TimeoutError<E> {
    /// Create a timeout error.
    pub fn timeout(duration: Duration) -> Self {
        Self::Timeout { duration }
    }

    /// Create an inner error.
    pub fn inner(error: E) -> Self {
        Self::Inner(error)
    }

    /// Returns true if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Returns true if this is an inner error.
    pub fn is_inner(&self) -> bool {
        matches!(self, Self::Inner(_))
    }

    /// Get the inner error if present.
    pub fn into_inner(self) -> Option<E> {
        match self {
            Self::Inner(e) => Some(e),
            Self::Timeout { .. } => None,
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for TimeoutError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout { duration } => write!(f, "operation timed out after {:?}", duration),
            Self::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for TimeoutError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Timeout { .. } => None,
            Self::Inner(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_retry_exhausted_display() {
        let err = RetryExhausted::new("connection failed", 3, Duration::from_millis(500));
        let display = format!("{}", err);
        assert!(display.contains("retry exhausted"));
        assert!(display.contains("3 attempts"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_retry_exhausted_into_error() {
        let err = RetryExhausted::new("test error", 5, Duration::from_secs(1));
        assert_eq!(err.into_error(), "test error");
    }

    #[test]
    fn test_timeout_error_timeout() {
        let err: TimeoutError<String> = TimeoutError::timeout(Duration::from_secs(5));
        assert!(err.is_timeout());
        assert!(!err.is_inner());
        assert!(err.into_inner().is_none());
    }

    #[test]
    fn test_timeout_error_inner() {
        let err = TimeoutError::inner("inner error".to_string());
        assert!(!err.is_timeout());
        assert!(err.is_inner());
        assert_eq!(err.into_inner(), Some("inner error".to_string()));
    }

    #[test]
    fn test_timeout_error_display() {
        let timeout: TimeoutError<String> = TimeoutError::timeout(Duration::from_secs(5));
        assert!(format!("{}", timeout).contains("timed out"));

        let inner = TimeoutError::inner("failed".to_string());
        assert_eq!(format!("{}", inner), "failed");
    }
}
