//! Tests for the zero-cost Effect trait system.

use crate::effect::compat::RunStandalone;
use crate::effect::prelude::*;

// Basic constructor tests
#[tokio::test]
async fn test_pure_returns_value() {
    let effect = pure::<_, String, ()>(42);
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_fail_returns_error() {
    let effect = fail::<i32, _, ()>("error".to_string());
    assert_eq!(effect.run_standalone().await, Err("error".to_string()));
}

// Map tests
#[tokio::test]
async fn test_map_transforms_value() {
    let effect = pure::<_, String, ()>(21).map(|x| x * 2);
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_map_on_failure_doesnt_execute() {
    let effect = fail::<i32, _, ()>("error".to_string()).map(|x| x * 2);
    assert_eq!(effect.run_standalone().await, Err("error".to_string()));
}

// MapErr tests
#[tokio::test]
async fn test_map_err_transforms_error() {
    let effect = fail::<i32, _, ()>("error").map_err(|e: &str| format!("wrapped: {}", e));
    assert_eq!(
        effect.run_standalone().await,
        Err("wrapped: error".to_string())
    );
}

#[tokio::test]
async fn test_map_err_preserves_success() {
    let effect = pure::<_, &str, ()>(42).map_err(|e| format!("wrapped: {}", e));
    assert_eq!(effect.run_standalone().await, Ok(42));
}

// AndThen tests
#[tokio::test]
async fn test_and_then_chains_effects() {
    let effect = pure::<_, String, ()>(21).and_then(|x| pure(x * 2));
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_and_then_propagates_error() {
    let effect = fail::<i32, _, ()>("error".to_string()).and_then(|x| pure(x * 2));
    assert_eq!(effect.run_standalone().await, Err("error".to_string()));
}

#[tokio::test]
async fn test_and_then_chain_failure() {
    let effect = pure::<_, String, ()>(21).and_then(|_| fail::<i32, _, ()>("error".to_string()));
    assert_eq!(effect.run_standalone().await, Err("error".to_string()));
}

// OrElse tests
#[tokio::test]
async fn test_or_else_recovers_from_error() {
    let effect = fail::<i32, _, ()>("error".to_string()).or_else(|_| pure::<_, String, ()>(42));
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_or_else_preserves_success() {
    let effect = pure::<_, String, ()>(42).or_else(|_| pure::<_, String, ()>(0));
    assert_eq!(effect.run_standalone().await, Ok(42));
}

// FromFn tests
#[tokio::test]
async fn test_from_fn_accesses_environment() {
    #[derive(Clone)]
    struct Env {
        value: i32,
    }

    let effect = from_fn(|env: &Env| Ok::<_, String>(env.value * 2));
    assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
}

// FromAsync tests
#[tokio::test]
async fn test_from_async_works() {
    let effect = from_async(|_: &()| async { Ok::<_, String>(42) });
    assert_eq!(effect.run_standalone().await, Ok(42));
}

// FromResult tests
#[tokio::test]
async fn test_from_result_ok() {
    let effect = from_result::<_, String, ()>(Ok(42));
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_from_result_err() {
    let effect = from_result::<i32, _, ()>(Err("error".to_string()));
    assert_eq!(effect.run_standalone().await, Err("error".to_string()));
}

// FromOption tests
#[tokio::test]
async fn test_from_option_some() {
    let effect = from_option::<_, String, ()>(Some(42), || "missing".to_string());
    assert_eq!(effect.run_standalone().await, Ok(42));
}

#[tokio::test]
async fn test_from_option_none() {
    let effect = from_option::<i32, _, ()>(None, || "missing".to_string());
    assert_eq!(effect.run_standalone().await, Err("missing".to_string()));
}

// Reader pattern tests
#[tokio::test]
async fn test_ask_clones_environment() {
    #[derive(Clone, PartialEq, Debug)]
    struct Env {
        value: i32,
    }

    let effect = ask::<String, Env>();
    assert_eq!(
        effect.execute(&Env { value: 42 }).await,
        Ok(Env { value: 42 })
    );
}

#[tokio::test]
async fn test_asks_queries_environment() {
    #[derive(Clone)]
    struct Env {
        value: i32,
    }

    let effect = asks::<_, String, _, _>(|env: &Env| env.value * 2);
    assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
}

#[tokio::test]
async fn test_local_modifies_environment() {
    #[derive(Clone)]
    struct OuterEnv {
        multiplier: i32,
    }
    #[derive(Clone)]
    struct InnerEnv {
        value: i32,
    }

    let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
    let effect = local(
        |outer: &OuterEnv| InnerEnv {
            value: 21 * outer.multiplier,
        },
        inner_effect,
    );

    assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
}

// Boxing tests
#[tokio::test]
async fn test_boxed_allows_collection_storage() {
    let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
        pure(1).boxed(),
        pure(2).map(|x| x * 2).boxed(),
        pure(3).and_then(|x| pure(x * 3)).boxed(),
    ];

    let mut results = Vec::new();
    for effect in effects {
        results.push(effect.run_standalone().await.unwrap());
    }
    assert_eq!(results, vec![1, 4, 9]);
}

#[tokio::test]
async fn test_boxed_allows_recursion() {
    fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
        if n <= 0 {
            pure(0).boxed()
        } else {
            pure(n)
                .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
                .boxed()
        }
    }

    assert_eq!(countdown(5).run_standalone().await, Ok(15)); // 5+4+3+2+1+0
}

#[tokio::test]
async fn test_boxed_allows_match_arms() {
    fn get_value(use_double: bool) -> BoxedEffect<i32, String, ()> {
        match use_double {
            true => pure(21).map(|x| x * 2).boxed(),
            false => pure(42).boxed(),
        }
    }

    assert_eq!(get_value(true).run_standalone().await, Ok(42));
    assert_eq!(get_value(false).run_standalone().await, Ok(42));
}

// Parallel tests
#[tokio::test]
async fn test_par_all_collects_successes() {
    let effects: Vec<BoxedEffect<i32, String, ()>> =
        vec![pure(1).boxed(), pure(2).boxed(), pure(3).boxed()];

    let result = par_all(effects, &()).await;
    assert_eq!(result, Ok(vec![1, 2, 3]));
}

#[tokio::test]
async fn test_par_all_collects_errors() {
    let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
        pure(1).boxed(),
        fail("error1".to_string()).boxed(),
        fail("error2".to_string()).boxed(),
    ];

    let result = par_all(effects, &()).await;
    assert_eq!(
        result,
        Err(vec!["error1".to_string(), "error2".to_string()])
    );
}

#[tokio::test]
async fn test_par_try_all_succeeds() {
    let effects: Vec<BoxedEffect<i32, String, ()>> = vec![pure(1).boxed(), pure(2).boxed()];

    let result = par_try_all(effects, &()).await;
    assert_eq!(result, Ok(vec![1, 2]));
}

#[tokio::test]
async fn test_par2_runs_heterogeneous_effects() {
    let e1 = pure::<_, String, ()>(42);
    let e2 = pure::<_, String, ()>("hello".to_string());

    let (r1, r2) = par2(e1, e2, &()).await;
    assert_eq!(r1, Ok(42));
    assert_eq!(r2, Ok("hello".to_string()));
}

#[tokio::test]
async fn test_par3_runs_three_effects() {
    let e1 = pure::<_, String, ()>(1);
    let e2 = pure::<_, String, ()>(2);
    let e3 = pure::<_, String, ()>(3);

    let (r1, r2, r3) = par3(e1, e2, e3, &()).await;
    assert_eq!(r1, Ok(1));
    assert_eq!(r2, Ok(2));
    assert_eq!(r3, Ok(3));
}

// Error type conversion test
#[tokio::test]
async fn test_error_type_conversion_chain() {
    #[derive(Debug, PartialEq)]
    enum AppError {
        Db(String),
        Network(String),
    }

    let effect = pure::<_, String, ()>(42)
        .map_err(AppError::Db)
        .and_then(|x| pure::<_, String, ()>(x * 2).map_err(AppError::Network));

    assert_eq!(effect.run_standalone().await, Ok(84));
}

// Complex chain test
#[tokio::test]
async fn test_complex_chain() {
    let effect = pure::<_, String, ()>(2)
        .map(|x| x * 3) // 6
        .and_then(|x| pure(x + 4)) // 10
        .map(|x| x * 2) // 20
        .and_then(|x| pure(x / 2)); // 10

    assert_eq!(effect.run_standalone().await, Ok(10));
}

// Bracket tests
#[tokio::test]
async fn test_bracket_basic() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let released = Arc::new(AtomicBool::new(false));
    let released_clone = released.clone();

    let effect = bracket(
        pure::<_, String, ()>(42),
        |resource| pure(resource * 2),
        move |_| {
            released_clone.store(true, Ordering::SeqCst);
            pure(())
        },
    );

    let result = effect.run_standalone().await;
    assert_eq!(result, Ok(84));
    assert!(released.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_bracket_releases_on_error() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let released = Arc::new(AtomicBool::new(false));
    let released_clone = released.clone();

    let effect = bracket(
        pure::<_, String, ()>(42),
        |_| fail::<i32, _, ()>("use error".to_string()),
        move |_| {
            released_clone.store(true, Ordering::SeqCst);
            pure(())
        },
    );

    let result = effect.run_standalone().await;
    assert_eq!(result, Err("use error".to_string()));
    assert!(released.load(Ordering::SeqCst));
}

// Zero-cost verification tests (compile-time checks)
#[test]
fn test_pure_size() {
    use std::marker::PhantomData;
    use std::mem::size_of;

    // Pure only stores the value + phantom data
    assert_eq!(
        size_of::<Pure<i32, String, ()>>(),
        size_of::<i32>() + size_of::<PhantomData<(String, ())>>()
    );
}

// Local combinator via EffectExt
#[tokio::test]
async fn test_effect_ext_local() {
    #[derive(Clone)]
    struct OuterEnv {
        multiplier: i32,
    }
    #[derive(Clone)]
    struct InnerEnv {
        value: i32,
    }

    let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
    let effect = inner_effect.local(|outer: &OuterEnv| InnerEnv {
        value: 21 * outer.multiplier,
    });

    assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
}

// Execute method test
#[tokio::test]
async fn test_execute_method() {
    let effect = pure::<_, String, ()>(42);
    assert_eq!(effect.execute(&()).await, Ok(42));
}
