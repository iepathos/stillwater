//! Pipeline Example
//!
//! Demonstrates data transformation pipelines using Effect composition.
//! Shows how to build complex data flows from simple transformations.
//!
//! Patterns covered:
//! - Sequential transformations with map
//! - Parallel processing
//! - Filtering and validation in pipelines
//! - Error handling in data flows
//! - Composition of pure and effectful operations

use stillwater::effect::prelude::*;

// ==================== Sequential Transformations ====================

/// Example 1: Simple data transformation pipeline
///
/// Demonstrates chaining pure transformations using map.
async fn example_sequential_transformations() {
    println!("\n=== Example 1: Sequential Transformations ===");

    #[derive(Clone)]
    struct Env;

    // Pipeline: number -> double -> add 10 -> to string
    let pipeline = pure::<_, String, Env>(5)
        .map(|x| x * 2)
        .tap(|x| {
            println!("  After doubling: {}", x);
            pure::<(), String, Env>(())
        })
        .map(|x| x + 10)
        .tap(|x| {
            println!("  After adding 10: {}", x);
            pure::<(), String, Env>(())
        })
        .map(|x| format!("Result: {}", x));

    let result = pipeline.run(&Env).await.unwrap();
    println!("Final: {}", result);
}

// ==================== Pure Functions in Pipelines ====================

/// Example 2: Composing pure functions
///
/// Demonstrates building pipelines from reusable pure functions.
async fn example_pure_functions() {
    println!("\n=== Example 2: Pure Functions in Pipelines ===");

    // Pure transformation functions
    fn normalize(s: String) -> String {
        s.trim().to_lowercase()
    }

    fn remove_special_chars(s: String) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    fn collapse_whitespace(s: String) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[derive(Clone)]
    struct Env;

    // Build pipeline from pure functions
    let input = "  Hello,   World!  ".to_string();
    let pipeline = pure::<_, String, Env>(input.clone())
        .tap(|s| {
            println!("  Input: '{}'", s);
            pure::<(), String, Env>(())
        })
        .map(normalize)
        .tap(|s| {
            println!("  Normalized: '{}'", s);
            pure::<(), String, Env>(())
        })
        .map(remove_special_chars)
        .tap(|s| {
            println!("  Special chars removed: '{}'", s);
            pure::<(), String, Env>(())
        })
        .map(collapse_whitespace)
        .tap(|s| {
            println!("  Whitespace collapsed: '{}'", s);
            pure::<(), String, Env>(())
        });

    let result = pipeline.run(&Env).await.unwrap();
    println!("Output: '{}'", result);
}

// ==================== Filtering Data ====================

/// Example 3: Filtering with check()
///
/// Demonstrates using check() to filter invalid data in pipelines.
async fn example_filtering() {
    println!("\n=== Example 3: Filtering with check() ===");

    #[derive(Clone)]
    struct Env;

    // Pipeline with validation
    fn process_age(age: i32) -> impl Effect<Output = String, Error = String, Env = Env> {
        pure::<_, String, Env>(age)
            .and_then(|a| {
                if a >= 0 {
                    pure::<_, String, Env>(a).boxed()
                } else {
                    fail::<_, _, Env>(format!("Age {} is negative", a)).boxed()
                }
            })
            .and_then(|a| {
                if a <= 150 {
                    pure::<_, String, Env>(a).boxed()
                } else {
                    fail::<_, _, Env>(format!("Age {} is too high", a)).boxed()
                }
            })
            .map(|a| format!("Valid age: {}", a))
    }

    // Valid age
    match process_age(25).run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Invalid age (negative)
    match process_age(-5).run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Invalid age (too high)
    match process_age(200).run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }
}

// ==================== Branching Pipelines ====================

