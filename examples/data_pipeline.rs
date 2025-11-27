//! Data Pipeline Example
//!
//! Demonstrates ETL (Extract, Transform, Load) patterns using Effect and Validation.
//! Shows how to build robust data pipelines that combine:
//! - Pure validation with error accumulation
//! - Effectful I/O operations
//! - Data transformation
//! - Error handling and context
//!
//! Pattern: Extract -> Validate -> Transform -> Load

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stillwater::{from_fn, ContextError, Effect, EffectContext, Validation};

// ==================== Domain Types ====================

#[derive(Debug, Clone)]
struct RawRecord {
    id: String,
    name: String,
    age: String,
    email: String,
}

#[derive(Debug, Clone)]
struct ValidatedRecord {
    id: u64,
    name: String,
    age: u32,
    email: String,
}

#[derive(Debug, Clone)]
struct TransformedRecord {
    id: u64,
    name: String,
    age_group: String,
    email_domain: String,
}

#[derive(Debug)]
struct PipelineStats {
    total: usize,
    valid: usize,
    invalid: usize,
    loaded: usize,
}

// ==================== Extract ====================

/// Mock data source
#[derive(Clone)]
struct DataSource {
    records: Vec<RawRecord>,
}

impl DataSource {
    fn new() -> Self {
        Self {
            records: vec![
                RawRecord {
                    id: "1".to_string(),
                    name: "Alice Smith".to_string(),
                    age: "28".to_string(),
                    email: "alice@example.com".to_string(),
                },
                RawRecord {
                    id: "2".to_string(),
                    name: "Bob Jones".to_string(),
                    age: "35".to_string(),
                    email: "bob@company.org".to_string(),
                },
                RawRecord {
                    id: "invalid".to_string(), // Invalid ID
                    name: "Charlie".to_string(),
                    age: "42".to_string(),
                    email: "charlie@test.com".to_string(),
                },
                RawRecord {
                    id: "4".to_string(),
                    name: "Diana Lee".to_string(),
                    age: "not-a-number".to_string(), // Invalid age
                    email: "diana@mail.com".to_string(),
                },
                RawRecord {
                    id: "5".to_string(),
                    name: "Eve Brown".to_string(),
                    age: "19".to_string(),
                    email: "invalid-email".to_string(), // Invalid email
                },
                RawRecord {
                    id: "6".to_string(),
                    name: "Frank Wilson".to_string(),
                    age: "67".to_string(),
                    email: "frank@service.net".to_string(),
                },
            ],
        }
    }

    fn fetch_all(&self) -> Vec<RawRecord> {
        self.records.clone()
    }
}

// ==================== Validate ====================

fn validate_id(id: &str) -> Validation<u64, Vec<String>> {
    match id.parse::<u64>() {
        Ok(n) => Validation::success(n),
        Err(_) => Validation::failure(vec![format!("Invalid ID: '{}'", id)]),
    }
}

fn validate_name(name: &str) -> Validation<String, Vec<String>> {
    if name.is_empty() {
        Validation::failure(vec!["Name cannot be empty".to_string()])
    } else if name.len() < 2 {
        Validation::failure(vec!["Name too short".to_string()])
    } else {
        Validation::success(name.to_string())
    }
}

fn validate_age(age: &str) -> Validation<u32, Vec<String>> {
    match age.parse::<u32>() {
        Ok(n) if (18..=120).contains(&n) => Validation::success(n),
        Ok(n) => Validation::failure(vec![format!("Age {} out of range (18-120)", n)]),
        Err(_) => Validation::failure(vec![format!("Invalid age: '{}'", age)]),
    }
}

fn validate_email(email: &str) -> Validation<String, Vec<String>> {
    if email.contains('@') && email.contains('.') {
        Validation::success(email.to_string())
    } else {
        Validation::failure(vec![format!("Invalid email: '{}'", email)])
    }
}

fn validate_record(raw: RawRecord) -> Validation<ValidatedRecord, Vec<String>> {
    Validation::<(u64, String, u32, String), Vec<String>>::all((
        validate_id(&raw.id),
        validate_name(&raw.name),
        validate_age(&raw.age),
        validate_email(&raw.email),
    ))
    .map(|(id, name, age, email)| ValidatedRecord {
        id,
        name,
        age,
        email,
    })
}

// ==================== Transform ====================

fn classify_age_group(age: u32) -> String {
    match age {
        18..=25 => "Young Adult".to_string(),
        26..=40 => "Adult".to_string(),
        41..=60 => "Middle Age".to_string(),
        _ => "Senior".to_string(),
    }
}

fn extract_email_domain(email: &str) -> String {
    email.split('@').nth(1).unwrap_or("unknown").to_string()
}

fn transform_record(validated: ValidatedRecord) -> TransformedRecord {
    TransformedRecord {
        id: validated.id,
        name: validated.name.clone(),
        age_group: classify_age_group(validated.age),
        email_domain: extract_email_domain(&validated.email),
    }
}

// ==================== Load ====================

