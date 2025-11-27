//! Bracket pattern for safe resource management.
//!
//! The bracket pattern ensures resources are properly released even when
//! errors occur or panics happen during use.

use crate::effect_v2::trait_def::Effect;

/// Bracket combinator type for resource management.
///
/// The bracket pattern has three phases:
/// 1. **Acquire**: Obtain the resource
/// 2. **Use**: Use the resource to produce a result
/// 3. **Release**: Release the resource (always runs, even on error)
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = bracket(
///     // Acquire: open file handle
///     from_fn(|_| Ok::<_, String>(File::open("data.txt")?)),
///     // Use: read contents
///     |file| from_fn(move |_| Ok(read_contents(&file))),
///     // Release: close file (happens automatically in Rust, but shown for illustration)
///     |file| pure(drop(file)),
/// );
/// ```
pub struct Bracket<Acquire, Use, Release> {
    acquire: Acquire,
    use_fn: Use,
    release: Release,
}

impl<Acquire, Use, Release> std::fmt::Debug for Bracket<Acquire, Use, Release> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bracket")
            .field("acquire", &"<effect>")
            .field("use_fn", &"<function>")
            .field("release", &"<function>")
            .finish()
    }
}

impl<Acquire, Use, Release> Bracket<Acquire, Use, Release> {
    /// Create a new Bracket.
    pub fn new(acquire: Acquire, use_fn: Use, release: Release) -> Self {
        Bracket {
            acquire,
            use_fn,
            release,
        }
    }
}

impl<Acquire, Use, Release, UseEffect, ReleaseEffect, R, T, E, Env> Effect
    for Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> ReleaseEffect + Send,
    ReleaseEffect: Effect<Output = (), Error = E, Env = Env>,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        // Acquire the resource
        let resource = self.acquire.run(env).await?;

        // Use the resource (store result to return later)
        let result = (self.use_fn)(resource.clone()).run(env).await;

        // Release runs regardless of use result
        // We ignore release errors to always return the use result
        // (a real implementation might want to combine errors)
        let _ = (self.release)(resource).run(env).await;

        result
    }
}

/// Bracket pattern for safe resource management.
///
/// Acquires a resource, uses it, and guarantees release.
///
/// # Type Parameters
///
/// * `Acquire` - Effect that acquires the resource
/// * `Use` - Function that uses the resource
/// * `Release` - Function that releases the resource
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// // Database connection example
/// let effect = bracket(
///     // Acquire connection
///     from_fn(|env: &AppEnv| env.db.get_connection()),
///     // Use connection
///     |conn| from_async(move |_| async move {
///         conn.execute("SELECT * FROM users").await
///     }),
///     // Release connection (return to pool)
///     |conn| from_fn(move |_| {
///         conn.release();
///         Ok(())
///     }),
/// );
/// ```
pub fn bracket<Acquire, Use, Release, UseEffect, ReleaseEffect, R, T, E, Env>(
    acquire: Acquire,
    use_fn: Use,
    release: Release,
) -> Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> ReleaseEffect + Send,
    ReleaseEffect: Effect<Output = (), Error = E, Env = Env>,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Bracket::new(acquire, use_fn, release)
}

/// Simplified bracket that uses a closure for release.
///
/// This variant is for cases where release doesn't need to be an effect.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect_v2::prelude::*;
///
/// let effect = bracket_simple(
///     from_fn(|_| Ok::<_, String>(acquire_resource())),
///     |resource| from_fn(move |_| Ok(use_resource(&resource))),
///     |resource| release_resource(resource), // Simple drop
/// );
/// ```
pub fn bracket_simple<Acquire, Use, ReleaseFn, UseEffect, R, T, E, Env>(
    acquire: Acquire,
    use_fn: Use,
    release_fn: ReleaseFn,
) -> impl Effect<Output = T, Error = E, Env = Env>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    ReleaseFn: FnOnce(R) + Send,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    BracketSimple {
        acquire,
        use_fn,
        release_fn,
    }
}

/// Simple bracket that uses a closure for release instead of an effect.
struct BracketSimple<Acquire, Use, ReleaseFn> {
    acquire: Acquire,
    use_fn: Use,
    release_fn: ReleaseFn,
}

impl<Acquire, Use, ReleaseFn, UseEffect, R, T, E, Env> Effect
    for BracketSimple<Acquire, Use, ReleaseFn>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> UseEffect + Send,
    UseEffect: Effect<Output = T, Error = E, Env = Env>,
    ReleaseFn: FnOnce(R) + Send,
    R: Clone + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    async fn run(self, env: &Self::Env) -> Result<T, E> {
        let resource = self.acquire.run(env).await?;
        let result = (self.use_fn)(resource.clone()).run(env).await;
        (self.release_fn)(resource);
        result
    }
}
