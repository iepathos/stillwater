//! Recover Patterns Example
//!
//! Demonstrates error recovery patterns for Effect-based computations.
//! Shows practical patterns including:
//! - Basic recover with predicates
//! - recover_with for Result-returning handlers
//! - recover_some for pattern matching errors
//! - fallback for default values
//! - fallback_to for alternative effects
//! - Chaining multiple recovers
//! - Composing predicates for sophisticated error handling
//! - Real-world scenarios (cache fallback, degraded mode, etc.)

use stillwater::effect::prelude::*;

// ==================== Basic Recover ====================

/// Example 1: Basic error recovery with predicates
///
/// Demonstrates recover() to handle specific errors while letting others propagate.
async fn example_basic_recover() {
    println!("\n=== Example 1: Basic Recover ===");

    #[derive(Debug, Clone, PartialEq)]
    enum CacheError {
        Miss,
    }

    // Simulate cache lookup that might fail
    fn fetch_from_cache(key: &str) -> impl Effect<Output = String, Error = CacheError, Env = ()> {
        let key = key.to_string();
        from_fn(move |_: &()| {
            println!("  Trying cache for key: {}", key);
            Err(CacheError::Miss) // Cache miss
        })
    }

    // Fallback to database
    fn fetch_from_db(key: &str) -> impl Effect<Output = String, Error = CacheError, Env = ()> {
        let key = key.to_string();
        from_fn(move |_: &()| {
            println!("  Fetching from database for key: {}", key);
            Ok(format!("value for {}", key))
        })
    }

    // Recover from cache misses by fetching from database
    let effect = fetch_from_cache("user:123").recover(
        |e: &CacheError| matches!(e, CacheError::Miss),
        |_| fetch_from_db("user:123"),
    );

    match effect.execute(&()).await {
        Ok(value) => println!("Success: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Recover With ====================

/// Example 2: Using recover_with for simple Result handlers
///
/// Demonstrates recover_with() when recovery doesn't need the environment.
async fn example_recover_with() {
    println!("\n=== Example 2: Recover With ===");

    #[derive(Debug, Clone, PartialEq)]
    enum ConfigError {
        MissingField(String),
    }

    fn parse_config() -> impl Effect<Output = String, Error = ConfigError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Parsing config...");
            Err(ConfigError::MissingField("timeout".to_string()))
        })
    }

    // Recover with default config for missing fields
    let effect = parse_config().recover_with(
        |e: &ConfigError| matches!(e, ConfigError::MissingField(_)),
        |e| {
            println!("  Using default for missing field: {:?}", e);
            Ok("default config".to_string())
        },
    );

    match effect.execute(&()).await {
        Ok(config) => println!("Config loaded: {}", config),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Recover Some ====================

/// Example 3: Pattern matching errors with recover_some
///
/// Demonstrates recover_some() for selective recovery based on error variants.
async fn example_recover_some() {
    println!("\n=== Example 3: Recover Some ===");

    #[derive(Debug, Clone, PartialEq)]
    enum ApiError {
        RateLimited { retry_after: u32 },
    }

    fn call_api() -> impl Effect<Output = String, Error = ApiError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Calling API...");
            Err(ApiError::RateLimited { retry_after: 5 })
        })
    }

    fn use_cached_response() -> impl Effect<Output = String, Error = ApiError, Env = ()> {
        pure("cached response".to_string())
    }

    // Pattern match on errors and decide recovery strategy
    let effect = call_api().recover_some(|e| match e {
        ApiError::RateLimited { retry_after } => {
            println!("  Rate limited, retry after {} seconds", retry_after);
            println!("  Using cached response instead");
            Some(use_cached_response().boxed())
        }
    });

    match effect.execute(&()).await {
        Ok(response) => println!("Response: {}", response),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Fallback ====================

/// Example 4: Simple fallback values
///
/// Demonstrates fallback() for providing default values on any error.
async fn example_fallback() {
    println!("\n=== Example 4: Fallback ===");

    #[derive(Debug, Clone)]
    enum CountError {
        NotInitialized,
    }

    fn get_count() -> impl Effect<Output = i32, Error = CountError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Getting count...");
            Err(CountError::NotInitialized)
        })
    }

    // Use 0 as default on any error
    let effect = get_count().fallback(0);

    match effect.execute(&()).await {
        Ok(count) => println!("Count: {}", count),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Fallback To ====================

/// Example 5: Fallback to alternative effect
///
/// Demonstrates fallback_to() for trying alternative effects.
async fn example_fallback_to() {
    println!("\n=== Example 5: Fallback To ===");

    #[derive(Debug, Clone)]
    enum ServiceError {
        Unavailable,
    }

    fn call_primary_service() -> impl Effect<Output = String, Error = ServiceError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Calling primary service...");
            Err(ServiceError::Unavailable)
        })
    }

    fn call_backup_service() -> impl Effect<Output = String, Error = ServiceError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Calling backup service...");
            Ok("response from backup".to_string())
        })
    }

    // Try backup service on any error from primary
    let effect = call_primary_service().fallback_to(call_backup_service());

    match effect.execute(&()).await {
        Ok(response) => println!("Response: {}", response),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Chaining Recovers ====================

/// Example 6: Chaining multiple recovery strategies
///
/// Demonstrates how to chain multiple recover calls for layered error handling.
async fn example_chained_recover() {
    println!("\n=== Example 6: Chained Recover ===");

    #[derive(Debug, Clone, PartialEq)]
    enum DataError {
        CacheMiss,
        DbConnectionFailed,
    }

    fn fetch_from_cache() -> impl Effect<Output = String, Error = DataError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Trying cache...");
            Err(DataError::CacheMiss)
        })
    }

    fn fetch_from_db() -> impl Effect<Output = String, Error = DataError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Trying database...");
            Err(DataError::DbConnectionFailed)
        })
    }

    fn fetch_from_api() -> impl Effect<Output = String, Error = DataError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Trying API...");
            Ok("data from API".to_string())
        })
    }

    // Chain multiple fallbacks: cache -> db -> api
    let effect = fetch_from_cache()
        .recover(
            |e: &DataError| matches!(e, DataError::CacheMiss),
            |_| fetch_from_db(),
        )
        .recover(
            |e: &DataError| matches!(e, DataError::DbConnectionFailed),
            |_| fetch_from_api(),
        );

    match effect.execute(&()).await {
        Ok(data) => println!("Data retrieved: {}", data),
        Err(e) => println!("All sources failed: {:?}", e),
    }
}

