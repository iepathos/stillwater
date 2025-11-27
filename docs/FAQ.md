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

Yes! Use `Effect::from_fn()` for sync operations. They'll be wrapped in ready futures.

### How do I test Effects?

Create simple mock environments (just data structures). Pure functions in your Effects need no mocking. See [testing_patterns example](../examples/testing_patterns.rs).

### Does Effect have performance overhead?

Minimal. Each combinator (`.map()`, `.and_then()`, etc.) allocates one small Box. For a chain of 10 combinators, that's 10 allocations. This is negligible for I/O-bound work where API calls take milliseconds to seconds. Effect compiles to efficient code similar to hand-written async functions.

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

Minimal. Effect allocates one Box per combinator in the chain. Validation is just an enum. Both compile to efficient code. For I/O-bound applications (API calls, database queries), allocation overhead is negligible compared to actual work.

### Can I use Stillwater in hot loops?

For pure computations, yes. For I/O effects, consider whether the abstraction cost is worth the testability benefit. Benchmark if performance is critical.

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

Specify type parameters explicitly:

```rust
// Instead of:
let effect = Effect::pure(42);

// Do:
let effect = Effect::<_, String, ()>::pure(42);
// Or:
let effect: Effect<i32, String> = Effect::pure(42);
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

Check your function signatures. `IO::read` expects functions returning `Result<T, E>`, not bare values:

```rust
// Wrong:
IO::read(|db: &Db| db.fetch_user(id))  // If fetch_user returns User directly

// Right:
IO::read(|db: &Db| Ok(db.fetch_user(id)))
// Or if fetch_user returns Result:
IO::read(|db: &Db| db.fetch_user(id))
```

## Getting More Help

- Read the [User Guide](guide/README.md)
- Check [PATTERNS.md](PATTERNS.md) for recipes
- See [examples/](../examples/) for working code
- Open an issue on [GitHub](https://github.com/iepathos/stillwater/issues)
