//! Refined types for compile-time validation guarantees
//!
//! This module implements the "parse, don't validate" pattern:
//! validate data once at system boundaries, then use types to
//! guarantee validity throughout the codebase.
//!
//! # Philosophy
//!
//! Instead of scattering validation checks throughout your code:
//!
//! ```rust,ignore
//! fn process_user(name: String, age: i32) -> Result<User, Error> {
//!     if name.is_empty() {
//!         return Err(Error::EmptyName);
//!     }
//!     if age <= 0 {
//!         return Err(Error::InvalidAge);
//!     }
//!     // What if we call another function that needs these checks?
//! }
//! ```
//!
//! Use refined types to encode invariants in the type system:
//!
//! ```rust,ignore
//! use stillwater::refined::{Refined, NonEmpty, Positive};
//!
//! type NonEmptyString = Refined<String, NonEmpty>;
//! type PositiveI32 = Refined<i32, Positive>;
//!
//! fn process_user(name: NonEmptyString, age: PositiveI32) -> User {
//!     // name is GUARANTEED non-empty by construction
//!     // age is GUARANTEED positive by construction
//!     // No runtime checks needed!
//! }
//! ```
//!
//! # Quick Start
//!
//! ```rust
//! use stillwater::refined::{Refined, NonEmpty, Positive};
//!
//! // Create refined types
//! type NonEmptyString = Refined<String, NonEmpty>;
//! type PositiveI32 = Refined<i32, Positive>;
//!
//! // Validate at the boundary
//! let name = NonEmptyString::new("Alice".to_string()).unwrap();
//! let age = PositiveI32::new(30).unwrap();
//!
//! // Use freely without checks
//! println!("Hello, {}! Age: {}", name.get(), age.get());
//! ```
//!
//! # Custom Predicates
//!
//! Define your own refinement predicates:
//!
//! ```rust
//! use stillwater::refined::{Refined, Predicate};
//!
//! // Define a predicate for valid email
//! pub struct ValidEmail;
//!
//! impl Predicate<String> for ValidEmail {
//!     type Error = &'static str;
//!
//!     fn check(value: &String) -> Result<(), Self::Error> {
//!         if value.contains('@') && value.contains('.') {
//!             Ok(())
//!         } else {
//!             Err("invalid email format")
//!         }
//!     }
//! }
//!
//! type Email = Refined<String, ValidEmail>;
//!
//! let email = Email::new("user@example.com".to_string()).unwrap();
//! ```
//!
//! # Integration with Validation
//!
//! Use with `Validation::and` for error accumulation:
//!
//! ```rust
//! use stillwater::{Validation, refined::{Refined, NonEmpty, Positive}};
//!
//! type NonEmptyString = Refined<String, NonEmpty>;
//! type PositiveI32 = Refined<i32, Positive>;
//!
//! fn validate_user(name: String, age: i32) -> Validation<(NonEmptyString, PositiveI32), Vec<&'static str>> {
//!     let v1 = NonEmptyString::validate_vec(name);
//!     let v2 = PositiveI32::validate_vec(age);
//!     v1.and(v2)
//! }
//!
//! // All errors collected
//! let result = validate_user("".to_string(), -5);
//! assert!(result.is_failure());
//! ```

mod aliases;
mod combinators;
mod effect;
pub mod predicates;
#[cfg(feature = "serde")]
mod serde_impl;
mod validation;

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

// Re-export core types
pub use aliases::*;
pub use combinators::{And, AndError, Not, NotError, Or, OrError};
pub use effect::{pure_refined, refine};
pub use predicates::collection::{MaxSize, MinSize};
pub use predicates::numeric::{InRange, Negative, NonNegative, NonZero, Positive};
pub use predicates::string::{MaxLength, MinLength, NonEmpty, Trimmed};
pub use validation::{FieldError, RefinedValidationExt, ValidationFieldExt};

