//! Predicate Combinators Example
//!
//! This example demonstrates how to use composable predicate combinators
//! for building complex validation logic from simple, reusable pieces.
//!
//! Run with: cargo run --example predicates

use stillwater::predicate::*;
use stillwater::Validation;

fn main() {
    println!("=== Predicate Combinators Example ===\n");

    string_predicates();
    number_predicates();
    collection_predicates();
    logical_combinators();
    validation_integration();
    real_world_example();
}

/// Demonstrates string predicates
fn string_predicates() {
    println!("--- String Predicates ---\n");

    // Length predicates
    let p = len_between(3, 10);
    println!("len_between(3, 10):");
    println!("  'ab': {}", p.check("ab")); // false - too short
    println!("  'abc': {}", p.check("abc")); // true - exactly min
    println!("  'hello': {}", p.check("hello")); // true - in range
    println!("  '12345678901': {}", p.check("12345678901")); // false - too long

    // Convenience length predicates
    println!("\nlen_min(3).check('hi'): {}", len_min(3).check("hi")); // false
    println!("len_max(5).check('hello'): {}", len_max(5).check("hello")); // true
    println!("len_eq(5).check('hello'): {}", len_eq(5).check("hello")); // true

    // Content predicates
    println!(
        "\nstarts_with('http').check('https://example.com'): {}",
        starts_with("http").check("https://example.com")
    ); // true
    println!(
        "ends_with('.rs').check('main.rs'): {}",
        ends_with(".rs").check("main.rs")
    ); // true
    println!(
        "contains('@').check('user@example.com'): {}",
        contains("@").check("user@example.com")
    ); // true

    // Character predicates
    println!(
        "\nis_alphabetic().check('hello'): {}",
        is_alphabetic().check("hello")
    ); // true
    println!(
        "is_alphanumeric().check('hello123'): {}",
        is_alphanumeric().check("hello123")
    ); // true
    println!(
        "is_numeric().check('12345'): {}",
        is_numeric().check("12345")
    ); // true
    println!("is_ascii().check('hello'): {}", is_ascii().check("hello")); // true
    println!("is_ascii().check('héllo'): {}", is_ascii().check("héllo")); // false

    // Custom character predicates
    let valid_identifier = all_chars(|c: char| c.is_alphanumeric() || c == '_');
    println!("\nall_chars(alphanumeric or '_'):");
    println!("  'user_name': {}", valid_identifier.check("user_name")); // true
    println!("  'user-name': {}", valid_identifier.check("user-name")); // false (hyphen)

    let has_digit = any_char(char::is_numeric);
    println!("\nany_char(numeric):");
    println!("  'hello123': {}", has_digit.check("hello123")); // true
    println!("  'hello': {}", has_digit.check("hello")); // false

    println!();
}

/// Demonstrates number predicates
fn number_predicates() {
    println!("--- Number Predicates ---\n");

    // Comparison predicates
    println!("gt(5).check(&6): {}", gt(5).check(&6)); // true
    println!("gt(5).check(&5): {}", gt(5).check(&5)); // false
    println!("ge(5).check(&5): {}", ge(5).check(&5)); // true
    println!("lt(5).check(&4): {}", lt(5).check(&4)); // true
    println!("le(5).check(&5): {}", le(5).check(&5)); // true
    println!("eq(5).check(&5): {}", eq(5).check(&5)); // true
    println!("ne(5).check(&4): {}", ne(5).check(&4)); // true

    // Range predicate
    let valid_age = between(0, 150);
    println!("\nbetween(0, 150):");
    println!("  25: {}", valid_age.check(&25)); // true
    println!("  -5: {}", valid_age.check(&-5)); // false
    println!("  200: {}", valid_age.check(&200)); // false

    // Sign predicates
    println!(
        "\npositive::<i32>().check(&5): {}",
        positive::<i32>().check(&5)
    ); // true
    println!(
        "positive::<i32>().check(&0): {}",
        positive::<i32>().check(&0)
    ); // false
    println!(
        "negative::<i32>().check(&-5): {}",
        negative::<i32>().check(&-5)
    ); // true
    println!(
        "non_negative::<i32>().check(&0): {}",
        non_negative::<i32>().check(&0)
    ); // true

    // Works with floats too
    let valid_probability = between(0.0_f64, 1.0_f64);
    println!("\nbetween(0.0, 1.0) for f64:");
    println!("  0.5: {}", valid_probability.check(&0.5)); // true
    println!("  1.5: {}", valid_probability.check(&1.5)); // false

    println!();
}

