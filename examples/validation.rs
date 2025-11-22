//! Validation Example
//!
//! Demonstrates the Validation type and error accumulation patterns.
//! Shows practical patterns including:
//! - Basic validation (Success/Failure)
//! - Combining multiple validations
//! - Error accumulation with Semigroup
//! - Mapping and transforming validations
//! - Business rule validations

use stillwater::Validation;

// ==================== Basic Validation ====================

/// Example 1: Simple validation
///
/// Demonstrates creating Success and Failure validations.
fn example_basic_validation() {
    println!("\n=== Example 1: Basic Validation ===");

    // Success case
    let success: Validation<i32, String> = Validation::success(42);
    println!("Success: {:?}", success);

    // Failure case
    let failure: Validation<i32, String> = Validation::failure("Something went wrong".to_string());
    println!("Failure: {:?}", failure);

    // Pattern matching
    match success {
        Validation::Success(value) => println!("Got value: {}", value),
        Validation::Failure(error) => println!("Got error: {}", error),
    }
}

// ==================== Simple Validation Functions ====================

/// Example 2: Building validation functions
///
/// Demonstrates creating reusable validation functions.
fn example_validation_functions() {
    println!("\n=== Example 2: Validation Functions ===");

    // Validation function: check if number is positive
    fn validate_positive(n: i32) -> Validation<i32, String> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(format!("{} is not positive", n))
        }
    }

    // Validation function: check if number is even
    fn validate_even(n: i32) -> Validation<i32, String> {
        if n % 2 == 0 {
            Validation::success(n)
        } else {
            Validation::failure(format!("{} is not even", n))
        }
    }

    println!("Validate positive:");
    println!("  10: {:?}", validate_positive(10));
    println!("  -5: {:?}", validate_positive(-5));

    println!("\nValidate even:");
    println!("  10: {:?}", validate_even(10));
    println!("  7: {:?}", validate_even(7));
}

// ==================== Combining Validations ====================

/// Example 3: Combining multiple validations
///
/// Demonstrates using and() to chain validations sequentially.
fn example_combining_validations() {
    println!("\n=== Example 3: Combining Validations ===");

    fn validate_positive(n: i32) -> Validation<i32, String> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(format!("{} must be positive", n))
        }
    }

    fn validate_even(n: i32) -> Validation<i32, String> {
        if n % 2 == 0 {
            Validation::success(n)
        } else {
            Validation::failure(format!("{} must be even", n))
        }
    }

    fn validate_less_than_100(n: i32) -> Validation<i32, String> {
        if n < 100 {
            Validation::success(n)
        } else {
            Validation::failure(format!("{} must be less than 100", n))
        }
    }

    // Chain validations with and()
    let result1 = validate_positive(50)
        .and(validate_even(50))
        .and(validate_less_than_100(50));
    println!("Validate 50 (positive, even, <100): {:?}", result1);

    // Fails on first check
    let result2 = validate_positive(-10)
        .and(validate_even(-10))
        .and(validate_less_than_100(-10));
    println!("Validate -10: {:?}", result2);

    // Passes first but fails second
    let result3 = validate_positive(7)
        .and(validate_even(7))
        .and(validate_less_than_100(7));
    println!("Validate 7: {:?}", result3);
}

// ==================== Error Accumulation ====================

