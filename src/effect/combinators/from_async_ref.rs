//! Async effect constructor that can borrow its environment across `.await`.

use std::marker::PhantomData;

use crate::effect::boxed::BoxFuture;
use crate::effect::trait_def::Effect;

/// An async effect whose future may borrow from the environment.
///
/// Unlike [`FromAsync`](crate::effect::combinators::FromAsync), this type uses
/// a boxed future so its lifetime can be tied to the environment passed to
/// [`Effect::run`].
pub struct FromAsyncRef<F, Env> {
    pub(crate) f: F,
    pub(crate) _phantom: PhantomData<Env>,
}

impl<F, Env> std::fmt::Debug for FromAsyncRef<F, Env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FromAsyncRef")
            .field("f", &"<function>")
            .finish()
    }
}

impl<F, Env> FromAsyncRef<F, Env> {
    /// Create an async effect from a lifetime-aware function.
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, T, E, Env> Effect for FromAsyncRef<F, Env>
where
    F: for<'a> FnOnce(&'a Env) -> BoxFuture<'a, Result<T, E>> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl std::future::Future<Output = Result<T, E>> + Send {
        (self.f)(env)
    }
}

#[cfg(test)]
mod tests {
    use crate::effect::constructors::from_async_ref;
    use crate::effect::ext::EffectExt;
    use crate::effect::trait_def::Effect;

    #[derive(Clone)]
    struct Env {
        value: String,
    }

    #[tokio::test]
    async fn borrows_environment_across_await() {
        let effect = from_async_ref(|env: &Env| {
            Box::pin(async move {
                tokio::task::yield_now().await;
                Ok::<_, String>(env.value.len())
            })
        });

        let env = Env {
            value: "stillwater".to_string(),
        };
        assert_eq!(effect.run(&env).await, Ok(10));
    }

    #[tokio::test]
    async fn propagates_error() {
        let effect = from_async_ref(|env: &Env| {
            Box::pin(async move {
                tokio::task::yield_now().await;
                Err::<usize, _>(env.value.clone())
            })
        });

        let env = Env {
            value: "borrowed failure".to_string(),
        };
        assert_eq!(effect.run(&env).await, Err("borrowed failure".to_string()));
    }

    #[tokio::test]
    async fn can_be_boxed() {
        let effect = from_async_ref(|env: &Env| {
            Box::pin(async move {
                tokio::task::yield_now().await;
                Ok::<_, String>(env.value.len())
            })
        })
        .boxed();

        let env = Env {
            value: "stillwater".to_string(),
        };
        assert_eq!(effect.run(&env).await, Ok(10));
    }
}
