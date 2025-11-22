//! Data pipeline example - testing composition of validation + effects + pure functions
//!
//! This simulates an ETL pipeline: Extract -> Transform -> Load
//! Tests how well stillwater handles real-world data processing.

use stillwater::{Effect, Validation, Semigroup, IO};
use std::path::PathBuf;

// ============================================================================
// Domain Types
// ============================================================================

#[derive(Debug, Clone)]
struct RawRecord {
    user_id: String,
    email: String,
    amount: String,
    timestamp: String,
}

#[derive(Debug, Clone)]
struct ValidRecord {
    user_id: u64,
    email: String,
    amount: f64,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct EnrichedRecord {
    user_id: u64,
    email: String,
    amount: f64,
    timestamp: u64,
    category: String,
    risk_score: f64,
}

#[derive(Debug)]
struct Report {
    total_amount: f64,
    record_count: usize,
    high_risk_count: usize,
}

// Reference data for enrichment
#[derive(Clone)]
struct ReferenceData {
    user_categories: Vec<(u64, String)>,
}

impl ReferenceData {
    fn get_category(&self, user_id: u64) -> Option<String> {
        self.user_categories
            .iter()
            .find(|(id, _)| *id == user_id)
            .map(|(_, cat)| cat.clone())
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
enum ValidationError {
    InvalidUserId { value: String, reason: String },
    InvalidEmail { value: String },
    InvalidAmount { value: String },
    InvalidTimestamp { value: String },
}

impl Semigroup for Vec<ValidationError> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

#[derive(Debug)]
enum PipelineError {
    FileRead(String),
    Parse(String),
    Validation(Vec<ValidationError>),
    Database(String),
    Enrichment(String),
}

impl From<Vec<ValidationError>> for PipelineError {
    fn from(errors: Vec<ValidationError>) -> Self {
        PipelineError::Validation(errors)
    }
}

// ============================================================================
// Pure Functions - Parsing
// ============================================================================

fn parse_csv_line(line: &str) -> Result<RawRecord, PipelineError> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() != 4 {
        return Err(PipelineError::Parse(format!("Expected 4 fields, got {}", parts.len())));
    }

    Ok(RawRecord {
        user_id: parts[0].to_string(),
        email: parts[1].to_string(),
        amount: parts[2].to_string(),
        timestamp: parts[3].to_string(),
    })
}

// ============================================================================
// Pure Functions - Validation
// ============================================================================

fn validate_user_id(id_str: &str) -> Validation<u64, Vec<ValidationError>> {
    match id_str.parse::<u64>() {
        Ok(id) if id > 0 => Validation::success(id),
        Ok(_) => Validation::failure(vec![ValidationError::InvalidUserId {
            value: id_str.to_string(),
            reason: "Must be > 0".to_string(),
        }]),
        Err(_) => Validation::failure(vec![ValidationError::InvalidUserId {
            value: id_str.to_string(),
            reason: "Not a number".to_string(),
        }]),
    }
}

fn validate_email(email: &str) -> Validation<String, Vec<ValidationError>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec![ValidationError::InvalidEmail {
            value: email.to_string(),
        }])
    }
}

fn validate_amount(amount_str: &str) -> Validation<f64, Vec<ValidationError>> {
    match amount_str.parse::<f64>() {
        Ok(amt) if amt >= 0.0 => Validation::success(amt),
        Ok(_) => Validation::failure(vec![ValidationError::InvalidAmount {
            value: amount_str.to_string(),
        }]),
        Err(_) => Validation::failure(vec![ValidationError::InvalidAmount {
            value: amount_str.to_string(),
        }]),
    }
}

fn validate_timestamp(ts_str: &str) -> Validation<u64, Vec<ValidationError>> {
    match ts_str.parse::<u64>() {
        Ok(ts) => Validation::success(ts),
        Err(_) => Validation::failure(vec![ValidationError::InvalidTimestamp {
            value: ts_str.to_string(),
        }]),
    }
}

fn validate_record(raw: RawRecord) -> Validation<ValidRecord, Vec<ValidationError>> {
    // Question: Is this the right way to compose validations?
    Validation::all((
        validate_user_id(&raw.user_id),
        validate_email(&raw.email),
        validate_amount(&raw.amount),
        validate_timestamp(&raw.timestamp),
    ))
    .map(|(user_id, email, amount, timestamp)| {
        ValidRecord {
            user_id,
            email,
            amount,
            timestamp,
        }
    })
}

