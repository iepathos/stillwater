# Frequently Asked Questions

## General

### What is Stillwater?

Stillwater is a Rust library providing pragmatic functional programming abstractions, focused on validation error accumulation and effect composition using the "pure core, imperative shell" pattern.

### Why "Stillwater"?

"Still" represents pure logic (calm, unchanging, referentially transparent). "Water" represents effects (flowing, dynamic, performing I/O). Together: "pure core, imperative shell."

### Is this a monad library?

Sort of. Validation is an Applicative Functor, Effect is a Reader/IO monad. But we focus on practical patterns, not category theory. You don't need to understand monads to use Stillwater effectively.

### What's the learning curve?

Low. If you understand Result and async/await, you can use Stillwater. The core concepts are:
- Validation accumulates errors (vs Result which short-circuits)
- Effect separates pure logic from I/O (for testability)

Advanced patterns are optional.

## Validation

### Why not just use Result?

Result short-circuits on the first error. Validation accumulates all errors, providing better UX for forms and APIs where users need to see all validation failures at once.

### Can I use the ? operator with Validation?

On nightly with `try_trait` feature, yes! But be aware: `?` fails fast (no accumulation). Use `Validation::all()` for error accumulation. See [Try Trait guide](guide/07-try-trait.md).

### What if I need more than 12 validations?

Use `Validation::all_vec()` for homogeneous collections, or nest tuples: `Validation::all((all1, all2, all3))`.

### How do I convert Validation to Result?

```rust
let result: Result<T, E> = validation.into_result();
```

### Do I need to implement Semigroup for every error type?

Only if you want to use `Validation::all()`. `Vec<T>`, `String`, and tuples already implement Semigroup. Most of the time you'll use `Vec<YourError>` which works out of the box.

## Effects

### Why Effect instead of just async fn?

Effect separates pure logic from I/O, making code more testable and composable. Pure functions need zero mocks! You can test business logic without databases, file systems, or network calls.

### Do I need tokio?

For async Effects, yes (or async-std). For sync-only code, no runtime needed.

### Can I use Effect with sync code?

Yes! Use `from_fn()` for sync operations. They'll be wrapped in ready futures.

### How do I test Effects?

Create simple mock environments (just data structures). Pure functions in your Effects need no mocking. See [testing_patterns example](../examples/testing_patterns.rs).

### Does Effect have performance overhead?

No! Stillwater follows the `futures` crate pattern: **zero-cost by default**. Each combinator returns a concrete type (like `Map`, `AndThen`) that the compiler can fully inline. No heap allocations occur for effect chains.

When you need type erasure (collections, recursion, match arms), use `.boxed()` which allocates once. For I/O-bound work, this is negligible.

### When should I use `.boxed()`?

Use `.boxed()` in exactly three cases:

1. **Collections**: Storing different effects in `Vec`, `HashMap`, etc.
2. **Recursion**: Breaking infinite type recursion
3. **Match arms**: When different branches return different effect types

```rust
// Collections
let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    pure(1).boxed(),
    pure(2).map(|x| x * 2).boxed(),
];

// Recursion
fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 { pure(0).boxed() }
    else { pure(n).and_then(move |_| countdown(n - 1)).boxed() }
}
```

### Why did the API change in 0.11.0?

Version 0.11.0 introduced a zero-cost Effect design following the `futures` crate pattern. The old API boxed every combinator; the new API uses concrete types by default.

Key changes:
- `Effect::pure(x)` → `pure(x)`
- `Effect<T, E, Env>` struct → `impl Effect<Output=T, Error=E, Env=Env>` trait
- Automatic boxing → Explicit `.boxed()` when needed

See [Migration Guide](MIGRATION.md) for detailed upgrade instructions.

### How do I migrate from 0.10.x to 0.11.0?

See the [Migration Guide](MIGRATION.md) for step-by-step instructions. Key steps:
1. Update imports to use `stillwater::prelude::*`
2. Change return types to `impl Effect<...>`
3. Replace `Effect::pure`, `Effect::fail` with `pure`, `fail`
4. Add `.boxed()` where type erasure is needed

## Error Handling

### Should I always use ContextError?

No. Use it at I/O boundaries and major operation boundaries where context helps debugging. Don't add context to pure functions or in hot loops.

### Can I mix Validation and Effect?

