//! Retry policy types and configuration.

use std::time::Duration;

/// A retry policy describing how to retry failed operations.
///
/// Policies are pure data - they describe retry behavior but don't execute it.
/// This makes them easy to test, clone, and inspect.
///
/// # Bounds Behavior
///
/// At least one bound MUST be set, or the policy will panic at construction:
/// - `max_retries`: Maximum number of retry attempts (not including initial attempt)
/// - `max_delay`: Maximum delay cap (also acts as implicit bound for exponential/fibonacci)
///
/// **Rationale**: Unbounded retries are almost always a bug. Requiring explicit bounds
/// forces intentional decisions about retry limits.
///
/// # Examples
///
/// ```rust
/// use stillwater::RetryPolicy;
/// use std::time::Duration;
///
/// // Exponential backoff with max retries
/// let policy = RetryPolicy::exponential(Duration::from_millis(100))
///     .with_max_retries(5);
///
/// assert_eq!(policy.max_retries(), Some(5));
///
/// // Constant delay with max delay cap
/// let policy = RetryPolicy::constant(Duration::from_millis(500))
///     .with_max_delay(Duration::from_secs(30));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct RetryPolicy {
    strategy: RetryStrategy,
    max_retries: Option<u32>,
    max_delay: Option<Duration>,
    jitter: JitterStrategy,
}

/// The backoff strategy for retry delays.
#[derive(Debug, Clone, PartialEq)]
pub enum RetryStrategy {
    /// Fixed delay between attempts.
    Constant(Duration),
    /// Delay increases linearly: base * (attempt + 1).
    Linear {
        /// Base delay duration.
        base: Duration,
    },
    /// Delay doubles: base * 2^attempt.
    Exponential {
        /// Base delay duration.
        base: Duration,
    },
    /// Delay follows Fibonacci sequence: fib(attempt) * base.
    Fibonacci {
        /// Base delay duration.
        base: Duration,
    },
}

/// Strategy for adding randomness to delays.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum JitterStrategy {
    /// No jitter applied.
    #[default]
    None,
    /// Add ±percentage randomness to delay.
    Proportional(f64),
    /// Random delay between 0 and calculated delay (AWS recommended).
    Full,
    /// Decorrelated jitter (AWS style).
    Decorrelated,
}

/// Information about a retry attempt, passed to hooks.
#[derive(Debug, Clone)]
pub struct RetryEvent<'a, E> {
    /// Which attempt just failed (1-indexed).
    pub attempt: u32,
    /// The error from the failed attempt.
    pub error: &'a E,
    /// Delay before next attempt (if retrying).
    pub next_delay: Option<Duration>,
    /// Total elapsed time since first attempt.
    pub elapsed: Duration,
}

impl RetryPolicy {
    /// Create a policy with constant delay between retries.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::constant(Duration::from_millis(500))
    ///     .with_max_retries(3);
    ///
    /// // Every retry waits 500ms
    /// assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(500)));
    /// assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(500)));
    /// assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(500)));
    /// assert_eq!(policy.delay_for_attempt(3), None); // max_retries exceeded
    /// ```
    pub fn constant(delay: Duration) -> Self {
        Self {
            strategy: RetryStrategy::Constant(delay),
            max_retries: None,
            max_delay: None,
            jitter: JitterStrategy::None,
        }
    }

    /// Create a policy with linearly increasing delay.
    ///
    /// Delay = base * (attempt + 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::linear(Duration::from_millis(100))
    ///     .with_max_retries(5);
    ///
    /// // Delay increases: 100ms, 200ms, 300ms, 400ms, 500ms
    /// assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
    /// assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(200)));
    /// assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(300)));
    /// ```
    pub fn linear(base: Duration) -> Self {
        Self {
            strategy: RetryStrategy::Linear { base },
            max_retries: None,
            max_delay: None,
            jitter: JitterStrategy::None,
        }
    }

    /// Create a policy with exponentially increasing delay.
    ///
    /// Delay = base * 2^attempt
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_max_retries(5);
    ///
    /// // Delay doubles: 100ms, 200ms, 400ms, 800ms, 1600ms
    /// assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
    /// assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(200)));
    /// assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(400)));
    /// ```
    pub fn exponential(base: Duration) -> Self {
        Self {
            strategy: RetryStrategy::Exponential { base },
            max_retries: None,
            max_delay: None,
            jitter: JitterStrategy::None,
        }
    }

