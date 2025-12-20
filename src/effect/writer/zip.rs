//! WriterZip combinator - combines two independent WriterEffects.

use crate::effect::writer::WriterEffect;
use crate::effect::Effect;
use crate::Semigroup;

/// Combines two WriterEffects, running them sequentially and returning both results.
///
/// Writes from both effects are combined using `Monoid::combine` in left-to-right
/// order, ensuring deterministic, predictable write ordering.
///
/// # Execution Order
///
/// Effects are executed sequentially (first, then second) for simplicity
/// and predictability.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let left = tell_one::<_, String, ()>("left".to_string()).map(|_| 1);
/// let right = tell_one::<_, String, ()>("right".to_string()).map(|_| 2);
///
/// let (result, logs) = left.zip(right).run_writer(&()).await;
/// assert_eq!(result, Ok((1, 2)));
/// assert_eq!(logs, vec!["left".to_string(), "right".to_string()]);
/// # });
/// ```
#[derive(Debug)]
pub struct WriterZip<E1, E2> {
    pub(crate) first: E1,
    pub(crate) second: E2,
}

impl<E1, E2> WriterZip<E1, E2> {
    /// Create a new WriterZip combinator from two effects.
    pub fn new(first: E1, second: E2) -> Self {
        WriterZip { first, second }
    }
}

impl<E1, E2> Effect for WriterZip<E1, E2>
where
    E1: WriterEffect,
    E2: WriterEffect<Error = E1::Error, Env = E1::Env, Writes = E1::Writes>,
{
    type Output = (E1::Output, E2::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let first_result = self.first.run(env).await?;
        let second_result = self.second.run(env).await?;
        Ok((first_result, second_result))
    }
}

impl<E1, E2> WriterEffect for WriterZip<E1, E2>
where
    E1: WriterEffect,
    E1::Writes: Semigroup,
    E2: WriterEffect<Error = E1::Error, Env = E1::Env, Writes = E1::Writes>,
{
    type Writes = E1::Writes;

    async fn run_writer(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result1, writes1) = self.first.run_writer(env).await;

        match result1 {
            Ok(value1) => {
                let (result2, writes2) = self.second.run_writer(env).await;
                let combined_writes = writes1.combine(writes2);

                match result2 {
                    Ok(value2) => (Ok((value1, value2)), combined_writes),
                    Err(e) => (Err(e), combined_writes),
                }
            }
            Err(e) => (Err(e), writes1),
        }
    }
}
