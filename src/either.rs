//! A semantically neutral sum type for representing one of two possible values.
//!
//! # Either vs Result
//!
//! `Either<L, R>` is a general-purpose sum type with no inherent success/failure semantics.
//! Unlike `Result`, neither variant implies an error condition.
//!
//! Use `Either` instead of `Result` when:
//! - Neither variant represents an error (e.g., cached vs fresh data)
//! - You need a sum type without error semantics
//! - You want to preserve both "sides" of a computation
//!
//! Use `Result` when:
//! - One variant clearly represents failure/error
//! - You want to use `?` operator for early returns
//!
//! # Right-Biased Convention
//!
//! By convention, `Either` is "right-biased": methods like `map` and `and_then`
//! operate on the `Right` variant. This matches the common FP convention where
//! `Right` is the "happy path" in computations.
//!
//! # Examples
//!
//! ```rust
//! use stillwater::Either;
//!
//! // Representing two valid data sources
//! fn get_data(from_cache: bool) -> Either<String, i32> {
//!     if from_cache {
//!         Either::left("cached".to_string())
//!     } else {
//!         Either::right(42)
//!     }
//! }
//!
//! // Process based on source
//! let data = get_data(true);
//! let description = data.fold(
//!     |cached| format!("From cache: {}", cached),
//!     |fresh| format!("Fresh value: {}", fresh),
//! );
//! assert_eq!(description, "From cache: cached");
//! ```

use crate::Validation;

/// A value that is either `Left(L)` or `Right(R)`.
///
/// `Either` is a general-purpose sum type with no inherent success/failure semantics.
/// Unlike `Result`, neither variant implies an error condition.
///
/// By convention, `Either` is "right-biased": methods like `map` and `and_then`
/// operate on the `Right` variant. This matches the common FP convention where
/// `Right` is the "happy path" in computations.
///
/// # When to Use
///
/// Use `Either` instead of `Result` when:
/// - Neither variant represents an error (e.g., cached vs fresh data)
/// - You need a sum type without error semantics
/// - You want to preserve both "sides" of a computation
///
/// Use `Result` when:
/// - One variant clearly represents failure/error
/// - You want to use `?` operator for early returns
///
/// # Example
///
/// ```rust
/// use stillwater::Either;
///
/// // Representing two valid outcomes
/// let left: Either<i32, &str> = Either::left(42);
/// let right: Either<i32, &str> = Either::right("hello");
///
/// // Pattern matching works naturally
/// match left {
///     Either::Left(n) => println!("Got left: {}", n),
///     Either::Right(s) => println!("Got right: {}", s),
/// }
///
/// // Fold to process both variants
/// let result = right.fold(
///     |n| format!("number: {}", n),
///     |s| format!("string: {}", s),
/// );
/// assert_eq!(result, "string: hello");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Either<L, R> {
    /// The left variant
    Left(L),
    /// The right variant
    Right(R),
}

impl<L, R> Either<L, R> {
    // ========== Constructors ==========

    /// Create a Left value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let e: Either<i32, &str> = Either::left(42);
    /// assert!(e.is_left());
    /// ```
    #[inline]
    pub fn left(value: L) -> Self {
        Either::Left(value)
    }

    /// Create a Right value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let e: Either<i32, &str> = Either::right("hello");
    /// assert!(e.is_right());
    /// ```
    #[inline]
    pub fn right(value: R) -> Self {
        Either::Right(value)
    }

    // ========== Predicates ==========

    /// Returns `true` if this is a `Left` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert!(left.is_left());
    /// assert!(!right.is_left());
    /// ```
    #[inline]
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Returns `true` if this is a `Right` value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert!(!left.is_right());
    /// assert!(right.is_right());
    /// ```
    #[inline]
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

    // ========== Extractors ==========

