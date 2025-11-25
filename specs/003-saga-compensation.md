---
number: 3
title: Saga Pattern and Compensation Workflows
category: foundation
priority: medium
status: draft
dependencies: [1, 2]
created: 2025-01-24
---

# Specification 003: Saga Pattern and Compensation Workflows

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 001 (Retry), Spec 002 (Resource Scopes)

## Context

Business processes often span multiple services or systems. When a step fails partway through, you need to undo the work already done. This is the **Saga pattern**—a sequence of local transactions with compensating actions.

### The Problem

```rust
// E-commerce checkout - what happens when step 3 fails?
async fn checkout(order: Order) -> Result<Receipt, Error> {
    // Step 1: Reserve inventory
    let reservation = inventory_service.reserve(&order.items).await?;

    // Step 2: Charge payment
    let charge = payment_service.charge(&order.payment).await?;

    // Step 3: Create shipment - THIS FAILS!
    let shipment = shipping_service.create(&order.address).await?;
    // ❌ Now we have:
    //    - Reserved inventory (needs unreserving)
    //    - Charged payment (needs refunding)
    //    - No shipment
    // Manual rollback is error-prone and hard to maintain

    Ok(Receipt { reservation, charge, shipment })
}
```

### Current Solutions

**Manual rollback**: Deeply nested error handling, easy to miss cases
```rust
match shipping_service.create(&order.address).await {
    Ok(shipment) => Ok(Receipt { ... }),
    Err(e) => {
        // Must remember to undo everything
        payment_service.refund(&charge).await?;  // What if THIS fails?
        inventory_service.unreserve(&reservation).await?;
        Err(e)
    }
}
```

**External orchestrators**: Temporal, Conductor, AWS Step Functions
- Powerful but heavyweight
- Require infrastructure
- Overkill for most applications

### What's Missing

A lightweight, library-level saga implementation that:
- Defines workflows as data
- Automatically runs compensations on failure
- Handles compensation failures gracefully
- Integrates with Effect composition
- Requires no external infrastructure

### Prior Art

- **Temporal.io**: Full workflow orchestration platform
- **AWS Step Functions**: Managed state machines
- **NServiceBus Sagas**: .NET saga implementation
- **Axon Framework**: Java CQRS/Event Sourcing with sagas
- **saga-rs**: Rust crate, but unmaintained and not async

## Objective

Add saga pattern support to stillwater that:

1. Models sagas as composable data structures
2. Automatically runs compensations in reverse order on failure
3. Handles compensation failures with configurable strategies
4. Supports both sequential and parallel step execution
5. Provides observability hooks for monitoring
6. Integrates naturally with Effect and retry patterns

## Requirements

### Functional Requirements

#### FR-1: Basic Saga Definition

```rust
let checkout_saga = Saga::new()
    .step(
        "reserve_inventory",
        |ctx: &CheckoutContext| reserve_items(&ctx.order),
        |ctx: &CheckoutContext, reservation: &Reservation| unreserve_items(reservation),
    )
    .step(
        "charge_payment",
        |ctx: &CheckoutContext| charge_card(&ctx.payment),
        |ctx: &CheckoutContext, charge: &Charge| refund_charge(charge),
    )
    .step(
        "create_shipment",
        |ctx: &CheckoutContext| create_shipment(&ctx.address),
        |ctx: &CheckoutContext, shipment: &Shipment| cancel_shipment(shipment),
    );

let result = checkout_saga.run(context, &env).await;
```

#### FR-2: Compensation Execution Order

```rust
// If step 3 fails:
// 1. Compensation for step 2 runs (refund)
// 2. Compensation for step 1 runs (unreserve)
// Compensations run in REVERSE order of successful steps

// If step 2 fails:
// 1. Compensation for step 1 runs (unreserve)
// Step 2 compensation does NOT run (step 2 never succeeded)
```

#### FR-3: Accessing Previous Step Results

