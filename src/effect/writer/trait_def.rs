//! WriterEffect trait definition.

use std::future::Future;

use crate::effect::Effect;
use crate::Monoid;

/// An effect that accumulates values of type `Writes` alongside computation.
///
/// The `Writes` type must be a `Monoid` to support:
/// - Empty writes (`Monoid::empty()`) for effects that don't write
/// - Combining writes (`Semigroup::combine`) when chaining effects
///
/// # Laws
///
/// WriterEffect extends Effect with additional structure:
///
/// 1. **Identity**: `tell(W::empty())` should be equivalent to `pure(())` with empty writes
/// 2. **Homomorphism**: `tell(a).and_then(|_| tell(b))` accumulates `a.combine(b)`
/// 3. **Associativity**: Writes accumulate left-to-right through chains
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = tell_one::<_, String, ()>("log 1".to_string())
///     .and_then(|_| tell_one("log 2".to_string()));
///
/// let (result, logs) = effect.run_writer(&()).await;
/// assert_eq!(result, Ok(()));
/// assert_eq!(logs, vec!["log 1".to_string(), "log 2".to_string()]);
/// # });
/// ```
pub trait WriterEffect: Effect {
    /// The type of values being accumulated.
    ///
    /// Must implement `Monoid` for identity and combination.
    type Writes: Monoid + Send;

    /// Execute this effect and return both result and accumulated writes.
    ///
    /// This is the primitive operation that all WriterEffect combinators
    /// build upon. Unlike `Effect::run` which only returns the result,
    /// `run_writer` also returns the accumulated writes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::writer::prelude::*;
    /// use stillwater::effect::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = tell::<_, String, ()>(vec!["hello".to_string()]);
    /// let (result, writes) = effect.run_writer(&()).await;
    ///
    /// assert_eq!(result, Ok(()));
    /// assert_eq!(writes, vec!["hello".to_string()]);
    /// # });
    /// ```
    fn run_writer(
        self,
        env: &Self::Env,
    ) -> impl Future<Output = (Result<Self::Output, Self::Error>, Self::Writes)> + Send;
}
