# Stillwater User Guide

Welcome to the Stillwater user guide! This guide will help you understand and master the core concepts of Stillwater.

## What is Stillwater?

Stillwater is a Rust library for pragmatic functional programming focused on two main problems:

1. **Validation**: Accumulating ALL errors instead of failing on the first one
2. **Effects**: Separating pure business logic from I/O operations

## Guide Structure

This guide is organized into progressive chapters, each building on the previous:

### Core Concepts

1. **[Semigroup](01-semigroup.md)** - The foundation for combining errors
   - What is a Semigroup?
   - Why it matters for validation
   - Implementing Semigroup for your types

2. **[Validation](02-validation.md)** - Error accumulation
   - The problem with Result
   - Using Validation for error accumulation
   - Combining validations
   - Real-world examples

3. **[Effects](03-effects.md)** - Pure core, imperative shell
   - Separating logic from I/O
   - Effect composition
   - Testing effectful code
   - Async support

8. **[Monoid](08-monoid.md)** - Identity elements for composition
   - What is a Monoid?
   - Extending Semigroup with identity
   - Numeric monoids (Sum, Product, Max, Min)
   - Using fold_all and reduce
   - Real-world aggregation patterns

### Supporting Features

4. **[Error Context](04-error-context.md)** - Better debugging
   - Adding context to errors
   - Error trails
   - Best practices

5. **[IO Module](05-io-module.md)** - Ergonomic helpers
   - IO::read, IO::write, IO::execute
   - Dependency injection patterns
   - Testing with mock environments

6. **[Helper Combinators](06-helper-combinators.md)** - Common patterns
   - traverse, sequence
   - Convenience functions
   - Building your own combinators

7. **[Try Trait](07-try-trait.md)** - Nightly feature
   - Using ? with Validation and Effect
   - When to enable try_trait
   - Migration path

### Advanced Patterns

12. **[Traverse Patterns](12-traverse-patterns.md)** - Working with collections
   - Collection validation with error accumulation
   - Effect processing over collections
   - Batch operations
   - Traverse vs sequence
   - Real-world examples

Note: Chapter 8 (Monoid) is listed under Core Concepts as it's fundamental to understanding error accumulation and composition patterns.

## How to Use This Guide

### For Beginners

Start with chapters 1-3 in order. These cover the core concepts you need to be productive with Stillwater.

### For Experienced Users

Jump to the chapters that interest you. Each chapter is self-contained with links to related concepts.

### Running Examples

All examples in this guide are runnable. You can find them in the [examples/](../../examples/) directory:

```bash
cargo run --example validation
cargo run --example effects
cargo run --example monoid
cargo run --example form_validation
```

## Quick Reference

### When to Use What

| Use Case | Tool | Example |
|----------|------|---------|
| Form validation | Validation | Collect all field errors |
| API request validation | Validation | Return all validation errors |
| Collection validation | traverse | Validate multiple items, accumulate errors |
| Batch processing | traverse_effect | Process collection with effects |
| Database operations | Effect | Separate logic from I/O |
| File operations | Effect + IO | Testable file processing |
| Error debugging | ContextError | Add context trails |
| Data aggregation | Monoid | Combine collections with fold_all |
| Numeric operations | Sum/Product | Aggregate numbers with identity |

### Common Patterns

```rust
use stillwater::prelude::*;

// Pattern 1: Independent validations
Validation::all((
    validate_email(input.email),
    validate_age(input.age),
))

// Pattern 2: Dependent validations
validate_email(email)
    .and_then(|email| check_email_available(email))

// Pattern 3: Effect with validation
Effect::from_validation(validate_user(input))
    .and_then(|user| save_to_db(user))
```

## Getting Help

- Check the [FAQ](../FAQ.md) for common questions
- Read the [API documentation](https://docs.rs/stillwater)
- See [PATTERNS.md](../PATTERNS.md) for recipes
- Compare to [other libraries](../COMPARISON.md)

## Next Steps

Ready to dive in? Start with [Chapter 1: Semigroup](01-semigroup.md)!