```rust
let saga = Saga::new()
    .step(
        "create_order",
        |ctx| create_order(&ctx.items),
        |_, order| delete_order(order),
    )
    .step_with_result(
        "reserve_inventory",
        // Access previous step's result
        |ctx, results| {
            let order: &Order = results.get("create_order")?;
            reserve_for_order(order)
        },
        |_, _, reservation| unreserve(reservation),
    )
    .step_with_result(
        "charge_payment",
        |ctx, results| {
            let order: &Order = results.get("create_order")?;
            charge_for_order(order)
        },
        |_, _, charge| refund(charge),
    );
```

#### FR-4: Conditional Steps

```rust
let saga = Saga::new()
    .step("reserve", reserve_items, unreserve_items)
    .step_if(
        |ctx| ctx.requires_approval,
        "approval",
        request_approval,
        revoke_approval,
    )
    .step("charge", charge_payment, refund_payment);
```

#### FR-5: Parallel Steps

```rust
let saga = Saga::new()
    .step("validate_order", validate, |_, _| Effect::pure(()))
    .parallel(vec![
        Step::new("reserve_inventory", reserve, unreserve),
        Step::new("check_fraud", check_fraud, |_, _| Effect::pure(())),
        Step::new("validate_address", validate_addr, |_, _| Effect::pure(())),
    ])
    .step("charge", charge, refund);

// Parallel steps run concurrently
// If any parallel step fails, compensations for successful parallel steps run
// Then compensations for prior sequential steps run
```

#### FR-6: Compensation Failure Handling

```rust
// Option 1: Ignore compensation failures (log and continue)
let saga = Saga::new()
    .compensation_strategy(CompensationStrategy::BestEffort)
    .step(...);

// Option 2: Retry compensations
let saga = Saga::new()
    .compensation_strategy(CompensationStrategy::Retry(
        RetryPolicy::exponential(Duration::from_millis(100)).with_max_retries(3)
    ))
    .step(...);

// Option 3: Fail on compensation failure
let saga = Saga::new()
    .compensation_strategy(CompensationStrategy::Strict)
    .step(...);

// Option 4: Custom handler
let saga = Saga::new()
    .on_compensation_failure(|step_name, error| {
        alert_ops_team(step_name, error);
        Effect::pure(())  // Continue with other compensations
    })
    .step(...);
```

#### FR-7: Saga Events and Observability

```rust
let saga = Saga::new()
    .step("reserve", reserve, unreserve)
    .step("charge", charge, refund)
    .on_step_start(|event| {
        log::info!("Starting step: {}", event.step_name);
        metrics::counter!("saga.step.started", 1, "step" => event.step_name);
        Effect::pure(())
    })
    .on_step_complete(|event| {
        log::info!("Completed step: {} in {:?}", event.step_name, event.duration);
        Effect::pure(())
    })
    .on_step_failed(|event| {
        log::error!("Step {} failed: {:?}", event.step_name, event.error);
        Effect::pure(())
    })
    .on_compensation_start(|event| {
        log::warn!("Running compensation for: {}", event.step_name);
        Effect::pure(())
    });
```

#### FR-8: Saga Composition

```rust
// Compose smaller sagas into larger ones
let payment_saga = Saga::new()
    .step("authorize", authorize_card, void_authorization)
    .step("capture", capture_payment, refund_payment);

let fulfillment_saga = Saga::new()
    .step("reserve", reserve_inventory, unreserve)
    .step("ship", create_shipment, cancel_shipment);

let order_saga = Saga::new()
    .step("create_order", create_order, delete_order)
    .include("payment", payment_saga)  // Embed sub-saga
    .include("fulfillment", fulfillment_saga)
    .step("notify", send_confirmation, |_, _| Effect::pure(()));
```

#### FR-9: Typed Results