    /// Returns the left value if present, consuming self.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.into_left(), Some(42));
    /// assert_eq!(right.into_left(), None);
    /// ```
    #[inline]
    pub fn into_left(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    /// Returns the right value if present, consuming self.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.into_right(), None);
    /// assert_eq!(right.into_right(), Some("hello"));
    /// ```
    #[inline]
    pub fn into_right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(r) => Some(r),
        }
    }

    /// Convert to `Either<&L, &R>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let e: Either<i32, String> = Either::left(42);
    /// let e_ref: Either<&i32, &String> = e.as_ref();
    /// assert_eq!(e_ref, Either::left(&42));
    /// ```
    #[inline]
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Convert to `Either<&mut L, &mut R>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let mut e: Either<i32, String> = Either::left(42);
    /// if let Either::Left(l) = e.as_mut() {
    ///     *l = 100;
    /// }
    /// assert_eq!(e, Either::left(100));
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    // ========== Transformations ==========

    /// Transform the left value, passing right values through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(21);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.map_left(|x| x * 2), Either::left(42));
    /// assert_eq!(right.map_left(|x| x * 2), Either::right("hello"));
    /// ```
    #[inline]
    pub fn map_left<L2, F>(self, f: F) -> Either<L2, R>
    where
        F: FnOnce(L) -> L2,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Transform the right value, passing left values through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<&str, i32> = Either::left("hello");
    /// let right: Either<&str, i32> = Either::right(21);
    ///
    /// assert_eq!(left.map_right(|x| x * 2), Either::left("hello"));
    /// assert_eq!(right.map_right(|x| x * 2), Either::right(42));
    /// ```
    #[inline]
    pub fn map_right<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> R2,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }

    /// Transform the right value (right-biased `map`).
    ///
    /// This is an alias for `map_right` following the right-biased convention.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let e: Either<&str, i32> = Either::right(21);
    /// assert_eq!(e.map(|x| x * 2), Either::right(42));
    /// ```
    #[inline]
    pub fn map<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> R2,
    {
        self.map_right(f)
    }

    /// Transform both variants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(1);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.bimap(|x| x + 1, |s| s.len()), Either::left(2));
    /// assert_eq!(right.bimap(|x| x + 1, |s| s.len()), Either::right(5));
    /// ```
    #[inline]
    pub fn bimap<L2, R2, F, G>(self, f: F, g: G) -> Either<L2, R2>
    where
        F: FnOnce(L) -> L2,
        G: FnOnce(R) -> R2,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(g(r)),
        }
    }

    /// Swap Left and Right.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.swap(), Either::right(42));
    /// assert_eq!(right.swap(), Either::left("hello"));
    /// ```
    #[inline]
    pub fn swap(self) -> Either<R, L> {
        match self {
            Either::Left(l) => Either::Right(l),
            Either::Right(r) => Either::Left(r),
        }
    }

    // ========== Folding ==========

    /// Fold both variants into a single value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.fold(|x| x.to_string(), |s| s.to_string()), "42");
    /// assert_eq!(right.fold(|x| x.to_string(), |s| s.to_string()), "hello");
    /// ```
    #[inline]
    pub fn fold<T, F, G>(self, left_fn: F, right_fn: G) -> T
    where
        F: FnOnce(L) -> T,
        G: FnOnce(R) -> T,
    {
        match self {
            Either::Left(l) => left_fn(l),
            Either::Right(r) => right_fn(r),
        }
    }

    /// Extract the left value, panicking if Right.
    ///
    /// # Panics
    ///
    /// Panics if the value is a `Right`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// assert_eq!(left.unwrap_left(), 42);
    /// ```
    #[inline]
    pub fn unwrap_left(self) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => panic!("called `Either::unwrap_left()` on a `Right` value"),
        }
    }

    /// Extract the right value, panicking if Left.
    ///
    /// # Panics
    ///
    /// Panics if the value is a `Left`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let right: Either<i32, &str> = Either::right("hello");
    /// assert_eq!(right.unwrap_right(), "hello");
    /// ```
    #[inline]
    pub fn unwrap_right(self) -> R {
        match self {
            Either::Left(_) => panic!("called `Either::unwrap_right()` on a `Left` value"),
            Either::Right(r) => r,
        }
    }

    /// Extract the left value with a custom panic message.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the value is a `Right`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// assert_eq!(left.expect_left("expected left"), 42);
    /// ```
    #[inline]
    pub fn expect_left(self, msg: &str) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => panic!("{}", msg),
        }
    }

    /// Extract the right value with a custom panic message.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the value is a `Left`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let right: Either<i32, &str> = Either::right("hello");
    /// assert_eq!(right.expect_right("expected right"), "hello");
    /// ```
    #[inline]
    pub fn expect_right(self, msg: &str) -> R {
        match self {
            Either::Left(_) => panic!("{}", msg),
            Either::Right(r) => r,
        }
    }

    /// Return the left value or a default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.left_or(0), 42);
    /// assert_eq!(right.left_or(0), 0);
    /// ```
    #[inline]
    pub fn left_or(self, default: L) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => default,
        }
    }

    /// Return the right value or a default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.right_or("default"), "default");
    /// assert_eq!(right.right_or("default"), "hello");
    /// ```
    #[inline]
    pub fn right_or(self, default: R) -> R {
        match self {
            Either::Left(_) => default,
            Either::Right(r) => r,
        }
    }

    /// Return the left value or compute it from the right.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(42);
    /// let right: Either<i32, &str> = Either::right("hello");
    ///
    /// assert_eq!(left.left_or_else(|s| s.len() as i32), 42);
    /// assert_eq!(right.left_or_else(|s| s.len() as i32), 5);
    /// ```
    #[inline]
    pub fn left_or_else<F>(self, f: F) -> L
    where
        F: FnOnce(R) -> L,
    {
        match self {
            Either::Left(l) => l,
            Either::Right(r) => f(r),
        }
    }

    /// Return the right value or compute it from the left.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, String> = Either::left(42);
    /// let right: Either<i32, String> = Either::right("hello".to_string());
    ///
    /// assert_eq!(left.right_or_else(|n| n.to_string()), "42");
    /// assert_eq!(right.right_or_else(|n| n.to_string()), "hello");
    /// ```
    #[inline]
    pub fn right_or_else<F>(self, f: F) -> R
    where
        F: FnOnce(L) -> R,
    {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => r,
        }
    }

    // ========== Monadic Operations (Right-Biased) ==========

    /// Chain a computation on the right value (right-biased flatMap).
    ///
    /// If this is a `Right`, applies `f` to the value. If this is a `Left`,
    /// passes the left value through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let right: Either<&str, i32> = Either::right(21);
    /// let left: Either<&str, i32> = Either::left("error");
    ///
    /// assert_eq!(right.and_then(|x| Either::right(x * 2)), Either::right(42));
    /// assert_eq!(left.and_then(|x| Either::right(x * 2)), Either::left("error"));
    /// ```
    #[inline]
    pub fn and_then<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> Either<L, R2>,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => f(r),
        }
    }

    /// Chain a computation on the left value.
    ///
    /// If this is a `Left`, applies `f` to the value. If this is a `Right`,
    /// passes the right value through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let left: Either<i32, &str> = Either::left(1);
    /// let right: Either<i32, &str> = Either::right("ok");
    ///
    /// assert_eq!(left.or_else(|_| Either::<i32, &str>::right("recovered")), Either::right("recovered"));
    /// assert_eq!(right.or_else(|x| Either::<i32, &str>::left(x * 2)), Either::right("ok"));
    /// ```
    #[inline]
    pub fn or_else<L2, F>(self, f: F) -> Either<L2, R>
    where
        F: FnOnce(L) -> Either<L2, R>,
    {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    // ========== Conversions ==========

    /// Convert to Result (Right becomes Ok, Left becomes Err).
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let right: Either<&str, i32> = Either::right(42);
    /// let left: Either<&str, i32> = Either::left("error");
    ///
    /// assert_eq!(right.into_result(), Ok(42));
    /// assert_eq!(left.into_result(), Err("error"));
    /// ```
    #[inline]
    pub fn into_result(self) -> Result<R, L> {
        match self {
            Either::Left(l) => Err(l),
            Either::Right(r) => Ok(r),
        }
    }

    /// Create from Result (Ok becomes Right, Err becomes Left).
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let ok: Either<&str, i32> = Either::from_result(Ok(42));
    /// let err: Either<&str, i32> = Either::from_result(Err("error"));
    ///
    /// assert_eq!(ok, Either::right(42));
    /// assert_eq!(err, Either::left("error"));
    /// ```
    #[inline]
    pub fn from_result(result: Result<R, L>) -> Self {
        match result {
            Ok(r) => Either::Right(r),
            Err(l) => Either::Left(l),
        }
    }

    /// Convert to Validation (Right becomes Success, Left becomes Failure).
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::{Either, Validation};
    ///
    /// let right: Either<&str, i32> = Either::right(42);
    /// let left: Either<&str, i32> = Either::left("error");
    ///
    /// assert_eq!(right.into_validation(), Validation::Success(42));
    /// assert_eq!(left.into_validation(), Validation::Failure("error"));
    /// ```
    #[inline]
    pub fn into_validation(self) -> Validation<R, L> {
        match self {
            Either::Left(l) => Validation::Failure(l),
            Either::Right(r) => Validation::Success(r),
        }
    }

    // ========== Iterator Support ==========

    /// Returns an iterator over the right value, if present.
    ///
    /// This is right-biased: only `Right` values yield an element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let right: Either<&str, i32> = Either::right(42);
    /// let left: Either<&str, i32> = Either::left("error");
    ///
    /// assert_eq!(right.iter().collect::<Vec<_>>(), vec![&42]);
    /// assert_eq!(left.iter().collect::<Vec<_>>(), Vec::<&i32>::new());
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &R> {
        self.as_ref().into_right().into_iter()
    }

    /// Returns a mutable iterator over the right value, if present.
    ///
    /// This is right-biased: only `Right` values yield an element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let mut right: Either<&str, i32> = Either::right(42);
    /// for val in right.iter_mut() {
    ///     *val *= 2;
    /// }
    /// assert_eq!(right, Either::right(84));
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut R> {
        self.as_mut().into_right().into_iter()
    }
}