/// A predicate that constrains values of type T.
///
/// Predicates are stateless - they only define the check logic.
/// The actual values are stored in [`Refined<T, P>`].
///
/// # Difference from `predicate::Predicate`
///
/// This trait is for *type-level* refinement: the predicate is part of
/// the type, guaranteeing invariants at compile time. The existing
/// `predicate::Predicate` trait is for runtime boolean checks.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::Predicate;
///
/// pub struct Even;
///
/// impl Predicate<i32> for Even {
///     type Error = &'static str;
///
///     fn check(value: &i32) -> Result<(), Self::Error> {
///         if value % 2 == 0 {
///             Ok(())
///         } else {
///             Err("value must be even")
///         }
///     }
/// }
/// ```
pub trait Predicate<T>: Send + Sync + 'static {
    /// Error returned when the predicate fails
    type Error: Send + Sync;

    /// Check if the value satisfies the predicate
    fn check(value: &T) -> Result<(), Self::Error>;

    /// Human-readable description of what this predicate requires
    fn description() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A value of type T that is guaranteed to satisfy predicate P.
///
/// `Refined` provides the "parse, don't validate" pattern:
/// - Validate once at construction
/// - Access freely without runtime checks
/// - Types document invariants
///
/// # Memory Layout
///
/// `Refined<T, P>` has the same memory layout as `T` (zero overhead).
/// The `PhantomData<P>` is zero-sized.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, NonEmpty};
///
/// type NonEmptyString = Refined<String, NonEmpty>;
///
/// fn greet(name: NonEmptyString) {
///     // name is guaranteed non-empty - no check needed!
///     println!("Hello, {}!", name.get());
/// }
///
/// // At the boundary, we validate
/// let name = NonEmptyString::new("Alice".to_string()).unwrap();
/// greet(name);
/// ```
pub struct Refined<T, P: Predicate<T>> {
    value: T,
    _predicate: PhantomData<P>,
}

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Create a new refined value, checking the predicate.
    ///
    /// Returns `Ok(Refined)` if the predicate passes,
    /// `Err(P::Error)` if it fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// let positive = Refined::<i32, Positive>::new(42);
    /// assert!(positive.is_ok());
    ///
    /// let not_positive = Refined::<i32, Positive>::new(-5);
    /// assert!(not_positive.is_err());
    /// ```
    pub fn new(value: T) -> Result<Self, P::Error> {
        P::check(&value)?;
        Ok(Self {
            value,
            _predicate: PhantomData,
        })
    }

    /// Get a reference to the inner value.
    ///
    /// This is zero-cost - no runtime check.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// let n = Refined::<i32, Positive>::new(42).unwrap();
    /// assert_eq!(*n.get(), 42);
    /// ```
    #[inline]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Consume the refined value, returning the inner value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// let n = Refined::<i32, Positive>::new(42).unwrap();
    /// let inner: i32 = n.into_inner();
    /// assert_eq!(inner, 42);
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Create a refined value without checking the predicate.
    ///
    /// # Safety (Logical)
    ///
    /// This is not unsafe in the Rust memory sense, but it bypasses
    /// the predicate check. The caller must guarantee that the
    /// predicate would pass. Use this for:
    /// - Deserializing known-valid data
    /// - Performance-critical code where validity is proven elsewhere
    /// - Internal code after transformation that preserves invariants
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// // Only use when you KNOW the value is valid
    /// let n = Refined::<i32, Positive>::new_unchecked(42);
    /// assert_eq!(*n.get(), 42);
    /// ```
    #[inline]
    pub fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _predicate: PhantomData,
        }
    }

    /// Map the inner value, re-checking the predicate.
    ///
    /// Returns `Err` if the new value doesn't satisfy the predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::refined::{Refined, Positive};
    ///
    /// let n = Refined::<i32, Positive>::new(42).unwrap();
    /// let doubled = n.try_map(|x| x * 2);
    /// assert!(doubled.is_ok());
    ///
    /// let negated = Refined::<i32, Positive>::new(5).unwrap().try_map(|x| -x);
    /// assert!(negated.is_err());
    /// ```
    pub fn try_map<F>(self, f: F) -> Result<Self, P::Error>
    where
        F: FnOnce(T) -> T,
    {
        Self::new(f(self.value))
    }
}