```rust
// Saga returns a typed result combining all step outputs
let saga = Saga::new()
    .step("order", create_order, delete_order)       // -> Order
    .step("payment", charge_card, refund)            // -> Charge
    .step("shipment", create_shipment, cancel);      // -> Shipment

let result: SagaResult<(Order, Charge, Shipment)> = saga.run(ctx, &env).await;

match result {
    SagaResult::Completed(order, charge, shipment) => { /* success */ }
    SagaResult::Compensated { failed_step, error, compensated_steps } => { /* handled */ }
    SagaResult::CompensationFailed { original_error, compensation_errors } => { /* uh oh */ }
}
```

#### FR-10: Integration with Retry

```rust
// Retry individual steps before triggering compensation
let saga = Saga::new()
    .step_with_retry(
        "charge_payment",
        charge_card,
        refund_card,
        RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(3)
            .retry_if(|e| e.is_transient()),
    )
    .step("ship", create_shipment, cancel_shipment);
```

### Non-Functional Requirements

#### NFR-1: No External Dependencies

- Pure Rust implementation
- No database required for simple sagas
- No message queue required

#### NFR-2: Deterministic Execution

- Same inputs produce same execution order
- Useful for testing and debugging

#### NFR-3: Memory Efficient

- Don't store unnecessary intermediate state
- Compensation functions receive only what they need

#### NFR-4: Clear Error Messages

- Saga failures include full execution history
- Clear indication of which step failed
- Which compensations succeeded/failed

## Acceptance Criteria

- [ ] `Saga` builder for defining workflows
- [ ] `Step` type with action and compensation
- [ ] Sequential step execution with automatic compensation
- [ ] Parallel step execution with `parallel()`
- [ ] Access to previous step results
- [ ] Conditional steps with `step_if()`
- [ ] Compensation strategies (BestEffort, Retry, Strict)
- [ ] Event hooks for observability
- [ ] Saga composition with `include()`
- [ ] Integration with `RetryPolicy`
- [ ] `SagaResult` type with success/compensated/failed variants
- [ ] Comprehensive unit tests
- [ ] Integration tests with simulated failures
- [ ] Documentation with examples
- [ ] Example file: `examples/saga_checkout.rs`

## Technical Details

### Implementation Approach

#### Core Types

```rust
/// A saga representing a sequence of steps with compensating actions.
pub struct Saga<Ctx, T, E, Env> {
    steps: Vec<SagaStep<Ctx, E, Env>>,
    compensation_strategy: CompensationStrategy,
    hooks: SagaHooks<E, Env>,
    _phantom: PhantomData<T>,
}

/// A single step in a saga.
pub struct SagaStep<Ctx, E, Env> {
    name: &'static str,
    action: StepAction<Ctx, E, Env>,
    compensation: StepCompensation<Ctx, E, Env>,
    condition: Option<Box<dyn Fn(&Ctx) -> bool + Send + Sync>>,
    retry_policy: Option<RetryPolicy>,
}

/// How the step action is executed.
enum StepAction<Ctx, E, Env> {
    /// Simple action that only needs context
    Simple(Box<dyn Fn(&Ctx) -> Effect<Box<dyn Any + Send>, E, Env> + Send + Sync>),
    /// Action that can access previous results
    WithResults(Box<dyn Fn(&Ctx, &StepResults) -> Effect<Box<dyn Any + Send>, E, Env> + Send + Sync>),
    /// Parallel actions
    Parallel(Vec<SagaStep<Ctx, E, Env>>),
}

/// Compensation function for a step.
type StepCompensation<Ctx, E, Env> =
    Box<dyn Fn(&Ctx, &dyn Any) -> Effect<(), E, Env> + Send + Sync>;

/// Results from completed steps, keyed by step name.
pub struct StepResults {
    results: HashMap<&'static str, Box<dyn Any + Send>>,
}

impl StepResults {
    pub fn get<T: 'static>(&self, step_name: &str) -> Option<&T> {
        self.results.get(step_name)?.downcast_ref()
    }
}

/// Strategy for handling compensation failures.
#[derive(Debug, Clone)]
pub enum CompensationStrategy {
    /// Log failures, continue with remaining compensations
    BestEffort,
    /// Retry failed compensations
    Retry(RetryPolicy),
    /// Fail immediately on first compensation failure
    Strict,
}

/// Result of saga execution.
#[derive(Debug)]
pub enum SagaResult<T, E> {
    /// All steps completed successfully.
    Completed(T),
    /// A step failed and compensations ran successfully.
    Compensated {
        /// Name of the step that failed
        failed_step: &'static str,
        /// The error from the failed step
        error: E,
        /// Steps that were compensated
        compensated_steps: Vec<&'static str>,
    },
    /// A step failed and some compensations also failed.
    CompensationFailed {
        /// The original step error
        original_error: E,
        /// Which step originally failed
        failed_step: &'static str,
        /// Errors from failed compensations
        compensation_errors: Vec<CompensationError<E>>,
    },
}

#[derive(Debug)]
pub struct CompensationError<E> {
    pub step_name: &'static str,
    pub error: E,
}
```