/// Example 4: Conditional branching
///
/// Demonstrates branching logic in pipelines.
async fn example_branching() {
    println!("\n=== Example 4: Conditional Branching ===");

    #[derive(Clone)]
    struct Env;

    fn classify_and_process(n: i32) -> impl Effect<Output = String, Error = String, Env = Env> {
        pure::<_, String, Env>(n).and_then(|num| {
            if num < 0 {
                pure::<_, String, Env>(format!("{} is negative", num))
            } else if num % 2 == 0 {
                pure::<_, String, Env>(format!("{} is even", num))
            } else {
                pure::<_, String, Env>(format!("{} is odd", num))
            }
        })
    }

    for n in [-3, 0, 4, 7] {
        let result = classify_and_process(n).run(&Env).await.unwrap();
        println!("  {}", result);
    }
}

// ==================== Effectful Transformations ====================

/// Example 5: Mixing pure and effectful operations
///
/// Demonstrates combining pure transformations with effects.
async fn example_effectful_transformations() {
    println!("\n=== Example 5: Mixing Pure and Effectful Operations ===");

    #[derive(Clone)]
    struct Logger {
        prefix: String,
    }

    #[derive(Clone)]
    struct Env {
        logger: Logger,
    }

    impl AsRef<Logger> for Env {
        fn as_ref(&self) -> &Logger {
            &self.logger
        }
    }

    // Effectful logging
    fn log_step<E: AsRef<Logger> + Clone + Send + Sync + 'static>(
        message: String,
        value: i32,
    ) -> impl Effect<Output = i32, Error = String, Env = E> {
        from_fn(move |env: &E| {
            let logger: &Logger = env.as_ref();
            println!("  [{}] {}: {}", logger.prefix, message, value);
            Ok(value)
        })
    }

    let env = Env {
        logger: Logger {
            prefix: "PIPELINE".to_string(),
        },
    };

    // Pipeline mixing pure and effectful operations
    let pipeline = pure::<_, String, Env>(10)
        .and_then(|x| log_step("Start".to_string(), x))
        .map(|x| x * 3)
        .and_then(|x| log_step("After multiply".to_string(), x))
        .map(|x| x + 5)
        .and_then(|x| log_step("After add".to_string(), x))
        .map(|x| format!("Final result: {}", x));

    let result = pipeline.run(&env).await.unwrap();
    println!("  {}", result);
}

// ==================== Error Recovery ====================

/// Example 6: Error handling and recovery
///
/// Demonstrates handling errors within pipelines.
async fn example_error_recovery() {
    println!("\n=== Example 6: Error Handling and Recovery ===");

    #[derive(Clone)]
    struct Env;

    fn parse_number(s: String) -> impl Effect<Output = i32, Error = String, Env = Env> {
        from_fn(move |_env: &Env| {
            s.parse::<i32>()
                .map_err(|_| format!("Failed to parse '{}'", s))
        })
    }

    fn safe_divide(a: i32, b: i32) -> impl Effect<Output = i32, Error = String, Env = Env> {
        from_fn(move |_env: &Env| {
            if b == 0 {
                Err("Division by zero".to_string())
            } else {
                Ok(a / b)
            }
        })
    }

    // Success case
    let pipeline1 = parse_number("20".to_string())
        .and_then(|n| safe_divide(n, 4))
        .map(|result| format!("Result: {}", result));

    match pipeline1.run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Parse error
    let pipeline2 = parse_number("not-a-number".to_string())
        .and_then(|n| safe_divide(n, 4))
        .map(|result| format!("Result: {}", result));

    match pipeline2.run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Division error
    let pipeline3 = parse_number("20".to_string())
        .and_then(|n| safe_divide(n, 0))
        .map(|result| format!("Result: {}", result));

    match pipeline3.run(&Env).await {
        Ok(msg) => println!("  ✓ {}", msg),
        Err(e) => println!("  ✗ Error: {}", e),
    }
}

// ==================== Collecting Results ====================

