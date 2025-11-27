//! Effects Example
//!
//! Demonstrates the Effect type and composition patterns.
//! Shows practical patterns including:
//! - Creating effects (pure, fail, from_fn, from_async)
//! - Mapping and transforming effects
//! - Chaining effects with and_then
//! - Error handling with map_err
//! - Helper combinators (tap, check, with)
//! - Environment-based dependency injection

use stillwater::{fail, from_async, from_fn, pure, Effect, EffectExt, RunStandalone};

// ==================== Basic Effects ====================

/// Example 1: Creating basic effects
///
/// Demonstrates pure() and fail() constructors.
async fn example_basic_effects() {
    println!("\n=== Example 1: Basic Effects ===");

    // Pure value - always succeeds
    let success_effect = pure::<_, String, ()>(42);
    let result = success_effect.run_standalone().await;
    println!("Pure effect: {:?}", result);

    // Failure - always fails
    let fail_effect = fail::<i32, _, ()>("something went wrong".to_string());
    let result = fail_effect.run_standalone().await;
    println!("Fail effect: {:?}", result);
}

// ==================== Creating Effects from Functions ====================

/// Example 2: Effects from synchronous functions
///
/// Demonstrates using from_fn() to create effects from pure functions.
async fn example_from_fn() {
    println!("\n=== Example 2: Effects from Functions ===");

    // Simple environment
    #[derive(Clone)]
    struct Env {
        multiplier: i32,
    }

    // Effect from a function that uses the environment
    let effect = from_fn(|env: &Env| Ok::<_, String>(env.multiplier * 2));

    let env = Env { multiplier: 21 };
    let result = effect.run(&env).await;
    println!("Result: {:?}", result);
}

// ==================== Mapping Effects ====================

/// Example 3: Transforming values with map
///
/// Demonstrates using map() to transform successful values.
async fn example_mapping() {
    println!("\n=== Example 3: Mapping Effects ===");

    #[derive(Clone)]
    struct Env {
        base_value: i32,
    }

    // Chain multiple transformations
    let effect = from_fn(|env: &Env| Ok::<_, String>(env.base_value))
        .map(|x| x * 2) // Double it
        .map(|x| x + 10) // Add 10
        .map(|x| format!("Result: {}", x)); // Convert to string

    let env = Env { base_value: 5 };
    let result = effect.run(&env).await.unwrap();
    println!("{}", result); // "Result: 20"
}

// ==================== Chaining Effects ====================

/// Example 4: Chaining effects with and_then
///
/// Demonstrates using and_then() to sequence effects that depend on previous results.
async fn example_chaining() {
    println!("\n=== Example 4: Chaining Effects ===");

    #[derive(Clone)]
    struct Database {
        value: i32,
    }

    #[derive(Clone)]
    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    // First effect: get value from database
    fn get_value() -> impl Effect<Output = i32, Error = String, Env = Env> {
        from_fn(|env: &Env| Ok::<_, String>(env.db.value))
    }

    // Second effect: validate and double the value
    fn validate_and_double(x: i32) -> impl Effect<Output = i32, Error = String, Env = Env> {
        from_fn(move |_: &Env| {
            if x > 0 {
                Ok(x * 2)
            } else {
                Err("Value must be positive".to_string())
            }
        })
    }

    let env = Env {
        db: Database { value: 10 },
    };
    let result = get_value().and_then(validate_and_double).run(&env).await;
    println!("Success case: {:?}", result);

    // Try with negative value
    let env2 = Env {
        db: Database { value: -5 },
    };
    let result2 = get_value().and_then(validate_and_double).run(&env2).await;
    println!("Failure case: {:?}", result2);
}

// ==================== Error Handling ====================

/// Example 5: Handling errors with map_err
///
/// Demonstrates using map_err() to transform error values.
async fn example_error_handling() {
    println!("\n=== Example 5: Error Handling ===");

    #[derive(Clone)]
    struct Env {
        value: i32,
    }

    // Effect that might fail
    let _effect = from_fn(|env: &Env| {
        if env.value > 0 {
            Ok::<_, &str>(env.value)
        } else {
            Err("negative")
        }
    })
    .map_err(|e| format!("Error: {} is not allowed", e));

    let env1 = Env { value: 42 };
    let effect1 = from_fn(|env: &Env| {
        if env.value > 0 {
            Ok::<_, &str>(env.value)
        } else {
            Err("negative")
        }
    })
    .map_err(|e| format!("Error: {} is not allowed", e));
    println!("Valid value: {:?}", effect1.run(&env1).await);

    let env2 = Env { value: -1 };
    let effect2 = from_fn(|env: &Env| {
        if env.value > 0 {
            Ok::<_, &str>(env.value)
        } else {
            Err("negative")
        }
    })
    .map_err(|e| format!("Error: {} is not allowed", e));
    println!("Invalid value: {:?}", effect2.run(&env2).await);
}

// ==================== Async Effects ====================

/// Example 6: Async effects
///
/// Demonstrates using from_async() for asynchronous operations.
async fn example_async_effects() {
    println!("\n=== Example 6: Async Effects ===");

    #[derive(Clone)]
    struct ApiClient {
        base_url: String,
    }

    #[derive(Clone)]
    struct Env {
        api: ApiClient,
    }

    impl AsRef<ApiClient> for Env {
        fn as_ref(&self) -> &ApiClient {
            &self.api
        }
    }

    let env = Env {
        api: ApiClient {
            base_url: "https://api.example.com".to_string(),
        },
    };

    // Async effect: simulate API call
    let fetch_user = from_async(|env: &Env| {
        let url = env.api.base_url.clone();
        async move {
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok::<_, String>(format!("User from {}", url))
        }
    });

    let result = fetch_user.run(&env).await.unwrap();
    println!("Fetched: {}", result);
}