// Flatten for nested Either
impl<L, R> Either<L, Either<L, R>> {
    /// Flatten a nested Either.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Either;
    ///
    /// let nested: Either<&str, Either<&str, i32>> = Either::right(Either::right(42));
    /// assert_eq!(nested.flatten(), Either::right(42));
    ///
    /// let inner_left: Either<&str, Either<&str, i32>> = Either::right(Either::left("inner"));
    /// assert_eq!(inner_left.flatten(), Either::left("inner"));
    ///
    /// let outer_left: Either<&str, Either<&str, i32>> = Either::left("outer");
    /// assert_eq!(outer_left.flatten(), Either::left("outer"));
    /// ```
    #[inline]
    pub fn flatten(self) -> Either<L, R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(inner) => inner,
        }
    }
}

// ========== Trait Implementations ==========

impl<L, R> From<Result<R, L>> for Either<L, R> {
    fn from(result: Result<R, L>) -> Self {
        Either::from_result(result)
    }
}

impl<L, R> From<Either<L, R>> for Result<R, L> {
    fn from(either: Either<L, R>) -> Self {
        either.into_result()
    }
}

impl<L, R> Default for Either<L, R>
where
    R: Default,
{
    /// Returns `Either::Right(R::default())`.
    fn default() -> Self {
        Either::Right(R::default())
    }
}

