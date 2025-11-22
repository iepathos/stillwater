//! Form validation example - testing ergonomics of error accumulation
//!
//! This example shows how we want validation to feel for a typical web form.

use stillwater::{Validation, Semigroup};

// Domain types
#[derive(Debug, Clone, PartialEq)]
struct Email(String);

#[derive(Debug, Clone, PartialEq)]
struct Password(String);

#[derive(Debug, Clone, PartialEq)]
struct Age(u8);

#[derive(Debug, PartialEq)]
struct User {
    email: Email,
    password: Password,
    age: Age,
}

// Raw input from form
struct SignupForm {
    email: String,
    password: String,
    password_confirm: String,
    age: String,
}

// Validation errors
#[derive(Debug, Clone, PartialEq)]
enum ValidationError {
    InvalidEmail { value: String, reason: String },
    PasswordTooShort { min_length: usize },
    PasswordMismatch,
    InvalidAge { value: String },
    AgeTooYoung { age: u8, minimum: u8 },
}

// Vec<ValidationError> is our error accumulator
impl Semigroup for Vec<ValidationError> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// Pure validation functions
fn validate_email(email: &str) -> Validation<Email, Vec<ValidationError>> {
    if email.is_empty() {
        return Validation::failure(vec![ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "Email cannot be empty".to_string(),
        }]);
    }

    if !email.contains('@') || !email.contains('.') {
        return Validation::failure(vec![ValidationError::InvalidEmail {
            value: email.to_string(),
            reason: "Must contain @ and .".to_string(),
        }]);
    }

    Validation::success(Email(email.to_string()))
}

fn validate_password(pwd: &str) -> Validation<Password, Vec<ValidationError>> {
    if pwd.len() < 8 {
        Validation::failure(vec![ValidationError::PasswordTooShort { min_length: 8 }])
    } else {
        Validation::success(Password(pwd.to_string()))
    }
}

fn validate_passwords_match(
    pwd: &str,
    confirm: &str,
) -> Validation<(), Vec<ValidationError>> {
    if pwd == confirm {
        Validation::success(())
    } else {
        Validation::failure(vec![ValidationError::PasswordMismatch])
    }
}

fn validate_age_input(age_str: &str) -> Validation<Age, Vec<ValidationError>> {
    // First, try to parse
    let age = match age_str.parse::<u8>() {
        Ok(a) => a,
        Err(_) => {
            return Validation::failure(vec![ValidationError::InvalidAge {
                value: age_str.to_string(),
            }])
        }
    };

    // Then validate range
    if age < 18 {
        Validation::failure(vec![ValidationError::AgeTooYoung {
            age,
            minimum: 18,
        }])
    } else {
        Validation::success(Age(age))
    }
}

// Composing validations - this is the key ergonomics test!
fn validate_signup_form(form: SignupForm) -> Validation<User, Vec<ValidationError>> {
    // Question: Does this feel natural?
    // We're validating multiple independent fields

    // Option 1: Using Validation::all with tuple
    let field_validations = (
        validate_email(&form.email),
        validate_password(&form.password),
        validate_age_input(&form.age),
    );

    // Combine field validations
    Validation::all(field_validations)
        .and_then(|(email, password, age)| {
            // Now check password match (depends on password being valid)
            validate_passwords_match(&form.password, &form.password_confirm)
                .map(|_| User { email, password, age })
        })
}

// Alternative composition style - which feels better?
fn validate_signup_form_alt1(form: SignupForm) -> Validation<User, Vec<ValidationError>> {
    // Option 2: Builder style?
    validate_email(&form.email)
        .and(validate_password(&form.password))
        .and(validate_age_input(&form.age))
        .and(validate_passwords_match(&form.password, &form.password_confirm))
        .map(|(email, password, age, _)| User { email, password, age })
}

fn validate_signup_form_alt2(form: SignupForm) -> Validation<User, Vec<ValidationError>> {
    // Option 3: Explicit accumulation?
    let email = validate_email(&form.email);
    let password = validate_password(&form.password);
    let age = validate_age_input(&form.age);
    let match_check = validate_passwords_match(&form.password, &form.password_confirm);

    // Combine all four
    Validation::all((email, password, age, match_check))
        .map(|(e, p, a, _)| User { email: e, password: p, age: a })
}

// Usage example
fn main() {
    println!("=== Form Validation Ergonomics Test ===\n");

    // Test 1: All valid
    println!("Test 1: Valid form");
    let valid_form = SignupForm {
        email: "user@example.com".to_string(),
        password: "secure123".to_string(),
        password_confirm: "secure123".to_string(),
        age: "25".to_string(),
    };

    match validate_signup_form(valid_form) {
        Validation::Success(user) => println!("✓ User created: {:?}", user),
        Validation::Failure(errors) => println!("✗ Errors: {:?}", errors),
    }

    println!("\n---\n");

    // Test 2: Multiple errors - this is the important one!
    println!("Test 2: Multiple validation errors");
    let invalid_form = SignupForm {
        email: "not-an-email".to_string(),
        password: "weak".to_string(),
        password_confirm: "different".to_string(),
        age: "15".to_string(),
    };

    match validate_signup_form(invalid_form) {
        Validation::Success(_) => println!("✓ Unexpected success"),
        Validation::Failure(errors) => {
            println!("✗ Found {} errors:", errors.len());
            for (i, err) in errors.iter().enumerate() {
                println!("  {}. {:?}", i + 1, err);
            }
        }
    }

    println!("\n---\n");

    // Test 3: Partial errors
    println!("Test 3: Some fields valid, some invalid");
    let partial_form = SignupForm {
        email: "valid@example.com".to_string(),
        password: "short".to_string(),
        password_confirm: "short".to_string(),
        age: "25".to_string(),
    };

    match validate_signup_form(partial_form) {
        Validation::Success(_) => println!("✓ Unexpected success"),
        Validation::Failure(errors) => {
            println!("✗ Found {} errors:", errors.len());
            for err in errors {
                println!("  - {:?}", err);
            }
        }
    }

    // Questions to answer:
    // 1. Does Validation::all() feel natural?
    // 2. Is the tuple syntax too verbose?
    // 3. Should we have a macro for common cases?
    // 4. How does error reporting feel?
    // 5. Is and_then for dependent validation clear?
}

/* Expected output:

=== Form Validation Ergonomics Test ===

Test 1: Valid form
✓ User created: User { email: Email("user@example.com"), password: Password("secure123"), age: Age(25) }

---

Test 2: Multiple validation errors
✗ Found 4 errors:
  1. InvalidEmail { value: "not-an-email", reason: "Must contain @ and ." }
  2. PasswordTooShort { min_length: 8 }
  3. PasswordMismatch
  4. AgeTooYoung { age: 15, minimum: 18 }

---

Test 3: Some fields valid, some invalid
✗ Found 1 errors:
  - PasswordTooShort { min_length: 8 }

*/
