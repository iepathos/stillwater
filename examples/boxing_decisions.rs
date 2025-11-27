//! Boxing Decisions Example
//!
//! Demonstrates when to use `.boxed()` vs zero-cost effects.
//! Stillwater follows the `futures` crate pattern: zero-cost by default,
//! explicit boxing when type erasure is needed.
//!
//! Run with: cargo run --example boxing_decisions

use stillwater::{fail, from_fn, pure, BoxedEffect, Effect, EffectExt, RunStandalone};

// ============================================================================
// ZERO-COST: Simple effect chains
// ============================================================================

/// Zero-cost effect chain - no heap allocation.
/// Each combinator returns a concrete type that the compiler can inline.
fn zero_cost_example() -> impl Effect<Output = i32, Error = String, Env = ()> {
    pure(1)
        .map(|x| x + 1)
        .map(|x| x * 2)
        .and_then(|x| pure(x + 10))
}

// ============================================================================
// BOXED: Storing in collections
// ============================================================================

/// Must use `.boxed()` to store different effects in a Vec.
/// Different effect chains have different concrete types, but BoxedEffect
/// gives them a uniform type for storage.
fn collection_example() -> Vec<BoxedEffect<i32, String, ()>> {
    vec![
        // Simple pure value
        pure(1).boxed(),
        // Effect with map
        pure(2).map(|x| x * 2).boxed(),
        // Effect with and_then
        pure(3).and_then(|x| pure(x * 3)).boxed(),
    ]
}

// ============================================================================
// BOXED: Recursive effects
// ============================================================================

/// Recursive effects require `.boxed()` to break the infinite type.
/// Without boxing, the compiler would try to compute an infinitely nested type.
fn recursive_sum(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n)
            .and_then(move |x| recursive_sum(x - 1).map(move |sum| x + sum))
            .boxed()
    }
}

/// Another recursive example: factorial
fn factorial(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 1 {
        pure(1).boxed()
    } else {
        pure(n)
            .and_then(move |x| factorial(x - 1).map(move |fact| x * fact))
            .boxed()
    }
}

// ============================================================================
// BOXED: Match arms with different types
// ============================================================================

#[derive(Debug, Clone, Copy)]
enum DataSource {
    Cache,
    Database,
    Remote,
}

/// Different match arms have different effect types, need `.boxed()`.
/// Each branch creates a different concrete type, but the function needs
/// to return a single type.
fn fetch_data(source: DataSource) -> BoxedEffect<String, String, ()> {
    match source {
        DataSource::Cache => {
            // Just a pure value
            pure("cached data".to_string()).boxed()
        }
        DataSource::Database => {
            // Effect with map - different type than pure alone
            pure("db").map(|s| format!("{} data", s)).boxed()
        }
        DataSource::Remote => {
            // Effect with and_then - yet another different type
            pure("remote")
                .and_then(|s| pure(format!("{} data", s)))
                .boxed()
        }
    }
}

// ============================================================================
// BOXED: Conditional logic with different effect types
// ============================================================================

/// When if/else branches create different effect types, boxing is needed.
fn conditional_effect(use_fallback: bool) -> BoxedEffect<i32, String, ()> {
    if use_fallback {
        // Simple fallback
        pure(42).boxed()
    } else {
        // More complex computation
        pure(10).map(|x| x * 2).and_then(|x| pure(x + 1)).boxed()
    }
}

// ============================================================================
// ZERO-COST: Returning impl Effect
// ============================================================================

/// When the return type is `impl Effect`, no boxing is needed.
/// The compiler infers the concrete type.
fn zero_cost_composition() -> impl Effect<Output = String, Error = String, Env = ()> {
    pure(42).map(|x| x * 2).map(|x| format!("Result: {}", x))
}

// ============================================================================
// ZERO-COST: Generic functions
// ============================================================================