impl<L, R> IntoIterator for Either<L, R> {
    type Item = R;
    type IntoIter = std::option::IntoIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_right().into_iter()
    }
}

impl<'a, L, R> IntoIterator for &'a Either<L, R> {
    type Item = &'a R;
    type IntoIter = std::option::IntoIter<&'a R>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().into_right().into_iter()
    }
}

// ========== Collection Utilities ==========

/// Partition an iterator of Either into two vectors.
///
/// # Example
///
/// ```rust
/// use stillwater::either::{Either, partition};
///
/// let items = vec![
///     Either::left(1),
///     Either::right("a"),
///     Either::left(2),
///     Either::right("b"),
/// ];
///
/// let (lefts, rights) = partition(items);
/// assert_eq!(lefts, vec![1, 2]);
/// assert_eq!(rights, vec!["a", "b"]);
/// ```
pub fn partition<L, R, I>(iter: I) -> (Vec<L>, Vec<R>)
where
    I: IntoIterator<Item = Either<L, R>>,
{
    let mut lefts = Vec::new();
    let mut rights = Vec::new();

    for item in iter {
        match item {
            Either::Left(l) => lefts.push(l),
            Either::Right(r) => rights.push(r),
        }
    }

    (lefts, rights)
}

/// Extract all Left values from an iterator.
///
/// # Example
///
/// ```rust
/// use stillwater::either::{Either, lefts};
///
/// let items = vec![
///     Either::left(1),
///     Either::right("a"),
///     Either::left(2),
/// ];
///
/// let left_values: Vec<_> = lefts(items).collect();
/// assert_eq!(left_values, vec![1, 2]);
/// ```
pub fn lefts<L, R, I>(iter: I) -> impl Iterator<Item = L>
where
    I: IntoIterator<Item = Either<L, R>>,
{
    iter.into_iter().filter_map(|e| e.into_left())
}