/// Demonstrates collection predicates
fn collection_predicates() {
    println!("--- Collection Predicates ---\n");

    let numbers = vec![1, 2, 3, 4, 5];
    let empty: Vec<i32> = vec![];

    // Emptiness predicates
    println!("is_empty().check(&[1,2,3]): {}", is_empty().check(&numbers)); // false
    println!("is_empty().check(&[]): {}", is_empty().check(&empty)); // true
    println!(
        "is_not_empty().check(&[1,2,3]): {}",
        is_not_empty().check(&numbers)
    ); // true

    // Length predicates
    println!(
        "\nhas_len(5).check(&[1,2,3,4,5]): {}",
        has_len(5).check(&numbers)
    ); // true
    println!(
        "has_min_len(3).check(&[1,2,3,4,5]): {}",
        has_min_len(3).check(&numbers)
    ); // true
    println!(
        "has_max_len(10).check(&[1,2,3,4,5]): {}",
        has_max_len(10).check(&numbers)
    ); // true

    // Element predicates
    let all_positive = all(positive::<i32>());
    println!("\nall(positive):");
    println!("  [1,2,3]: {}", all_positive.check(&vec![1, 2, 3])); // true
    println!("  [1,-2,3]: {}", all_positive.check(&vec![1, -2, 3])); // false

    let any_gt_10 = any(gt(10));
    println!("\nany(gt(10)):");
    println!("  [1,2,15]: {}", any_gt_10.check(&vec![1, 2, 15])); // true
    println!("  [1,2,3]: {}", any_gt_10.check(&vec![1, 2, 3])); // false

    // Contains element
    println!(
        "\ncontains_element(3).check(&[1,2,3]): {}",
        contains_element(3).check(&vec![1, 2, 3])
    ); // true
    println!(
        "contains_element(5).check(&[1,2,3]): {}",
        contains_element(5).check(&vec![1, 2, 3])
    ); // false

    println!();
}

/// Demonstrates logical combinators (and, or, not, all_of, any_of, none_of)
fn logical_combinators() {
    println!("--- Logical Combinators ---\n");

    // AND combinator
    let in_range = gt(0).and(lt(100));
    println!("gt(0).and(lt(100)):");
    println!("  50: {}", in_range.check(&50)); // true
    println!("  0: {}", in_range.check(&0)); // false
    println!("  100: {}", in_range.check(&100)); // false

    // OR combinator
    let extreme = lt(0).or(gt(100));
    println!("\nlt(0).or(gt(100)):");
    println!("  -5: {}", extreme.check(&-5)); // true
    println!("  150: {}", extreme.check(&150)); // true
    println!("  50: {}", extreme.check(&50)); // false

    // NOT combinator
    let not_positive = positive::<i32>().not();
    println!("\npositive::<i32>().not():");
    println!("  -5: {}", not_positive.check(&-5)); // true
    println!("  0: {}", not_positive.check(&0)); // true
    println!("  5: {}", not_positive.check(&5)); // false

    // Chaining multiple combinators
    let complex = gt(0).and(lt(100)).or(eq(0));
    println!("\ngt(0).and(lt(100)).or(eq(0)):");
    println!("  0: {}", complex.check(&0)); // true (matches eq(0))
    println!("  50: {}", complex.check(&50)); // true (in range)
    println!("  -5: {}", complex.check(&-5)); // false

    // all_of - all predicates must be true (requires same type)
    let all_greater = all_of([gt(0), gt(-10), gt(-100)]);
    println!("\nall_of([gt(0), gt(-10), gt(-100)]):");
    println!("  50: {}", all_greater.check(&50)); // true
    println!("  -50: {}", all_greater.check(&-50)); // false (fails gt(0))

    // any_of - at least one must be true
    let special_values = any_of([eq(1), eq(5), eq(10)]);
    println!("\nany_of([eq(1), eq(5), eq(10)]):");
    println!("  5: {}", special_values.check(&5)); // true
    println!("  7: {}", special_values.check(&7)); // false

    // none_of - none must be true
    let no_special = none_of([eq(1), eq(5), eq(10)]);
    println!("\nnone_of([eq(1), eq(5), eq(10)]):");
    println!("  7: {}", no_special.check(&7)); // true
    println!("  5: {}", no_special.check(&5)); // false

    println!();
}