// ============================================================================
// Pure Functions - Transformation
// ============================================================================

fn calculate_risk_score(amount: f64, category: &str) -> f64 {
    let base_risk = amount / 1000.0;
    let category_multiplier = match category {
        "high-risk" => 2.0,
        "medium-risk" => 1.0,
        _ => 0.5,
    };
    base_risk * category_multiplier
}

fn enrich_record(valid: ValidRecord, ref_data: &ReferenceData) -> Result<EnrichedRecord, PipelineError> {
    let category = ref_data
        .get_category(valid.user_id)
        .unwrap_or_else(|| "unknown".to_string());

    let risk_score = calculate_risk_score(valid.amount, &category);

    Ok(EnrichedRecord {
        user_id: valid.user_id,
        email: valid.email,
        amount: valid.amount,
        timestamp: valid.timestamp,
        category,
        risk_score,
    })
}

fn aggregate_records(records: Vec<EnrichedRecord>) -> Report {
    let total_amount = records.iter().map(|r| r.amount).sum();
    let record_count = records.len();
    let high_risk_count = records.iter().filter(|r| r.risk_score > 1.0).count();

    Report {
        total_amount,
        record_count,
        high_risk_count,
    }
}

// ============================================================================
// Environment
// ============================================================================

struct PipelineEnv {
    db: Database,
}

struct Database {
    reference_data: ReferenceData,
}

impl Database {
    fn load_reference_data(&self) -> Result<ReferenceData, PipelineError> {
        Ok(self.reference_data.clone())
    }

    fn save_enriched_records(&self, records: &[EnrichedRecord]) -> Result<(), PipelineError> {
        println!("  [DB] Saving {} enriched records", records.len());
        Ok(())
    }
}

// ============================================================================
// Pipeline Composition
// ============================================================================

// This is the key test: Does a real-world pipeline feel natural?

fn read_csv_file(path: PathBuf) -> Effect<String, PipelineError, ()> {
    Effect::from_fn(move |_| {
        // Simulate file read
        Ok("1,user1@example.com,100.50,1234567890\n\
            2,user2@example.com,250.00,1234567891\n\
            3,invalid-email,500.00,1234567892\n\
            4,user4@example.com,-50.00,1234567893".to_string())
    })
}

fn parse_csv(content: String) -> Effect<Vec<RawRecord>, PipelineError, ()> {
    let lines: Vec<_> = content.lines().collect();
    let results: Result<Vec<_>, _> = lines
        .iter()
        .map(|line| parse_csv_line(line))
        .collect();

    Effect::from_result(results)
}

fn validate_all_records(raw_records: Vec<RawRecord>) -> Effect<Vec<ValidRecord>, PipelineError, ()> {
    // Question: Should we:
    // A) Fail entire batch if any record invalid?
    // B) Collect all validation errors?
    // C) Filter out invalid records and continue?

    // Approach A: Fail on any error (strict)
    let validations: Vec<_> = raw_records
        .into_iter()
        .map(validate_record)
        .collect();

    // Combine all validations
    let combined = Validation::all(validations);

    Effect::from_validation(combined)
}

// Alternative: Filter out invalid records
fn validate_records_permissive(raw_records: Vec<RawRecord>) -> Effect<Vec<ValidRecord>, PipelineError, ()> {
    let mut valid = Vec::new();
    let mut errors = Vec::new();

    for raw in raw_records {
        match validate_record(raw) {
            Validation::Success(v) => valid.push(v),
            Validation::Failure(e) => errors.extend(e),
        }
    }

    if !errors.is_empty() {
        println!("  [Warning] {} validation errors, continuing with {} valid records",
                 errors.len(), valid.len());
    }

    Effect::pure(valid)
}

fn enrich_records(valid_records: Vec<ValidRecord>) -> Effect<Vec<EnrichedRecord>, PipelineError, PipelineEnv> {
    // Load reference data
    IO::query(|db: &Database| {
        db.load_reference_data()
    })
    .and_then(move |ref_data| {
        // Enrich each record
        let enriched: Result<Vec<_>, _> = valid_records
            .into_iter()
            .map(|rec| enrich_record(rec, &ref_data))
            .collect();

        Effect::from_result(enriched)
    })
}

