//! Zip combinator - combines two independent effects into a tuple.

use crate::effect::trait_def::Effect;

/// Combines two effects, running them sequentially and returning both results.
///
/// This is zero-cost: no heap allocation occurs. The `Zip` struct stores
/// both effects inline.
///
/// # Execution Order
///
/// Effects are executed sequentially (first, then second) for simplicity
/// and predictability. Use `zip_par` variants (via `par2`, `par3`, etc.)
/// when concurrent execution is needed.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = pure::<_, String, ()>(1).zip(pure(2));
/// assert_eq!(effect.execute(&()).await, Ok((1, 2)));
/// ```
#[derive(Debug)]
pub struct Zip<E1, E2> {
    pub(crate) first: E1,
    pub(crate) second: E2,
}

impl<E1, E2> Zip<E1, E2> {
    /// Create a new Zip combinator from two effects.
    pub fn new(first: E1, second: E2) -> Self {
        Zip { first, second }
    }
}

impl<E1, E2> Effect for Zip<E1, E2>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
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

/// Combines three effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs. Returns a flat tuple
/// `(T1, T2, T3)` rather than nested `((T1, T2), T3)`.
///
/// # Example
///
/// ```rust,ignore
/// use stillwater::effect::prelude::*;
///
/// let effect = zip3(pure(1), pure(2), pure(3));
/// assert_eq!(effect.execute(&()).await, Ok((1, 2, 3)));
/// ```
#[derive(Debug)]
pub struct Zip3<E1, E2, E3> {
    e1: E1,
    e2: E2,
    e3: E3,
}

impl<E1, E2, E3> Zip3<E1, E2, E3> {
    /// Create a new Zip3 combinator from three effects.
    pub fn new(e1: E1, e2: E2, e3: E3) -> Self {
        Zip3 { e1, e2, e3 }
    }
}

impl<E1, E2, E3> Effect for Zip3<E1, E2, E3>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output, E3::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        Ok((r1, r2, r3))
    }
}

/// Combines four effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs.
#[derive(Debug)]
pub struct Zip4<E1, E2, E3, E4> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
}

impl<E1, E2, E3, E4> Zip4<E1, E2, E3, E4> {
    /// Create a new Zip4 combinator from four effects.
    pub fn new(e1: E1, e2: E2, e3: E3, e4: E4) -> Self {
        Zip4 { e1, e2, e3, e4 }
    }
}

impl<E1, E2, E3, E4> Effect for Zip4<E1, E2, E3, E4>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output, E3::Output, E4::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        let r4 = self.e4.run(env).await?;
        Ok((r1, r2, r3, r4))
    }
}

/// Combines five effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs.
#[derive(Debug)]
pub struct Zip5<E1, E2, E3, E4, E5> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
}

impl<E1, E2, E3, E4, E5> Zip5<E1, E2, E3, E4, E5> {
    /// Create a new Zip5 combinator from five effects.
    pub fn new(e1: E1, e2: E2, e3: E3, e4: E4, e5: E5) -> Self {
        Zip5 { e1, e2, e3, e4, e5 }
    }
}

impl<E1, E2, E3, E4, E5> Effect for Zip5<E1, E2, E3, E4, E5>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (E1::Output, E2::Output, E3::Output, E4::Output, E5::Output);
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        let r4 = self.e4.run(env).await?;
        let r5 = self.e5.run(env).await?;
        Ok((r1, r2, r3, r4, r5))
    }
}

/// Combines six effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs.
#[derive(Debug)]
pub struct Zip6<E1, E2, E3, E4, E5, E6> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
}

impl<E1, E2, E3, E4, E5, E6> Zip6<E1, E2, E3, E4, E5, E6> {
    /// Create a new Zip6 combinator from six effects.
    pub fn new(e1: E1, e2: E2, e3: E3, e4: E4, e5: E5, e6: E6) -> Self {
        Zip6 {
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
        }
    }
}

impl<E1, E2, E3, E4, E5, E6> Effect for Zip6<E1, E2, E3, E4, E5, E6>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (
        E1::Output,
        E2::Output,
        E3::Output,
        E4::Output,
        E5::Output,
        E6::Output,
    );
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        let r4 = self.e4.run(env).await?;
        let r5 = self.e5.run(env).await?;
        let r6 = self.e6.run(env).await?;
        Ok((r1, r2, r3, r4, r5, r6))
    }
}

/// Combines seven effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs.
#[derive(Debug)]
pub struct Zip7<E1, E2, E3, E4, E5, E6, E7> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
    e7: E7,
}

impl<E1, E2, E3, E4, E5, E6, E7> Zip7<E1, E2, E3, E4, E5, E6, E7> {
    /// Create a new Zip7 combinator from seven effects.
    pub fn new(e1: E1, e2: E2, e3: E3, e4: E4, e5: E5, e6: E6, e7: E7) -> Self {
        Zip7 {
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
        }
    }
}

impl<E1, E2, E3, E4, E5, E6, E7> Effect for Zip7<E1, E2, E3, E4, E5, E6, E7>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
    E7: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (
        E1::Output,
        E2::Output,
        E3::Output,
        E4::Output,
        E5::Output,
        E6::Output,
        E7::Output,
    );
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        let r4 = self.e4.run(env).await?;
        let r5 = self.e5.run(env).await?;
        let r6 = self.e6.run(env).await?;
        let r7 = self.e7.run(env).await?;
        Ok((r1, r2, r3, r4, r5, r6, r7))
    }
}

/// Combines eight effects into a flat tuple.
///
/// This is zero-cost: no heap allocation occurs.
#[derive(Debug)]
pub struct Zip8<E1, E2, E3, E4, E5, E6, E7, E8> {
    e1: E1,
    e2: E2,
    e3: E3,
    e4: E4,
    e5: E5,
    e6: E6,
    e7: E7,
    e8: E8,
}

impl<E1, E2, E3, E4, E5, E6, E7, E8> Zip8<E1, E2, E3, E4, E5, E6, E7, E8> {
    /// Create a new Zip8 combinator from eight effects.
    #[allow(clippy::too_many_arguments)]
    pub fn new(e1: E1, e2: E2, e3: E3, e4: E4, e5: E5, e6: E6, e7: E7, e8: E8) -> Self {
        Zip8 {
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
            e8,
        }
    }
}

impl<E1, E2, E3, E4, E5, E6, E7, E8> Effect for Zip8<E1, E2, E3, E4, E5, E6, E7, E8>
where
    E1: Effect,
    E2: Effect<Error = E1::Error, Env = E1::Env>,
    E3: Effect<Error = E1::Error, Env = E1::Env>,
    E4: Effect<Error = E1::Error, Env = E1::Env>,
    E5: Effect<Error = E1::Error, Env = E1::Env>,
    E6: Effect<Error = E1::Error, Env = E1::Env>,
    E7: Effect<Error = E1::Error, Env = E1::Env>,
    E8: Effect<Error = E1::Error, Env = E1::Env>,
{
    type Output = (
        E1::Output,
        E2::Output,
        E3::Output,
        E4::Output,
        E5::Output,
        E6::Output,
        E7::Output,
        E8::Output,
    );
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let r1 = self.e1.run(env).await?;
        let r2 = self.e2.run(env).await?;
        let r3 = self.e3.run(env).await?;
        let r4 = self.e4.run(env).await?;
        let r5 = self.e5.run(env).await?;
        let r6 = self.e6.run(env).await?;
        let r7 = self.e7.run(env).await?;
        let r8 = self.e8.run(env).await?;
        Ok((r1, r2, r3, r4, r5, r6, r7, r8))
    }
}