#### Saga Builder

```rust
impl<Ctx, E, Env> Saga<Ctx, (), E, Env>
where
    Ctx: Send + Sync + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    /// Create a new empty saga.
    pub fn new() -> Self {
        Saga {
            steps: Vec::new(),
            compensation_strategy: CompensationStrategy::BestEffort,
            hooks: SagaHooks::default(),
            _phantom: PhantomData,
        }
    }

    /// Add a step to the saga.
    pub fn step<T, A, C, AFut, CFut>(
        self,
        name: &'static str,
        action: A,
        compensation: C,
    ) -> Saga<Ctx, (T,), E, Env>  // Type accumulates
    where
        T: Send + 'static,
        A: Fn(&Ctx) -> Effect<T, E, Env> + Send + Sync + 'static,
        C: Fn(&Ctx, &T) -> Effect<(), E, Env> + Send + Sync + 'static,
    {
        // Add step, return saga with accumulated type
    }

    /// Add a conditional step.
    pub fn step_if<T, P, A, C>(
        self,
        predicate: P,
        name: &'static str,
        action: A,
        compensation: C,
    ) -> Saga<Ctx, (Option<T>,), E, Env>
    where
        P: Fn(&Ctx) -> bool + Send + Sync + 'static,
        // ...
    {
        // Add conditional step
    }

    /// Add parallel steps.
    pub fn parallel(self, steps: Vec<Step<Ctx, E, Env>>) -> Self {
        // Add parallel step group
    }

    /// Set compensation strategy.
    pub fn compensation_strategy(mut self, strategy: CompensationStrategy) -> Self {
        self.compensation_strategy = strategy;
        self
    }

    /// Include a sub-saga.
    pub fn include<T2>(self, name: &'static str, saga: Saga<Ctx, T2, E, Env>) -> Self {
        // Flatten sub-saga steps into this saga
    }
}
```

#### Saga Execution

