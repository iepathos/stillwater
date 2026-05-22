//! Shared helpers for Stillwater performance benchmarks.

use stillwater::Validation;

/// Number of fields validated per benchmark iteration.
pub const FIELD_COUNT: usize = 8;

/// Test inputs — all valid so success-path overhead is measured.
pub fn fields() -> [&'static str; FIELD_COUNT] {
    ["valid"; FIELD_COUNT]
}

/// Stillwater validation for a single field.
pub fn validate_field(field: &str) -> Validation<i32, Vec<&'static str>> {
    if field == "valid" {
        Validation::success(field.len() as i32)
    } else {
        Validation::failure(vec!["invalid field"])
    }
}

/// Manual `Result`-based validation for a single field.
pub fn validate_field_manual(field: &str) -> Result<i32, &'static str> {
    if field == "valid" {
        Ok(field.len() as i32)
    } else {
        Err("invalid field")
    }
}

/// Accumulate validations with Stillwater.
pub fn stillwater_accumulate(fields: &[&str]) -> Validation<Vec<i32>, Vec<&'static str>> {
    let vals: Vec<_> = fields.iter().map(|&f| validate_field(f)).collect();
    Validation::all_vec(vals)
}

/// Accumulate validations manually with `Result`.
pub fn manual_accumulate(fields: &[&str]) -> Result<Vec<i32>, Vec<&'static str>> {
    let mut errors = Vec::new();
    let mut results = Vec::new();

    for &field in fields {
        match validate_field_manual(field) {
            Ok(v) => results.push(v),
            Err(e) => errors.push(e),
        }
    }

    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}