/// Demonstrates integration with the Validation type
fn validation_integration() {
    println!("--- Validation Integration ---\n");

    // Using validate() function
    let result = validate(String::from("hello"), len_min(3), "too short");
    println!("validate('hello', len_min(3), 'too short'): {:?}", result);

    let result = validate(String::from("hi"), len_min(3), "too short");
    println!("validate('hi', len_min(3), 'too short'): {:?}", result);

    // Using validate_with() for custom error messages
    let result = validate_with(String::from("hi"), len_min(3), |s| {
        format!("'{}' is too short (min 3 chars)", s)
    });
    println!(
        "\nvalidate_with('hi', len_min(3), <error_fn>): {:?}",
        result
    );

    // Using ensure() method on Validation for chaining
    let result = Validation::success(String::from("hello"))
        .ensure(len_min(3), "too short")
        .ensure(len_max(10), "too long")
        .ensure(is_alphabetic(), "must be alphabetic");
    println!("\nValidation chain with ensure():");
    println!("  'hello': {:?}", result);

    let result = Validation::success(String::from("hello123"))
        .ensure(len_min(3), "too short")
        .ensure(len_max(10), "too long")
        .ensure(is_alphabetic(), "must be alphabetic");
    println!("  'hello123': {:?}", result);

    // Validating numbers
    let result = validate(25, between(0, 150), "age out of range");
    println!(
        "\nvalidate(25, between(0, 150), 'age out of range'): {:?}",
        result
    );

    let result = validate(-5, between(0, 150), "age out of range");
    println!(
        "validate(-5, between(0, 150), 'age out of range'): {:?}",
        result
    );

    // Validating collections
    let result: Validation<Vec<i32>, &str> = validate(
        vec![1, 2, 3],
        all(positive::<i32>()),
        "all values must be positive",
    );
    println!("\nvalidate([1,2,3], all(positive()), ...): {:?}", result);

    let result: Validation<Vec<i32>, &str> = validate(
        vec![1, -2, 3],
        all(positive::<i32>()),
        "all values must be positive",
    );
    println!("validate([1,-2,3], all(positive()), ...): {:?}", result);

    println!();
}

/// Real-world example: user registration validation
fn real_world_example() {
    println!("--- Real World Example: User Registration ---\n");

    #[derive(Debug, Clone)]
    struct UserInput {
        username: String,
        email: String,
        age: i32,
        tags: Vec<String>,
    }

    #[derive(Debug)]
    struct User {
        username: String,
        email: String,
        age: i32,
        tags: Vec<String>,
    }

    // Validation functions return Validation<(), Vec<String>> for error accumulation
    fn validate_username(username: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if !len_between(3, 20).check(username) {
            errors.push("Username must be 3-20 characters".to_string());
        }
        if !all_chars(|c: char| c.is_alphanumeric() || c == '_').check(username) {
            errors.push("Username must contain only letters, numbers, and underscores".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_email(email: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if !not_empty().check(email) {
            errors.push("Email is required".to_string());
        } else if !contains("@").check(email) {
            errors.push("Email must contain @".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_age(age: i32) -> Validation<(), Vec<String>> {
        if between(0, 150).check(&age) {
            Validation::success(())
        } else {
            Validation::failure(vec![format!("Age {} is out of valid range (0-150)", age)])
        }
    }

    fn validate_tags(tags: &[String]) -> Validation<(), Vec<String>> {
        if has_max_len(5).check(tags) {
            Validation::success(())
        } else {
            Validation::failure(vec!["Maximum 5 tags allowed".to_string()])
        }
    }

    fn validate_user(input: UserInput) -> Validation<User, Vec<String>> {
        let v1 = validate_username(&input.username);
        let v2 = validate_email(&input.email);
        let v3 = validate_age(input.age);
        let v4 = validate_tags(&input.tags);

        // Combine all validations - errors are accumulated
        Validation::<((), (), (), ()), Vec<String>>::all((v1, v2, v3, v4)).map(|_| User {
            username: input.username,
            email: input.email,
            age: input.age,
            tags: input.tags,
        })
    }

    // Valid user
    let valid_input = UserInput {
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        age: 25,
        tags: vec!["rust".to_string(), "programming".to_string()],
    };
    println!("Valid input:");
    println!("  {:?}", valid_input);
    match validate_user(valid_input) {
        Validation::Success(user) => println!("  Result: Success({:?})", user),
        Validation::Failure(errors) => println!("  Result: Failure({:?})", errors),
    }

    // Invalid user - multiple errors
    let invalid_input = UserInput {
        username: "ab".to_string(),      // too short
        email: "invalid".to_string(),    // no @
        age: 200,                        // out of range
        tags: vec!["a".to_string(); 10], // too many
    };
    println!("\nInvalid input:");
    println!("  {:?}", invalid_input);
    match validate_user(invalid_input) {
        Validation::Success(user) => println!("  Result: Success({:?})", user),
        Validation::Failure(errors) => {
            println!("  Result: Failure with {} errors:", errors.len());
            for error in errors {
                println!("    - {}", error);
            }
        }
    }
}