Yes! Common pattern:

```rust
Effect::from_validation(validate_data(data))
    .and_then(|valid| save_to_db(valid))
```

Validate first (pure, accumulates errors), then lift to Effect for I/O.

### How do I handle errors from third-party libraries?

Map them to your error types:

```rust
IO::execute(|env| async {
    env.db.query()
        .await
        .map_err(|e| MyError::Database(e.to_string()))
})
```

## Performance

### Is there overhead?

No! The Effect trait is zero-cost by default:
- Each combinator returns a concrete type (like `Map<AndThen<Pure<...>, F>, G>`)
- The compiler can fully inline the effect chain
- No heap allocations occur

Validation is just an enum with no overhead. Both compile to efficient code identical to hand-written async functions.

### When does allocation happen?

Only when you explicitly call `.boxed()`:
- Storing effects in collections
- Recursive effects
- Match arms with different effect types

For I/O-bound applications (API calls, database queries), boxing overhead is negligible compared to actual work.

### Can I use Stillwater in hot loops?

Yes! The zero-cost design means you can use Effects in performance-sensitive code. Just avoid `.boxed()` in the hot path. For tight loops, benchmark to confirm.

### Can I use no_std?

Not currently. Effect requires `std` for async/boxing. Future versions may add no_std support for Validation.

## Comparison

### vs anyhow/thiserror

Those are for error handling. Stillwater is for validation (accumulation) and effect composition (separation). Use together! Stillwater for business logic, anyhow for error propagation.

### vs frunk

frunk focuses on HLists and type-level programming. Stillwater focuses on practical validation and effects with a lower learning curve.

### vs monadic

monadic uses macros for do-notation (`rdrdo!`). Stillwater uses method chaining (more idiomatic Rust).

### vs hand-rolling

Hand-rolling works but requires boilerplate. Stillwater provides tested, composable abstractions that follow best practices.

### vs just using Result everywhere?

Result is perfect for operations that should fail fast. Use Result for that! Use Validation when you want ALL errors (forms, API validation). Use Effect when you want testable I/O separation.

## Contributing

### How can I help?

See [CONTRIBUTING.md](../CONTRIBUTING.md). We welcome:
- Bug reports
- Documentation improvements
- Examples
- Feature requests
- Performance improvements

### What's the roadmap?

See specs in [specs/](../specs/) directory for planned features. Major upcoming features:
- Parallel effect execution
- More combinators
- Additional examples
- Performance optimizations

### Is this production-ready?

Yes! Stillwater 0.1 is stable with comprehensive tests (111 unit tests, 58 doc tests). The 0.x version indicates the API may evolve based on community feedback.

## Common Issues

### "Cannot infer type for Effect"

Specify type parameters explicitly on constructor functions:

```rust
// Instead of:
let effect = pure(42);

// Do:
let effect = pure::<_, String, ()>(42);
```

### "expected struct, found opaque type"

You're returning `impl Effect` but the caller expects a concrete type. Either:
1. Use `.boxed()` to get `BoxedEffect`
2. Update the caller to accept `impl Effect`

### "recursive type has infinite size"

Use `.boxed()` for recursive effects:
```rust
fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
    if n <= 0 { pure(0).boxed() }
    else { pure(n).and_then(move |_| countdown(n - 1)).boxed() }
}
```

### "Validation::all doesn't compile"

Make sure your error type implements Semigroup:

```rust
use stillwater::Semigroup;

impl Semigroup for MyError {
    fn combine(self, other: Self) -> Self {
        // Combine errors
    }
}
```

Or use `Vec<MyError>` which already implements Semigroup.

### "Effect.run() returns nested Results"

Check your function signatures. `from_fn` expects functions returning `Result<T, E>`, not bare values:

```rust
// Wrong:
from_fn(|db: &Db| db.fetch_user(id))  // If fetch_user returns User directly

// Right:
from_fn(|db: &Db| Ok(db.fetch_user(id)))
// Or if fetch_user returns Result:
from_fn(|db: &Db| db.fetch_user(id))
```

## Getting More Help

- Read the [User Guide](guide/README.md)
- Check [PATTERNS.md](PATTERNS.md) for recipes
- See [examples/](../examples/) for working code
- Open an issue on [GitHub](https://github.com/iepathos/stillwater/issues)
