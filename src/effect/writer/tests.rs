//! Tests for WriterEffect.

use std::convert::Infallible;

use crate::effect::prelude::*;
use crate::effect::writer::prelude::*;
use crate::monoid::Sum;

// ============================================================================
// Core Functionality Tests
// ============================================================================

#[tokio::test]
async fn test_tell_emits_value() {
    let effect = tell::<_, String, ()>(vec!["hello".to_string()]);
    let (result, writes) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(()));
    assert_eq!(writes, vec!["hello".to_string()]);
}

#[tokio::test]
async fn test_tell_one_convenience() {
    let effect = tell_one::<_, String, ()>("hello".to_string());
    let (result, writes) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(()));
    assert_eq!(writes, vec!["hello".to_string()]);
}

#[tokio::test]
async fn test_tap_tell_logs_result() {
    let effect = into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(42))
        .tap_tell(|n| vec![format!("Result: {}", n)]);

    let (result, writes) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(42));
    assert_eq!(writes, vec!["Result: 42".to_string()]);
}

#[tokio::test]
async fn test_writes_accumulate_across_and_then() {
    let effect = tell_one::<_, String, ()>("step 1".to_string())
        .and_then(|_| tell_one("step 2".to_string()))
        .and_then(|_| tell_one("step 3".to_string()));

    let (result, writes) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(()));
    assert_eq!(
        writes,
        vec![
            "step 1".to_string(),
            "step 2".to_string(),
            "step 3".to_string(),
        ]
    );
}

#[tokio::test]
async fn test_censor_transforms_writes() {
    let effect = tell_one::<_, String, ()>("debug: something".to_string())
        .and_then(|_| tell_one("info: important".to_string()))
        .censor(|logs| {
            logs.into_iter()
                .filter(|l| !l.starts_with("debug"))
                .collect()
        });

    let (_, writes) = effect.run_writer(&()).await;

    assert_eq!(writes, vec!["info: important".to_string()]);
}

#[tokio::test]
async fn test_listen_includes_writes_in_output() {
    let effect = tell_one::<_, String, ()>("logged".to_string())
        .map(|_| 42)
        .listen();

    let (result, writes) = effect.run_writer(&()).await;

    assert_eq!(result, Ok((42, vec!["logged".to_string()])));
    assert_eq!(writes, vec!["logged".to_string()]);
}

#[tokio::test]
async fn test_pass_transforms_writes_from_output() {
    let effect = tell::<_, String, ()>(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        .map(|_| (42, |logs: Vec<String>| logs.into_iter().take(2).collect()))
        .pass();

    let (result, logs) = effect.run_writer(&()).await;
    assert_eq!(result, Ok(42));
    assert_eq!(logs, vec!["a".to_string(), "b".to_string()]);
}

#[tokio::test]
async fn test_error_preserves_writes_up_to_failure() {
    let effect = tell_one::<_, String, ()>("before error".to_string())
        .and_then(|_| into_writer::<_, _, Vec<String>>(fail::<(), String, ()>("boom".into())))
        .and_then(|_| tell_one("after error".to_string()));

    let (result, writes) = effect.run_writer(&()).await;

    assert!(result.is_err());
    assert_eq!(writes, vec!["before error".to_string()]);
}

#[tokio::test]
async fn test_with_sum_monoid() {
    let effect = tell::<_, String, ()>(Sum(1))
        .and_then(|_| tell(Sum(2)))
        .and_then(|_| tell(Sum(3)));

    let (_, writes) = effect.run_writer(&()).await;

    assert_eq!(writes, Sum(6));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[derive(Clone)]
struct TestEnv {
    multiplier: i32,
}

#[tokio::test]
async fn test_writer_with_environment() {
    fn compute(
        x: i32,
    ) -> impl WriterEffect<Output = i32, Writes = Vec<String>, Error = String, Env = TestEnv> {
        into_writer::<_, _, Vec<String>>(asks::<_, String, TestEnv, _>(|env| env.multiplier))
            .tap_tell(|m| vec![format!("Multiplier: {}", m)])
            .map(move |m| x * m)
            .tap_tell(|result| vec![format!("Result: {}", result)])
    }

    let env = TestEnv { multiplier: 3 };
    let (result, logs) = compute(7).run_writer(&env).await;

    assert_eq!(result, Ok(21));
    assert_eq!(
        logs,
        vec!["Multiplier: 3".to_string(), "Result: 21".to_string(),]
    );
}

#[tokio::test]
async fn test_mixed_writer_and_regular_effects() {
    // Verify regular effects lift cleanly into writer context
    let effect = into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(10))
        .and_then(|n| tell_one(format!("Got: {}", n)).map(move |_| n * 2))
        .tap_tell(|result| vec![format!("Final: {}", result)]);

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(20));
    assert_eq!(logs, vec!["Got: 10".to_string(), "Final: 20".to_string()]);
}

