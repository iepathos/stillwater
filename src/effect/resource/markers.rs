//! Resource kind markers for compile-time resource tracking.
//!
//! This module defines the `ResourceKind` trait for marking resource types,
//! along with common resource markers for file handles, database connections,
//! locks, transactions, and network sockets.
//!
//! All marker types are zero-sized, incurring no runtime overhead.
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::resource::*;
//!
//! // Define a custom resource kind
//! pub struct MyCustomRes;
//! impl ResourceKind for MyCustomRes {
//!     const NAME: &'static str = "MyCustom";
//! }
//! ```

/// Marker trait for resource kinds.
///
/// Implement this trait to define custom resource types for compile-time tracking.
/// All resource kinds are zero-sized types that exist only at the type level.
///
/// The `NAME` constant provides a human-readable name for error messages
/// and debugging purposes.
///
/// # Requirements
///
/// Implementors must be `Send + Sync + 'static` to ensure resource tracking
/// works across async boundaries.
pub trait ResourceKind: Send + Sync + 'static {
    /// Human-readable name for error messages and debugging.
    const NAME: &'static str;
}

/// File handle resource marker.
///
/// Use this to track file handle acquisition and release.
///
/// # Example
///
/// ```rust,ignore
/// fn open_file(path: &str) -> impl ResourceEffect<Acquires = Has<FileRes>> {
///     // ... opens file and marks acquisition
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileRes;

impl ResourceKind for FileRes {
    const NAME: &'static str = "File";
}

/// Database connection resource marker.
///
/// Use this to track database connection pool acquisitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DbRes;

impl ResourceKind for DbRes {
    const NAME: &'static str = "Database";
}

/// Lock/mutex resource marker.
///
/// Use this to track lock acquisitions and ensure proper release.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LockRes;

impl ResourceKind for LockRes {
    const NAME: &'static str = "Lock";
}

/// Transaction resource marker.
///
/// Use this to enforce transaction protocols (begin → operations → commit/rollback).
///
/// # Example
///
/// ```rust,ignore
/// fn begin_tx() -> impl ResourceEffect<Acquires = Has<TxRes>> { ... }
/// fn commit(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
/// fn rollback(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxRes;

impl ResourceKind for TxRes {
    const NAME: &'static str = "Transaction";
}

/// Network socket resource marker.
///
/// Use this to track socket connections and ensure proper cleanup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SocketRes;

impl ResourceKind for SocketRes {
    const NAME: &'static str = "Socket";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_kind_names() {
        assert_eq!(FileRes::NAME, "File");
        assert_eq!(DbRes::NAME, "Database");
        assert_eq!(LockRes::NAME, "Lock");
        assert_eq!(TxRes::NAME, "Transaction");
        assert_eq!(SocketRes::NAME, "Socket");
    }

    #[test]
    fn custom_resource_kind() {
        struct CustomRes;
        impl ResourceKind for CustomRes {
            const NAME: &'static str = "Custom";
        }
        assert_eq!(CustomRes::NAME, "Custom");
    }

    #[test]
    fn resource_markers_are_zero_sized() {
        assert_eq!(std::mem::size_of::<FileRes>(), 0);
        assert_eq!(std::mem::size_of::<DbRes>(), 0);
        assert_eq!(std::mem::size_of::<LockRes>(), 0);
        assert_eq!(std::mem::size_of::<TxRes>(), 0);
        assert_eq!(std::mem::size_of::<SocketRes>(), 0);
    }

    #[test]
    fn resource_markers_implement_debug() {
        // Verify Debug is implemented
        let _ = format!("{:?}", FileRes);
        let _ = format!("{:?}", DbRes);
        let _ = format!("{:?}", LockRes);
        let _ = format!("{:?}", TxRes);
        let _ = format!("{:?}", SocketRes);
    }
}