#[derive(Clone)]
struct DataWarehouse {
    records: Arc<Mutex<HashMap<u64, TransformedRecord>>>,
}

impl DataWarehouse {
    fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn insert(&self, record: TransformedRecord) {
        self.records.lock().unwrap().insert(record.id, record);
    }

    fn count(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    fn get_all(&self) -> Vec<TransformedRecord> {
        self.records.lock().unwrap().values().cloned().collect()
    }
}

// ==================== Environment ====================

#[derive(Clone)]
struct Env {
    source: DataSource,
    warehouse: DataWarehouse,
}

impl Env {
    fn new() -> Self {
        Self {
            source: DataSource::new(),
            warehouse: DataWarehouse::new(),
        }
    }
}

impl AsRef<DataSource> for Env {
    fn as_ref(&self) -> &DataSource {
        &self.source
    }
}

impl AsRef<DataWarehouse> for Env {
    fn as_ref(&self) -> &DataWarehouse {
        &self.warehouse
    }
}

// ==================== Pipeline ====================

/// Extract raw records from source
fn extract_records<Env: AsRef<DataSource> + Clone + Send + Sync + 'static>(
) -> impl Effect<Output = Vec<RawRecord>, Error = ContextError<String>, Env = Env> {
    from_fn(|env: &Env| {
        let source: &DataSource = env.as_ref();
        Ok(source.fetch_all())
    })
    .context("extracting records from source".to_string())
}

/// Load transformed record to warehouse
fn load_record<Env: AsRef<DataWarehouse> + Clone + Send + Sync + 'static>(
    record: TransformedRecord,
) -> impl Effect<Output = (), Error = ContextError<String>, Env = Env> {
    from_fn(move |env: &Env| {
        let warehouse: &DataWarehouse = env.as_ref();
        warehouse.insert(record);
        Ok(())
    })
    .context("loading record to warehouse".to_string())
}

/// Run the complete ETL pipeline
async fn run_pipeline<
    Env: AsRef<DataSource> + AsRef<DataWarehouse> + Clone + Send + Sync + 'static,
>(
    env: &Env,
) -> Result<PipelineStats, ContextError<String>> {
    println!("Starting ETL pipeline...\n");

    // Extract
    println!("=== EXTRACT ===");
    let records = extract_records().run(env).await?;
    println!("Extracted {} records\n", records.len());

    // Validate, Transform, Load
    println!("=== VALIDATE, TRANSFORM, LOAD ===");
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for (idx, raw) in records.iter().enumerate() {
        println!("Record {} (ID: {})", idx + 1, raw.id);

        // Validate
        let validation = validate_record(raw.clone());

        match validation {
            Validation::Success(validated) => {
                println!("  ✓ Validation passed");

                // Transform
                let transformed = transform_record(validated);
                println!(
                    "  → Transformed: {} ({}, {})",
                    transformed.name, transformed.age_group, transformed.email_domain
                );

                // Load
                load_record(transformed).run(env).await?;
                println!("  ✓ Loaded to warehouse");

                valid_count += 1;
            }
            Validation::Failure(errors) => {
                println!("  ✗ Validation failed ({} errors):", errors.len());
                for error in &errors {
                    println!("      - {}", error);
                }
                invalid_count += 1;
            }
        }
        println!();
    }

    let warehouse: &DataWarehouse = env.as_ref();
    let loaded = warehouse.count();

    Ok(PipelineStats {
        total: records.len(),
        valid: valid_count,
        invalid: invalid_count,
        loaded,
    })
}

// ==================== Analysis ====================

async fn analyze_results<Env: AsRef<DataWarehouse> + Sync + 'static>(env: &Env) {
    println!("=== ANALYSIS ===");

    let warehouse: &DataWarehouse = env.as_ref();
    let records = warehouse.get_all();

    // Group by age group
    let mut age_groups: HashMap<String, usize> = HashMap::new();
    for record in &records {
        *age_groups.entry(record.age_group.clone()).or_insert(0) += 1;
    }

    println!("Age group distribution:");
    for (group, count) in &age_groups {
        println!("  {}: {}", group, count);
    }

    // Group by email domain
    let mut domains: HashMap<String, usize> = HashMap::new();
    for record in &records {
        *domains.entry(record.email_domain.clone()).or_insert(0) += 1;
    }

    println!("\nEmail domain distribution:");
    for (domain, count) in &domains {
        println!("  {}: {}", domain, count);
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("Data Pipeline Example");
    println!("=====================\n");

    let env = Env::new();

    // Run pipeline
    match run_pipeline(&env).await {
        Ok(stats) => {
            println!("=== PIPELINE COMPLETE ===");
            println!("Total records: {}", stats.total);
            println!("Valid records: {}", stats.valid);
            println!("Invalid records: {}", stats.invalid);
            println!("Loaded to warehouse: {}", stats.loaded);
            println!();

            // Analyze results
            analyze_results(&env).await;
        }
        Err(e) => {
            println!("Pipeline failed: {}", e);
        }
    }

    println!("\n=== All examples completed successfully! ===");
}