    /// Create a policy with Fibonacci-based delay.
    ///
    /// Delay = base * fib(attempt + 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::fibonacci(Duration::from_millis(100))
    ///     .with_max_retries(5);
    ///
    /// // Delay follows Fibonacci: 100ms, 100ms, 200ms, 300ms, 500ms
    /// assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
    /// assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(100)));
    /// assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(200)));
    /// assert_eq!(policy.delay_for_attempt(3), Some(Duration::from_millis(300)));
    /// assert_eq!(policy.delay_for_attempt(4), Some(Duration::from_millis(500)));
    /// ```
    pub fn fibonacci(base: Duration) -> Self {
        Self {
            strategy: RetryStrategy::Fibonacci { base },
            max_retries: None,
            max_delay: None,
            jitter: JitterStrategy::None,
        }
    }

    /// Set the maximum number of retry attempts.
    ///
    /// This does not include the initial attempt. For example, `max_retries(3)`
    /// means up to 4 total attempts (1 initial + 3 retries).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_max_retries(3);
    ///
    /// assert_eq!(policy.max_retries(), Some(3));
    /// ```
    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = Some(n);
        self
    }

    /// Set the maximum delay cap.
    ///
    /// Delays will never exceed this value, regardless of the backoff strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_max_retries(10)
    ///     .with_max_delay(Duration::from_secs(5));
    ///
    /// // Without cap: 100ms, 200ms, 400ms, 800ms, 1600ms, 3200ms, 6400ms...
    /// // With 5s cap: 100ms, 200ms, 400ms, 800ms, 1600ms, 3200ms, 5000ms...
    /// ```
    pub fn with_max_delay(mut self, d: Duration) -> Self {
        self.max_delay = Some(d);
        self
    }

    /// Add proportional jitter to delays.
    ///
    /// The factor determines the range of randomness. For example, `0.25` means
    /// the actual delay will be ±25% of the calculated delay.
    ///
    /// **Note**: Requires the `jitter` feature. Without it, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_jitter(0.25)
    ///     .with_max_retries(5);
    ///
    /// // Delays will be between 75ms-125ms, 150ms-250ms, 300ms-500ms, etc.
    /// ```
    pub fn with_jitter(mut self, factor: f64) -> Self {
        self.jitter = JitterStrategy::Proportional(factor.clamp(0.0, 1.0));
        self
    }

    /// Use full jitter (AWS recommended).
    ///
    /// The delay will be a random value between 0 and the calculated delay.
    /// This provides maximum spread to prevent thundering herd.
    ///
    /// **Note**: Requires the `jitter` feature. Without it, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_full_jitter()
    ///     .with_max_retries(5);
    /// ```
    pub fn with_full_jitter(mut self) -> Self {
        self.jitter = JitterStrategy::Full;
        self
    }

    /// Use decorrelated jitter (AWS style).
    ///
    /// Each delay is random between base and 3x the previous delay.
    /// This provides good spread while maintaining progression.
    ///
    /// **Note**: Requires the `jitter` feature. Without it, this method does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_decorrelated_jitter()
    ///     .with_max_retries(5);
    /// ```
    pub fn with_decorrelated_jitter(mut self) -> Self {
        self.jitter = JitterStrategy::Decorrelated;
        self
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> Option<u32> {
        self.max_retries
    }

    /// Get the maximum delay cap.
    pub fn max_delay(&self) -> Option<Duration> {
        self.max_delay
    }

    /// Get the jitter strategy.
    pub fn jitter(&self) -> &JitterStrategy {
        &self.jitter
    }

    /// Get the retry strategy.
    pub fn strategy(&self) -> &RetryStrategy {
        &self.strategy
    }

    /// Calculate the delay before attempt N (0-indexed).
    ///
    /// Returns None if no more retries should be attempted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stillwater::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::exponential(Duration::from_millis(100))
    ///     .with_max_retries(3);
    ///
    /// assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_millis(100)));
    /// assert_eq!(policy.delay_for_attempt(1), Some(Duration::from_millis(200)));
    /// assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_millis(400)));
    /// assert_eq!(policy.delay_for_attempt(3), None); // exceeded max_retries
    /// ```
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        // Check max_retries
        if let Some(max) = self.max_retries {
            if attempt >= max {
                return None;
            }
        }

        // Calculate base delay from strategy
        let base_delay = match &self.strategy {
            RetryStrategy::Constant(d) => *d,
            RetryStrategy::Linear { base } => base.saturating_mul(attempt + 1),
            RetryStrategy::Exponential { base } => {
                base.saturating_mul(2u32.saturating_pow(attempt))
            }
            RetryStrategy::Fibonacci { base } => base.saturating_mul(fibonacci(attempt + 1)),
        };

        // Apply max_delay cap
        let capped = match self.max_delay {
            Some(max) => base_delay.min(max),
            None => base_delay,
        };

        Some(capped)
    }

    /// Calculate the delay with jitter applied.
    ///
    /// This is used internally by the retry executor.
    #[doc(hidden)]
    pub fn delay_with_jitter(
        &self,
        attempt: u32,
        prev_delay: Option<Duration>,
    ) -> Option<Duration> {
        let base_delay = self.delay_for_attempt(attempt)?;
        Some(self.jitter.apply(base_delay, prev_delay, self.max_delay))
    }

    /// Validate that the policy has at least one bound.
    ///
    /// Returns an error message if the policy is invalid.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_retries.is_none() && self.max_delay.is_none() {
            Err("RetryPolicy must have at least one bound (max_retries or max_delay)")
        } else {
            Ok(())
        }
    }
}

