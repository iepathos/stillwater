//! Context error handling with error trails
//!
//! This module provides the `ContextError` type, which wraps errors and accumulates
//! context information as they propagate up the call stack. This makes debugging
//! significantly easier by preserving a trail of breadcrumbs showing what operations
//! were being attempted when the error occurred.
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```
//! use stillwater::ContextError;
//!
//! let err = ContextError::new("file not found")
//!     .context("reading config file")
//!     .context("initializing application");
//!
//! assert_eq!(err.inner(), &"file not found");
//! assert_eq!(err.context_trail().len(), 2);
//! ```
//!
//! ## With Effect
//!
//! ```
//! use stillwater::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let effect = Effect::<i32, _, ()>::fail("database error")
//!     .context("querying user table")
//!     .context("loading user profile");
//!
//! match effect.run_standalone().await {
//!     Err(ctx_err) => {
//!         assert_eq!(ctx_err.inner(), &"database error");
//!         assert_eq!(ctx_err.context_trail().len(), 2);
//!     }
//!     Ok(_) => panic!("Expected error"),
//! }
//! # });
//! ```

use std::error::Error as StdError;
use std::fmt;

/// An error wrapper that accumulates context as it propagates
///
/// `ContextError<E>` wraps an underlying error of type `E` and maintains a trail
/// of context messages that describe what operations were being attempted when
/// the error occurred.
///
/// # Examples
///
/// ```
/// use stillwater::ContextError;
///
/// let err = ContextError::new("connection refused")
///     .context("connecting to database")
///     .context("initializing user service");
///
/// println!("{}", err);
/// // Output:
/// // Error: connection refused
/// //   -> connecting to database
/// //     -> initializing user service
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextError<E> {
    error: E,
    context: Vec<String>,
}

impl<E> ContextError<E> {
    /// Create a new context error
    ///
    /// This wraps an error with an empty context trail. Use the `context` method
    /// to add context layers.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::ContextError;
    ///
    /// let err = ContextError::new("base error");
    /// assert_eq!(err.inner(), &"base error");
    /// assert_eq!(err.context_trail(), &[] as &[String]);
    /// ```
    pub fn new(error: E) -> Self {
        ContextError {
            error,
            context: Vec::new(),
        }
    }

    /// Add a context layer
    ///
    /// Appends a context message to the trail. Context messages are accumulated
    /// in the order they're added, representing the call stack from inner to outer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::ContextError;
    ///
    /// let err = ContextError::new("parse error")
    ///     .context("reading config file")
    ///     .context("initializing app");
    ///
    /// assert_eq!(err.context_trail(), &["reading config file", "initializing app"]);
    /// ```
    pub fn context(mut self, msg: impl Into<String>) -> Self {
        self.context.push(msg.into());
        self
    }

    /// Get the underlying error
    ///
    /// Returns a reference to the wrapped error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::ContextError;
    ///
    /// let err = ContextError::new("base error").context("operation failed");
    /// assert_eq!(err.inner(), &"base error");
    /// ```
    pub fn inner(&self) -> &E {
        &self.error
    }

    /// Consume and return the underlying error
    ///
    /// Unwraps the context error and returns the original error, discarding
    /// the context trail.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::ContextError;
    ///
    /// let err = ContextError::new("base error").context("operation failed");
    /// let inner = err.into_inner();
    /// assert_eq!(inner, "base error");
    /// ```
    pub fn into_inner(self) -> E {
        self.error
    }

    /// Get the context trail
    ///
    /// Returns a slice of all context messages in the order they were added.
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::ContextError;
    ///
    /// let err = ContextError::new("error")
    ///     .context("step 1")
    ///     .context("step 2");
    ///
    /// assert_eq!(err.context_trail(), &["step 1", "step 2"]);
    /// ```
    pub fn context_trail(&self) -> &[String] {
        &self.context
    }
}

impl<E: fmt::Display> fmt::Display for ContextError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display underlying error
        write!(f, "Error: {}", self.error)?;

        // Display context trail with indentation
        for ctx in &self.context {
            write!(f, "\n  -> {}", ctx)?;
        }

        Ok(())
    }
}

impl<E: StdError + 'static> StdError for ContextError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_error_new() {
        let err = ContextError::new("base error");
        assert_eq!(err.inner(), &"base error");
        assert_eq!(err.context_trail(), &[] as &[String]);
    }

    #[test]
    fn test_context_accumulation() {
        let err = ContextError::new("base error")
            .context("first context")
            .context("second context");

        assert_eq!(err.context_trail(), &["first context", "second context"]);
    }

    #[test]
    fn test_context_into_string() {
        let err = ContextError::new("base error")
            .context(String::from("owned string"))
            .context("borrowed str");

        assert_eq!(err.context_trail(), &["owned string", "borrowed str"]);
    }

    #[test]
    fn test_display_format_no_context() {
        let err = ContextError::new("file not found");
        let output = format!("{}", err);
        assert_eq!(output, "Error: file not found");
    }

    #[test]
    fn test_display_format_with_context() {
        let err = ContextError::new("file not found")
            .context("reading config")
            .context("initializing app");

        let output = format!("{}", err);
        assert!(output.contains("Error: file not found"));
        assert!(output.contains("-> reading config"));
        assert!(output.contains("-> initializing app"));

        // Verify indentation is consistent
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Error: file not found");
        assert_eq!(lines[1], "  -> reading config");
        assert_eq!(lines[2], "  -> initializing app");
    }

    #[test]
    fn test_into_inner() {
        let err = ContextError::new("base error")
            .context("context 1")
            .context("context 2");

        let inner = err.into_inner();
        assert_eq!(inner, "base error");
    }

    #[test]
    fn test_clone() {
        let err = ContextError::new("error").context("context");
        let cloned = err.clone();

        assert_eq!(err.inner(), cloned.inner());
        assert_eq!(err.context_trail(), cloned.context_trail());
    }

    #[test]
    fn test_eq() {
        let err1 = ContextError::new("error").context("context");
        let err2 = ContextError::new("error").context("context");
        let err3 = ContextError::new("error").context("different");

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_trait() {
        use std::error::Error;

        let inner_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ctx_err = ContextError::new(inner_err).context("reading config");

        // Should implement Error trait
        let _: &dyn Error = &ctx_err;

        // Source should point to inner error
        assert!(ctx_err.source().is_some());
    }

    #[test]
    fn test_debug_format() {
        let err = ContextError::new("error").context("context");
        let debug_output = format!("{:?}", err);

        // Should contain both error and context
        assert!(debug_output.contains("error"));
        assert!(debug_output.contains("context"));
    }

    #[test]
    fn test_multiple_context_layers() {
        let err = ContextError::new("base")
            .context("layer 1")
            .context("layer 2")
            .context("layer 3")
            .context("layer 4");

        assert_eq!(err.context_trail().len(), 4);
        assert_eq!(
            err.context_trail(),
            &["layer 1", "layer 2", "layer 3", "layer 4"]
        );
    }
}