```rust
impl<Ctx, T, E, Env> Saga<Ctx, T, E, Env>
where
    Ctx: Send + Sync + 'static,
    E: Send + Clone + 'static,
    Env: Sync + 'static,
{
    /// Execute the saga.
    pub fn run(self, context: Ctx, env: &Env) -> Effect<SagaResult<T, E>, E, Env> {
        Effect::from_async(move |env| async move {
            let mut completed_steps: Vec<CompletedStep<E, Env>> = Vec::new();
            let mut results = StepResults::new();

            for step in &self.steps {
                // Check condition
                if let Some(cond) = &step.condition {
                    if !cond(&context) {
                        continue;
                    }
                }

                // Run hooks
                self.hooks.on_step_start(&step.name);

                // Execute step (with retry if configured)
                let step_result = match &step.retry_policy {
                    Some(policy) => {
                        execute_step(&step, &context, &results)
                            .with_retry(policy.clone())
                            .run(env)
                            .await
                    }
                    None => {
                        execute_step(&step, &context, &results)
                            .run(env)
                            .await
                    }
                };

                match step_result {
                    Ok(value) => {
                        // Store result for later steps
                        results.insert(step.name, value.clone());

                        // Store for potential compensation
                        completed_steps.push(CompletedStep {
                            name: step.name,
                            result: value,
                            compensation: step.compensation.clone(),
                        });

                        self.hooks.on_step_complete(&step.name);
                    }
                    Err(error) => {
                        self.hooks.on_step_failed(&step.name, &error);

                        // Run compensations in reverse order
                        let compensation_result = self.run_compensations(
                            &context,
                            completed_steps,
                            env,
                        ).await;

                        return Ok(match compensation_result {
                            Ok(compensated) => SagaResult::Compensated {
                                failed_step: step.name,
                                error,
                                compensated_steps: compensated,
                            },
                            Err(comp_errors) => SagaResult::CompensationFailed {
                                original_error: error,
                                failed_step: step.name,
                                compensation_errors: comp_errors,
                            },
                        });
                    }
                }
            }

            // All steps completed successfully
            Ok(SagaResult::Completed(self.build_result(results)))
        })
    }

    async fn run_compensations(
        &self,
        context: &Ctx,
        completed_steps: Vec<CompletedStep<E, Env>>,
        env: &Env,
    ) -> Result<Vec<&'static str>, Vec<CompensationError<E>>> {
        let mut compensated = Vec::new();
        let mut errors = Vec::new();

        // Reverse order!
        for step in completed_steps.into_iter().rev() {
            self.hooks.on_compensation_start(&step.name);

            let comp_result = match &self.compensation_strategy {
                CompensationStrategy::BestEffort => {
                    (step.compensation)(context, &step.result)
                        .run(env)
                        .await
                }
                CompensationStrategy::Retry(policy) => {
                    (step.compensation)(context, &step.result)
                        .with_retry(policy.clone())
                        .run(env)
                        .await
                        .map_err(|e| e.final_error)
                }
                CompensationStrategy::Strict => {
                    (step.compensation)(context, &step.result)
                        .run(env)
                        .await
                }
            };

            match comp_result {
                Ok(()) => {
                    compensated.push(step.name);
                }
                Err(e) => {
                    match &self.compensation_strategy {
                        CompensationStrategy::BestEffort => {
                            tracing::error!(
                                "Compensation failed for step '{}': {:?}",
                                step.name, e
                            );
                            // Continue with other compensations
                        }
                        CompensationStrategy::Strict => {
                            errors.push(CompensationError {
                                step_name: step.name,
                                error: e,
                            });
                            // Stop immediately
                            break;
                        }
                        CompensationStrategy::Retry(_) => {
                            // Retry already happened, this is final failure
                            errors.push(CompensationError {
                                step_name: step.name,
                                error: e,
                            });
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(compensated)
        } else {
            Err(errors)
        }
    }
}
```

### Architecture Changes

New module structure:

```
src/
├── lib.rs           # Re-export saga types
├── effect.rs        # Existing
├── saga/
│   ├── mod.rs       # Module root, re-exports
│   ├── builder.rs   # Saga builder API
│   ├── step.rs      # Step, StepAction, StepCompensation
│   ├── executor.rs  # Saga execution logic
│   ├── result.rs    # SagaResult, CompensationError
│   ├── strategy.rs  # CompensationStrategy
│   └── hooks.rs     # SagaHooks, event types
```

### Data Structures

