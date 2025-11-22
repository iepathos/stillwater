//! Testing patterns example - demonstrating testability benefits
//!
//! This shows how stillwater's separation of pure/effectful makes testing easier.

use stillwater::{Effect, Validation, IO};

// ============================================================================
// Domain
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: ProductId,
    name: String,
    price: Money,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ProductId(u64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Money(f64);

#[derive(Debug, Clone, PartialEq)]
struct Cart {
    items: Vec<(Product, u32)>, // (product, quantity)
}

#[derive(Debug, Clone, PartialEq)]
struct Order {
    cart: Cart,
    subtotal: Money,
    discount: Money,
    tax: Money,
    total: Money,
}

#[derive(Debug, Clone, Copy)]
enum CustomerTier {
    Regular,
    Premium,
    VIP,
}

#[derive(Debug, Clone)]
struct Customer {
    id: CustomerId,
    tier: CustomerTier,
    loyalty_points: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CustomerId(u64);

// ============================================================================
// Pure Business Logic (Easy to Test - No Mocks!)
// ============================================================================

fn calculate_subtotal(cart: &Cart) -> Money {
    let total = cart
        .items
        .iter()
        .map(|(product, qty)| product.price.0 * (*qty as f64))
        .sum();
    Money(total)
}

fn calculate_discount(tier: CustomerTier, subtotal: Money, loyalty_points: u32) -> Money {
    let tier_discount = match tier {
        CustomerTier::Regular => 0.0,
        CustomerTier::Premium => 0.10,
        CustomerTier::VIP => 0.20,
    };

    let loyalty_discount = (loyalty_points as f64 / 100.0) * 0.01;
    let total_discount = (tier_discount + loyalty_discount).min(0.30); // Cap at 30%

    Money(subtotal.0 * total_discount)
}

fn calculate_tax(amount: Money) -> Money {
    Money(amount.0 * 0.08) // 8% tax
}

fn create_order(cart: Cart, customer: &Customer) -> Order {
    let subtotal = calculate_subtotal(&cart);
    let discount = calculate_discount(customer.tier, subtotal, customer.loyalty_points);
    let taxable_amount = Money(subtotal.0 - discount.0);
    let tax = calculate_tax(taxable_amount);
    let total = Money(taxable_amount.0 + tax.0);

    Order {
        cart,
        subtotal,
        discount,
        tax,
        total,
    }
}

fn validate_cart(cart: &Cart) -> Validation<(), Vec<String>> {
    if cart.items.is_empty() {
        Validation::failure(vec!["Cart cannot be empty".to_string()])
    } else {
        Validation::success(())
    }
}

// ============================================================================
// Effects (Needs Environment)
// ============================================================================

#[derive(Debug)]
enum CheckoutError {
    InvalidCart(Vec<String>),
    CustomerNotFound(CustomerId),
    InsufficientInventory(ProductId),
    PaymentFailed(String),
}

impl From<Vec<String>> for CheckoutError {
    fn from(errors: Vec<String>) -> Self {
        CheckoutError::InvalidCart(errors)
    }
}

struct ShopEnv {
    customer_repo: CustomerRepository,
    inventory: Inventory,
    payment_gateway: PaymentGateway,
}

trait CustomerRepository {
    fn find_customer(&self, id: CustomerId) -> Result<Customer, CheckoutError>;
}

trait Inventory {
    fn check_availability(&self, product_id: ProductId, qty: u32) -> Result<(), CheckoutError>;
    fn reserve_items(&mut self, items: &[(Product, u32)]) -> Result<(), CheckoutError>;
}

trait PaymentGateway {
    fn charge(&self, customer_id: CustomerId, amount: Money) -> Result<String, CheckoutError>;
}

fn checkout_effect(
    cart: Cart,
    customer_id: CustomerId,
) -> Effect<Order, CheckoutError, ShopEnv> {
    // Validate cart
    Effect::from_validation(validate_cart(&cart))
        .map(|_| cart.clone())

        // Fetch customer
        .and_then(move |cart| {
            IO::query(move |customer_repo: &dyn CustomerRepository| {
                customer_repo.find_customer(customer_id)
            })
            .map(move |customer| (cart, customer))
        })

        // Create order (pure!)
        .map(|(cart, customer)| create_order(cart, &customer))

        // Check inventory
        .and_then(|order| {
            let items = order.cart.items.clone();
            IO::query(move |inventory: &dyn Inventory| {
                for (product, qty) in &items {
                    inventory.check_availability(product.id, *qty)?;
                }
                Ok(())
            })
            .map(move |_| order)
        })

        // Process payment
        .and_then(move |order| {
            IO::execute(move |payment: &dyn PaymentGateway| {
                payment.charge(customer_id, order.total)
            })
            .map(move |_| order)
        })

        // Reserve inventory
        .and_then(|order| {
            let items = order.cart.items.clone();
            IO::execute(move |inventory: &mut dyn Inventory| {
                inventory.reserve_items(&items)
            })
            .map(move |_| order)
        })
}

// ============================================================================
// Tests - Pure Functions (No Mocking!)
// ============================================================================

#[cfg(test)]
mod pure_tests {
    use super::*;

    #[test]
    fn test_calculate_subtotal_empty_cart() {
        let cart = Cart { items: vec![] };
        let subtotal = calculate_subtotal(&cart);
        assert_eq!(subtotal, Money(0.0));
    }

    #[test]
    fn test_calculate_subtotal_single_item() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(10.0),
                },
                2,
            )],
        };
        let subtotal = calculate_subtotal(&cart);
        assert_eq!(subtotal, Money(20.0));
    }

    #[test]
    fn test_calculate_subtotal_multiple_items() {
        let cart = Cart {
            items: vec![
                (
                    Product {
                        id: ProductId(1),
                        name: "Widget".to_string(),
                        price: Money(10.0),
                    },
                    2,
                ),
                (
                    Product {
                        id: ProductId(2),
                        name: "Gadget".to_string(),
                        price: Money(15.0),
                    },
                    3,
                ),
            ],
        };
        let subtotal = calculate_subtotal(&cart);
        assert_eq!(subtotal, Money(65.0));
    }

    #[test]
    fn test_regular_customer_no_discount() {
        let discount = calculate_discount(CustomerTier::Regular, Money(100.0), 0);
        assert_eq!(discount, Money(0.0));
    }

    #[test]
    fn test_premium_customer_10_percent() {
        let discount = calculate_discount(CustomerTier::Premium, Money(100.0), 0);
        assert_eq!(discount, Money(10.0));
    }

    #[test]
    fn test_vip_customer_20_percent() {
        let discount = calculate_discount(CustomerTier::VIP, Money(100.0), 0);
        assert_eq!(discount, Money(20.0));
    }

    #[test]
    fn test_loyalty_points_add_to_discount() {
        // 500 points = 5% additional discount
        let discount = calculate_discount(CustomerTier::Premium, Money(100.0), 500);
        assert_eq!(discount, Money(15.0)); // 10% + 5%
    }

    #[test]
    fn test_discount_capped_at_30_percent() {
        // VIP (20%) + 2000 points (20%) should cap at 30%
        let discount = calculate_discount(CustomerTier::VIP, Money(100.0), 2000);
        assert_eq!(discount, Money(30.0)); // Capped
    }

    #[test]
    fn test_calculate_tax() {
        let tax = calculate_tax(Money(100.0));
        assert_eq!(tax, Money(8.0));
    }

    #[test]
    fn test_create_order_integrates_calculations() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(100.0),
                },
                1,
            )],
        };

        let customer = Customer {
            id: CustomerId(1),
            tier: CustomerTier::Premium,
            loyalty_points: 0,
        };

        let order = create_order(cart, &customer);

        assert_eq!(order.subtotal, Money(100.0));
        assert_eq!(order.discount, Money(10.0)); // 10% premium
        assert_eq!(order.tax, Money(7.2));       // 8% of (100 - 10)
        assert_eq!(order.total, Money(97.2));    // 90 + 7.2
    }

    #[test]
    fn test_validate_cart_empty_fails() {
        let cart = Cart { items: vec![] };
        match validate_cart(&cart) {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0], "Cart cannot be empty");
            }
            _ => panic!("Expected validation failure"),
        }
    }

    #[test]
    fn test_validate_cart_with_items_succeeds() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(10.0),
                },
                1,
            )],
        };
        assert!(matches!(validate_cart(&cart), Validation::Success(_)));
    }
}

