//! NonEmptyVec Example
//!
//! Demonstrates the NonEmptyVec type for collections guaranteed to have at least one element.
//! Shows practical patterns including:
//! - Creating non-empty vectors safely
//! - Operations that never fail (head, last)
//! - Functional operations (map, filter)
//! - Integration with Validation for error accumulation
//! - Semigroup instance for concatenation

use stillwater::{NonEmptyVec, Semigroup, Validation};

// ==================== Basic Creation ====================

/// Example 1: Creating NonEmptyVec instances
///
/// Demonstrates different ways to construct a NonEmptyVec.
fn example_basic_creation() {
    println!("\n=== Example 1: Basic Creation ===");

    // Create from head + tail
    let nev1 = NonEmptyVec::new(1, vec![2, 3, 4]);
    println!("new(1, vec![2, 3, 4]): {:?}", nev1);
    println!("  head: {}, len: {}", nev1.head(), nev1.len());

    // Create singleton (single element)
    let nev2 = NonEmptyVec::singleton(42);
    println!("\nsingleton(42): {:?}", nev2);
    println!("  head: {}, len: {}", nev2.head(), nev2.len());

    // Safe creation from Vec (returns Option)
    let vec_with_items = vec![10, 20, 30];
    match NonEmptyVec::from_vec(vec_with_items) {
        Some(nev) => println!("\nfrom_vec(vec![10, 20, 30]): {:?}", nev),
        None => println!("\nfrom_vec returned None (empty vec)"),
    }

    // Empty vec returns None
    let empty_vec: Vec<i32> = vec![];
    match NonEmptyVec::from_vec(empty_vec) {
        Some(nev) => println!("from_vec(vec![]): {:?}", nev),
        None => println!("from_vec(vec![]): None (as expected)"),
    }
}

// ==================== Safe Operations ====================

/// Example 2: Operations that never fail
///
/// Demonstrates operations that are guaranteed to succeed on NonEmptyVec.
fn example_safe_operations() {
    println!("\n=== Example 2: Safe Operations ===");

    let nev = NonEmptyVec::new(10, vec![20, 30, 40, 50]);
    println!("NonEmptyVec: {:?}", nev);

    // head() always succeeds (no Option needed)
    println!("  head: {}", nev.head());

    // last() always succeeds
    println!("  last: {}", nev.last());

    // tail is always available (might be empty slice)
    println!("  tail: {:?}", nev.tail());

    // len is always >= 1
    println!("  len: {}", nev.len());
    println!("  is_empty: {}", nev.is_empty()); // Always false

    // Indexing works like Vec
    println!("  nev[0]: {}", nev[0]);
    println!("  nev[2]: {}", nev[2]);

    // Singleton example
    let single = NonEmptyVec::singleton(99);
    println!("\nSingleton: {:?}", single);
    println!("  head: {}, last: {}", single.head(), single.last());
    println!("  tail: {:?}", single.tail()); // Empty slice
}

// ==================== Mutation ====================

/// Example 3: Push and pop operations
///
/// Demonstrates mutable operations on NonEmptyVec.
fn example_mutation() {
    println!("\n=== Example 3: Mutation ===");

    let mut nev = NonEmptyVec::singleton(1);
    println!("Start: {:?}", nev);

    // Push adds to the end
    nev.push(2);
    nev.push(3);
    nev.push(4);
    println!("After pushes: {:?}", nev);

    // Pop removes from the end (returns Option)
    println!("\nPopping elements:");
    while let Some(value) = nev.pop() {
        println!("  Popped: {}, remaining: {:?}", value, nev);
    }

    // Can't pop the last element (head)
    println!("Final state: {:?} (can't pop head)", nev);
    println!("Attempt to pop: {:?}", nev.pop()); // None
}

// ==================== Functional Operations ====================

/// Example 4: Map, filter, and iteration
///
/// Demonstrates functional programming operations on NonEmptyVec.
fn example_functional_operations() {
    println!("\n=== Example 4: Functional Operations ===");

    let nev = NonEmptyVec::new(1, vec![2, 3, 4, 5]);
    println!("Original: {:?}", nev);

    // map transforms all elements (preserves non-emptiness)
    let doubled = nev.clone().map(|x| x * 2);
    println!("map(|x| x * 2): {:?}", doubled);

    // map can change type
    let strings = nev.clone().map(|x| format!("Item {}", x));
    println!("map to strings: {:?}", strings);

    // filter may return empty Vec (since not all elements might pass)
    let evens = nev.clone().filter(|x| x % 2 == 0);
    println!("filter(even): {:?}", evens);

    let all_positive = nev.clone().filter(|x| x > &0);
    println!("filter(positive): {:?}", all_positive);

    // iter() for read-only iteration
    print!("iter sum: ");
    let sum: i32 = nev.iter().sum();
    println!("{}", sum);

    // into_iter() consumes the NonEmptyVec
    let nev_copy = nev.clone();
    let collected: Vec<_> = nev_copy.into_iter().collect();
    println!("into_iter().collect(): {:?}", collected);

    // Convert to regular Vec when needed
    let vec = nev.into_vec();
    println!("into_vec(): {:?}", vec);
}

// ==================== Semigroup ====================

