//! Refined Types Example
//!
//! This example demonstrates the "parse, don't validate" pattern using refined types.
//! Refined types encode invariants in the type system, so once validated at boundaries,
//! the values are guaranteed to satisfy their constraints throughout the codebase.
//!
//! Run with: cargo run --example refined

use stillwater::refined::{
    And, BoundedString, FieldError, MaxLength, MinLength, NonEmpty, NonEmptyString,
    NonEmptyTrimmedString, NonZeroU32, Or, Percentage, Port, Positive, PositiveI32, Predicate,
    Refined, RefinedValidationExt, Trimmed, ValidationFieldExt,
};
use stillwater::Validation;

fn main() {
    println!("=== Refined Types Example ===\n");

    basic_usage();
    type_aliases();
    custom_predicates();
    predicate_combinators();
    validation_integration();
    real_world_example();
}

/// Demonstrates basic refined type usage
fn basic_usage() {
    println!("--- Basic Usage ---\n");

    // Create refined types - validate at the boundary
    let name = NonEmptyString::new("Alice".to_string());
    println!("NonEmptyString::new(\"Alice\"): {:?}", name.is_ok());

    let empty = NonEmptyString::new("".to_string());
    println!("NonEmptyString::new(\"\"): {:?}", empty.is_err());

    // Once created, access is zero-cost
    if let Ok(name) = name {
        // get() returns a reference to the inner value
        println!("name.get(): {}", name.get());

        // Deref allows direct access
        println!("name.len(): {}", name.len());

        // into_inner() consumes the wrapper
        let inner: String = name.into_inner();
        println!("inner: {}", inner);
    }

    // try_map allows transformations while re-checking the predicate
    let n = PositiveI32::new(42).unwrap();
    let doubled = n.try_map(|x| x * 2);
    println!("\nPositiveI32(42).try_map(|x| x * 2): {:?}", doubled);

    let negated = PositiveI32::new(5).unwrap().try_map(|x| -x);
    println!("PositiveI32(5).try_map(|x| -x): {:?}", negated.is_err());

    println!();
}

/// Demonstrates convenient type aliases
fn type_aliases() {
    println!("--- Type Aliases ---\n");

    // String aliases
    println!("String aliases:");
    println!(
        "  NonEmptyString::new(\"hello\"): {}",
        NonEmptyString::new("hello".to_string()).is_ok()
    );
    println!(
        "  NonEmptyTrimmedString::new(\"  hello  \"): {}",
        NonEmptyTrimmedString::new("  hello  ".to_string()).is_err()
    );

    // Numeric aliases
    println!("\nNumeric aliases:");
    println!("  PositiveI32::new(42): {}", PositiveI32::new(42).is_ok());
    println!("  PositiveI32::new(0): {}", PositiveI32::new(0).is_err());
    println!("  NonZeroU32::new(1): {}", NonZeroU32::new(1).is_ok());
    println!("  NonZeroU32::new(0): {}", NonZeroU32::new(0).is_err());

    // Domain-specific aliases
    println!("\nDomain-specific aliases:");
    println!("  Port::new(443): {}", Port::new(443).is_ok());
    println!("  Port::new(0): {}", Port::new(0).is_err());

    println!("  Percentage::new(50): {}", Percentage::new(50).is_ok());
    println!("  Percentage::new(101): {}", Percentage::new(101).is_err());

    // Bounded strings
    type Username = BoundedString<20>;
    println!(
        "\n  BoundedString<20>::new(\"alice\"): {}",
        Username::new("alice".to_string()).is_ok()
    );
    println!(
        "  BoundedString<20>::new(\"this_is_a_very_long_username\"): {}",
        Username::new("this_is_a_very_long_username".to_string()).is_err()
    );

    println!();
}