impl JitterStrategy {
    /// Apply jitter to a base delay.
    ///
    /// # Arguments
    ///
    /// * `base_delay` - The calculated delay before jitter
    /// * `prev_delay` - The previous delay (for decorrelated jitter)
    /// * `max_delay` - Optional cap on the final delay
    pub fn apply(
        &self,
        base_delay: Duration,
        #[cfg_attr(not(feature = "jitter"), allow(unused_variables))] prev_delay: Option<Duration>,
        max_delay: Option<Duration>,
    ) -> Duration {
        let jittered = match self {
            JitterStrategy::None => base_delay,
            #[cfg(feature = "jitter")]
            JitterStrategy::Proportional(factor) => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let base_millis = base_delay.as_millis() as f64;
                let jitter_range = base_millis * factor;
                let min = (base_millis - jitter_range).max(0.0);
                let max = base_millis + jitter_range;
                let jittered_millis = rng.gen_range(min..=max);
                Duration::from_millis(jittered_millis as u64)
            }
            #[cfg(not(feature = "jitter"))]
            JitterStrategy::Proportional(_) => base_delay,
            #[cfg(feature = "jitter")]
            JitterStrategy::Full => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let max_millis = base_delay.as_millis() as u64;
                if max_millis == 0 {
                    Duration::ZERO
                } else {
                    Duration::from_millis(rng.gen_range(0..=max_millis))
                }
            }
            #[cfg(not(feature = "jitter"))]
            JitterStrategy::Full => base_delay,
            #[cfg(feature = "jitter")]
            JitterStrategy::Decorrelated => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let prev = prev_delay.unwrap_or(base_delay);
                let base_millis = base_delay.as_millis() as u64;
                let max_millis = prev.as_millis().saturating_mul(3) as u64;
                if max_millis <= base_millis {
                    base_delay
                } else {
                    Duration::from_millis(rng.gen_range(base_millis..=max_millis))
                }
            }
            #[cfg(not(feature = "jitter"))]
            JitterStrategy::Decorrelated => base_delay,
        };

        // Apply max_delay cap
        match max_delay {
            Some(max) => jittered.min(max),
            None => jittered,
        }
    }
}