/// Example 5: Combining NonEmptyVecs
///
/// Demonstrates using the Semigroup instance to concatenate NonEmptyVecs.
fn example_semigroup() {
    println!("\n=== Example 5: Semigroup (Concatenation) ===");

    let nev1 = NonEmptyVec::new(1, vec![2, 3]);
    let nev2 = NonEmptyVec::new(4, vec![5, 6]);
    let nev3 = NonEmptyVec::singleton(7);

    println!("nev1: {:?}", nev1);
    println!("nev2: {:?}", nev2);
    println!("nev3: {:?}", nev3);

    // combine concatenates two NonEmptyVecs
    let combined = nev1.combine(nev2);
    println!("\nnev1.combine(nev2): {:?}", combined);

    // Can chain multiple combines
    let all_combined = combined.combine(nev3);
    println!("all combined: {:?}", all_combined);

    // Useful for accumulating non-empty results
    let words = NonEmptyVec::new("Hello".to_string(), vec![]);
    let more_words = NonEmptyVec::new("World".to_string(), vec!["!".to_string()]);
    let sentence = words.combine(more_words);
    println!("\nCombining strings: {:?}", sentence);
}

// ==================== Validation Integration ====================

/// Example 6: Using NonEmptyVec with Validation
///
/// Demonstrates how NonEmptyVec ensures validation failures always have errors.
fn example_validation_integration() {
    println!("\n=== Example 6: Validation Integration ===");

    // Validation with NonEmptyVec ensures at least one error
    type ValidationError = String;
    type ValidResult<T> = Validation<T, NonEmptyVec<ValidationError>>;

    fn validate_email(email: &str) -> ValidResult<String> {
        if email.contains('@') {
            Validation::success(email.to_string())
        } else {
            Validation::fail("Email must contain @".to_string())
        }
    }

    fn validate_age(age: i32) -> ValidResult<i32> {
        if age >= 18 {
            Validation::success(age)
        } else {
            Validation::fail(format!("Age must be >= 18, got {}", age))
        }
    }

    fn validate_name(name: &str) -> ValidResult<String> {
        if !name.is_empty() {
            Validation::success(name.to_string())
        } else {
            Validation::fail("Name cannot be empty".to_string())
        }
    }

    // Success case
    println!("Valid inputs:");
    let result = validate_email("user@example.com")
        .and(validate_age(25))
        .and(validate_name("Alice"));

    match result {
        Validation::Success(((email, age), name)) => {
            println!("  ✓ Success: {} is {} years old", name, age);
            println!("    Email: {}", email);
        }
        Validation::Failure(errors) => {
            println!("  ✗ Errors:");
            for error in errors.iter() {
                println!("    - {}", error);
            }
        }
    }

    // Failure case - accumulates all errors
    println!("\nInvalid inputs:");
    let result = validate_email("invalid-email")
        .and(validate_age(15))
        .and(validate_name(""));

    match result {
        Validation::Success(_) => println!("  ✓ Success"),
        Validation::Failure(errors) => {
            println!("  ✗ {} errors accumulated:", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("    {}. {}", i + 1, error);
            }
        }
    }
}

// ==================== Real-world Use Case ====================

/// Example 7: Processing non-empty batches
///
/// Demonstrates a real-world scenario where NonEmptyVec prevents errors.
fn example_real_world_batch_processing() {
    println!("\n=== Example 7: Real-world - Batch Processing ===");

    #[derive(Debug, Clone)]
    struct Task {
        id: u32,
        priority: u8,
    }

    // Process a batch of tasks (guaranteed non-empty)
    fn process_batch(tasks: NonEmptyVec<Task>) -> (Task, Vec<Task>) {
        // Can safely get highest priority without Option
        let mut tasks_vec = tasks.into_vec();
        tasks_vec.sort_by_key(|t| std::cmp::Reverse(t.priority));

        let highest = tasks_vec.remove(0);
        (highest, tasks_vec)
    }

    // Create a batch
    let batch = NonEmptyVec::new(
        Task { id: 1, priority: 5 },
        vec![
            Task { id: 2, priority: 8 },
            Task { id: 3, priority: 3 },
            Task {
                id: 4,
                priority: 10,
            },
        ],
    );

    println!("Processing batch with {} tasks", batch.len());
    for task in batch.iter() {
        println!("  Task #{}: priority {}", task.id, task.priority);
    }

    let (highest, rest) = process_batch(batch);
    println!(
        "\nHighest priority: Task #{} (priority {})",
        highest.id, highest.priority
    );
    println!("Remaining {} tasks:", rest.len());
    for task in &rest {
        println!("  Task #{}: priority {}", task.id, task.priority);
    }

    // Type system prevents processing empty batches
    // This won't compile:
    // let empty_batch = NonEmptyVec::from_vec(vec![]).unwrap(); // Runtime panic!
}

// ==================== Aggregations ====================

/// Example 8: Safe aggregations (min, max, etc.)
///
/// Demonstrates operations that require non-empty collections.
fn example_aggregations() {
    println!("\n=== Example 8: Safe Aggregations ===");

    let numbers = NonEmptyVec::new(42, vec![17, 99, 8, 56]);
    println!("Numbers: {:?}", numbers);

    // These operations never fail on NonEmptyVec
    let min = numbers.iter().min().unwrap(); // Safe unwrap
    let max = numbers.iter().max().unwrap(); // Safe unwrap
    let sum: i32 = numbers.iter().sum();
    let avg = sum as f64 / numbers.len() as f64;

    println!("  min: {}", min);
    println!("  max: {}", max);
    println!("  sum: {}", sum);
    println!("  avg: {:.2}", avg);

    // Product example
    let factors = NonEmptyVec::new(2, vec![3, 5]);
    let product: i32 = factors.iter().product();
    println!("\nFactors: {:?}", factors);
    println!("  product: {}", product);
}

// ==================== Main ====================

fn main() {
    println!("NonEmptyVec Examples");
    println!("===================");

    example_basic_creation();
    example_safe_operations();
    example_mutation();
    example_functional_operations();
    example_semigroup();
    example_validation_integration();
    example_real_world_batch_processing();
    example_aggregations();

    println!("\n=== All examples completed! ===");
}
