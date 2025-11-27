---
number: 035
title: Comparison Documentation Enhancement with Before/After Examples
category: foundation
priority: medium
status: draft
dependencies: [025]
created: 2025-11-27
---

# Specification 035: Comparison Documentation Enhancement with Before/After Examples

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 025 (Documentation Update for Zero-Cost Effect API)

## Context

The existing `docs/COMPARISON.md` provides a good overview comparing Stillwater to alternatives (frunk, monadic, anyhow, validator), but it primarily uses prose descriptions and comparison tables. To drive adoption and demonstrate concrete value, the documentation needs **extensive before/after code examples** that show:

1. **Reduced boilerplate** - How Stillwater eliminates repetitive code patterns
2. **Improved readability** - Side-by-side comparisons showing cleaner code
3. **Real-world scenarios** - Common patterns from production applications
4. **Measurable improvements** - Line count reductions, complexity metrics

The Rust community is skeptical of new abstractions unless they see clear, tangible benefits. Abstract feature comparisons are less convincing than concrete code transformations.

### Current Gap Analysis

The existing `COMPARISON.md`:
- ✓ Has comparison tables (good for quick reference)
- ✓ Lists when to use each library
- ✓ Includes some migration snippets
- ✗ Lacks comprehensive before/after examples
- ✗ Missing real-world scenario comparisons
- ✗ No boilerplate reduction metrics
- ✗ Limited examples showing Stillwater + other crates together

## Objective

Transform `COMPARISON.md` into a compelling document that demonstrates Stillwater's value through **extensive, concrete before/after code examples** across multiple real-world scenarios, showing measurable reductions in boilerplate and complexity.

## Requirements

### Functional Requirements

#### FR1: Form Validation Comparison Suite

Add comprehensive before/after examples for form validation:

- **MUST** show traditional `Result` chain vs `Validation::all()`
- **MUST** demonstrate 3+ field validation with error accumulation
- **MUST** include nested object validation example
- **MUST** show line count reduction (target: 40-60% reduction)
- **SHOULD** include conditional validation logic

#### FR2: Error Context Preservation Comparison

Add before/after examples for error context handling:

- **MUST** compare manual error wrapping vs `.context()` chain
- **MUST** show nested function call error trails
- **MUST** demonstrate debugging improvements
- **SHOULD** show comparison with `anyhow` context

#### FR3: Dependency Injection Comparison

Add before/after examples for DI patterns:

- **MUST** show parameter threading vs Reader pattern
- **MUST** demonstrate 3+ dependency scenario
- **MUST** show testing improvements (mock injection)
- **SHOULD** compare with Arc<dyn Trait> pattern

#### FR4: Async Service Composition Comparison

Add before/after examples for async operations:

- **MUST** show raw async/await vs Effect composition
- **MUST** demonstrate error handling in async chains
- **MUST** show retry logic comparison
- **SHOULD** include parallel operation examples

#### FR5: Real-World Scenario Sections

Create dedicated sections for common use cases:

- **MUST** include "API Request Handler" scenario
- **MUST** include "Database Transaction" scenario
- **MUST** include "Configuration Validation" scenario
- **SHOULD** include "Event Processing Pipeline" scenario

#### FR6: Boilerplate Metrics

Add measurable improvement metrics:

- **MUST** include line-of-code comparisons for each example
- **MUST** show cyclomatic complexity reduction where applicable
- **SHOULD** include "number of error handling branches" comparison

#### FR7: Complementary Usage Examples

Expand examples showing Stillwater with other crates:

- **MUST** show Stillwater + anyhow integration
- **MUST** show Stillwater + validator integration
- **SHOULD** show Stillwater + thiserror integration
- **SHOULD** show Stillwater + tracing integration

### Non-Functional Requirements

#### NFR1: Code Example Quality

- All examples MUST compile and be tested
- Examples MUST use the new zero-cost API (per Spec 025)
- Examples MUST be realistic, not contrived
- Examples SHOULD be copy-paste ready

#### NFR2: Visual Clarity

- Before/after examples MUST be side-by-side or clearly labeled
- Line count reductions MUST be explicitly stated
- Key improvements MUST be highlighted with comments

#### NFR3: Accessibility

