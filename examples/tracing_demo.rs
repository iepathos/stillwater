//! Production-grade tracing patterns for Stillwater Effects
//!
//! Run with: cargo run --example tracing_demo --features "tracing async"
//!
//! This example demonstrates:
//! - Semantic spans with business data (user_id, order_id)
//! - Context for error narratives
//! - Quiet happy path, verbose errors

use stillwater::prelude::*;

#[tokio::main]
async fn main() {
    // Development: human-readable with file/line on errors
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .compact()
        .init();

    println!("=== Order Processing Workflow ===\n");

    // Happy path - minimal output
    process_order("user-123".to_string(), "order-456".to_string()).await;

    println!("\n=== Failing Workflow (step 2) ===\n");

    // Error at fetch_order
    process_order("user-789".to_string(), "order-bad".to_string()).await;

    println!("\n=== Failing Workflow (step 4) ===\n");

    // Error at charge_payment - shows longer context trail
    process_order("user-456".to_string(), "order-broke".to_string()).await;
}

async fn process_order(user_id: String, order_id: String) {
    // Clone for use in error reporting
    let user_id_log = user_id.clone();
    let order_id_log = order_id.clone();

    // Simple flat chain - context marks where each operation failed
    let result = fetch_user(user_id)
        .context("loading user profile")
        .and_then(|user| fetch_order(order_id, &user).context("fetching order details"))
        .and_then(|order| validate_order(order).context("validating order"))
        .and_then(|order| charge_payment(order).context("processing payment"))
        .and_then(|receipt| send_confirmation(receipt).context("sending confirmation"))
        .run(&())
        .await;

    match result {
        Ok(confirmation_id) => {
            tracing::info!(
                user_id = %user_id_log,
                order_id = %order_id_log,
                confirmation_id = %confirmation_id,
                "order completed successfully"
            );
        }
        Err(e) => {
            // Error shows exactly which step failed via context
            tracing::error!(
                user_id = %user_id_log,
                order_id = %order_id_log,
                "failed: {}",
                e
            );
        }
    }
}

// Each function uses semantic spans with relevant business data

fn fetch_user(user_id: String) -> Effect<User, String, ()> {
    let span_user_id = user_id.clone();

    Effect::from_fn(move |_| {
        Ok(User {
            id: user_id.clone(),
            name: "Alice".to_string(),
        })
    })
    .instrument(tracing::debug_span!("fetch_user", user_id = %span_user_id))
}

fn fetch_order(order_id: String, user: &User) -> Effect<Order, String, ()> {
    let user_id = user.id.clone();
    let span_order_id = order_id.clone();
    let span_user_id = user_id.clone();

    Effect::from_fn(move |_| {
        // "order-bad" triggers failure
        if order_id == "order-bad" {
            return Err("order not found in database".to_string());
        }
        Ok(Order {
            id: order_id.clone(),
            user_id: user_id.clone(),
            amount: 99_99,
        })
    })
    .instrument(tracing::debug_span!(
        "fetch_order",
        order_id = %span_order_id,
        user_id = %span_user_id
    ))
}

fn validate_order(order: Order) -> Effect<Order, String, ()> {
    let span_order_id = order.id.clone();
    let span_amount = order.amount;

    Effect::from_fn(move |_| {
        if order.amount == 0 {
            return Err("order amount cannot be zero".to_string());
        }
        Ok(order.clone())
    })
    .instrument(tracing::debug_span!(
        "validate_order",
        order_id = %span_order_id,
        amount_cents = %span_amount
    ))
}

fn charge_payment(order: Order) -> Effect<Receipt, String, ()> {
    let span_order_id = order.id.clone();
    let span_amount = order.amount;

    Effect::from_fn(move |_| {
        // "order-broke" triggers payment failure
        if order.id == "order-broke" {
            return Err("insufficient funds".to_string());
        }
        Ok(Receipt {
            order_id: order.id.clone(),
            transaction_id: "txn-abc123".to_string(),
        })
    })
    .instrument(tracing::info_span!(
        "charge_payment",
        order_id = %span_order_id,
        amount_cents = %span_amount
    ))
}

fn send_confirmation(receipt: Receipt) -> Effect<String, String, ()> {
    let span_order_id = receipt.order_id.clone();

    Effect::from_fn(move |_| {
        tracing::debug!(transaction_id = %receipt.transaction_id, "email sent");
        Ok(format!("conf-{}", receipt.transaction_id))
    })
    .instrument(tracing::debug_span!(
        "send_confirmation",
        order_id = %span_order_id
    ))
}

// Domain types
#[derive(Clone)]
struct User {
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Clone)]
struct Order {
    id: String,
    #[allow(dead_code)]
    user_id: String,
    amount: u32,
}

struct Receipt {
    order_id: String,
    transaction_id: String,
}