#[tokio::test]
async fn test_traverse_writer_accumulates_all() {
    let items = vec![1, 2, 3];
    let effect = traverse_writer(items, |n| {
        tell_one::<_, String, ()>(format!("Processing {}", n)).map(move |_| n * 10)
    });

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(vec![10, 20, 30]));
    assert_eq!(
        logs,
        vec![
            "Processing 1".to_string(),
            "Processing 2".to_string(),
            "Processing 3".to_string(),
        ]
    );
}

#[tokio::test]
async fn test_fold_writer_accumulates_all() {
    let items = vec![1, 2, 3, 4];
    let effect = fold_writer(items, 0, |acc, n| {
        tell_one::<_, String, ()>(format!("Adding {} to {}", n, acc)).map(move |_| acc + n)
    });

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(10));
    assert_eq!(
        logs,
        vec![
            "Adding 1 to 0".to_string(),
            "Adding 2 to 1".to_string(),
            "Adding 3 to 3".to_string(),
            "Adding 4 to 6".to_string(),
        ]
    );
}

#[tokio::test]
async fn test_zip_combines_writes_left_to_right() {
    let left = tell_one::<_, String, ()>("left".to_string()).map(|_| 1);
    let right = tell_one::<_, String, ()>("right".to_string()).map(|_| 2);

    let (result, logs) = left.zip(right).run_writer(&()).await;

    assert_eq!(result, Ok((1, 2)));
    assert_eq!(logs, vec!["left".to_string(), "right".to_string()]);
}

