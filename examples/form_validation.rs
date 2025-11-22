//! Form Validation Example
//!
//! Demonstrates practical form validation with error accumulation.
//! Shows how Validation excels at collecting all validation errors
//! instead of failing on the first one.
//!
//! Includes:
//! - Contact form validation
//! - User profile validation
//! - Payment form validation
//! - Multi-field cross-validation

use stillwater::Validation;

// ==================== Contact Form ====================

/// Example 1: Simple contact form validation
///
/// Demonstrates validating a contact form and collecting all errors.
fn example_contact_form() {
    println!("\n=== Example 1: Contact Form ===");

    #[derive(Debug)]
    struct ContactForm {
        name: String,
        email: String,
        message: String,
    }

    fn validate_name(name: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if name.is_empty() {
            errors.push("Name is required".to_string());
        } else if name.len() < 2 {
            errors.push("Name must be at least 2 characters".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_email(email: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if email.is_empty() {
            errors.push("Email is required".to_string());
        } else if !email.contains('@') {
            errors.push("Email must contain @".to_string());
        } else if !email.contains('.') {
            errors.push("Email must contain domain".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_message(message: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if message.is_empty() {
            errors.push("Message is required".to_string());
        } else if message.len() < 10 {
            errors.push("Message must be at least 10 characters".to_string());
        } else if message.len() > 1000 {
            errors.push("Message must not exceed 1000 characters".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_contact_form(form: ContactForm) -> Validation<ContactForm, Vec<String>> {
        let v1 = validate_name(&form.name);
        let v2 = validate_email(&form.email);
        let v3 = validate_message(&form.message);
        Validation::<((), (), ()), Vec<String>>::all((v1, v2, v3)).map(|_| form)
    }

    // Valid form
    let valid_form = ContactForm {
        name: "Alice Smith".to_string(),
        email: "alice@example.com".to_string(),
        message: "Hello, I would like more information about your services.".to_string(),
    };

    match validate_contact_form(valid_form) {
        Validation::Success(_) => println!("✓ Valid contact form"),
        Validation::Failure(errors) => {
            println!("✗ Invalid contact form:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid form - collects all errors
    let invalid_form = ContactForm {
        name: "A".to_string(),
        email: "invalid-email".to_string(),
        message: "Hi".to_string(),
    };

    println!();
    match validate_contact_form(invalid_form) {
        Validation::Success(_) => println!("✓ Valid contact form"),
        Validation::Failure(errors) => {
            println!("✗ Invalid contact form ({} errors):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== User Profile ====================

/// Example 2: User profile validation
///
/// Demonstrates validating a user profile with multiple fields.
fn example_user_profile() {
    println!("\n=== Example 2: User Profile ===");

    #[derive(Debug)]
    struct UserProfile {
        username: String,
        age: u32,
        bio: String,
        website: String,
    }

    fn validate_username(username: &str) -> Validation<(), Vec<String>> {
        if username.len() >= 3
            && username.len() <= 20
            && username.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            Validation::success(())
        } else {
            Validation::failure(vec![
                "Username must be 3-20 characters and contain only letters, numbers, and underscores".to_string()
            ])
        }
    }

    fn validate_age(age: u32) -> Validation<(), Vec<String>> {
        if age >= 13 && age <= 120 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Age must be between 13 and 120 (got {})",
                age
            )])
        }
    }

    fn validate_bio(bio: &str) -> Validation<(), Vec<String>> {
        if bio.len() <= 500 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Bio must not exceed 500 characters (got {})",
                bio.len()
            )])
        }
    }

    fn validate_website(website: &str) -> Validation<(), Vec<String>> {
        if website.is_empty() || website.starts_with("http://") || website.starts_with("https://") {
            Validation::success(())
        } else {
            Validation::failure(vec![
                "Website must start with http:// or https://".to_string()
            ])
        }
    }

    fn validate_user_profile(profile: UserProfile) -> Validation<UserProfile, Vec<String>> {
        let v1 = validate_username(&profile.username);
        let v2 = validate_age(profile.age);
        let v3 = validate_bio(&profile.bio);
        let v4 = validate_website(&profile.website);
        Validation::<((), (), (), ()), Vec<String>>::all((v1, v2, v3, v4)).map(|_| profile)
    }

    // Valid profile
    let valid = UserProfile {
        username: "alice_123".to_string(),
        age: 25,
        bio: "Software engineer interested in functional programming.".to_string(),
        website: "https://alice.dev".to_string(),
    };

    match validate_user_profile(valid) {
        Validation::Success(profile) => println!("✓ Valid profile: {}", profile.username),
        Validation::Failure(errors) => {
            println!("✗ Invalid profile:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid profile - multiple errors
    let invalid = UserProfile {
        username: "ab".to_string(),
        age: 10,
        bio: "x".repeat(600),
        website: "alice.dev".to_string(),
    };

    println!();
    match validate_user_profile(invalid) {
        Validation::Success(_) => println!("✓ Valid profile"),
        Validation::Failure(errors) => {
            println!("✗ Invalid profile ({} errors):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Payment Form ====================

/// Example 3: Payment form validation
///
/// Demonstrates validating payment information with complex rules.
fn example_payment_form() {
    println!("\n=== Example 3: Payment Form ===");

    #[derive(Debug)]
    struct PaymentForm {
        card_number: String,
        expiry_month: u32,
        expiry_year: u32,
        cvv: String,
        amount: f64,
    }

    fn validate_card_number(card: &str) -> Validation<(), Vec<String>> {
        let digits_only: String = card.chars().filter(|c| c.is_numeric()).collect();

        if digits_only.len() == 16 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Card number must be 16 digits (got {})",
                digits_only.len()
            )])
        }
    }

    fn validate_expiry(month: u32, year: u32) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if month < 1 || month > 12 {
            errors.push(format!("Invalid month: {}", month));
        }

        if year < 2025 || year > 2035 {
            errors.push(format!("Invalid year: {}", year));
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_cvv(cvv: &str) -> Validation<(), Vec<String>> {
        if cvv.len() == 3 && cvv.chars().all(|c| c.is_numeric()) {
            Validation::success(())
        } else {
            Validation::failure(vec!["CVV must be 3 digits".to_string()])
        }
    }

    fn validate_amount(amount: f64) -> Validation<(), Vec<String>> {
        if amount > 0.0 && amount <= 10000.0 {
            Validation::success(())
        } else {
            Validation::failure(vec![format!(
                "Amount must be between $0.01 and $10,000 (got ${:.2})",
                amount
            )])
        }
    }

    fn validate_payment_form(form: PaymentForm) -> Validation<PaymentForm, Vec<String>> {
        let v1 = validate_card_number(&form.card_number);
        let v2 = validate_expiry(form.expiry_month, form.expiry_year);
        let v3 = validate_cvv(&form.cvv);
        let v4 = validate_amount(form.amount);
        Validation::<((), (), (), ()), Vec<String>>::all((v1, v2, v3, v4)).map(|_| form)
    }

    // Valid payment
    let valid = PaymentForm {
        card_number: "4532-1234-5678-9010".to_string(),
        expiry_month: 12,
        expiry_year: 2027,
        cvv: "123".to_string(),
        amount: 99.99,
    };

    match validate_payment_form(valid) {
        Validation::Success(_) => println!("✓ Valid payment form"),
        Validation::Failure(errors) => {
            println!("✗ Invalid payment:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid payment - multiple errors
    let invalid = PaymentForm {
        card_number: "1234".to_string(),
        expiry_month: 13,
        expiry_year: 2040,
        cvv: "12".to_string(),
        amount: -50.0,
    };

    println!();
    match validate_payment_form(invalid) {
        Validation::Success(_) => println!("✓ Valid payment form"),
        Validation::Failure(errors) => {
            println!("✗ Invalid payment ({} errors):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Cross-Field Validation ====================

/// Example 4: Cross-field validation
///
/// Demonstrates validating fields that depend on each other.
fn example_cross_field_validation() {
    println!("\n=== Example 4: Cross-Field Validation ===");

    #[derive(Debug)]
    struct RegistrationForm {
        password: String,
        confirm_password: String,
        email: String,
        alternate_email: String,
    }

    fn validate_password_strength(password: &str) -> Validation<(), Vec<String>> {
        let mut errors = Vec::new();

        if password.len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain an uppercase letter".to_string());
        }
        if !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain a lowercase letter".to_string());
        }
        if !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain a number".to_string());
        }

        if errors.is_empty() {
            Validation::success(())
        } else {
            Validation::failure(errors)
        }
    }

    fn validate_passwords_match(password: &str, confirm: &str) -> Validation<(), Vec<String>> {
        if password == confirm {
            Validation::success(())
        } else {
            Validation::failure(vec!["Passwords do not match".to_string()])
        }
    }

    fn validate_emails_different(email: &str, alternate: &str) -> Validation<(), Vec<String>> {
        if email.is_empty() || alternate.is_empty() || email != alternate {
            Validation::success(())
        } else {
            Validation::failure(vec![
                "Alternate email must be different from primary email".to_string()
            ])
        }
    }

    fn validate_registration(form: RegistrationForm) -> Validation<RegistrationForm, Vec<String>> {
        let v1 = validate_password_strength(&form.password);
        let v2 = validate_passwords_match(&form.password, &form.confirm_password);
        let v3 = validate_emails_different(&form.email, &form.alternate_email);
        Validation::<((), (), ()), Vec<String>>::all((v1, v2, v3)).map(|_| form)
    }

    // Valid registration
    let valid = RegistrationForm {
        password: "Secret123".to_string(),
        confirm_password: "Secret123".to_string(),
        email: "user@example.com".to_string(),
        alternate_email: "user.backup@example.com".to_string(),
    };

    match validate_registration(valid) {
        Validation::Success(_) => println!("✓ Valid registration"),
        Validation::Failure(errors) => {
            println!("✗ Invalid registration:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid - password issues and mismatch
    let invalid1 = RegistrationForm {
        password: "weak".to_string(),
        confirm_password: "different".to_string(),
        email: "user@example.com".to_string(),
        alternate_email: "backup@example.com".to_string(),
    };

    println!();
    match validate_registration(invalid1) {
        Validation::Success(_) => println!("✓ Valid registration"),
        Validation::Failure(errors) => {
            println!("✗ Invalid registration ({} errors):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Invalid - duplicate emails
    let invalid2 = RegistrationForm {
        password: "Secret123".to_string(),
        confirm_password: "Secret123".to_string(),
        email: "user@example.com".to_string(),
        alternate_email: "user@example.com".to_string(),
    };

    println!();
    match validate_registration(invalid2) {
        Validation::Success(_) => println!("✓ Valid registration"),
        Validation::Failure(errors) => {
            println!("✗ Invalid registration ({} errors):", errors.len());
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
}

// ==================== Main ====================

fn main() {
    println!("Form Validation Examples");
    println!("========================");

    example_contact_form();
    example_user_profile();
    example_payment_form();
    example_cross_field_validation();

    println!("\n=== All examples completed successfully! ===");
}