// ==================== Helper Combinators ====================

/// Example 7: Using tap for side effects
///
/// Demonstrates tap() to perform side effects while passing the value through.
async fn example_tap() {
    println!("\n=== Example 7: Using tap() ===");

    #[derive(Clone)]
    struct Env {
        value: i32,
    }

    let effect = from_fn(|env: &Env| Ok::<_, String>(env.value))
        .tap(|x| {
            println!("  [DEBUG] Got value: {}", x);
            pure::<(), String, Env>(())
        })
        .map(|x| x * 2)
        .tap(|x| {
            println!("  [DEBUG] After doubling: {}", x);
            pure::<(), String, Env>(())
        })
        .map(|x| x + 5);

    let env = Env { value: 10 };
    let result = effect.run(&env).await.unwrap();
    println!("Final result: {}", result);
}

/// Example 8: Using check for conditional validation
///
/// Demonstrates check() to validate values with a predicate.
async fn example_check() {
    println!("\n=== Example 8: Using check() ===");

    #[derive(Clone)]
    struct Env {
        age: i32,
    }

    let env1 = Env { age: 25 };
    let result1 = from_fn(|env: &Env| Ok::<_, String>(env.age))
        .and_then(|age| {
            from_fn(move |_: &Env| {
                if age >= 18 {
                    Ok(age)
                } else {
                    Err(format!("Age {} is below minimum (18)", age))
                }
            })
        })
        .run(&env1)
        .await;
    println!("Adult: {:?}", result1);

    let env2 = Env { age: 16 };
    let result2 = from_fn(|env: &Env| Ok::<_, String>(env.age))
        .and_then(|age| {
            from_fn(move |_: &Env| {
                if age >= 18 {
                    Ok(age)
                } else {
                    Err(format!("Age {} is below minimum (18)", age))
                }
            })
        })
        .run(&env2)
        .await;
    println!("Minor: {:?}", result2);
}

/// Example 9: Using with to combine effects
///
/// Demonstrates with() to run effects in sequence and combine results.
async fn example_with() {
    println!("\n=== Example 9: Using with() ===");

    #[derive(Clone)]
    struct Config {
        width: i32,
        height: i32,
    }

    #[derive(Clone)]
    struct Env {
        config: Config,
    }

    impl AsRef<Config> for Env {
        fn as_ref(&self) -> &Config {
            &self.config
        }
    }

    // Get width and height as separate effects, then combine
    let area_effect = from_fn(|env: &Env| Ok::<_, String>(env.config.width))
        .with(|_w| from_fn(|env: &Env| Ok::<_, String>(env.config.height)))
        .map(|(w, h)| w * h);

    let env = Env {
        config: Config {
            width: 10,
            height: 5,
        },
    };

    let area = area_effect.run(&env).await.unwrap();
    println!("Area: {}", area);
}

// ==================== Combining Multiple Effects ====================

/// Example 10: Real-world composition
///
/// Demonstrates combining multiple patterns into a realistic workflow.
async fn example_composition() {
    println!("\n=== Example 10: Real-world Composition ===");

    #[derive(Clone)]
    struct User {
        id: u64,
        name: String,
        age: i32,
    }

    #[derive(Clone)]
    struct Database {
        users: Vec<User>,
    }

    #[derive(Clone)]
    struct Env {
        db: Database,
    }

    impl AsRef<Database> for Env {
        fn as_ref(&self) -> &Database {
            &self.db
        }
    }

    // Find user by ID
    fn find_user(user_id: u64) -> impl Effect<Output = User, Error = String, Env = Env> {
        from_fn(move |env: &Env| {
            env.db
                .users
                .iter()
                .find(|u| u.id == user_id)
                .cloned()
                .ok_or_else(|| format!("User {} not found", user_id))
        })
    }

    // Validate user age
    fn validate_adult(user: User) -> impl Effect<Output = User, Error = String, Env = Env> {
        from_fn(move |_: &Env| {
            if user.age >= 18 {
                Ok(user.clone())
            } else {
                Err(format!("User {} is not an adult", user.name))
            }
        })
    }

    // Format greeting
    fn greet(user: User) -> impl Effect<Output = String, Error = String, Env = Env> {
        pure(format!("Hello, {}!", user.name))
    }

    // Compose the workflow
    let workflow = find_user(1)
        .tap(|u| {
            println!("  Found user: {}", u.name);
            pure::<(), String, Env>(())
        })
        .and_then(validate_adult)
        .tap(|u| {
            println!("  Validated user: {}", u.name);
            pure::<(), String, Env>(())
        })
        .and_then(greet);

    let env = Env {
        db: Database {
            users: vec![
                User {
                    id: 1,
                    name: "Alice".to_string(),
                    age: 25,
                },
                User {
                    id: 2,
                    name: "Bob".to_string(),
                    age: 16,
                },
            ],
        },
    };

    // Success case
    match workflow.run(&env).await {
        Ok(greeting) => println!("Success: {}", greeting),
        Err(e) => println!("Error: {}", e),
    }

    // Try with minor
    let workflow2 = find_user(2).and_then(validate_adult).and_then(greet);
    match workflow2.run(&env).await {
        Ok(greeting) => println!("Success: {}", greeting),
        Err(e) => println!("Error: {}", e),
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Effects Examples");
    println!("================");

    example_basic_effects().await;
    example_from_fn().await;
    example_mapping().await;
    example_chaining().await;
    example_error_handling().await;
    example_async_effects().await;
    example_tap().await;
    example_check().await;
    example_with().await;
    example_composition().await;

    println!("\n=== All examples completed successfully! ===");
}
