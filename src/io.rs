//! IO module for creating Effect instances from I/O operations
//!
//! This module provides convenient helpers for creating Effects from I/O operations,
//! with automatic environment extraction using trait bounds.
//!
//! # Overview
//!
//! The IO module provides four main helpers:
//! - `IO::read()` - Create effects from read-only synchronous operations
//! - `IO::write()` - Create effects from mutating synchronous operations
//! - `IO::read_async()` - Create effects from read-only async operations
//! - `IO::write_async()` - Create effects from mutating async operations
//!
//! # Environment Pattern
//!
//! The IO module uses `AsRef<T>` for automatic environment extraction.
//! This allows you to have a composite environment and extract specific services:
//!
//! ```
//! # use stillwater::IO;
//! # use std::convert::Infallible;
//! struct Database {
//!     users: Vec<String>,
//! }
//!
//! impl Database {
//!     fn find_user(&self, id: u64) -> Option<String> {
//!         self.users.get(id as usize).cloned()
//!     }
//! }
//!
//! struct AppEnv {
//!     db: Database,
//! }
//!
//! impl AsRef<Database> for AppEnv {
//!     fn as_ref(&self) -> &Database {
//!         &self.db
//!     }
//! }
//!
//! # tokio_test::block_on(async {
//! let env = AppEnv {
//!     db: Database {
//!         users: vec!["Alice".to_string()],
//!     },
//! };
//!
//! // Type inference figures out we need Database from AppEnv
//! let effect = IO::read(|db: &Database| db.find_user(0));
//! let result = effect.run(&env).await;
//! assert_eq!(result, Ok(Some("Alice".to_string())));
//! # });
//! ```
//!
//! # Read vs Write
//!
//! The distinction between `read` and `write` is semantic:
//! - **read**: Query operations that don't modify state (e.g., database SELECT, file read)
//! - **write**: Operations that modify state (e.g., database INSERT, file write)
//!
//! Both take `&T` due to Effect's immutable environment design. For true mutation,
//! use interior mutability (Arc<Mutex<T>>, RefCell, etc.):
//!
//! ```
//! # use stillwater::IO;
//! # use std::sync::{Arc, Mutex};
//! # use std::collections::HashMap;
//! struct Cache {
//!     data: Arc<Mutex<HashMap<u64, String>>>,
//! }
//!
//! impl Cache {
//!     fn set(&self, key: u64, value: String) {
//!         self.data.lock().unwrap().insert(key, value);
//!     }
//! }
//!
//! struct Env {
//!     cache: Cache,
//! }
//!
//! impl AsRef<Cache> for Env {
//!     fn as_ref(&self) -> &Cache {
//!         &self.cache
//!     }
//! }
//!
//! # tokio_test::block_on(async {
//! let env = Env {
//!     cache: Cache {
//!         data: Arc::new(Mutex::new(HashMap::new())),
//!     },
//! };
//!
//! let effect = IO::write(|cache: &Cache| {
//!     cache.set(1, "value".to_string());
//! });
//!
//! effect.run(&env).await.unwrap();
//! # });
//! ```

use std::convert::Infallible;
use std::future::Future;

use crate::Effect;

/// Helper for creating I/O effects
///
/// This is a zero-sized type that acts as a namespace for IO helper functions.
/// All methods are static and create Effect instances from I/O operations.
#[derive(Debug, Clone, Copy)]
pub struct IO;