/// Example 4: Accumulating errors from multiple validations
///
/// Demonstrates using all() to collect ALL errors, not just the first one.
/// This is the key feature that distinguishes Validation from Result.
fn example_error_accumulation() {
    println!("\n=== Example 4: Error Accumulation ===");

    fn validate_min_length(s: &str, min: usize) -> Validation<(), Vec<String>> {
        if s.len() >= min {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Must be at least {} characters (got {})",
                min,
                s.len()
            )])
        }
    }

    fn validate_has_uppercase(s: &str) -> Validation<(), Vec<String>> {
        if s.chars().any(|c| c.is_uppercase()) {
            Validation::success(())
        } else {
            Validation::failure(vec![
                "Must contain at least one uppercase letter".to_string()
            ])
        }
    }

    fn validate_has_number(s: &str) -> Validation<(), Vec<String>> {
        if s.chars().any(|c| c.is_numeric()) {
            Validation::success(())
        } else {
            Validation::failure(vec!["Must contain at least one number".to_string()])
        }
    }

    fn validate_password(password: &str) -> Validation<String, Vec<String>> {
        let v1 = validate_min_length(password, 8);
        let v2 = validate_has_uppercase(password);
        let v3 = validate_has_number(password);
        Validation::<((), (), ()), Vec<String>>::all((v1, v2, v3)).map(|_| password.to_string())
    }

    // Good password
    println!("Password 'Secret123':");
    match validate_password("Secret123") {
        Validation::Success(_) => println!("  Valid!"),
        Validation::Failure(errors) => {
            println!("  Invalid:");
            for error in errors {
                println!("    - {}", error);
            }
        }
    }

    // Bad password - accumulates ALL errors
    println!("\nPassword 'weak':");
    match validate_password("weak") {
        Validation::Success(_) => println!("  Valid!"),
        Validation::Failure(errors) => {
            println!("  Invalid ({} errors):", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Mapping and Transforming ====================

/// Example 5: Using map to transform validated values
///
/// Demonstrates how to transform values inside Validation.
fn example_mapping() {
    println!("\n=== Example 5: Mapping and Transforming ===");

    fn validate_age(age: i32) -> Validation<i32, String> {
        if (0..=150).contains(&age) {
            Validation::success(age)
        } else {
            Validation::failure(format!("Age {} is invalid", age))
        }
    }

    // Transform age to age category
    let result = validate_age(25).map(|age| {
        if age < 18 {
            "Minor"
        } else if age < 65 {
            "Adult"
        } else {
            "Senior"
        }
    });

    println!("Age 25 category: {:?}", result);

    // Map over failure - doesn't execute
    let result2 = validate_age(200).map(|age| format!("Valid age: {}", age));
    println!("Age 200: {:?}", result2);
}

// ==================== Business Rules Validation ====================

/// Example 6: Practical business rules validation
///
/// Demonstrates a real-world scenario: validating an order.
fn example_business_rules() {
    println!("\n=== Example 6: Business Rules Validation ===");

    #[derive(Debug)]
    struct Order {
        quantity: u32,
        unit_price: f64,
        discount_percent: f64,
    }

    impl Order {
        fn total(&self) -> f64 {
            let subtotal = self.quantity as f64 * self.unit_price;
            subtotal * (1.0 - self.discount_percent / 100.0)
        }
    }

    fn validate_quantity(qty: u32) -> Validation<(), Vec<String>> {
        if qty > 0 && qty <= 1000 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Quantity must be between 1 and 1000 (got {})",
                qty
            )])
        }
    }

    fn validate_price(price: f64) -> Validation<(), Vec<String>> {
        if price > 0.0 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!("Price must be positive (got {})", price)])
        }
    }

    fn validate_discount(discount: f64) -> Validation<(), Vec<String>> {
        if (0.0..=100.0).contains(&discount) {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Discount must be between 0 and 100 (got {})",
                discount
            )])
        }
    }

    fn validate_order(order: &Order) -> Validation<Order, Vec<String>> {
        let v1 = validate_quantity(order.quantity);
        let v2 = validate_price(order.unit_price);
        let v3 = validate_discount(order.discount_percent);
        Validation::<((), (), ()), Vec<String>>::all((v1, v2, v3)).map(|_| Order {
            quantity: order.quantity,
            unit_price: order.unit_price,
            discount_percent: order.discount_percent,
        })
    }

    // Valid order
    let valid_order = Order {
        quantity: 10,
        unit_price: 99.99,
        discount_percent: 10.0,
    };
    println!("\nValid order:");
    match validate_order(&valid_order) {
        Validation::Success(order) => {
            println!("  Order validated!");
            println!("  Total: ${:.2}", order.total());
        }
        Validation::Failure(errors) => {
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid order - multiple errors
    let invalid_order = Order {
        quantity: 0,
        unit_price: -10.0,
        discount_percent: 150.0,
    };
    println!("\nInvalid order:");
    match validate_order(&invalid_order) {
        Validation::Success(_) => println!("  Order validated!"),
        Validation::Failure(errors) => {
            println!("  {} validation errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Collecting Results ====================

/// Example 7: Validating collections with all_vec
///
/// Demonstrates validating a collection of items.
fn example_collection_validation() {
    println!("\n=== Example 7: Collection Validation ===");

    fn validate_email(email: &str) -> Validation<String, Vec<String>> {
        if email.contains('@') && email.len() > 3 {
            Validation::success(email.to_string())
        } else {
            Validation::failure(vec![format!("Invalid email: {}", email)])
        }
    }

    let emails = ["alice@example.com", "bob@test.com", "charlie@mail.org"];

    println!("Valid emails:");
    let validations: Vec<_> = emails.iter().map(|e| validate_email(e)).collect();
    match Validation::all_vec(validations) {
        Validation::Success(valid_emails) => {
            println!("  All {} emails are valid:", valid_emails.len());
            for email in valid_emails {
                println!("    - {}", email);
            }
        }
        Validation::Failure(errors) => {
            println!("  Validation failed:");
            for error in errors {
                println!("    - {}", error);
            }
        }
    }

    // Mix of valid and invalid
    let mixed_emails = ["alice@example.com", "invalid", "bob@test.com", "bad"];

    println!("\nMixed emails:");
    let validations: Vec<_> = mixed_emails.iter().map(|e| validate_email(e)).collect();
    match Validation::all_vec(validations) {
        Validation::Success(valid_emails) => {
            println!("  All emails are valid: {:?}", valid_emails);
        }
        Validation::Failure(errors) => {
            println!("  {} validation errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}

// ==================== Main ====================

fn main() {
    println!("Validation Examples");
    println!("===================");

    example_basic_validation();
    example_validation_functions();
    example_combining_validations();
    example_error_accumulation();
    example_mapping();
    example_business_rules();
    example_collection_validation();

    println!("\n=== All examples completed successfully! ===");
}