/// Demonstrates defining custom predicates
fn custom_predicates() {
    println!("--- Custom Predicates ---\n");

    // Define a predicate for even numbers
    pub struct Even;

    impl Predicate<i32> for Even {
        type Error = &'static str;

        fn check(value: &i32) -> Result<(), Self::Error> {
            if value % 2 == 0 {
                Ok(())
            } else {
                Err("value must be even")
            }
        }

        fn description() -> &'static str {
            "even number"
        }
    }

    type EvenI32 = Refined<i32, Even>;

    println!("Custom Even predicate:");
    println!("  EvenI32::new(42): {}", EvenI32::new(42).is_ok());
    println!("  EvenI32::new(41): {}", EvenI32::new(41).is_err());

    // Define a predicate for valid email (simplified)
    pub struct ValidEmail;

    impl Predicate<String> for ValidEmail {
        type Error = &'static str;

        fn check(value: &String) -> Result<(), Self::Error> {
            if value.contains('@') && value.contains('.') && value.len() >= 5 {
                Ok(())
            } else {
                Err("invalid email format")
            }
        }
    }

    type Email = Refined<String, ValidEmail>;

    println!("\nCustom ValidEmail predicate:");
    println!(
        "  Email::new(\"user@example.com\"): {}",
        Email::new("user@example.com".to_string()).is_ok()
    );
    println!(
        "  Email::new(\"invalid\"): {}",
        Email::new("invalid".to_string()).is_err()
    );

    println!();
}

/// Demonstrates predicate combinators
fn predicate_combinators() {
    println!("--- Predicate Combinators ---\n");

    // And combinator: both predicates must hold
    type CleanString = Refined<String, And<NonEmpty, Trimmed>>;

    println!("And<NonEmpty, Trimmed>:");
    println!(
        "  CleanString::new(\"hello\"): {}",
        CleanString::new("hello".to_string()).is_ok()
    );
    println!(
        "  CleanString::new(\"\"): {}",
        CleanString::new("".to_string()).is_err()
    );
    println!(
        "  CleanString::new(\"  hello  \"): {}",
        CleanString::new("  hello  ".to_string()).is_err()
    );

    // Complex composition with And
    type ValidUsername = Refined<String, And<And<NonEmpty, Trimmed>, MaxLength<20>>>;

    println!("\nAnd<And<NonEmpty, Trimmed>, MaxLength<20>>:");
    println!(
        "  ValidUsername::new(\"alice\"): {}",
        ValidUsername::new("alice".to_string()).is_ok()
    );
    println!(
        "  ValidUsername::new(\"\"): {}",
        ValidUsername::new("".to_string()).is_err()
    );
    println!(
        "  ValidUsername::new(\"this_is_way_too_long_for_a_username\"): {}",
        ValidUsername::new("this_is_way_too_long_for_a_username".to_string()).is_err()
    );

    // Or combinator: at least one predicate must hold
    use stillwater::refined::Negative;
    type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;

    println!("\nOr<Positive, Negative> (alternative NonZero):");
    println!("  NonZeroAlt::new(5): {}", NonZeroAlt::new(5).is_ok());
    println!("  NonZeroAlt::new(-5): {}", NonZeroAlt::new(-5).is_ok());
    println!("  NonZeroAlt::new(0): {}", NonZeroAlt::new(0).is_err());

    // Not combinator
    use stillwater::refined::Not;
    type NotPositive = Refined<i32, Not<Positive>>;

    println!("\nNot<Positive>:");
    println!("  NotPositive::new(0): {}", NotPositive::new(0).is_ok());
    println!("  NotPositive::new(-5): {}", NotPositive::new(-5).is_ok());
    println!("  NotPositive::new(5): {}", NotPositive::new(5).is_err());

    // MinLength and MaxLength together
    type Password = Refined<String, And<MinLength<8>, MaxLength<128>>>;

    println!("\nAnd<MinLength<8>, MaxLength<128>> (Password):");
    println!(
        "  Password::new(\"short\"): {}",
        Password::new("short".to_string()).is_err()
    );
    println!(
        "  Password::new(\"secure_password_123\"): {}",
        Password::new("secure_password_123".to_string()).is_ok()
    );

    println!();
}

