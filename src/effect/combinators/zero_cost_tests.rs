//! Zero-cost validation tests for recovery combinators.

#[cfg(test)]
mod tests {
    use crate::effect::combinators::fallback::Fallback;
    use crate::effect::combinators::fallback_to::FallbackTo;
    use crate::effect::combinators::pure::Pure;
    use crate::effect::combinators::recover::Recover;
    use crate::effect::combinators::recover_some::RecoverSome;
    use crate::effect::combinators::recover_with::RecoverWith;
    use std::mem::size_of;

    #[test]
    fn test_recover_size_is_zero_cost() {
        type InnerEffect = Pure<i32, String, ()>;
        type Predicate = fn(&String) -> bool;
        type Handler = fn(String) -> Pure<i32, String, ()>;
        type RecoverEffect = Recover<InnerEffect, Predicate, Handler, Pure<i32, String, ()>>;

        // Recover should NOT allocate on heap - size is stack-only (may have alignment padding)
        // The size should be reasonable (not Box-sized which would indicate heap allocation)
        assert!(size_of::<RecoverEffect>() < 100); // Reasonable stack size
        assert!(size_of::<RecoverEffect>() >= size_of::<InnerEffect>()); // At least as big as inner
    }

    #[test]
    fn test_recover_with_size_is_zero_cost() {
        type InnerEffect = Pure<i32, String, ()>;
        type RecoverWithEffect = RecoverWith<InnerEffect, fn(&String) -> bool, fn(String) -> Result<i32, String>>;

        // RecoverWith should NOT allocate on heap - size is stack-only
        assert!(size_of::<RecoverWithEffect>() < 100); // Reasonable stack size
        assert!(size_of::<RecoverWithEffect>() >= size_of::<InnerEffect>());
    }

    #[test]
    fn test_recover_some_size_is_zero_cost() {
        type InnerEffect = Pure<i32, String, ()>;
        type RecoverSomeEffect = RecoverSome<InnerEffect, fn(String) -> Option<Pure<i32, String, ()>>, Pure<i32, String, ()>>;

        // RecoverSome should NOT allocate on heap - size is stack-only
        assert!(size_of::<RecoverSomeEffect>() < 100); // Reasonable stack size
        assert!(size_of::<RecoverSomeEffect>() >= size_of::<InnerEffect>());
    }

    #[test]
    fn test_fallback_size_is_zero_cost() {
        type InnerEffect = Pure<i32, String, ()>;
        type FallbackEffect = Fallback<InnerEffect>;

        // Fallback should be sum of inner + default value
        let expected = size_of::<InnerEffect>() + size_of::<i32>();

        assert_eq!(size_of::<FallbackEffect>(), expected);
    }

    #[test]
    fn test_fallback_to_size_is_zero_cost() {
        type PrimaryEffect = Pure<i32, String, ()>;
        type AlternativeEffect = Pure<i32, String, ()>;
        type FallbackToEffect = FallbackTo<PrimaryEffect, AlternativeEffect>;

        // FallbackTo should be sum of primary + alternative
        let expected = size_of::<PrimaryEffect>() + size_of::<AlternativeEffect>();

        assert_eq!(size_of::<FallbackToEffect>(), expected);
    }
}