// Debug implementation
impl<T: fmt::Debug, P: Predicate<T>> fmt::Debug for Refined<T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Refined")
            .field("value", &self.value)
            .field("predicate", &std::any::type_name::<P>())
            .finish()
    }
}

// Clone when T: Clone
impl<T: Clone, P: Predicate<T>> Clone for Refined<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _predicate: PhantomData,
        }
    }
}

// PartialEq delegates to inner
impl<T: PartialEq, P: Predicate<T>> PartialEq for Refined<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P: Predicate<T>> Eq for Refined<T, P> {}

// PartialOrd delegates to inner
impl<T: PartialOrd, P: Predicate<T>> PartialOrd for Refined<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

// Ord delegates to inner
impl<T: Ord, P: Predicate<T>> Ord for Refined<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

// Hash delegates to inner
impl<T: Hash, P: Predicate<T>> Hash for Refined<T, P> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

// AsRef for transparent access
impl<T, P: Predicate<T>> AsRef<T> for Refined<T, P> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// Deref for ergonomic access
impl<T, P: Predicate<T>> std::ops::Deref for Refined<T, P> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

// Display when T: Display
impl<T: fmt::Display, P: Predicate<T>> fmt::Display for Refined<T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Custom test predicate
    struct Even;

    impl Predicate<i32> for Even {
        type Error = &'static str;

        fn check(value: &i32) -> Result<(), Self::Error> {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err("value must be even")
            }
        }
    }

    type EvenI32 = Refined<i32, Even>;

    #[test]
    fn test_new_success() {
        let result = EvenI32::new(42);
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().get(), 42);
    }

    #[test]
    fn test_new_failure() {
        let result = EvenI32::new(41);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "value must be even");
    }

    #[test]
    fn test_get() {
        let n = EvenI32::new(42).unwrap();
        assert_eq!(*n.get(), 42);
    }

    #[test]
    fn test_into_inner() {
        let n = EvenI32::new(42).unwrap();
        assert_eq!(n.into_inner(), 42);
    }

    #[test]
    fn test_new_unchecked() {
        // Can create with unchecked (even invalid values)
        let n = EvenI32::new_unchecked(41);
        assert_eq!(*n.get(), 41);
    }

    #[test]
    fn test_try_map_success() {
        let n = EvenI32::new(42).unwrap();
        let doubled = n.try_map(|x| x * 2);
        assert!(doubled.is_ok());
        assert_eq!(*doubled.unwrap().get(), 84);
    }

    #[test]
    fn test_try_map_failure() {
        let n = EvenI32::new(42).unwrap();
        let odd = n.try_map(|x| x + 1);
        assert!(odd.is_err());
    }

    #[test]
    fn test_clone() {
        let n = EvenI32::new(42).unwrap();
        let cloned = n.clone();
        assert_eq!(*n.get(), *cloned.get());
    }

    #[test]
    fn test_partial_eq() {
        let a = EvenI32::new(42).unwrap();
        let b = EvenI32::new(42).unwrap();
        let c = EvenI32::new(44).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_ord() {
        let a = EvenI32::new(42).unwrap();
        let b = EvenI32::new(44).unwrap();
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(EvenI32::new(42).unwrap());
        set.insert(EvenI32::new(42).unwrap()); // duplicate
        set.insert(EvenI32::new(44).unwrap());

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_as_ref() {
        let n = EvenI32::new(42).unwrap();
        let r: &i32 = n.as_ref();
        assert_eq!(*r, 42);
    }

    #[test]
    fn test_deref() {
        let n = EvenI32::new(42).unwrap();
        // Deref allows direct access
        assert_eq!(*n, 42);
    }

    #[test]
    fn test_display() {
        let n = EvenI32::new(42).unwrap();
        assert_eq!(format!("{}", n), "42");
    }

    #[test]
    fn test_debug() {
        let n = EvenI32::new(42).unwrap();
        let debug = format!("{:?}", n);
        assert!(debug.contains("Refined"));
        assert!(debug.contains("42"));
    }
}
