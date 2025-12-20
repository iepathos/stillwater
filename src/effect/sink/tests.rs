//! Comprehensive tests for SinkEffect.

// Import only what we need to avoid method conflicts between EffectExt and SinkEffectExt
use crate::effect::sink::prelude::*;
use crate::effect::{asks, fail, pure};

mod emit_tests {
    use super::*;

    #[tokio::test]
    async fn emit_single_item() {
        let effect = emit::<_, String, ()>("hello".to_string());

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["hello".to_string()]);
    }

    #[tokio::test]
    async fn emit_many_items() {
        let effect = emit_many::<_, _, String, ()>(vec!["a", "b", "c"]);

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn emit_as_plain_effect_is_noop() {
        use crate::effect::Effect;
        let effect = emit::<_, String, ()>("hello".to_string());

        let result = effect.run(&()).await;

        assert_eq!(result, Ok(()));
    }
}

mod into_sink_tests {
    use super::*;

    #[tokio::test]
    async fn lift_pure_effect() {
        let effect = into_sink::<_, _, String>(pure::<_, String, ()>(42));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(42));
        assert!(collected.is_empty());
    }

    #[tokio::test]
    async fn lift_asks_effect() {
        #[derive(Clone)]
        struct Env {
            value: i32,
        }

        let effect = into_sink::<_, _, String>(asks::<_, String, Env, _>(|env: &Env| env.value));

        let env = Env { value: 42 };
        let (result, collected) = effect.run_collecting(&env).await;

        assert_eq!(result, Ok(42));
        assert!(collected.is_empty());
    }
}

mod and_then_tests {
    use super::*;

    #[tokio::test]
    async fn chain_two_effects() {
        let effect =
            emit::<_, String, ()>("first".to_string()).and_then(|_| emit("second".to_string()));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn chain_three_effects() {
        let effect = emit::<_, String, ()>("a".to_string())
            .and_then(|_| emit("b".to_string()))
            .and_then(|_| emit("c".to_string()));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn chain_passes_value() {
        let effect = into_sink::<_, _, String>(pure::<_, String, ()>(10))
            .and_then(|n| emit(format!("got: {}", n)).map(move |_| n * 2));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(20));
        assert_eq!(collected, vec!["got: 10"]);
    }
}

mod map_tests {
    use super::*;

    #[tokio::test]
    async fn map_transforms_output() {
        let effect = emit::<_, String, ()>("log".to_string())
            .map(|_| 42)
            .map(|n| n * 2);

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(84));
        assert_eq!(collected, vec!["log"]);
    }
}

mod map_err_tests {
    use super::*;

    #[tokio::test]
    async fn map_err_transforms_error() {
        let effect: SinkMapErr<_, _> = into_sink::<_, _, String>(fail::<i32, i32, ()>(42))
            .map_err(|e| format!("Error: {}", e));

        let (result, _) = effect.run_collecting(&()).await;

        assert_eq!(result, Err("Error: 42".to_string()));
    }

    #[tokio::test]
    async fn map_err_preserves_emissions() {
        let effect = emit::<String, String, ()>("before".to_string())
            .map_err(|e: String| format!("wrapped: {}", e));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["before"]);
    }
}

mod or_else_tests {
    use super::*;

    #[tokio::test]
    async fn or_else_recovers_from_error() {
        let effect = into_sink::<_, _, String>(fail::<i32, String, ()>("error".to_string()))
            .or_else(|_| emit::<String, String, ()>("recovered".to_string()).map(|_| 42));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(42));
        assert_eq!(collected, vec!["recovered"]);
    }

    #[tokio::test]
    async fn or_else_preserves_prior_emissions() {
        let effect = emit::<String, String, ()>("before error".to_string())
            .and_then(|_| into_sink::<_, (), String>(fail::<(), String, ()>("oops".to_string())))
            .or_else(|_| emit::<String, String, ()>("recovered".to_string()));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(collected, vec!["before error", "recovered"]);
    }
}

mod zip_tests {
    use super::*;

    #[tokio::test]
    async fn zip_combines_effects() {
        let left = emit::<_, String, ()>("left".to_string()).map(|_| 1);
        let right = emit::<_, String, ()>("right".to_string()).map(|_| 2);

        let (result, collected) = left.zip(right).run_collecting(&()).await;

        assert_eq!(result, Ok((1, 2)));
        assert_eq!(collected, vec!["left", "right"]);
    }
}

mod tap_emit_tests {
    use super::*;

    #[tokio::test]
    async fn tap_emit_adds_derived_value() {
        let effect = into_sink::<_, _, String>(pure::<_, String, ()>(42))
            .tap_emit(|n| format!("Result: {}", n));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(42));
        assert_eq!(collected, vec!["Result: 42"]);
    }

    #[tokio::test]
    async fn tap_emit_preserves_output() {
        let effect = emit::<_, String, ()>("start".to_string())
            .map(|_| 100)
            .tap_emit(|n| format!("value: {}", n));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(100));
        assert_eq!(collected, vec!["start", "value: 100"]);
    }
}

mod traverse_sink_tests {
    use super::*;

    #[tokio::test]
    async fn traverse_processes_all_items() {
        let items = vec![1, 2, 3];
        let effect = traverse_sink(items, |n| {
            emit::<_, String, ()>(format!("Processing {}", n)).map(move |_| n * 10)
        });

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(vec![10, 20, 30]));
        assert_eq!(
            collected,
            vec!["Processing 1", "Processing 2", "Processing 3"]
        );
    }

    #[tokio::test]
    async fn traverse_stops_on_error() {
        let items = vec![1, 2, 3];
        let effect = traverse_sink(items, |n| {
            if n == 2 {
                // Use boxed to unify types
                into_sink::<_, _, String>(fail::<i32, String, ()>("error at 2".to_string()))
                    .boxed_sink()
            } else {
                emit::<_, String, ()>(format!("Processing {}", n))
                    .map(move |_| n)
                    .boxed_sink()
            }
        });

        let (result, collected) = effect.run_collecting(&()).await;

        assert!(result.is_err());
        assert_eq!(collected, vec!["Processing 1"]);
    }
}

