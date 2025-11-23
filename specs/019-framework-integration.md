---
number: 019
title: Web Framework Integration (Axum, Actix)
category: compatibility
priority: medium
status: draft
dependencies: [017]
created: 2025-11-22
---

# Specification 019: Web Framework Integration

**Category**: compatibility
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 017 (Serde integration)

## Context

Stillwater is ideal for web applications, but integration with popular frameworks (Axum, Actix) should be seamless. Common patterns:

- Extracting environment from request extensions
- Returning Validation errors as JSON responses
- Running Effects in request handlers
- Dependency injection via environment

Currently, users must manually convert between Stillwater types and framework types.

## Objective

Provide framework integrations that make using Stillwater with Axum and Actix ergonomic, with minimal boilerplate.

## Requirements

### Functional Requirements

- Axum extractors for environment
- Axum `IntoResponse` for `Validation`
- Actix extractors for environment
- Actix responders for `Validation`
- Middleware for environment setup
- Examples for both frameworks

### Acceptance Criteria

- [ ] Axum integration crate: `stillwater-axum`
- [ ] Actix integration crate: `stillwater-actix`
- [ ] `FromRequest` extractors for environment
- [ ] `IntoResponse`/`Responder` for Validation
- [ ] Complete examples for both frameworks
- [ ] Documentation guide
- [ ] All tests pass

## Technical Details

### Axum Integration

```rust
// stillwater-axum/src/lib.rs

use axum::{
    extract::{FromRequestParts, rejection::*},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use stillwater::Validation;

/// Extract environment from request extensions.
///
/// # Example
///
/// ```rust
/// async fn handler(
///     Env(env): Env<AppEnv>,
/// ) -> Result<Json<User>, ValidationError> {
///     let user = fetch_user().run(&env)?;
///     Ok(Json(user))
/// }
/// ```
pub struct Env<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Env<T>
where
    T: Clone + Send + Sync + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<T>()
            .cloned()
            .map(Env)
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Environment not found"))
    }
}

/// Convert Validation to HTTP response.
impl<T, E> IntoResponse for Validation<T, E>
where
    T: Serialize,
    E: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            Validation::Success(value) => {
                (StatusCode::OK, Json(value)).into_response()
            }
            Validation::Failure(errors) => {
                (StatusCode::BAD_REQUEST, Json(errors)).into_response()
            }
        }
    }
}
```

### Example Usage

```rust
use axum::{Router, routing::post};
use stillwater_axum::Env;

#[derive(Clone)]
struct AppEnv {
    db: Database,
}

async fn create_user(
    Env(env): Env<AppEnv>,
    Json(input): Json<UserInput>,
) -> impl IntoResponse {
    validate_and_create_user(input)
        .run(&env)
        .await
}

let app = Router::new()
    .route("/users", post(create_user))
    .layer(Extension(app_env));
```

## Documentation Requirements

- Integration guides for both frameworks
- Complete example apps
- Migration guide from traditional error handling

## Success Metrics

- Minimal boilerplate in handlers
- Natural integration with framework patterns
- Positive user feedback