impl IO {
    /// Create an effect from a read-only synchronous operation
    ///
    /// The closure receives an immutable reference to a service extracted from
    /// the environment via `AsRef<T>`. Type inference determines which service
    /// to extract based on the closure parameter type.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The service type (inferred from closure parameter)
    /// - `R`: The return type (inferred from closure return)
    /// - `F`: The closure type (inferred)
    /// - `Env`: The environment type (must implement `AsRef<T>`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use stillwater::IO;
    /// struct Database {
    ///     users: Vec<String>,
    /// }
    ///
    /// impl Database {
    ///     fn count_users(&self) -> usize {
    ///         self.users.len()
    ///     }
    /// }
    ///
    /// struct Env {
    ///     db: Database,
    /// }
    ///
    /// impl AsRef<Database> for Env {
    ///     fn as_ref(&self) -> &Database {
    ///         &self.db
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let env = Env {
    ///     db: Database {
    ///         users: vec!["Alice".to_string(), "Bob".to_string()],
    ///     },
    /// };
    ///
    /// let effect = IO::read(|db: &Database| db.count_users());
    /// let count = effect.run(&env).await;
    /// assert_eq!(count, Ok(2));
    /// # });
    /// ```
    pub fn read<T, R, F, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &Env| Ok(f(env.as_ref())))
    }

    /// Create an effect from a mutating synchronous operation
    ///
    /// Note: Since Effect runs with `&Env`, true mutation requires interior
    /// mutability (RefCell, Mutex, etc.). This helper is semantically distinct
    /// from `read` to indicate intent, but both take `&T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The service type (inferred from closure parameter)
    /// - `R`: The return type (inferred from closure return)
    /// - `F`: The closure type (inferred)
    /// - `Env`: The environment type (must implement `AsRef<T>`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use stillwater::IO;
    /// # use std::sync::{Arc, Mutex};
    /// struct Logger {
    ///     messages: Arc<Mutex<Vec<String>>>,
    /// }
    ///
    /// impl Logger {
    ///     fn log(&self, msg: String) {
    ///         self.messages.lock().unwrap().push(msg);
    ///     }
    /// }
    ///
    /// struct Env {
    ///     logger: Logger,
    /// }
    ///
    /// impl AsRef<Logger> for Env {
    ///     fn as_ref(&self) -> &Logger {
    ///         &self.logger
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let env = Env {
    ///     logger: Logger {
    ///         messages: Arc::new(Mutex::new(Vec::new())),
    ///     },
    /// };
    ///
    /// let effect = IO::write(|logger: &Logger| {
    ///     logger.log("Hello".to_string());
    /// });
    ///
    /// effect.run(&env).await.unwrap();
    /// assert_eq!(env.logger.messages.lock().unwrap().len(), 1);
    /// # });
    /// ```
    pub fn write<T, R, F, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> R + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_fn(move |env: &Env| Ok(f(env.as_ref())))
    }

    /// Create an effect from an async read-only operation
    ///
    /// The closure receives an immutable reference to a service and returns
    /// a Future. This is useful for async I/O operations like network requests
    /// or async database queries.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The service type (inferred from closure parameter)
    /// - `R`: The return type (inferred from Future output)
    /// - `F`: The closure type (inferred)
    /// - `Fut`: The Future type (inferred)
    /// - `Env`: The environment type (must implement `AsRef<T>`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use stillwater::IO;
    /// # use std::future::ready;
    /// struct Database {
    ///     value: String,
    /// }
    ///
    /// struct Env {
    ///     db: Database,
    /// }
    ///
    /// impl AsRef<Database> for Env {
    ///     fn as_ref(&self) -> &Database {
    ///         &self.db
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let env = Env {
    ///     db: Database {
    ///         value: "query result".to_string(),
    ///     },
    /// };
    ///
    /// let effect = IO::read_async(|db: &Database| {
    ///     let result = db.value.clone();
    ///     ready(result)
    /// });
    ///
    /// let result = effect.run(&env).await;
    /// assert!(result.is_ok());
    /// # });
    /// ```
    pub fn read_async<T, R, F, Fut, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_async(move |env: &Env| {
            let fut = f(env.as_ref());
            async move { Ok(fut.await) }
        })
    }

    /// Create an effect from an async mutating operation
    ///
    /// Note: Since Effect runs with `&Env`, true mutation requires interior
    /// mutability (RefCell, Mutex, etc.). This helper is semantically distinct
    /// from `read_async` to indicate intent, but both take `&T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: The service type (inferred from closure parameter)
    /// - `R`: The return type (inferred from Future output)
    /// - `F`: The closure type (inferred)
    /// - `Fut`: The Future type (inferred)
    /// - `Env`: The environment type (must implement `AsRef<T>`)
    ///
    /// # Examples
    ///
    /// ```
    /// # use stillwater::IO;
    /// # use std::sync::{Arc, Mutex};
    /// # use std::future::ready;
    /// struct Cache {
    ///     data: Arc<Mutex<Vec<String>>>,
    /// }
    ///
    /// struct Env {
    ///     cache: Cache,
    /// }
    ///
    /// impl AsRef<Cache> for Env {
    ///     fn as_ref(&self) -> &Cache {
    ///         &self.cache
    ///     }
    /// }
    ///
    /// # tokio_test::block_on(async {
    /// let env = Env {
    ///     cache: Cache {
    ///         data: Arc::new(Mutex::new(Vec::new())),
    ///     },
    /// };
    ///
    /// let effect = IO::write_async(|cache: &Cache| {
    ///     cache.data.lock().unwrap().push("value".to_string());
    ///     ready(())
    /// });
    ///
    /// effect.run(&env).await.unwrap();
    /// assert_eq!(env.cache.data.lock().unwrap().len(), 1);
    /// # });
    /// ```
    pub fn write_async<T, R, F, Fut, Env>(f: F) -> Effect<R, Infallible, Env>
    where
        F: FnOnce(&T) -> Fut + Send + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
        T: Send + Sync + 'static,
        Env: AsRef<T> + Send + Sync + 'static,
    {
        Effect::from_async(move |env: &Env| {
            let fut = f(env.as_ref());
            async move { Ok(fut.await) }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Test read with simple service
    #[tokio::test]
    async fn test_io_read_simple() {
        struct Database {
            value: i32,
        }

        impl Database {
            fn get_value(&self) -> i32 {
                self.value
            }
        }

        struct Env {
            db: Database,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }

        let env = Env {
            db: Database { value: 42 },
        };

        let effect = IO::read(|db: &Database| db.get_value());
        let result = effect.run(&env).await;

        assert_eq!(result, Ok(42));
    }

    // Test read with user data
    #[tokio::test]
    async fn test_io_read_user_data() {
        #[derive(Clone, PartialEq, Debug)]
        struct User {
            id: u64,
            name: String,
        }

        struct Database {
            users: Vec<User>,
        }

        impl Database {
            fn find_user(&self, id: u64) -> Option<User> {
                self.users.iter().find(|u| u.id == id).cloned()
            }
        }

        struct Env {
            db: Database,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }

        let env = Env {
            db: Database {
                users: vec![User {
                    id: 1,
                    name: "Alice".to_string(),
                }],
            },
        };

        let effect = IO::read(|db: &Database| db.find_user(1));
        let result = effect.run(&env).await;

        assert_eq!(
            result,
            Ok(Some(User {
                id: 1,
                name: "Alice".to_string()
            }))
        );
    }

    // Test write with interior mutability
    #[tokio::test]
    async fn test_io_write_with_mutex() {
        struct Logger {
            messages: Arc<Mutex<Vec<String>>>,
        }

        impl Logger {
            fn log(&self, msg: String) {
                self.messages.lock().unwrap().push(msg);
            }
        }

        struct Env {
            logger: Logger,
        }

        impl AsRef<Logger> for Env {
            fn as_ref(&self) -> &Logger {
                &self.logger
            }
        }

        let env = Env {
            logger: Logger {
                messages: Arc::new(Mutex::new(Vec::new())),
            },
        };

        let effect = IO::write(|logger: &Logger| {
            logger.log("Hello".to_string());
        });

        effect.run(&env).await.unwrap();
        assert_eq!(env.logger.messages.lock().unwrap().len(), 1);
        assert_eq!(env.logger.messages.lock().unwrap()[0], "Hello");
    }

    // Test read_async
    #[tokio::test]
    async fn test_io_read_async() {
        use std::future::ready;

        struct Database {
            value: String,
        }

        struct Env {
            db: Database,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }

        let env = Env {
            db: Database {
                value: "Result of: SELECT * FROM users".to_string(),
            },
        };

        let effect = IO::read_async(|db: &Database| {
            let value = db.value.clone();
            ready(value)
        });

        let result = effect.run(&env).await;
        assert_eq!(result, Ok("Result of: SELECT * FROM users".to_string()));
    }

    // Test write_async
    #[tokio::test]
    async fn test_io_write_async() {
        use std::future::ready;

        struct Cache {
            data: Arc<Mutex<Vec<String>>>,
        }

        struct Env {
            cache: Cache,
        }

        impl AsRef<Cache> for Env {
            fn as_ref(&self) -> &Cache {
                &self.cache
            }
        }

        let env = Env {
            cache: Cache {
                data: Arc::new(Mutex::new(Vec::new())),
            },
        };

        let effect = IO::write_async(|cache: &Cache| {
            cache.data.lock().unwrap().push("value".to_string());
            ready(())
        });

        effect.run(&env).await.unwrap();
        assert_eq!(env.cache.data.lock().unwrap().len(), 1);
    }

    // Test multiple services in same environment
    #[tokio::test]
    async fn test_multiple_services() {
        struct Database {
            data: String,
        }
        struct Cache {
            data: String,
        }
        struct Logger {
            data: String,
        }

        struct Env {
            db: Database,
            cache: Cache,
            logger: Logger,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }
        impl AsRef<Cache> for Env {
            fn as_ref(&self) -> &Cache {
                &self.cache
            }
        }
        impl AsRef<Logger> for Env {
            fn as_ref(&self) -> &Logger {
                &self.logger
            }
        }

        let env = Env {
            db: Database {
                data: "db data".to_string(),
            },
            cache: Cache {
                data: "cache data".to_string(),
            },
            logger: Logger {
                data: "logger data".to_string(),
            },
        };

        // Type inference figures out which service to use
        let db_effect = IO::read(|db: &Database| db.data.clone());
        let cache_effect = IO::read(|cache: &Cache| cache.data.clone());
        let logger_effect = IO::read(|logger: &Logger| logger.data.clone());

        assert_eq!(db_effect.run(&env).await, Ok("db data".to_string()));
        assert_eq!(cache_effect.run(&env).await, Ok("cache data".to_string()));
        assert_eq!(logger_effect.run(&env).await, Ok("logger data".to_string()));
    }

    // Test composition with Effect combinators
    #[tokio::test]
    async fn test_composition_with_combinators() {
        struct Database {
            value: i32,
        }

        struct Env {
            db: Database,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }

        let env = Env {
            db: Database { value: 10 },
        };

        let effect = IO::read(|db: &Database| db.value)
            .map(|x| x * 2)
            .and_then(|x| IO::read(move |db: &Database| x + db.value));

        let result = effect.run(&env).await;
        assert_eq!(result, Ok(30)); // (10 * 2) + 10
    }

    // Integration test: cache-aside pattern
    #[tokio::test]
    async fn test_real_world_composition() {
        use std::future::ready;

        struct Database {
            data: HashMap<u64, String>,
        }

        struct Cache {
            data: Arc<Mutex<HashMap<u64, String>>>,
        }

        impl Cache {
            fn get(&self, id: u64) -> Option<String> {
                self.data.lock().unwrap().get(&id).cloned()
            }

            fn set(&self, id: u64, value: String) {
                self.data.lock().unwrap().insert(id, value);
            }
        }

        struct Env {
            db: Database,
            cache: Cache,
        }

        impl AsRef<Database> for Env {
            fn as_ref(&self) -> &Database {
                &self.db
            }
        }

        impl AsRef<Cache> for Env {
            fn as_ref(&self) -> &Cache {
                &self.cache
            }
        }

        let mut db_data = HashMap::new();
        db_data.insert(1, "Alice".to_string());

        let env = Env {
            db: Database { data: db_data },
            cache: Cache {
                data: Arc::new(Mutex::new(HashMap::new())),
            },
        };

        // First call - should hit database
        let effect = IO::read(move |cache: &Cache| cache.get(1)).and_then(|cached| {
            if cached.is_some() {
                Effect::pure(cached)
            } else {
                IO::read_async(|db: &Database| {
                    let value = db.data.get(&1).cloned();
                    ready(value)
                })
                .and_then(move |value| {
                    if let Some(ref v) = value {
                        let v = v.clone();
                        IO::write(move |cache: &Cache| {
                            cache.set(1, v);
                        })
                        .map(move |_| value.clone())
                    } else {
                        Effect::pure(value)
                    }
                })
            }
        });

        let result = effect.run(&env).await;
        assert_eq!(result, Ok(Some("Alice".to_string())));

        // Should be cached now
        let cached = env.cache.get(1);
        assert_eq!(cached, Some("Alice".to_string()));
    }
}