mod fold_sink_tests {
    use super::*;

    #[tokio::test]
    async fn fold_accumulates_values() {
        let items = vec![1, 2, 3, 4];
        let effect = fold_sink(items, 0, |acc, n| {
            emit::<_, String, ()>(format!("Adding {} to {}", n, acc)).map(move |_| acc + n)
        });

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(10));
        assert_eq!(
            collected,
            vec![
                "Adding 1 to 0",
                "Adding 2 to 1",
                "Adding 3 to 3",
                "Adding 4 to 6",
            ]
        );
    }
}

mod boxed_sink_tests {
    use super::*;

    #[tokio::test]
    async fn boxed_sink_works() {
        let effect = emit::<_, String, ()>("hello".to_string())
            .map(|_| 42)
            .boxed_sink();

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(42));
        assert_eq!(collected, vec!["hello"]);
    }

    #[tokio::test]
    async fn heterogeneous_collection() {
        let effects: Vec<BoxedSinkEffect<i32, String, (), String>> = vec![
            emit("a".to_string()).map(|_| 1).boxed_sink(),
            emit("b".to_string()).map(|_| 2).boxed_sink(),
            into_sink::<_, _, String>(pure::<_, String, ()>(3)).boxed_sink(),
        ];

        let mut results = Vec::new();
        let mut all_logs = Vec::new();

        for effect in effects {
            let (result, logs) = effect.run_collecting(&()).await;
            results.push(result.unwrap());
            all_logs.extend(logs);
        }

        assert_eq!(results, vec![1, 2, 3]);
        assert_eq!(all_logs, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn conditional_boxing() {
        fn conditional_log(flag: bool) -> BoxedSinkEffect<i32, String, (), String> {
            if flag {
                emit("enabled".to_string()).map(|_| 1).boxed_sink()
            } else {
                into_sink::<_, _, String>(pure::<_, String, ()>(0)).boxed_sink()
            }
        }

        let (result_true, logs_true) = conditional_log(true).run_collecting(&()).await;
        assert_eq!(result_true, Ok(1));
        assert_eq!(logs_true, vec!["enabled"]);

        let (result_false, logs_false) = conditional_log(false).run_collecting(&()).await;
        assert_eq!(result_false, Ok(0));
        assert!(logs_false.is_empty());
    }
}

mod run_ignore_emissions_tests {
    use super::*;

    #[tokio::test]
    async fn run_ignore_emissions_discards_output() {
        let result = emit::<_, String, ()>("hello".to_string())
            .and_then(|_| emit("world".to_string()))
            .map(|_| 42)
            .run_ignore_emissions(&())
            .await;

        assert_eq!(result, Ok(42));
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn error_preserves_prior_emissions() {
        let effect = emit::<String, String, ()>("before error".to_string())
            .and_then(|_| emit::<String, String, ()>("about to fail".to_string()))
            .and_then(|_| {
                into_sink::<_, (), String>(fail::<i32, String, ()>("something broke".to_string()))
            })
            .and_then(|n| emit::<String, String, ()>("never reached".to_string()).map(move |_| n));

        let (result, collected) = effect.run_collecting(&()).await;

        assert!(result.is_err());
        assert_eq!(collected, vec!["before error", "about to fail"]);
    }
}

mod integration_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn complex_pipeline() {
        let effect = emit::<_, String, ()>("Starting".to_string())
            .and_then(|_| into_sink(pure::<_, String, ()>(vec![1, 2, 3])))
            .and_then(|items| {
                traverse_sink(items, |n| {
                    emit(format!("Processing item {}", n)).map(move |_| n * 10)
                })
            })
            .tap_emit(|results| format!("Final results: {:?}", results));

        let (result, collected) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(vec![10, 20, 30]));
        assert_eq!(
            collected,
            vec![
                "Starting",
                "Processing item 1",
                "Processing item 2",
                "Processing item 3",
                "Final results: [10, 20, 30]",
            ]
        );
    }

    #[tokio::test]
    async fn dual_execution_patterns() {
        // Create a logging effect that works with both patterns
        fn log_operation<Env: Clone + Send + Sync>(
            name: &str,
            value: i32,
        ) -> impl SinkEffect<Output = i32, Error = String, Env = Env, Item = String> {
            let name = name.to_string();
            emit::<_, String, Env>(format!("Starting: {}", name))
                .and_then(move |_| into_sink(pure::<_, String, Env>(value * 2)))
                .tap_emit(move |v| format!("Result: {}", v))
        }

        // Testing pattern
        let effect = log_operation::<()>("multiply", 21);
        let (result, logs) = effect.run_collecting(&()).await;

        assert_eq!(result, Ok(42));
        assert_eq!(logs.len(), 2);
        assert!(logs[0].contains("Starting"));
        assert!(logs[1].contains("Result"));

        // Production pattern (stream immediately)
        let effect = log_operation::<()>("multiply", 21);
        let streamed = Arc::new(Mutex::new(Vec::new()));
        let streamed_clone = Arc::clone(&streamed);
        let result = effect
            .run_with_sink(&(), move |log| {
                let streamed = Arc::clone(&streamed_clone);
                async move {
                    streamed.lock().expect("mutex").push(log);
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(streamed.lock().expect("mutex").len(), 2);
    }
}
