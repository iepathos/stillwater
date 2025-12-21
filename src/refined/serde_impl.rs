//! Serde support for refined types (feature-gated)
//!
//! This module provides `Serialize` and `Deserialize` implementations
//! for [`Refined<T, P>`] when the `serde` feature is enabled.
//!
//! # Example
//!
//! ```rust,ignore
//! use serde::{Deserialize, Serialize};
//! use stillwater::refined::{Refined, NonEmpty, Positive};
//!
//! type NonEmptyString = Refined<String, NonEmpty>;
//! type PositiveI32 = Refined<i32, Positive>;
//!
//! #[derive(Serialize, Deserialize)]
//! struct User {
//!     name: NonEmptyString,  // Validated on deserialize
//!     age: PositiveI32,      // Validated on deserialize
//! }
//!
//! // Deserialization validates automatically
//! let json = r#"{"name": "Alice", "age": 25}"#;
//! let user: User = serde_json::from_str(json).unwrap();
//!
//! // Invalid data fails deserialization
//! let bad_json = r#"{"name": "", "age": 25}"#;
//! let result: Result<User, _> = serde_json::from_str(bad_json);
//! assert!(result.is_err());
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use super::{Predicate, Refined};

impl<T, P> Serialize for Refined<T, P>
where
    T: Serialize,
    P: Predicate<T>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().serialize(serializer)
    }
}

impl<'de, T, P> Deserialize<'de> for Refined<T, P>
where
    T: Deserialize<'de>,
    P: Predicate<T>,
    P::Error: fmt::Display,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = T::deserialize(deserializer)?;
        Refined::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::predicates::numeric::Positive;
    use crate::refined::predicates::string::NonEmpty;

    type NonEmptyString = Refined<String, NonEmpty>;
    type PositiveI32 = Refined<i32, Positive>;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct User {
        name: NonEmptyString,
        age: PositiveI32,
    }

    #[test]
    fn test_serialize() {
        let name = NonEmptyString::new("Alice".to_string()).unwrap();
        let age = PositiveI32::new(25).unwrap();
        let user = User { name, age };

        let json = serde_json::to_string(&user).unwrap();
        assert_eq!(json, r#"{"name":"Alice","age":25}"#);
    }

    #[test]
    fn test_deserialize_success() {
        let json = r#"{"name":"Alice","age":25}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.name.get(), "Alice");
        assert_eq!(*user.age.get(), 25);
    }

    #[test]
    fn test_deserialize_empty_name_fails() {
        let json = r#"{"name":"","age":25}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"));
    }

    #[test]
    fn test_deserialize_negative_age_fails() {
        let json = r#"{"name":"Alice","age":-5}"#;
        let result: Result<User, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("positive"));
    }

    #[test]
    fn test_roundtrip() {
        let name = NonEmptyString::new("Bob".to_string()).unwrap();
        let age = PositiveI32::new(30).unwrap();
        let original = User { name, age };

        let json = serde_json::to_string(&original).unwrap();
        let restored: User = serde_json::from_str(&json).unwrap();

        assert_eq!(original, restored);
    }
}