/// Demonstrates validation integration with error accumulation
fn validation_integration() {
    println!("--- Validation Integration ---\n");

    // Single validation
    let result = PositiveI32::validate(42);
    println!(
        "PositiveI32::validate(42).is_success(): {}",
        result.is_success()
    );

    let result = PositiveI32::validate(-5);
    println!(
        "PositiveI32::validate(-5).is_failure(): {}",
        result.is_failure()
    );

    // Error accumulation with validate_vec
    println!("\nError accumulation:");

    fn validate_user(
        name: String,
        age: i32,
    ) -> Validation<(NonEmptyString, PositiveI32), Vec<&'static str>> {
        let v1 = NonEmptyString::validate_vec(name);
        let v2 = PositiveI32::validate_vec(age);
        v1.and(v2)
    }

    let result = validate_user("Alice".to_string(), 25);
    println!(
        "  validate_user(\"Alice\", 25).is_success(): {}",
        result.is_success()
    );

    let result = validate_user("".to_string(), -5);
    if let Validation::Failure(errors) = &result {
        println!("  validate_user(\"\", -5) errors: {}", errors.len());
    }

    // Field context with with_field
    println!("\nField context:");

    let result = NonEmptyString::validate("".to_string()).with_field("username");
    if let Validation::Failure(err) = result {
        println!("  Field: {}", err.field);
        println!("  Error: {}", err.error);
    }

    // Using validate_field for direct field context
    let result = NonEmptyString::validate_field("".to_string(), "email");
    if let Validation::Failure(err) = result {
        println!("\n  validate_field error: {}", err);
    }

    // Combined field validation
    println!("\nCombined field validation:");

    fn validate_registration(
        username: String,
        age: i32,
    ) -> Validation<(NonEmptyString, PositiveI32), Vec<FieldError<&'static str>>> {
        let v1 = NonEmptyString::validate(username)
            .with_field("username")
            .map_err(|e| vec![e]);
        let v2 = PositiveI32::validate(age)
            .with_field("age")
            .map_err(|e| vec![e]);
        v1.and(v2)
    }

    let result = validate_registration("".to_string(), -5);
    if let Validation::Failure(errors) = result {
        for err in &errors {
            println!("  {}: {}", err.field, err.error);
        }
    }

    println!();
}

/// Real-world example: User registration
fn real_world_example() {
    println!("--- Real-World Example: User Registration ---\n");

    // Define domain types
    type Username = Refined<String, And<And<NonEmpty, Trimmed>, MaxLength<30>>>;
    type Password = Refined<String, And<MinLength<8>, MaxLength<128>>>;

    // User struct with refined types - invariants encoded in types
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        username: Username,
        password: Password,
        age: PositiveI32,
        port: Option<Port>,
    }

    // Validation function that accumulates all errors
    fn validate_registration(
        username: String,
        password: String,
        age: i32,
        port: Option<u16>,
    ) -> Validation<User, Vec<String>> {
        let v_username = Username::validate(username).map_err(|e| vec![format!("username: {}", e)]);
        let v_password = Password::validate(password).map_err(|e| vec![format!("password: {}", e)]);
        let v_age = PositiveI32::validate(age).map_err(|e| vec![format!("age: {}", e)]);
        let v_port = match port {
            Some(p) => Port::validate(p)
                .map(Some)
                .map_err(|e| vec![format!("port: {}", e)]),
            None => Validation::Success(None),
        };

        v_username.and(v_password).and(v_age).and(v_port).map(
            |(((username, password), age), port)| User {
                username,
                password,
                age,
                port,
            },
        )
    }

    // Valid registration
    let result = validate_registration(
        "alice".to_string(),
        "secure_password_123".to_string(),
        25,
        Some(8080),
    );
    println!("Valid registration:");
    match result {
        Validation::Success(user) => {
            println!("  Success! Username: {}", user.username.get());
        }
        Validation::Failure(errors) => {
            println!("  Errors: {:?}", errors);
        }
    }

    // Invalid registration - multiple errors
    let result = validate_registration(
        "".to_string(),      // empty username
        "short".to_string(), // password too short
        -5,                  // negative age
        Some(0),             // invalid port
    );
    println!("\nInvalid registration (all fields wrong):");
    match result {
        Validation::Success(_) => {
            println!("  Unexpected success");
        }
        Validation::Failure(errors) => {
            println!("  {} errors found:", errors.len());
            for err in errors {
                println!("    - {}", err);
            }
        }
    }

    // Function that accepts only validated data
    fn process_user(user: User) {
        // No validation needed here - types guarantee constraints!
        println!(
            "\nProcessing user {} (age {})",
            user.username.get(),
            user.age.get()
        );
        if let Some(port) = user.port {
            println!("  Custom port: {}", port.get());
        }
    }

    // Only valid users can reach this function
    if let Validation::Success(user) = validate_registration(
        "bob".to_string(),
        "another_secure_password".to_string(),
        30,
        None,
    ) {
        process_user(user);
    }

    println!();
}
