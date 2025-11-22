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

use stillwater::Effect;

// ==================== Sequential Transformations ====================

/// Example 1: Simple data transformation pipeline
///
/// Demonstrates chaining pure transformations using map.
async fn example_sequential_transformations() {
    println!("\n=== Example 1: Sequential Transformations ===");

    struct Env;

    // Pipeline: number -> double -> add 10 -> to string
    let pipeline = Effect::pure(5)
        .map(|x| x * 2)
        .tap(|x| {
            println!("  After doubling: {}", x);
            Effect::<(), String, Env>::pure(())
        })
        .map(|x| x + 10)
        .tap(|x| {
            println!("  After adding 10: {}", x);
            Effect::<(), String, Env>::pure(())
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

    struct Env;

    // Build pipeline from pure functions
    let input = "  Hello,   World!  ".to_string();
    let pipeline = Effect::pure(input.clone())
        .tap(|s| {
            println!("  Input: '{}'", s);
            Effect::<(), String, Env>::pure(())
        })
        .map(normalize)
        .tap(|s| {
            println!("  Normalized: '{}'", s);
            Effect::<(), String, Env>::pure(())
        })
        .map(remove_special_chars)
        .tap(|s| {
            println!("  Special chars removed: '{}'", s);
            Effect::<(), String, Env>::pure(())
        })
        .map(collapse_whitespace)
        .tap(|s| {
            println!("  Whitespace collapsed: '{}'", s);
            Effect::<(), String, Env>::pure(())
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

    struct Env;

    // Pipeline with validation
    fn process_age(age: i32) -> Effect<String, String, Env> {
        Effect::pure(age)
            .and_then(|a| {
                if a >= 0 {
                    Effect::pure(a)
                } else {
                    Effect::fail(format!("Age {} is negative", a))
                }
            })
            .and_then(|a| {
                if a <= 150 {
                    Effect::pure(a)
                } else {
                    Effect::fail(format!("Age {} is too high", a))
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

    struct Env;

    fn classify_and_process(n: i32) -> Effect<String, String, Env> {
        Effect::pure(n).and_then(|num| {
            if num < 0 {
                Effect::pure(format!("{} is negative", num))
            } else if num % 2 == 0 {
                Effect::pure(format!("{} is even", num))
            } else {
                Effect::pure(format!("{} is odd", num))
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

    struct Logger {
        prefix: String,
    }

    struct Env {
        logger: Logger,
    }

    impl AsRef<Logger> for Env {
        fn as_ref(&self) -> &Logger {
            &self.logger
        }
    }

    // Effectful logging
    fn log_step<Env: AsRef<Logger> + Sync + 'static>(
        message: String,
        value: i32,
    ) -> Effect<i32, String, Env> {
        Effect::from_fn(move |env: &Env| {
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
    let pipeline = Effect::pure(10)
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

    struct Env;

    fn parse_number(s: String) -> Effect<i32, String, Env> {
        Effect::from_fn(move |_env: &Env| {
            s.parse::<i32>()
                .map_err(|_| format!("Failed to parse '{}'", s))
        })
    }

    fn safe_divide(a: i32, b: i32) -> Effect<i32, String, Env> {
        Effect::from_fn(move |_env: &Env| {
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

    struct Env;

    fn process_item(item: i32) -> Effect<String, String, Env> {
        Effect::pure(item)
            .and_then(|n| {
                if n > 0 {
                    Effect::pure(n)
                } else {
                    Effect::fail(format!("{} is not positive", n))
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

    struct Env;

    // Stage 1: Clean and tokenize
    fn clean_and_tokenize(raw: RawData) -> Effect<CleanData, String, Env> {
        Effect::pure(raw.text)
            .map(|s| s.to_lowercase())
            .map(|s| {
                s.chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
            })
            .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
            .and_then(|words| {
                if !words.is_empty() {
                    Effect::pure(words)
                } else {
                    Effect::fail("No valid words found".to_string())
                }
            })
            .map(|words| CleanData { words })
    }

    // Stage 2: Compute statistics
    fn compute_stats(data: CleanData) -> Effect<Statistics, String, Env> {
        Effect::pure(data).map(|d| {
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
    fn analyze_text(raw: RawData) -> Effect<Statistics, String, Env> {
        clean_and_tokenize(raw).and_then(compute_stats)
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