#[tokio::test]
async fn test_boxed_writer_in_collection() {
    let effects: Vec<BoxedWriterEffect<i32, Infallible, (), Vec<String>>> = vec![
        tell_one::<_, Infallible, ()>("a".to_string())
            .map(|_| 1)
            .boxed_writer(),
        tell_one::<_, Infallible, ()>("b".to_string())
            .map(|_| 2)
            .boxed_writer(),
        tell_one::<_, Infallible, ()>("c".to_string())
            .map(|_| 3)
            .boxed_writer(),
    ];

    let mut results = Vec::new();
    let mut all_logs = Vec::new();

    for effect in effects {
        let (result, logs) = effect.run_writer(&()).await;
        results.push(result.unwrap());
        all_logs.extend(logs);
    }

    assert_eq!(results, vec![1, 2, 3]);
    assert_eq!(
        all_logs,
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

// ============================================================================
// Custom Environment Tests (Debtmap-style)
// ============================================================================

/// Trait representing an analysis environment
trait AnalysisEnv {
    fn config(&self) -> &Config;
}

#[derive(Clone)]
struct Config {
    threshold: u32,
}

#[derive(Clone)]
struct RealEnv {
    config: Config,
}

impl AnalysisEnv for RealEnv {
    fn config(&self) -> &Config {
        &self.config
    }
}

/// Helper to query config
fn asks_config<U, Env, F>(f: F) -> impl Effect<Output = U, Error = String, Env = Env>
where
    Env: AnalysisEnv + Clone + Send + Sync,
    F: Fn(&Config) -> U + Send + Sync + 'static,
    U: Send + 'static,
{
    asks(move |env: &Env| f(env.config()))
}

#[derive(Debug, Clone, PartialEq)]
enum AuditEvent {
    Started,
    ThresholdUsed(u32),
    Completed(i32),
}

#[tokio::test]
async fn test_writer_with_custom_env() {
    fn analyze<Env>(
        value: i32,
    ) -> impl WriterEffect<Output = i32, Error = String, Env = Env, Writes = Vec<AuditEvent>>
    where
        Env: AnalysisEnv + Clone + Send + Sync + 'static,
    {
        tell_one::<_, String, Env>(AuditEvent::Started).and_then(move |_| {
            into_writer::<_, _, Vec<AuditEvent>>(asks_config::<u32, Env, _>(|cfg| cfg.threshold))
                .tap_tell(|t| vec![AuditEvent::ThresholdUsed(*t)])
                .map(move |t| if value > t as i32 { value } else { t as i32 })
                .tap_tell(|result| vec![AuditEvent::Completed(*result)])
        })
    }

    let env = RealEnv {
        config: Config { threshold: 10 },
    };

    let (result, events) = analyze::<RealEnv>(15).run_writer(&env).await;

    assert_eq!(result, Ok(15));
    assert_eq!(
        events,
        vec![
            AuditEvent::Started,
            AuditEvent::ThresholdUsed(10),
            AuditEvent::Completed(15),
        ]
    );
}

// ============================================================================
// or_else Tests
// ============================================================================

#[tokio::test]
async fn test_or_else_preserves_writes_on_recovery() {
    let effect = tell_one::<_, String, ()>("before error".to_string())
        .and_then(|_| into_writer::<_, _, Vec<String>>(fail::<(), String, ()>("boom".into())))
        .or_else(|_| tell_one::<_, String, ()>("recovered".to_string()));

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(()));
    assert_eq!(
        logs,
        vec!["before error".to_string(), "recovered".to_string()]
    );
}

#[tokio::test]
async fn test_or_else_no_recovery_on_success() {
    // Use String error type and a proper recovery path
    let effect = tell_one::<_, String, ()>("success".to_string())
        .map(|_| 42)
        .or_else(|_| tell_one::<_, String, ()>("not used".to_string()).map(|_| 0));

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(42));
    assert_eq!(logs, vec!["success".to_string()]);
}

// ============================================================================
// Map Tests
// ============================================================================

#[tokio::test]
async fn test_map_preserves_writes() {
    let effect = tell_one::<_, String, ()>("logged".to_string())
        .map(|_| 42)
        .map(|n| n * 2);

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(84));
    assert_eq!(logs, vec!["logged".to_string()]);
}

#[tokio::test]
async fn test_map_err_preserves_writes() {
    let effect = tell_one::<_, Infallible, ()>("logged".to_string())
        .map(|_| 42)
        .map_err(|e: Infallible| -> String { match e {} });

    let (result, logs): (Result<i32, String>, _) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(42));
    assert_eq!(logs, vec!["logged".to_string()]);
}

// ============================================================================
// run_ignore_writes Tests
// ============================================================================

#[tokio::test]
async fn test_run_ignore_writes() {
    let effect = tell_one::<_, String, ()>("logged".to_string())
        .and_then(|_| tell_one("more logs".to_string()))
        .map(|_| 42);

    let result = effect.run_ignore_writes(&()).await;

    assert_eq!(result, Ok(42));
    // Logs are discarded
}

// ============================================================================
// Empty Writes Tests
// ============================================================================

#[tokio::test]
async fn test_into_writer_has_empty_writes() {
    let effect = into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(42));

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(42));
    assert!(logs.is_empty());
}

// ============================================================================
// Chaining Complexity Tests
// ============================================================================

#[tokio::test]
async fn test_complex_chain() {
    let effect = tell_one::<_, String, ()>("start".to_string())
        .and_then(|_| into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(10)))
        .tap_tell(|n| vec![format!("got {}", n)])
        .map(|n| n * 2)
        .tap_tell(|n| vec![format!("doubled to {}", n)])
        .and_then(|n| tell_one(format!("final: {}", n)).map(move |_| n))
        .censor(|logs| logs.into_iter().filter(|l| !l.starts_with("got")).collect());

    let (result, logs) = effect.run_writer(&()).await;

    assert_eq!(result, Ok(20));
    assert_eq!(
        logs,
        vec![
            "start".to_string(),
            "doubled to 20".to_string(),
            "final: 20".to_string(),
        ]
    );
}
