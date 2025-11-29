//! Tests for recovery combinators.

#[cfg(test)]
mod tests {
    use crate::effect::prelude::*;
    use crate::predicate::PredicateExt;

    #[derive(Debug, Clone, PartialEq)]
    enum TestError {
        Recoverable(String),
        Fatal(String),
    }

    impl TestError {
        fn is_recoverable(&self) -> bool {
            matches!(self, TestError::Recoverable(_))
        }
    }

    // AC2: pure(5).recover(...) returns Ok(5) (no recovery needed)
    #[tokio::test]
    async fn test_recover_on_success() {
        let effect = pure::<_, TestError, ()>(5).recover(
            |_: &TestError| panic!("predicate should not be called"),
            |_: TestError| pure::<i32, TestError, ()>(0),
        );
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    // AC3: fail("err").recover(|_| true, |_| pure(42)) returns Ok(42)
    #[tokio::test]
    async fn test_recover_on_matching_error() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover(|e: &TestError| e.is_recoverable(), |_| pure(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    // AC4: fail("err").recover(|_| false, ...) returns Err("err")
    #[tokio::test]
    async fn test_recover_on_non_matching_error() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .recover(|e: &TestError| e.is_recoverable(), |_| pure(42));
        assert_eq!(
            effect.execute(&()).await,
            Err(TestError::Fatal("boom".into()))
        );
    }

    // AC5: Predicate only called on error, not on success
    #[tokio::test]
    async fn test_recover_not_called_on_success() {
        let effect = pure::<_, TestError, ()>(5).recover(
            |_: &TestError| panic!("predicate should not be called"),
            |_: TestError| pure::<i32, TestError, ()>(0),
        );
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    // AC6-AC8: recover_with tests
    #[tokio::test]
    async fn test_recover_with_result() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover_with(|e: &TestError| e.is_recoverable(), |_| Ok(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_with_transforms_error() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into())).recover_with(
            |e: &TestError| e.is_recoverable(),
            |_| Err(TestError::Fatal("transformed".into())),
        );
        assert_eq!(
            effect.execute(&()).await,
            Err(TestError::Fatal("transformed".into()))
        );
    }

    // AC9-AC11: recover_some tests
    #[tokio::test]
    async fn test_recover_some_matches() {
        let effect =
            fail::<i32, _, ()>(TestError::Recoverable("oops".into())).recover_some(|e| match e {
                TestError::Recoverable(_) => Some(pure(42)),
                _ => None,
            });
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_some_no_match() {
        let effect =
            fail::<i32, _, ()>(TestError::Fatal("boom".into())).recover_some(|e| match e {
                TestError::Recoverable(_) => Some(pure(42)),
                _ => None,
            });
        assert_eq!(
            effect.execute(&()).await,
            Err(TestError::Fatal("boom".into()))
        );
    }

    // AC12-AC14: fallback tests
    #[tokio::test]
    async fn test_fallback() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into())).fallback(0);
        assert_eq!(effect.execute(&()).await, Ok(0));
    }

    #[tokio::test]
    async fn test_fallback_not_used_on_success() {
        let effect = pure::<_, TestError, ()>(5).fallback(0);
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    // AC15-AC16: fallback_to tests
    #[tokio::test]
    async fn test_fallback_to() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into())).fallback_to(pure(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_fallback_to_not_used_on_success() {
        let effect = pure::<_, TestError, ()>(5)
            .fallback_to(fail(TestError::Fatal("should not run".into())));
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    // AC17-AC19: Chaining tests
    #[tokio::test]
    async fn test_chained_recover() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover(
                |e: &TestError| matches!(e, TestError::Fatal(_)),
                |_| pure(0),
            )
            .recover(
                |e: &TestError| matches!(e, TestError::Recoverable(_)),
                |_| pure(42),
            );
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_first_matching_recover_handles_error() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover(
                |e: &TestError| matches!(e, TestError::Recoverable(_)),
                |_| pure(1),
            )
            .recover(|_: &TestError| true, |_| pure(2));
        assert_eq!(effect.execute(&()).await, Ok(1));
    }

    #[tokio::test]
    async fn test_unmatched_errors_propagate() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .recover(
                |e: &TestError| matches!(e, TestError::Recoverable(_)),
                |_| pure(1),
            )
            .recover(
                |e: &TestError| matches!(e, TestError::Recoverable(_)),
                |_| pure(2),
            );
        assert_eq!(
            effect.execute(&()).await,
            Err(TestError::Fatal("boom".into()))
        );
    }

    // AC21: Works with other combinators
    #[tokio::test]
    async fn test_recover_with_other_combinators() {
        let effect = pure::<_, TestError, ()>(5)
            .map(|x| x * 2)
            .and_then(|x| {
                if x > 5 {
                    fail(TestError::Recoverable("too big".into())).boxed()
                } else {
                    pure(x).boxed()
                }
            })
            .recover(|e: &TestError| e.is_recoverable(), |_| pure(5))
            .map(|x| x + 1);
        assert_eq!(effect.execute(&()).await, Ok(6));
    }

    // Predicate composition tests
    #[tokio::test]
    async fn test_recover_with_predicate_composition() {
        // Define reusable predicates
        let is_recoverable = |e: &TestError| matches!(e, TestError::Recoverable(_));
        let is_timeout =
            |e: &TestError| matches!(e, TestError::Recoverable(s) if s.contains("timeout"));

        // Compose predicates
        let should_retry = is_recoverable.and(is_timeout);

        let effect = fail::<i32, _, ()>(TestError::Recoverable("timeout".into()))
            .recover(should_retry, |_| pure(42));

        assert_eq!(effect.execute(&()).await, Ok(42));

        // Non-timeout recoverable errors don't match
        let effect = fail::<i32, _, ()>(TestError::Recoverable("other".into()))
            .recover(should_retry, |_| pure(42));

        assert!(effect.execute(&()).await.is_err());
    }

    #[tokio::test]
    async fn test_recover_with_or_predicate() {
        let is_recoverable = |e: &TestError| matches!(e, TestError::Recoverable(_));
        let is_specific_fatal =
            |e: &TestError| matches!(e, TestError::Fatal(s) if s.contains("retryable"));

        // Either recoverable OR specific fatal errors
        let can_retry = is_recoverable.or(is_specific_fatal);

        // Recoverable error recovers
        let effect = fail::<i32, _, ()>(TestError::Recoverable("err".into()))
            .recover(can_retry, |_| pure(1));
        assert_eq!(effect.execute(&()).await, Ok(1));

        // Specific fatal error also recovers
        let effect = fail::<i32, _, ()>(TestError::Fatal("retryable".into()))
            .recover(can_retry, |_| pure(2));
        assert_eq!(effect.execute(&()).await, Ok(2));

        // Other fatal errors don't recover
        let effect = fail::<i32, _, ()>(TestError::Fatal("permanent".into()))
            .recover(can_retry, |_| pure(3));
        assert!(effect.execute(&()).await.is_err());
    }
}