/// Example 7: Processing multiple items
///
/// Demonstrates processing collections through pipelines.
async fn example_batch_processing() {
    println!("\n=== Example 7: Batch Processing ===");

    #[derive(Clone)]
    struct Env;

    fn process_item(item: i32) -> impl Effect<Output = String, Error = String, Env = Env> {
        pure::<_, String, Env>(item)
            .and_then(|n| {
                if n > 0 {
                    pure::<_, String, Env>(n).boxed()
                } else {
                    fail::<_, _, Env>(format!("{} is not positive", n)).boxed()
                }
            })
            .map(|n| n * 2)
            .map(|n| format!("Processed: {}", n))
    }

    let items = vec![1, 2, -3, 4, 5];

    println!("  Processing {} items...", items.len());
    for item in items {
        match process_item(item).run(&Env).await {
            Ok(result) => println!("    ✓ {}", result),
            Err(e) => println!("    ✗ Skipped ({})", e),
        }
    }
}

// ==================== Complex Pipeline ====================

/// Example 8: Real-world data processing pipeline
///
/// Demonstrates a realistic multi-stage pipeline.
async fn example_complex_pipeline() {
    println!("\n=== Example 8: Complex Pipeline ===");

    #[derive(Debug, Clone)]
    struct RawData {
        text: String,
    }

    #[derive(Debug)]
    struct CleanData {
        words: Vec<String>,
    }

    #[derive(Debug)]
    struct Statistics {
        word_count: usize,
        avg_word_length: f64,
    }

    #[derive(Clone)]
    struct Env;

    // Stage 1: Clean and tokenize
    fn clean_and_tokenize(
        raw: RawData,
    ) -> impl Effect<Output = CleanData, Error = String, Env = Env> {
        pure::<_, String, Env>(raw.text)
            .map(|s| s.to_lowercase())
            .map(|s| {
                s.chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
            })
            .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
            .and_then(|words| {
                if !words.is_empty() {
                    pure::<_, String, Env>(words).boxed()
                } else {
                    fail::<_, _, Env>("No valid words found".to_string()).boxed()
                }
            })
            .map(|words| CleanData { words })
    }

    // Stage 2: Compute statistics
    fn compute_stats(data: CleanData) -> impl Effect<Output = Statistics, Error = String, Env = Env> {
        pure::<_, String, Env>(data).map(|d| {
            let word_count = d.words.len();
            let total_chars: usize = d.words.iter().map(|w| w.len()).sum();
            let avg_word_length = total_chars as f64 / word_count as f64;

            Statistics {
                word_count,
                avg_word_length,
            }
        })
    }

    // Complete pipeline
    fn analyze_text(
        raw: RawData,
    ) -> impl Effect<Output = Statistics, Error = String, Env = Env> {
        clean_and_tokenize(raw).and_then_auto(compute_stats)
    }

    // Test with valid data
    let data1 = RawData {
        text: "Hello, World! This is a test.".to_string(),
    };

    match analyze_text(data1).run(&Env).await {
        Ok(stats) => {
            println!("  ✓ Analysis complete:");
            println!("    Words: {}", stats.word_count);
            println!("    Avg word length: {:.2}", stats.avg_word_length);
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Test with invalid data (no words)
    let data2 = RawData {
        text: "!!!".to_string(),
    };

    match analyze_text(data2).run(&Env).await {
        Ok(_) => println!("  ✓ Analysis complete"),
        Err(e) => println!("  ✗ Error: {}", e),
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Pipeline Examples");
    println!("=================");
    println!();
    println!("Demonstrates building data transformation pipelines:");
    println!("- Chain operations with map and and_then");
    println!("- Mix pure and effectful transformations");
    println!("- Handle errors gracefully");
    println!("- Process data in stages");

    example_sequential_transformations().await;
    example_pure_functions().await;
    example_filtering().await;
    example_branching().await;
    example_effectful_transformations().await;
    example_error_recovery().await;
    example_batch_processing().await;
    example_complex_pipeline().await;

    println!("\n=== All examples completed successfully! ===");
}