/// Generic functions can work with any Effect without boxing.
async fn run_and_print<E>(effect: E)
where
    E: Effect<Output = i32, Error = String, Env = ()>,
{
    match effect.run(&()).await {
        Ok(value) => println!("  Success: {}", value),
        Err(e) => println!("  Error: {}", e),
    }
}

// ============================================================================
// BOXED: Error handling with recovery
// ============================================================================

/// When recovery branches return different types, boxing is needed.
fn with_recovery() -> BoxedEffect<i32, String, ()> {
    fail::<i32, _, ()>("primary failed".to_string())
        .or_else(|_| pure(42)) // recovery path
        .boxed()
}

// ============================================================================
// ZERO-COST: Environment access
// ============================================================================

#[derive(Clone)]
struct AppEnv {
    multiplier: i32,
}

/// Effects that access the environment are still zero-cost.
fn env_effect() -> impl Effect<Output = i32, Error = String, Env = AppEnv> {
    from_fn(|env: &AppEnv| Ok(env.multiplier * 10))
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=== Boxing Decisions Demo ===\n");

    // Zero-cost chain
    println!("1. Zero-cost effect chain:");
    let result = zero_cost_example().run_standalone().await;
    println!("   Result: {:?}", result);
    println!("   (No heap allocation - compiler inlines everything)\n");

    // Collection of effects
    println!("2. Collection of boxed effects:");
    let effects = collection_example();
    println!("   {} effects in collection:", effects.len());
    for (i, effect) in effects.into_iter().enumerate() {
        let result = effect.run(&()).await;
        println!("   Effect {}: {:?}", i, result);
    }
    println!("   (Boxing needed for uniform type in Vec)\n");

    // Recursive effect
    println!("3. Recursive sum (1+2+3+4+5):");
    let sum = recursive_sum(5).run(&()).await;
    println!("   Sum: {:?}", sum);
    println!("   (Boxing breaks infinite type recursion)\n");

    // Factorial
    println!("4. Recursive factorial (5!):");
    let fact = factorial(5).run(&()).await;
    println!("   Factorial: {:?}", fact);
    println!();

    // Match arms
    println!("5. Fetch data from different sources:");
    for source in [DataSource::Cache, DataSource::Database, DataSource::Remote] {
        let data = fetch_data(source).run(&()).await;
        println!("   {:?}: {:?}", source, data);
    }
    println!("   (Boxing unifies different match arm types)\n");

    // Conditional
    println!("6. Conditional effect:");
    let primary = conditional_effect(false).run(&()).await;
    let fallback = conditional_effect(true).run(&()).await;
    println!("   Primary path: {:?}", primary);
    println!("   Fallback path: {:?}", fallback);
    println!();

    // Zero-cost impl Effect
    println!("7. Zero-cost with impl Effect:");
    let result = zero_cost_composition().run_standalone().await;
    println!("   Result: {:?}", result);
    println!("   (Concrete type, no boxing)\n");

    // Generic function
    println!("8. Generic function (zero-cost):");
    run_and_print(pure(100)).await;
    run_and_print(pure(50).map(|x| x * 2)).await;
    println!("   (Works with any Effect, no boxing)\n");

    // Recovery
    println!("9. Effect with recovery:");
    let result = with_recovery().run(&()).await;
    println!("   Result: {:?}", result);
    println!();

    // Environment access
    println!("10. Environment access (zero-cost):");
    let env = AppEnv { multiplier: 7 };
    let result = env_effect().run(&env).await;
    println!("   Result: {:?}", result);
    println!("   (Environment access is still zero-cost)\n");

    // Summary
    println!("=== Summary ===");
    println!();
    println!("Use ZERO-COST (no .boxed()) when:");
    println!("  - Simple effect chains");
    println!("  - Returning impl Effect<...>");
    println!("  - Generic functions");
    println!("  - Environment access");
    println!();
    println!("Use BOXED (.boxed()) when:");
    println!("  - Storing effects in collections");
    println!("  - Recursive effects");
    println!("  - Match/if arms with different effect types");
    println!();
    println!("=== All examples completed successfully! ===");
}