// ============================================================================
// Tests - Effects (With Mock Environment)
// ============================================================================

#[cfg(test)]
mod effect_tests {
    use super::*;

    // Mock implementations
    struct MockCustomerRepo {
        customers: Vec<Customer>,
    }

    impl CustomerRepository for MockCustomerRepo {
        fn find_customer(&self, id: CustomerId) -> Result<Customer, CheckoutError> {
            self.customers
                .iter()
                .find(|c| c.id == id)
                .cloned()
                .ok_or(CheckoutError::CustomerNotFound(id))
        }
    }

    struct MockInventory {
        available: bool,
    }

    impl Inventory for MockInventory {
        fn check_availability(&self, _product_id: ProductId, _qty: u32) -> Result<(), CheckoutError> {
            if self.available {
                Ok(())
            } else {
                Err(CheckoutError::InsufficientInventory(ProductId(1)))
            }
        }

        fn reserve_items(&mut self, _items: &[(Product, u32)]) -> Result<(), CheckoutError> {
            Ok(())
        }
    }

    struct MockPaymentGateway {
        should_fail: bool,
    }

    impl PaymentGateway for MockPaymentGateway {
        fn charge(&self, _customer_id: CustomerId, _amount: Money) -> Result<String, CheckoutError> {
            if self.should_fail {
                Err(CheckoutError::PaymentFailed("Card declined".to_string()))
            } else {
                Ok("txn_12345".to_string())
            }
        }
    }