- Examples SHOULD progress from simple to complex
- Each example SHOULD be understandable in isolation
- Terminology SHOULD be consistent throughout

## Acceptance Criteria

### Form Validation Section

- [ ] **AC1**: Traditional Result chain example (15+ lines)
- [ ] **AC2**: Stillwater Validation equivalent (8-10 lines)
- [ ] **AC3**: Line reduction percentage stated explicitly
- [ ] **AC4**: Nested object validation before/after
- [ ] **AC5**: Conditional validation before/after

### Error Context Section

- [ ] **AC6**: Manual error wrapping example (3+ nested calls)
- [ ] **AC7**: Stillwater `.context()` equivalent
- [ ] **AC8**: Error trail output comparison shown

### Dependency Injection Section

- [ ] **AC9**: Parameter threading example (3+ deps, 3+ functions)
- [ ] **AC10**: Reader pattern equivalent
- [ ] **AC11**: Test code comparison showing mock injection

### Async Composition Section

- [ ] **AC12**: Raw async chain with error handling
- [ ] **AC13**: Effect chain equivalent
- [ ] **AC14**: Retry logic before/after

### Real-World Scenarios Section

- [ ] **AC15**: Complete API handler scenario
- [ ] **AC16**: Database transaction scenario
- [ ] **AC17**: Config validation scenario

### Metrics

- [ ] **AC18**: Summary table with all LOC reductions
- [ ] **AC19**: Each major example has explicit metrics

### Integration Examples

- [ ] **AC20**: Stillwater + anyhow working example
- [ ] **AC21**: Stillwater + validator working example

### Quality

- [ ] **AC22**: All code examples compile (`cargo test --doc`)
- [ ] **AC23**: No broken internal links

## Technical Details

### Example Structure Template

Each before/after comparison should follow this structure:

```markdown
### [Scenario Name]

**Problem**: [One sentence describing the pain point]

**Before** (Traditional Rust):
```rust
// X lines
[code]
```

**After** (With Stillwater):
```rust
// Y lines - Z% reduction
[code]
```

**Key Improvements**:
- [Bullet point 1]
- [Bullet point 2]
```

### Proposed New Sections

#### 1. Form Validation Deep Dive

```rust
// BEFORE: Traditional Result chain (22 lines)
fn validate_user_registration(input: &RegistrationInput) -> Result<ValidatedUser, Vec<String>> {
    let mut errors = Vec::new();

    let email = match validate_email(&input.email) {
        Ok(e) => Some(e),
        Err(e) => { errors.push(e); None }
    };

    let age = match validate_age(input.age) {
        Ok(a) => Some(a),
        Err(e) => { errors.push(e); None }
    };

    let username = match validate_username(&input.username) {
        Ok(u) => Some(u),
        Err(e) => { errors.push(e); None }
    };

    if errors.is_empty() {
        Ok(ValidatedUser {
            email: email.unwrap(),
            age: age.unwrap(),
            username: username.unwrap(),
        })
    } else {
        Err(errors)
    }
}

// AFTER: Stillwater Validation (8 lines - 64% reduction)
fn validate_user_registration(input: &RegistrationInput) -> Validation<ValidatedUser, Vec<String>> {
    Validation::all((
        validate_email(&input.email),
        validate_age(input.age),
        validate_username(&input.username),
    ))
    .map(|(email, age, username)| ValidatedUser { email, age, username })
}
```

#### 2. Dependency Threading Deep Dive

