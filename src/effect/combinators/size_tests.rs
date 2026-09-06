//! Size smoke tests for representative recovery combinators.
//!
//! These checks catch some accidental size growth. They do not establish inline
//! field storage, absence of heap allocation, or runtime performance.

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
    fn recover_has_bounded_size() {
        type InnerEffect = Pure<i32, String, ()>;
        type Predicate = fn(&String) -> bool;
        type Handler = fn(String) -> Pure<i32, String, ()>;
        type RecoverEffect = Recover<InnerEffect, Predicate, Handler, Pure<i32, String, ()>>;

        // A coarse budget for this instantiation, not an allocation assertion.
        assert!(size_of::<RecoverEffect>() < 100);
        assert!(size_of::<RecoverEffect>() >= size_of::<InnerEffect>()); // At least as big as inner
    }

    #[test]
    fn recover_with_has_bounded_size() {
        type InnerEffect = Pure<i32, String, ()>;
        type RecoverWithEffect =
            RecoverWith<InnerEffect, fn(&String) -> bool, fn(String) -> Result<i32, String>>;

        assert!(size_of::<RecoverWithEffect>() < 100);
        assert!(size_of::<RecoverWithEffect>() >= size_of::<InnerEffect>());
    }

    #[test]
    fn recover_some_has_bounded_size() {
        type InnerEffect = Pure<i32, String, ()>;
        type RecoverSomeEffect = RecoverSome<
            InnerEffect,
            fn(String) -> Option<Pure<i32, String, ()>>,
            Pure<i32, String, ()>,
        >;

        assert!(size_of::<RecoverSomeEffect>() < 100);
        assert!(size_of::<RecoverSomeEffect>() >= size_of::<InnerEffect>());
    }

    #[test]
    fn fallback_has_expected_size_for_i32() {
        type InnerEffect = Pure<i32, String, ()>;
        type FallbackEffect = Fallback<InnerEffect>;

        // Fallback should be sum of inner + default value
        let expected = size_of::<InnerEffect>() + size_of::<i32>();

        assert_eq!(size_of::<FallbackEffect>(), expected);
    }

    #[test]
    fn fallback_to_has_expected_size_for_i32() {
        type PrimaryEffect = Pure<i32, String, ()>;
        type AlternativeEffect = Pure<i32, String, ()>;
        type FallbackToEffect = FallbackTo<PrimaryEffect, AlternativeEffect>;

        // FallbackTo should be sum of primary + alternative
        let expected = size_of::<PrimaryEffect>() + size_of::<AlternativeEffect>();

        assert_eq!(size_of::<FallbackToEffect>(), expected);
    }
}