```rust
/// Event emitted when a step starts.
#[derive(Debug, Clone)]
pub struct StepStartEvent<'a> {
    pub step_name: &'a str,
    pub step_index: usize,
    pub total_steps: usize,
}

/// Event emitted when a step completes.
#[derive(Debug, Clone)]
pub struct StepCompleteEvent<'a> {
    pub step_name: &'a str,
    pub duration: Duration,
}

/// Event emitted when a step fails.
#[derive(Debug, Clone)]
pub struct StepFailedEvent<'a, E> {
    pub step_name: &'a str,
    pub error: &'a E,
    pub duration: Duration,
}

/// Event emitted when compensation starts.
#[derive(Debug, Clone)]
pub struct CompensationStartEvent<'a> {
    pub step_name: &'a str,
    pub reason: &'a str,  // Name of failed step
}

/// Hooks for saga observability.
pub struct SagaHooks<E, Env> {
    pub on_step_start: Option<Box<dyn Fn(StepStartEvent) -> Effect<(), E, Env> + Send + Sync>>,
    pub on_step_complete: Option<Box<dyn Fn(StepCompleteEvent) -> Effect<(), E, Env> + Send + Sync>>,
    pub on_step_failed: Option<Box<dyn Fn(StepFailedEvent<E>) -> Effect<(), E, Env> + Send + Sync>>,
    pub on_compensation_start: Option<Box<dyn Fn(CompensationStartEvent) -> Effect<(), E, Env> + Send + Sync>>,
}
```

### APIs and Interfaces

#### Saga Builder

```rust
impl<Ctx, T, E, Env> Saga<Ctx, T, E, Env> {
    pub fn new() -> Saga<Ctx, (), E, Env>;

    // Step types
    pub fn step<U, A, C>(self, name: &'static str, action: A, compensation: C) -> Saga<Ctx, ..., E, Env>;
    pub fn step_if<U, P, A, C>(self, pred: P, name: &'static str, action: A, comp: C) -> Saga<Ctx, ..., E, Env>;
    pub fn step_with_result<U, A, C>(self, name: &'static str, action: A, comp: C) -> Saga<Ctx, ..., E, Env>;
    pub fn step_with_retry<U, A, C>(self, name: &'static str, action: A, comp: C, policy: RetryPolicy) -> Saga<Ctx, ..., E, Env>;
    pub fn parallel(self, steps: Vec<Step<Ctx, E, Env>>) -> Self;

    // Configuration
    pub fn compensation_strategy(self, strategy: CompensationStrategy) -> Self;
    pub fn on_compensation_failure<F>(self, handler: F) -> Self;

    // Hooks
    pub fn on_step_start<F>(self, f: F) -> Self;
    pub fn on_step_complete<F>(self, f: F) -> Self;
    pub fn on_step_failed<F>(self, f: F) -> Self;
    pub fn on_compensation_start<F>(self, f: F) -> Self;

    // Composition
    pub fn include<T2>(self, name: &'static str, saga: Saga<Ctx, T2, E, Env>) -> Self;

    // Execution
    pub fn run(self, context: Ctx, env: &Env) -> Effect<SagaResult<T, E>, E, Env>;
}
```

#### Step Builder (for parallel)

```rust
pub struct Step<Ctx, E, Env> { ... }

impl<Ctx, E, Env> Step<Ctx, E, Env> {
    pub fn new<T, A, C>(name: &'static str, action: A, compensation: C) -> Self;
    pub fn with_retry(self, policy: RetryPolicy) -> Self;
}
```

## Dependencies

- **Prerequisites**:
  - Spec 001 (Retry) - For `step_with_retry` and compensation retry
  - Spec 002 (Resource Scopes) - Conceptual alignment, not direct dependency
- **Affected Components**:
  - `lib.rs` - Re-export saga types