// ==================== Predicate Composition ====================

/// Example 7: Composing predicates for sophisticated error handling
///
/// Demonstrates using predicate combinators (.and(), .or()) for complex conditions.
async fn example_predicate_composition() {
    println!("\n=== Example 7: Predicate Composition ===");

    #[derive(Debug, Clone, PartialEq)]
    enum NetworkError {
        Timeout { duration: u32 },
    }

    fn make_request() -> impl Effect<Output = String, Error = NetworkError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Making request...");
            Err(NetworkError::Timeout { duration: 5000 })
        })
    }

    fn retry_request() -> impl Effect<Output = String, Error = NetworkError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Retrying request...");
            Ok("success on retry".to_string())
        })
    }

    // Define reusable predicate
    let is_timeout = |e: &NetworkError| matches!(e, NetworkError::Timeout { .. });

    // Use predicate for retry decision
    let is_retryable = is_timeout;

    let effect = make_request().recover(is_retryable, |_| retry_request());

    match effect.execute(&()).await {
        Ok(response) => println!("Response: {}", response),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Real-World: Cache Fallback ====================

/// Example 8: Multi-tier cache with fallback
///
/// Demonstrates a realistic caching scenario with multiple tiers.
async fn example_cache_fallback() {
    println!("\n=== Example 8: Multi-Tier Cache Fallback ===");

    #[derive(Debug, Clone, PartialEq)]
    enum CacheError {
        L1Miss,
        L2Miss,
    }

    fn fetch_l1_cache(key: &str) -> impl Effect<Output = String, Error = CacheError, Env = ()> {
        let key = key.to_string();
        from_fn(move |_: &()| {
            println!("  L1 cache lookup for: {}", key);
            Err(CacheError::L1Miss)
        })
    }

    fn fetch_l2_cache(key: &str) -> impl Effect<Output = String, Error = CacheError, Env = ()> {
        let key = key.to_string();
        from_fn(move |_: &()| {
            println!("  L2 cache lookup for: {}", key);
            Err(CacheError::L2Miss)
        })
    }

    fn fetch_store(key: &str) -> impl Effect<Output = String, Error = CacheError, Env = ()> {
        let key = key.to_string();
        from_fn(move |_: &()| {
            println!("  Store lookup for: {}", key);
            Ok(format!("value for {}", key))
        })
    }

    let key = "product:456";

    // L1 -> L2 -> Store cascade
    let effect = fetch_l1_cache(key)
        .recover(
            |e: &CacheError| matches!(e, CacheError::L1Miss),
            |_| fetch_l2_cache(key),
        )
        .recover(
            |e: &CacheError| matches!(e, CacheError::L2Miss),
            |_| fetch_store(key),
        );

    match effect.execute(&()).await {
        Ok(value) => println!("Retrieved: {}", value),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Real-World: Degraded Mode ====================

/// Example 9: Graceful degradation on service failures
///
/// Demonstrates providing reduced functionality when services fail.
async fn example_degraded_mode() {
    println!("\n=== Example 9: Degraded Mode ===");

    #[derive(Debug, Clone)]
    struct UserProfile {
        name: String,
        recommendations: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ProfileError {
        ServiceUnavailable,
    }

    fn fetch_full_profile() -> impl Effect<Output = UserProfile, Error = ProfileError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Fetching full profile with recommendations...");
            Err(ProfileError::ServiceUnavailable)
        })
    }

    fn fetch_basic_profile() -> impl Effect<Output = UserProfile, Error = ProfileError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Fetching basic profile (degraded mode)...");
            Ok(UserProfile {
                name: "User".to_string(),
                recommendations: vec![],
            })
        })
    }

    // Try full profile, fall back to basic profile on service errors
    let is_service_error = |e: &ProfileError| matches!(e, ProfileError::ServiceUnavailable);

    let effect = fetch_full_profile().recover(is_service_error, |_| fetch_basic_profile());

    match effect.execute(&()).await {
        Ok(profile) => {
            println!("Profile loaded: {}", profile.name);
            if profile.recommendations.is_empty() {
                println!("  (Running in degraded mode - no recommendations)");
            } else {
                println!("  Recommendations: {:?}", profile.recommendations);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Real-World: Multiple Endpoints ====================

/// Example 10: Try multiple API endpoints
///
/// Demonstrates attempting different endpoints with different recovery strategies.
async fn example_multiple_endpoints() {
    println!("\n=== Example 10: Multiple Endpoints ===");

    #[derive(Debug, Clone, PartialEq)]
    enum ApiError {
        EndpointDeprecated,
    }

    fn call_v2_endpoint() -> impl Effect<Output = String, Error = ApiError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Calling /api/v2/data...");
            Err(ApiError::EndpointDeprecated)
        })
    }

    fn call_v1_endpoint() -> impl Effect<Output = String, Error = ApiError, Env = ()> {
        from_fn(|_: &()| {
            println!("  Calling /api/v1/data (legacy)...");
            Ok("data from v1".to_string())
        })
    }

    // Only fall back to v1 if v2 is deprecated, not for other errors
    let effect = call_v2_endpoint().recover(
        |e: &ApiError| matches!(e, ApiError::EndpointDeprecated),
        |_| call_v1_endpoint(),
    );

    match effect.execute(&()).await {
        Ok(data) => println!("Data: {}", data),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Combining with Other Combinators ====================

/// Example 11: Recover combined with map and and_then
///
/// Demonstrates recover in a larger effect pipeline.
async fn example_combined_pipeline() {
    println!("\n=== Example 11: Combined Pipeline ===");

    #[derive(Debug, Clone, PartialEq)]
    enum Error {
        Parse,
        Validation,
        NotFound,
    }

    fn fetch_data() -> impl Effect<Output = String, Error = Error, Env = ()> {
        from_fn(|_: &()| {
            println!("  Fetching data...");
            Err(Error::NotFound)
        })
    }

    fn fetch_default() -> impl Effect<Output = String, Error = Error, Env = ()> {
        pure("42".to_string())
    }

    fn parse(s: String) -> impl Effect<Output = i32, Error = Error, Env = ()> {
        from_fn(move |_: &()| {
            println!("  Parsing: {}", s);
            s.parse::<i32>().map_err(|_| Error::Parse)
        })
    }

    fn validate(n: i32) -> impl Effect<Output = i32, Error = Error, Env = ()> {
        from_fn(move |_: &()| {
            println!("  Validating: {}", n);
            if n > 0 && n < 100 {
                Ok(n)
            } else {
                Err(Error::Validation)
            }
        })
    }

    // Full pipeline: fetch -> recover -> parse -> validate -> transform
    let effect = fetch_data()
        .recover(
            |e: &Error| matches!(e, Error::NotFound),
            |_| fetch_default(),
        )
        .and_then(parse)
        .and_then(validate)
        .map(|n| n * 2);

    match effect.execute(&()).await {
        Ok(result) => println!("Final result: {}", result),
        Err(e) => println!("Error: {:?}", e),
    }
}

// ==================== Main ====================

#[tokio::main]
async fn main() {
    println!("======================================");
    println!("      Recover Patterns Example        ");
    println!("======================================");

    example_basic_recover().await;
    example_recover_with().await;
    example_recover_some().await;
    example_fallback().await;
    example_fallback_to().await;
    example_chained_recover().await;
    example_predicate_composition().await;
    example_cache_fallback().await;
    example_degraded_mode().await;
    example_multiple_endpoints().await;
    example_combined_pipeline().await;

    println!("\n======================================");
    println!("           Examples Complete           ");
    println!("======================================");
}
