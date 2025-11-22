#![cfg(feature = "try_trait")]
#![feature(try_trait_v2)]

use stillwater::Validation;

#[test]
fn test_validation_question_mark() {
    fn parse_and_validate(s: &str) -> Validation<i32, Vec<String>> {
        let num: i32 = s.parse().map_err(|e| vec![format!("Parse error: {}", e)])?;

        if num >= 0 {
            Validation::success(num)
        } else {
            Validation::failure(vec!["Number must be positive".to_string()])
        }
    }

    assert_eq!(parse_and_validate("42"), Validation::Success(42));
    assert!(parse_and_validate("-5").is_failure());
    assert!(parse_and_validate("abc").is_failure());
}

#[test]
fn test_mixing_result_and_validation() {
    fn validate_positive(n: i32) -> Validation<i32, Vec<String>> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Must be positive".to_string()])
        }
    }

    fn process(s: &str) -> Validation<String, Vec<String>> {
        // Result from standard library function
        let parsed: i32 = s.parse().map_err(|_| vec!["Invalid number".to_string()])?;

        // Validation from our function
        let validated = validate_positive(parsed)?;

        Validation::success(format!("Processed: {}", validated))
    }

    assert_eq!(
        process("42"),
        Validation::Success("Processed: 42".to_string())
    );
    assert!(process("-5").is_failure());
    assert!(process("abc").is_failure());
}

#[test]
fn test_real_world_form_validation_with_question_mark() {
    #[derive(Debug, PartialEq)]
    enum ValidationError {
        InvalidEmail,
        PasswordTooShort,
        AgeTooYoung,
    }

    #[derive(Debug, PartialEq)]
    struct RegistrationData {
        email: String,
        password: String,
        age: u8,
    }

    fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
        if email.contains('@') {
            Validation::success(email.to_string())
        } else {
            Validation::failure(vec![ValidationError::InvalidEmail])
        }
    }

    fn validate_password(pwd: &str) -> Validation<String, Vec<ValidationError>> {
        if pwd.len() >= 8 {
            Validation::success(pwd.to_string())
        } else {
            Validation::failure(vec![ValidationError::PasswordTooShort])
        }
    }

    fn validate_age(age: u8) -> Validation<u8, Vec<ValidationError>> {
        if age >= 18 {
            Validation::success(age)
        } else {
            Validation::failure(vec![ValidationError::AgeTooYoung])
        }
    }

    fn validate_form(
        email: &str,
        password: &str,
        age: u8,
    ) -> Validation<RegistrationData, Vec<ValidationError>> {
        let email = validate_email(email)?;
        let password = validate_password(password)?;
        let age = validate_age(age)?;

        Validation::success(RegistrationData {
            email,
            password,
            age,
        })
    }

    // Valid form
    let result = validate_form("user@example.com", "password123", 25);
    assert!(result.is_success());

    // Note: With ?, we get fail-fast behavior, not error accumulation
    // First error stops the chain
    let result = validate_form("invalid", "short", 15);
    assert_eq!(
        result,
        Validation::Failure(vec![ValidationError::InvalidEmail])
    );
    // Only sees first error (email), doesn't check password or age
}

#[test]
fn test_question_mark_short_circuits() {
    let mut call_count = 0;

    fn check_positive(n: i32) -> Validation<i32, Vec<String>> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Must be positive".to_string()])
        }
    }

    fn process_with_side_effect(a: i32, call_count: &mut i32) -> Validation<i32, Vec<String>> {
        let result_a = check_positive(a)?;
        *call_count += 1;
        Validation::success(result_a)
    }

    // Should short-circuit and not increment call_count
    let result = process_with_side_effect(-1, &mut call_count);
    assert!(result.is_failure());
    assert_eq!(call_count, 0);

    // Should succeed and increment call_count
    let result = process_with_side_effect(1, &mut call_count);
    assert!(result.is_success());
    assert_eq!(call_count, 1);
}

#[test]
fn test_chained_validations_with_question_mark() {
    fn validate_step1(n: i32) -> Validation<i32, Vec<String>> {
        if n > 0 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Step 1: must be positive".to_string()])
        }
    }

    fn validate_step2(n: i32) -> Validation<i32, Vec<String>> {
        if n < 100 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Step 2: must be less than 100".to_string()])
        }
    }

    fn validate_step3(n: i32) -> Validation<i32, Vec<String>> {
        if n % 2 == 0 {
            Validation::success(n)
        } else {
            Validation::failure(vec!["Step 3: must be even".to_string()])
        }
    }

    fn validate_all_steps(n: i32) -> Validation<i32, Vec<String>> {
        let n = validate_step1(n)?;
        let n = validate_step2(n)?;
        let n = validate_step3(n)?;
        Validation::success(n)
    }

    // Success case
    assert_eq!(validate_all_steps(42), Validation::Success(42));

    // Fails at step 1
    assert_eq!(
        validate_all_steps(-5),
        Validation::Failure(vec!["Step 1: must be positive".to_string()])
    );

    // Fails at step 2
    assert_eq!(
        validate_all_steps(150),
        Validation::Failure(vec!["Step 2: must be less than 100".to_string()])
    );

    // Fails at step 3
    assert_eq!(
        validate_all_steps(43),
        Validation::Failure(vec!["Step 3: must be even".to_string()])
    );
}