- **External Dependencies**:
  - `tracing` (existing, for logging)
  - `tokio` (existing, for async)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_saga_all_steps_succeed() {
        let saga = Saga::new()
            .step("step1", |_| Effect::pure(1), |_, _| Effect::pure(()))
            .step("step2", |_| Effect::pure(2), |_, _| Effect::pure(()))
            .step("step3", |_| Effect::pure(3), |_, _| Effect::pure(()));

        let result = saga.run((), &()).await.unwrap();

        match result {
            SagaResult::Completed((a, b, c)) => {
                assert_eq!((a, b, c), (1, 2, 3));
            }
            _ => panic!("Expected Completed"),
        }
    }

    #[tokio::test]
    async fn test_saga_compensates_on_failure() {
        let compensated = Arc::new(Mutex::new(Vec::new()));
        let comp1 = compensated.clone();
        let comp2 = compensated.clone();

        let saga = Saga::new()
            .step(
                "step1",
                |_| Effect::pure(1),
                move |_, _| {
                    comp1.lock().unwrap().push("comp1");
                    Effect::pure(())
                },
            )
            .step(
                "step2",
                |_| Effect::pure(2),
                move |_, _| {
                    comp2.lock().unwrap().push("comp2");
                    Effect::pure(())
                },
            )
            .step(
                "step3",
                |_| Effect::<i32, String, ()>::fail("step3 failed".into()),
                |_, _| Effect::pure(()),
            );

        let result = saga.run((), &()).await.unwrap();

        match result {
            SagaResult::Compensated { failed_step, compensated_steps, .. } => {
                assert_eq!(failed_step, "step3");
                assert_eq!(compensated_steps, vec!["step2", "step1"]);
            }
            _ => panic!("Expected Compensated"),
        }

        // Check compensation ran in reverse order
        assert_eq!(*compensated.lock().unwrap(), vec!["comp2", "comp1"]);
    }

    #[tokio::test]
    async fn test_saga_handles_compensation_failure() {
        let saga = Saga::new()
            .compensation_strategy(CompensationStrategy::BestEffort)
            .step(
                "step1",
                |_| Effect::pure(1),
                |_, _| Effect::<(), String, ()>::fail("comp1 failed".into()),
            )
            .step(
                "step2",
                |_| Effect::<i32, String, ()>::fail("step2 failed".into()),
                |_, _| Effect::pure(()),
            );

        let result = saga.run((), &()).await.unwrap();

        // BestEffort means we get Compensated, not CompensationFailed
        assert!(matches!(result, SagaResult::Compensated { .. }));
    }

    #[tokio::test]
    async fn test_saga_conditional_step() {
        let saga = Saga::new()
            .step("always", |_| Effect::pure(1), |_, _| Effect::pure(()))
            .step_if(
                |ctx: &bool| *ctx,  // Only if context is true
                "conditional",
                |_| Effect::pure(2),
                |_, _| Effect::pure(()),
            );

        // With condition true
        let result = saga.clone().run(true, &()).await.unwrap();
        assert!(matches!(result, SagaResult::Completed((1, Some(2)))));

        // With condition false
        let result = saga.run(false, &()).await.unwrap();
        assert!(matches!(result, SagaResult::Completed((1, None))));
    }

    #[tokio::test]
    async fn test_saga_step_accesses_previous_result() {
        let saga = Saga::new()
            .step("create", |_| Effect::pure(42), |_, _| Effect::pure(()))
            .step_with_result(
                "use",
                |_, results| {
                    let prev: &i32 = results.get("create").unwrap();
                    Effect::pure(*prev * 2)
                },
                |_, _, _| Effect::pure(()),
            );

        let result = saga.run((), &()).await.unwrap();
        assert!(matches!(result, SagaResult::Completed((42, 84))));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_checkout_saga_happy_path() {
    let env = TestEnv::new();

    let saga = Saga::new()
        .step(
            "reserve",
            |ctx: &Order| reserve_inventory(&ctx.items),
            |_, res| unreserve(&res),
        )
        .step(
            "charge",
            |ctx| charge_card(&ctx.payment),
            |_, charge| refund(&charge),
        )
        .step(
            "ship",
            |ctx| create_shipment(&ctx.address),
            |_, ship| cancel_shipment(&ship),
        );

    let order = Order::test_order();
    let result = saga.run(order, &env).await.unwrap();

    assert!(matches!(result, SagaResult::Completed(_)));
}

#[tokio::test]
async fn test_checkout_saga_payment_failure_compensates() {
    let env = TestEnv::new()
        .with_payment_failure();  // Simulate payment failure

    let saga = checkout_saga();
    let order = Order::test_order();

    let result = saga.run(order, &env).await.unwrap();

    match result {
        SagaResult::Compensated { failed_step, .. } => {
            assert_eq!(failed_step, "charge");
            // Verify inventory was unreserved
            assert!(env.inventory_is_unreserved());
        }
        _ => panic!("Expected compensation"),
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn compensation_count_matches_completed_steps(
        fail_at: usize,
        total_steps in 1usize..10
    ) {
        let fail_at = fail_at % (total_steps + 1);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let compensated = Arc::new(AtomicUsize::new(0));

        let mut saga = Saga::new();
        for i in 0..total_steps {
            let comp = compensated.clone();
            if i == fail_at {
                saga = saga.step(
                    Box::leak(format!("step{}", i).into_boxed_str()),
                    move |_| Effect::<i32, String, ()>::fail("fail".into()),
                    move |_, _| Effect::pure(()),
                );
            } else {
                saga = saga.step(
                    Box::leak(format!("step{}", i).into_boxed_str()),
                    move |_| Effect::pure(i as i32),
                    move |_, _| {
                        comp.fetch_add(1, Ordering::SeqCst);
                        Effect::pure(())
                    },
                );
            }
        }

        rt.block_on(async {
            let _ = saga.run((), &()).await;
        });

        // Number of compensations should equal steps that completed before failure
        prop_assert_eq!(compensated.load(Ordering::SeqCst), fail_at);
    }
}
```

## Documentation Requirements

### Code Documentation
- Comprehensive rustdoc for Saga builder
- Examples for each step type
- Common patterns documented

### User Documentation
- Add "Saga Pattern" chapter to user guide
- Document compensation strategies
- Provide real-world examples (checkout, booking, etc.)

### Architecture Updates
- Document saga module in DESIGN.md
- Explain design decisions

## Implementation Notes

### Type Accumulation Challenge

The saga builder needs to track the types of all step results. This is challenging in Rust without variadic generics. Options:

**Option A: Type-level list (HList)**
```rust
Saga<Ctx, HNil, E, Env>
    .step(...) -> Saga<Ctx, HCons<T1, HNil>, E, Env>
    .step(...) -> Saga<Ctx, HCons<T2, HCons<T1, HNil>>, E, Env>
```
Complex, but provides full type safety.

**Option B: Tuple accumulation (up to reasonable limit)**
```rust
Saga<Ctx, (), E, Env>
    .step(...) -> Saga<Ctx, (T1,), E, Env>
    .step(...) -> Saga<Ctx, (T1, T2), E, Env>
    // Macro generates up to (T1, T2, ..., T12)
```
Simpler, works for most cases.

**Option C: Type-erased results**
```rust
Saga<Ctx, E, Env>  // No type parameter for results
    .run(...) -> SagaResult<StepResults, E>  // HashMap<&str, Box<dyn Any>>
```
Simplest, but loses type safety.

**Recommendation**: Start with Option C for MVP, consider Option B for typed results in future.

### Parallel Step Complexity

Parallel steps need careful handling:
- All start simultaneously
- If any fails, cancel others (structured concurrency)
- Run compensations for completed parallel steps
- Then run compensations for prior sequential steps

Consider using `tokio::select!` or `futures::select_all`.

### Serialization (Future)

For persistent sagas (survives restarts), we'd need:
- Serializable step state
- Checkpoint storage
- Resume from checkpoint

This is out of scope for initial implementation.

## Migration and Compatibility

### Breaking Changes
None - this is a purely additive feature.

### Feature Flags
```toml
[features]
default = ["saga"]
saga = []  # Saga pattern support
```

### Deprecations
None.

## Future Considerations (Out of Scope)

1. **Persistent Sagas**: Survive process restarts
2. **Distributed Sagas**: Across multiple services
3. **Saga Versioning**: Handle schema changes
4. **Visual Saga Editor**: GUI for designing sagas
5. **Saga Analytics**: Execution statistics and bottleneck detection

---

*"When step 3 fails, steps 2 and 1 must be undone. Automatically. Every time. Without exception."*
