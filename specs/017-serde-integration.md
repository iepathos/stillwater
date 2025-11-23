---
number: 017
title: Serde Integration
category: compatibility
priority: medium
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 017: Serde Integration

**Category**: compatibility
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

Many Rust applications serialize data to JSON, TOML, or other formats for APIs, configuration, or storage. Stillwater's types (`Validation`, `ContextError`, etc.) should be serializable to enable:

- Returning validation errors as JSON from web APIs
- Serializing configuration with context errors
- Storing validation results in databases

Currently, these types don't implement `Serialize`/`Deserialize`, requiring manual conversion.

## Objective

Add optional serde support behind a feature flag, enabling serialization of Stillwater types without imposing serde as a required dependency.

## Requirements

### Functional Requirements

- Add `serde` feature flag (optional)
- Implement `Serialize` for `Validation<T, E>` where T, E: Serialize
- Implement `Deserialize` for `Validation<T, E>` where T, E: Deserialize
- Implement `Serialize` for `ContextError<E>` where E: Serialize
- Implement `Serialize` for `NonEmptyVec<T>` where T: Serialize
- Sensible JSON representation (human-readable)

### Acceptance Criteria

- [ ] `serde` feature flag in Cargo.toml
- [ ] Serialize/Deserialize for Validation
- [ ] Serialize for ContextError (with context trail)
- [ ] Serialize for NonEmptyVec
- [ ] Examples showing JSON serialization
- [ ] Tests verify roundtrip (serialize → deserialize)
- [ ] Documentation updated

## Technical Details

### Implementation

```rust
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Validation<T, E> {
    Success(T),
    Failure(E),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ContextError<E> {
    error: E,
    context: Vec<String>,
}

// JSON representation:
// {
//   "error": "NotFound",
//   "context": ["Fetching user", "Loading profile"]
// }
```

### Example Usage

```rust
#[derive(Serialize)]
struct ValidationResponse {
    errors: Vec<String>,
}

let validation: Validation<User, Vec<String>> = validate_user(input);

match validation {
    Validation::Failure(errors) => {
        let response = ValidationResponse { errors };
        Json(response) // Axum response
    }
    Validation::Success(user) => {
        Json(user)
    }
}
```

## Documentation Requirements

- Add `docs/guide/13-serde-integration.md`
- Example in README showing JSON API usage
- Rustdoc examples with serde

## Success Metrics

- Clean JSON output
- Roundtrip serialization works
- No overhead when feature disabled
