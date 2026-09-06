# Stillwater Specifications

Specifications are design inputs, not a promise that every draft belongs in the public API.
Their front-matter status is authoritative.

## Status lifecycle

- `draft`: problem and design are still being explored.
- `ready`: accepted for implementation with current requirements.
- `in-progress`: actively being implemented.
- `complete`: implemented and verified.
- `parked`: intentionally deferred; the spec records a possible direction but is not roadmap work.

A parked spec must state concrete reactivation criteria. Reactivation starts with a fresh
architecture review; old performance or trait-bound claims are not automatically accepted.

## Current specifications

| Spec | Topic | Status |
|------|-------|--------|
| [003](003-saga-compensation.md) | Saga compensation | Draft |
| [017](017-serde-integration.md) | Serde integration | Draft |
| [019](019-framework-integration.md) | Framework integration | Draft |
| [030](030-result-extension-trait.md) | Result extension trait | Draft |
| [034](034-effect-with-flat-tuple-combinators.md) | Flat tuple Effect combinators | Parked |
| [035](035-enhanced-validation-assertions.md) | Enhanced validation assertions | Ready |
| [039](039-circuit-breaker.md) | Circuit breaker | Parked |

## Quality gate

Before moving a spec to `complete`, its behavior must compile, have focused tests, pass the
full relevant test suite and lints, and be reflected in user documentation. New effect APIs
must also justify why composition with the core primitives is insufficient.