fn save_records(enriched: Vec<EnrichedRecord>) -> Effect<Vec<EnrichedRecord>, PipelineError, PipelineEnv> {
    let enriched_clone = enriched.clone();
    IO::execute(move |db: &Database| {
        db.save_enriched_records(&enriched)
    })
    .map(move |_| enriched_clone)
}

// Full pipeline
fn process_data_pipeline(
    input_path: PathBuf,
) -> Effect<Report, PipelineError, PipelineEnv> {
    read_csv_file(input_path)
        // Parse
        .and_then(parse_csv)

        // Validate (strict - fail on any error)
        .and_then(validate_all_records)

        // Enrich (needs reference data from DB)
        .and_then(enrich_records)

        // Save enriched records
        .and_then(save_records)

        // Generate report (pure)
        .map(aggregate_records)
}

// Alternative: Permissive validation
fn process_data_pipeline_permissive(
    input_path: PathBuf,
) -> Effect<Report, PipelineError, PipelineEnv> {
    read_csv_file(input_path)
        .and_then(parse_csv)
        .and_then(validate_records_permissive) // Different validation strategy
        .and_then(enrich_records)
        .and_then(save_records)
        .map(aggregate_records)
}

// ============================================================================
// Usage
// ============================================================================

fn main() {
    println!("=== Data Pipeline Composition Test ===\n");

    let env = PipelineEnv {
        db: Database {
            reference_data: ReferenceData {
                user_categories: vec![
                    (1, "low-risk".to_string()),
                    (2, "high-risk".to_string()),
                    (3, "medium-risk".to_string()),
                ],
            },
        },
    };

    // Test 1: Strict validation (fails on any invalid record)
    println!("Test 1: Strict pipeline (fail on any error)");
    let effect = process_data_pipeline(PathBuf::from("data.csv"));

    match effect.run(&env) {
        Ok(report) => {
            println!("✓ Pipeline completed:");
            println!("  Total amount: ${:.2}", report.total_amount);
            println!("  Records processed: {}", report.record_count);
            println!("  High risk records: {}", report.high_risk_count);
        }
        Err(err) => {
            println!("✗ Pipeline failed: {:?}", err);
            if let PipelineError::Validation(errors) = err {
                println!("\n  Validation errors:");
                for e in errors {
                    println!("    - {:?}", e);
                }
            }
        }
    }

    println!("\n---\n");

    // Test 2: Permissive validation (continue with valid records)
    println!("Test 2: Permissive pipeline (filter invalid, continue)");
    let effect = process_data_pipeline_permissive(PathBuf::from("data.csv"));

    match effect.run(&env) {
        Ok(report) => {
            println!("✓ Pipeline completed:");
            println!("  Total amount: ${:.2}", report.total_amount);
            println!("  Records processed: {}", report.record_count);
            println!("  High risk records: {}", report.high_risk_count);
        }
        Err(err) => {
            println!("✗ Pipeline failed: {:?}", err);
        }
    }

    // Ergonomics questions:
    // 1. Is the pipeline flow clear and readable?
    // 2. Is it easy to switch between strict/permissive validation?
    // 3. Should we have helpers for "validate and filter"?
    // 4. Is the pure/effect separation obvious?
    // 5. Would parallel processing be easy to add?
    // 6. How would we add retries or error recovery?
}

/* Expected output:

=== Data Pipeline Composition Test ===

Test 1: Strict pipeline (fail on any error)
✗ Pipeline failed: Validation([InvalidEmail { value: "invalid-email" }, InvalidAmount { value: "-50.00" }])

  Validation errors:
    - InvalidEmail { value: "invalid-email" }
    - InvalidAmount { value: "-50.00" }

---

Test 2: Permissive pipeline (filter invalid, continue)
  [Warning] 2 validation errors, continuing with 2 valid records
  [DB] Saving 2 enriched records
✓ Pipeline completed:
  Total amount: $350.50
  Records processed: 2
  High risk records: 1

*/