    #[test]
    fn test_checkout_success() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(100.0),
                },
                1,
            )],
        };

        let env = ShopEnv {
            customer_repo: MockCustomerRepo {
                customers: vec![Customer {
                    id: CustomerId(1),
                    tier: CustomerTier::Premium,
                    loyalty_points: 0,
                }],
            },
            inventory: MockInventory { available: true },
            payment_gateway: MockPaymentGateway { should_fail: false },
        };

        let effect = checkout_effect(cart, CustomerId(1));
        let result = effect.run(&env);

        assert!(result.is_ok());
        let order = result.unwrap();
        assert_eq!(order.total, Money(97.2));
    }

    #[test]
    fn test_checkout_customer_not_found() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(10.0),
                },
                1,
            )],
        };

        let env = ShopEnv {
            customer_repo: MockCustomerRepo { customers: vec![] },
            inventory: MockInventory { available: true },
            payment_gateway: MockPaymentGateway { should_fail: false },
        };

        let effect = checkout_effect(cart, CustomerId(999));
        let result = effect.run(&env);

        assert!(matches!(result, Err(CheckoutError::CustomerNotFound(_))));
    }

    #[test]
    fn test_checkout_insufficient_inventory() {
        let cart = Cart {
            items: vec![(
                Product {
                    id: ProductId(1),
                    name: "Widget".to_string(),
                    price: Money(10.0),
                },
                1,
            )],
        };

        let env = ShopEnv {
            customer_repo: MockCustomerRepo {
                customers: vec![Customer {
                    id: CustomerId(1),
                    tier: CustomerTier::Regular,
                    loyalty_points: 0,
                }],
            },
            inventory: MockInventory { available: false },
            payment_gateway: MockPaymentGateway { should_fail: false },
        };

        let effect = checkout_effect(cart, CustomerId(1));
        let result = effect.run(&env);

        assert!(matches!(result, Err(CheckoutError::InsufficientInventory(_))));
    }

    // Note: How much easier is this compared to mocking everything
    // in traditional imperative code?
}

fn main() {
    println!("=== Testing Patterns Example ===\n");
    println!("This example demonstrates testability benefits.");
    println!("Run `cargo test` to see the tests in action.\n");
    println!("Key observations:");
    println!("  1. Pure functions need ZERO mocking");
    println!("  2. Business logic is 100% testable");
    println!("  3. Effects use simple mock environments");
    println!("  4. Clear separation makes testing obvious");
}