```rust
// BEFORE: Parameter threading (service with 3 deps, 3 operations)
async fn process_order(
    db: &Database,
    cache: &Cache,
    email_service: &EmailService,
    order: Order,
) -> Result<Receipt, Error> {
    let user = fetch_user(db, order.user_id).await?;
    let inventory = check_inventory(db, cache, &order.items).await?;
    let receipt = create_receipt(db, &user, &order, &inventory).await?;
    send_confirmation(email_service, &user, &receipt).await?;
    Ok(receipt)
}

async fn fetch_user(db: &Database, id: UserId) -> Result<User, Error> { ... }
async fn check_inventory(db: &Database, cache: &Cache, items: &[Item]) -> Result<Inventory, Error> { ... }
async fn create_receipt(db: &Database, user: &User, order: &Order, inv: &Inventory) -> Result<Receipt, Error> { ... }
async fn send_confirmation(email: &EmailService, user: &User, receipt: &Receipt) -> Result<(), Error> { ... }

// AFTER: Reader pattern (no parameter threading)
fn process_order(order: Order) -> impl Effect<Output = Receipt, Error = Error, Env = AppEnv> {
    asks(|env: &AppEnv| env.db.fetch_user(order.user_id))
        .and_then(move |user| {
            asks(move |env: &AppEnv| env.check_inventory(&order.items))
                .and_then(move |inventory| {
                    asks(move |env: &AppEnv| env.db.create_receipt(&user, &order, &inventory))
                        .and_then(move |receipt| {
                            asks(move |env: &AppEnv| env.email.send_confirmation(&user, &receipt))
                                .map(move |_| receipt)
                        })
                })
        })
}

// Test becomes trivial - no mocking frameworks needed
#[test]
fn test_process_order() {
    let test_env = AppEnv {
        db: InMemoryDb::with_test_data(),
        cache: NoOpCache,
        email: RecordingEmailService::new(),
    };
    let result = process_order(test_order).run(&test_env).await;
    assert!(result.is_ok());
}
```

#### 3. Boilerplate Summary Table

| Scenario | Before (LOC) | After (LOC) | Reduction |
|----------|--------------|-------------|-----------|
| 3-field validation | 22 | 8 | 64% |
| 5-field validation | 38 | 12 | 68% |
| Nested validation | 45 | 15 | 67% |
| Dependency threading (3 deps) | 28 | 18 | 36% |
| Error context chain (4 levels) | 24 | 12 | 50% |
| Retry with backoff | 35 | 10 | 71% |

### Files to Modify

| File | Changes |
|------|---------|
| `docs/COMPARISON.md` | Major expansion with all new sections |

### Documentation Test File

Create `docs/comparison_examples.rs` (or add to existing test suite) to verify all code examples compile:

```rust
#[cfg(test)]
mod comparison_doc_tests {
    // All before/after examples should be tested here
    // to ensure they compile and work as documented
}
```

## Dependencies

### Prerequisites
- Spec 025 (Documentation Update for Zero-Cost Effect API) - new API must be documented first

### Affected Components
- `docs/COMPARISON.md`
- Possibly new test file for doc examples

### External Dependencies
- None new

## Testing Strategy

### Documentation Tests

- **Unit Tests**: All code examples must be embedded as doc tests or in a dedicated test module
- **Compilation Check**: `cargo test --doc` must pass
- **Example Verification**: Each "After" example must produce the same result as its "Before" counterpart

### Review Criteria

- Technical accuracy verified by running examples
- Clarity reviewed by someone unfamiliar with Stillwater
- Metrics spot-checked for accuracy

## Documentation Requirements

### Code Documentation
- Each example should have inline comments explaining key points
- Metrics should be calculated and verified, not estimated

### User Documentation
- This spec IS about documentation improvement
- No additional architecture docs needed

## Implementation Notes

### Writing Guidelines

1. **Start with real pain points** - Each example should address a genuine frustration
2. **Keep examples self-contained** - Don't require context from other sections
3. **Show complete code** - Include imports, type definitions where helpful
4. **Be honest about trade-offs** - If Stillwater adds complexity somewhere, acknowledge it
5. **Use consistent naming** - Same variable names, same scenario across comparisons

### Common Pitfalls to Avoid

- Don't cherry-pick trivial examples where improvement is minimal
- Don't hide complexity by omitting necessary code
- Don't make "before" examples artificially bad
- Don't ignore cases where traditional Rust is better

### Suggested Section Order

1. Quick wins (validation) - most compelling, easiest to understand
2. Error handling - common need, clear improvement
3. Dependency injection - more advanced, but high value
4. Async composition - for users already bought in
5. Real-world scenarios - putting it all together
6. Complementary usage - showing Stillwater plays well with others

## Migration and Compatibility

No breaking changes - this is documentation improvement only.

## Success Metrics

### Quantitative
- All code examples compile
- Average LOC reduction across examples ≥ 40%
- At least 8 complete before/after comparisons

### Qualitative
- Examples are realistic and relatable
- Improvements are genuinely compelling
- Documentation helps drive adoption decisions

---

*"Show, don't tell. Code speaks louder than prose."*