/// Extract all Right values from an iterator.
///
/// # Example
///
/// ```rust
/// use stillwater::either::{Either, rights};
///
/// let items = vec![
///     Either::left(1),
///     Either::right("a"),
///     Either::right("b"),
/// ];
///
/// let right_values: Vec<_> = rights(items).collect();
/// assert_eq!(right_values, vec!["a", "b"]);
/// ```
pub fn rights<L, R, I>(iter: I) -> impl Iterator<Item = R>
where
    I: IntoIterator<Item = Either<L, R>>,
{
    iter.into_iter().filter_map(|e| e.into_right())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        assert!(Either::<i32, &str>::left(42).is_left());
        assert!(Either::<i32, &str>::right("hello").is_right());
    }

    #[test]
    fn test_predicates() {
        let left: Either<i32, &str> = Either::left(42);
        let right: Either<i32, &str> = Either::right("hello");

        assert!(left.is_left());
        assert!(!left.is_right());
        assert!(!right.is_left());
        assert!(right.is_right());
    }

    #[test]
    fn test_into_left_into_right() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.into_left(), Some(42));

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.into_right(), Some("hello"));

        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.into_right(), None);

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.into_left(), None);
    }

    #[test]
    fn test_as_ref() {
        let e: Either<i32, String> = Either::left(42);
        let e_ref: Either<&i32, &String> = e.as_ref();
        assert_eq!(e_ref, Either::left(&42));
    }

    #[test]
    fn test_as_mut() {
        let mut e: Either<i32, String> = Either::left(42);
        if let Either::Left(l) = e.as_mut() {
            *l = 100;
        }
        assert_eq!(e, Either::left(100));
    }

    #[test]
    fn test_map_left() {
        let e: Either<i32, &str> = Either::left(21);
        assert_eq!(e.map_left(|x| x * 2), Either::left(42));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.map_left(|x| x * 2), Either::right("hello"));
    }

    #[test]
    fn test_map_right() {
        let e: Either<i32, i32> = Either::right(21);
        assert_eq!(e.map_right(|x| x * 2), Either::right(42));

        let e: Either<i32, i32> = Either::left(100);
        assert_eq!(e.map_right(|x| x * 2), Either::left(100));
    }

    #[test]
    fn test_map() {
        let e: Either<&str, i32> = Either::right(21);
        assert_eq!(e.map(|x| x * 2), Either::right(42));
    }

    #[test]
    fn test_bimap() {
        let e: Either<i32, &str> = Either::left(1);
        assert_eq!(e.bimap(|x| x + 1, |s| s.len()), Either::left(2));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.bimap(|x| x + 1, |s| s.len()), Either::right(5));
    }

    #[test]
    fn test_swap() {
        let e: Either<i32, &str> = Either::left(42);
        assert_eq!(e.swap(), Either::right(42));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.swap(), Either::left("hello"));
    }

    #[test]
    fn test_fold() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.fold(|x| x.to_string(), |s| s.to_string()), "42");

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.fold(|x| x.to_string(), |s| s.to_string()), "hello");
    }

    #[test]
    fn test_unwrap_left() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.unwrap_left(), 42);
    }

    #[test]
    #[should_panic(expected = "called `Either::unwrap_left()` on a `Right` value")]
    fn test_unwrap_left_panics() {
        let right: Either<i32, &str> = Either::right("hello");
        right.unwrap_left();
    }

    #[test]
    fn test_unwrap_right() {
        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.unwrap_right(), "hello");
    }

    #[test]
    #[should_panic(expected = "called `Either::unwrap_right()` on a `Left` value")]
    fn test_unwrap_right_panics() {
        let left: Either<i32, &str> = Either::left(42);
        left.unwrap_right();
    }

    #[test]
    fn test_expect_left() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.expect_left("expected left"), 42);
    }

    #[test]
    #[should_panic(expected = "expected left value")]
    fn test_expect_left_panics() {
        let right: Either<i32, &str> = Either::right("hello");
        right.expect_left("expected left value");
    }

    #[test]
    fn test_expect_right() {
        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.expect_right("expected right"), "hello");
    }

    #[test]
    #[should_panic(expected = "expected right value")]
    fn test_expect_right_panics() {
        let left: Either<i32, &str> = Either::left(42);
        left.expect_right("expected right value");
    }

    #[test]
    fn test_left_or() {
        let left: Either<i32, &str> = Either::left(42);
        let right: Either<i32, &str> = Either::right("hello");

        assert_eq!(left.left_or(0), 42);
        assert_eq!(right.left_or(0), 0);
    }

    #[test]
    fn test_right_or() {
        let left: Either<i32, &str> = Either::left(42);
        let right: Either<i32, &str> = Either::right("hello");

        assert_eq!(left.right_or("default"), "default");
        assert_eq!(right.right_or("default"), "hello");
    }

    #[test]
    fn test_left_or_else() {
        let left: Either<i32, &str> = Either::left(42);
        let right: Either<i32, &str> = Either::right("hello");

        assert_eq!(left.left_or_else(|s| s.len() as i32), 42);
        assert_eq!(right.left_or_else(|s| s.len() as i32), 5);
    }

    #[test]
    fn test_right_or_else() {
        let left: Either<i32, String> = Either::left(42);
        let right: Either<i32, String> = Either::right("hello".to_string());

        assert_eq!(left.right_or_else(|n| n.to_string()), "42");
        assert_eq!(right.right_or_else(|n| n.to_string()), "hello");
    }

    #[test]
    fn test_and_then() {
        let e: Either<&str, i32> = Either::right(21);
        assert_eq!(e.and_then(|x| Either::right(x * 2)), Either::right(42));

        let e: Either<&str, i32> = Either::left("error");
        assert_eq!(e.and_then(|x| Either::right(x * 2)), Either::left("error"));
    }

    #[test]
    fn test_or_else() {
        let e: Either<i32, &str> = Either::left(1);
        assert_eq!(
            e.or_else(|_| Either::<i32, &str>::right("recovered")),
            Either::right("recovered")
        );

        let e: Either<i32, &str> = Either::right("ok");
        assert_eq!(
            e.or_else(|x| Either::<i32, &str>::left(x * 2)),
            Either::right("ok")
        );
    }

    #[test]
    fn test_result_conversion() {
        let ok: Result<i32, &str> = Ok(42);
        let either: Either<&str, i32> = ok.into();
        assert_eq!(either, Either::right(42));

        let err: Result<i32, &str> = Err("error");
        let either: Either<&str, i32> = err.into();
        assert_eq!(either, Either::left("error"));

        // Round-trip
        let original: Either<&str, i32> = Either::right(42);
        let result: Result<i32, &str> = original.into();
        let back: Either<&str, i32> = result.into();
        assert_eq!(back, Either::right(42));
    }

    #[test]
    fn test_into_validation() {
        let right: Either<&str, i32> = Either::right(42);
        assert_eq!(right.into_validation(), Validation::Success(42));

        let left: Either<&str, i32> = Either::left("error");
        assert_eq!(left.into_validation(), Validation::Failure("error"));
    }

    #[test]
    fn test_partition() {
        let items = vec![
            Either::left(1),
            Either::right("a"),
            Either::left(2),
            Either::right("b"),
        ];

        let (lefts, rights) = partition(items);
        assert_eq!(lefts, vec![1, 2]);
        assert_eq!(rights, vec!["a", "b"]);
    }

    #[test]
    fn test_lefts() {
        let items = vec![Either::left(1), Either::right("a"), Either::left(2)];

        let left_values: Vec<_> = lefts(items).collect();
        assert_eq!(left_values, vec![1, 2]);
    }

    #[test]
    fn test_rights() {
        let items = vec![Either::left(1), Either::right("a"), Either::right("b")];

        let right_values: Vec<_> = rights(items).collect();
        assert_eq!(right_values, vec!["a", "b"]);
    }

    #[test]
    fn test_flatten() {
        let nested: Either<&str, Either<&str, i32>> = Either::right(Either::right(42));
        assert_eq!(nested.flatten(), Either::right(42));

        let nested: Either<&str, Either<&str, i32>> = Either::right(Either::left("inner"));
        assert_eq!(nested.flatten(), Either::left("inner"));

        let nested: Either<&str, Either<&str, i32>> = Either::left("outer");
        assert_eq!(nested.flatten(), Either::left("outer"));
    }

    #[test]
    fn test_iter() {
        let right: Either<&str, i32> = Either::right(42);
        let collected: Vec<_> = right.iter().collect();
        assert_eq!(collected, vec![&42]);

        let left: Either<&str, i32> = Either::left("error");
        let collected: Vec<_> = left.iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_iter_mut() {
        let mut right: Either<&str, i32> = Either::right(42);
        for val in right.iter_mut() {
            *val *= 2;
        }
        assert_eq!(right, Either::right(84));

        let mut left: Either<&str, i32> = Either::left("error");
        for val in left.iter_mut() {
            *val *= 2; // This won't run
        }
        assert_eq!(left, Either::left("error"));
    }

    #[test]
    fn test_into_iter() {
        let right: Either<&str, i32> = Either::right(42);
        let collected: Vec<_> = right.into_iter().collect();
        assert_eq!(collected, vec![42]);

        let left: Either<&str, i32> = Either::left("error");
        let collected: Vec<_> = left.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_default() {
        let e: Either<&str, i32> = Either::default();
        assert_eq!(e, Either::right(0));
    }

    // Property-like tests
    #[test]
    fn test_swap_involution() {
        let e: Either<i32, i32> = Either::left(42);
        assert_eq!(e.swap().swap(), e);

        let e: Either<i32, i32> = Either::right(42);
        assert_eq!(e.swap().swap(), e);
    }

    #[test]
    fn test_functor_identity() {
        let e: Either<(), i32> = Either::right(42);
        assert_eq!(e.map(|v| v), Either::right(42));
    }

    #[test]
    fn test_functor_composition() {
        let f = |v: i32| v + 1;
        let g = |v: i32| v * 2;

        let e: Either<(), i32> = Either::right(10);
        assert_eq!(e.map(f).map(g), Either::right(10).map(|v| g(f(v))));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_swap_involution(x: i32) {
            let e: Either<i32, i32> = Either::left(x);
            prop_assert_eq!(e.swap().swap(), e);

            let e: Either<i32, i32> = Either::right(x);
            prop_assert_eq!(e.swap().swap(), e);
        }

        #[test]
        fn prop_functor_identity(x: i32) {
            let e: Either<(), i32> = Either::right(x);
            prop_assert_eq!(e.map(|v| v), Either::right(x));
        }

        #[test]
        fn prop_functor_composition(x: i32) {
            let f = |v: i32| v.wrapping_add(1);
            let g = |v: i32| v.wrapping_mul(2);

            let e: Either<(), i32> = Either::right(x);
            prop_assert_eq!(
                e.map(f).map(g),
                Either::right(x).map(|v| g(f(v)))
            );
        }

        #[test]
        fn prop_result_roundtrip(x: i32) {
            let either: Either<(), i32> = Either::right(x);
            let result: Result<i32, ()> = either.into();
            let back: Either<(), i32> = result.into();
            prop_assert_eq!(back, Either::right(x));
        }

        #[test]
        fn prop_bimap_swap_commutes(x: i32) {
            let f = |v: i32| v.wrapping_add(1);
            let g = |v: i32| v.wrapping_mul(2);

            let e: Either<i32, i32> = Either::left(x);
            prop_assert_eq!(
                e.bimap(f, g).swap(),
                e.swap().bimap(g, f)
            );

            let e: Either<i32, i32> = Either::right(x);
            prop_assert_eq!(
                e.bimap(f, g).swap(),
                e.swap().bimap(g, f)
            );
        }
    }
}
