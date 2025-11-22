# Stillwater Examples - Ergonomics Testing

These examples are written **as if the API already exists** to test how it feels before we implement it.

## Examples

### 1. `form_validation.rs` - Validation Basics
**Tests:** Error accumulation across multiple fields

**Key scenarios:**
- All fields valid ✓
- Multiple validation errors (email, password, age)
- Partial errors (some valid, some invalid)

**API features tested:**
```rust
Validation::all((email, password, age))
validate_signup_form()  // Composition
Validation::Success vs Failure
```

**Questions raised:**
- Is `Validation::all()` with tuples the right approach?
- How to handle dependent validations (password confirmation)?
- Should we have a macro for large forms?

---

### 2. `user_registration.rs` - Effect Composition
**Tests:** Pure core / imperative shell pattern

**Key scenarios:**
- Successful user registration
- Validation errors at start
- Business rule violation (duplicate email)
- Non-critical failure handling (email service)

**API features tested:**
```rust
Effect::from_validation()
IO::query() / IO::execute()
.and_then() chaining
.or_else() for recovery
.run(&env)
```

**Questions raised:**
- Is Effect chaining readable?
- Is IO::query vs IO::execute clear?
- How to handle non-critical failures elegantly?
- Too much `.map_err()` boilerplate?

---

### 3. `error_context.rs` - Context Accumulation
**Tests:** Error context chaining through call stacks

**Key scenarios:**
- File not found (see full context trail)
- Parse error (context shows where in pipeline)
- Nested initialization steps

**API features tested:**
```rust
.context("description")
ContextError<E> wrapper
context_trail() for debugging
```

**Questions raised:**
- Is `.context()` easy enough to add?
- Should context be strings or structured?
- Is the error output format helpful?
- Should we auto-capture file:line?

---

### 4. `data_pipeline.rs` - Real-World ETL
**Tests:** Complex data processing pipeline

**Key scenarios:**
- Parse CSV → Validate → Enrich → Save → Report
- Strict validation (fail on any error)
- Permissive validation (filter invalid, continue)

**API features tested:**
```rust
Validation::all() over collections
Effect chaining for multi-step pipeline
Pure functions vs I/O boundaries
Different validation strategies
```

**Questions raised:**
- Is pipeline flow clear and readable?
- Should we provide both strict/permissive helpers?
- How to report filtered errors?
- Would parallel processing be easy to add?

---

### 5. `testing_patterns.rs` - Testability Benefits
**Tests:** How separation makes testing easier

**Key scenarios:**
- Pure business logic tests (NO MOCKING!)
- Effect tests with mock environments
- Complex discount calculation
- Multi-step checkout process

**API features tested:**
```rust
Pure functions standalone
Effect.run(&mock_env) for testing
Simple mock trait implementations
```

**Questions raised:**
- Is the benefit obvious?
- Are mock environments easy to create?
- Does this feel better than traditional testing?

## Usage

These examples won't compile yet (the library doesn't exist!), but you can:

1. **Read through them** to evaluate API ergonomics
2. **Identify pain points** before implementation
3. **Suggest improvements** to the API design
4. **Use as reference** when implementing

## Key Insights from Examples

### What Feels Good ✓

1. **Validation accumulation** - Getting all errors at once is powerful
2. **Effect chaining** - Reads top-to-bottom naturally
3. **Pure/effect separation** - Makes testing trivial
4. **Context chaining** - Error trails are super helpful
5. **Type safety** - Compiler guides you to correct composition

### Potential Pain Points ⚠️

1. **Nested closures** - Can get deep in complex effects
2. **Type verbosity** - `Effect<T, E, Env>` is long (type aliases help)
3. **Error conversion** - `.map_err()` appears frequently
4. **Move semantics** - Cloning to move into closures

### Design Questions ❓

See [ERGONOMICS_REVIEW.md](./ERGONOMICS_REVIEW.md) for detailed discussion of:
- Validation composition strategies
- IO module design
- Error handling patterns
- Async support
- Helper methods vs minimal API

## Next Steps

1. **Review these examples**
   - Do they feel natural?
   - What would you change?
   - Any missing use cases?

2. **Make API decisions**
   - Based on insights from examples
   - Document choices in DESIGN.md

3. **Implement MVP**
   - Start with core types
   - Make examples actually compile
   - Iterate based on real usage

4. **Gather feedback**
   - Share with Rust community
   - Get early adopters
   - Refine before 1.0

## Running Examples

Once the library is implemented:

```bash
# Run individual examples
cargo run --example form_validation
cargo run --example user_registration
cargo run --example error_context
cargo run --example data_pipeline
cargo run --example testing_patterns

# Run tests in testing_patterns
cargo test --example testing_patterns
```

## Contributing Example Scenarios

Have a use case we haven't covered? Add an example!

Missing scenarios:
- [ ] Async/await integration
- [ ] Streaming data processing
- [ ] Web framework integration (Axum/Actix)
- [ ] Database ORM integration (SQLx/Diesel)
- [ ] Concurrent/parallel validation
- [ ] Retry/timeout patterns
- [ ] State machine integration

---

**Remember:** These examples are exploratory. The API will evolve based on what we learn!