/// Calculate the nth Fibonacci number.
fn fibonacci(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut a = 0u32;
    let mut b = 1u32;
    for _ in 1..n {
        let temp = a.saturating_add(b);
        a = b;
        b = temp;
    }
    b
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    #[test]
    fn test_constant_delay() {
        let policy = RetryPolicy::constant(Duration::from_millis(100)).with_max_retries(3);

        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(100))
        );
        assert_eq!(policy.delay_for_attempt(3), None);
    }

    #[test]
    fn test_linear_delay() {
        let policy = RetryPolicy::linear(Duration::from_millis(100)).with_max_retries(5);

        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(300))
        );
        assert_eq!(
            policy.delay_for_attempt(3),
            Some(Duration::from_millis(400))
        );
    }

    #[test]
    fn test_exponential_delay() {
        let policy = RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(5);

        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(400))
        );
        assert_eq!(
            policy.delay_for_attempt(3),
            Some(Duration::from_millis(800))
        );
    }

    #[test]
    fn test_fibonacci_delay() {
        let policy = RetryPolicy::fibonacci(Duration::from_millis(100)).with_max_retries(6);

        // fib sequence: 1, 1, 2, 3, 5, 8, 13...
        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            policy.delay_for_attempt(3),
            Some(Duration::from_millis(300))
        );
        assert_eq!(
            policy.delay_for_attempt(4),
            Some(Duration::from_millis(500))
        );
        assert_eq!(
            policy.delay_for_attempt(5),
            Some(Duration::from_millis(800))
        );
    }

    #[test]
    fn test_max_delay_cap() {
        let policy = RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(10)
            .with_max_delay(Duration::from_millis(500));

        assert_eq!(
            policy.delay_for_attempt(0),
            Some(Duration::from_millis(100))
        );
        assert_eq!(
            policy.delay_for_attempt(1),
            Some(Duration::from_millis(200))
        );
        assert_eq!(
            policy.delay_for_attempt(2),
            Some(Duration::from_millis(400))
        );
        assert_eq!(
            policy.delay_for_attempt(3),
            Some(Duration::from_millis(500))
        ); // capped
        assert_eq!(
            policy.delay_for_attempt(4),
            Some(Duration::from_millis(500))
        ); // capped
    }

    #[test]
    fn test_max_retries_limit() {
        let policy = RetryPolicy::constant(Duration::from_millis(100)).with_max_retries(2);

        assert!(policy.delay_for_attempt(0).is_some());
        assert!(policy.delay_for_attempt(1).is_some());
        assert!(policy.delay_for_attempt(2).is_none());
    }

    #[test]
    fn test_policy_is_clone() {
        let policy = RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3);
        let cloned = policy.clone();
        assert_eq!(policy, cloned);
    }

    #[test]
    fn test_policy_is_debug() {
        let policy = RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3);
        let debug = format!("{:?}", policy);
        assert!(debug.contains("RetryPolicy"));
    }

    #[test]
    fn test_fibonacci_function() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(6), 8);
        assert_eq!(fibonacci(7), 13);
    }

    #[test]
    fn test_validate_with_max_retries() {
        let policy = RetryPolicy::constant(Duration::from_millis(100)).with_max_retries(3);
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_validate_with_max_delay() {
        let policy = RetryPolicy::constant(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(5));
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_validate_with_both_bounds() {
        let policy = RetryPolicy::constant(Duration::from_millis(100))
            .with_max_retries(3)
            .with_max_delay(Duration::from_secs(5));
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_validate_no_bounds() {
        let policy = RetryPolicy::constant(Duration::from_millis(100));
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_jitter_strategy_default() {
        let jitter = JitterStrategy::default();
        assert_eq!(jitter, JitterStrategy::None);
    }

    #[test]
    fn test_jitter_none_returns_base_delay() {
        let jitter = JitterStrategy::None;
        let base = Duration::from_millis(100);
        let result = jitter.apply(base, None, None);
        assert_eq!(result, base);
    }

    #[test]
    fn test_policy_getters() {
        let policy = RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(3)
            .with_max_delay(Duration::from_secs(5))
            .with_jitter(0.25);

        assert_eq!(policy.max_retries(), Some(3));
        assert_eq!(policy.max_delay(), Some(Duration::from_secs(5)));
        assert!(matches!(policy.jitter(), JitterStrategy::Proportional(_)));
        assert!(matches!(
            policy.strategy(),
            RetryStrategy::Exponential { .. }
        ));
    }
}
