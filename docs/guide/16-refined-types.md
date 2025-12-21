# Refined Types: Parse, Don't Validate

## The Problem

Validation checks are often scattered throughout codebases:

```rust
fn process_user(name: String, age: i32) -> Result<User, Error> {
    if name.is_empty() {
        return Err(Error::EmptyName);
    }
    if age <= 0 {
        return Err(Error::InvalidAge);
    }

    // What if we call another function?
    // Does it also need to check name and age?
    save_user(&name, age)?;

    Ok(User { name, age })
}

fn save_user(name: &str, age: i32) -> Result<(), Error> {
    // Do we need to validate again? Maybe...
    // The type system doesn't tell us if name is valid
    db.insert(name, age)
}
```

This leads to:
- Redundant validation checks
- Uncertainty about whether data is valid
- Runtime errors when validation is forgotten
- No compiler help to catch missing checks

## The Solution: Refined Types

Refined types encode invariants in the type system. Once validated at the boundary, the type *guarantees* validity:

```rust
use stillwater::refined::{Refined, NonEmpty, Positive};

type NonEmptyString = Refined<String, NonEmpty>;
type PositiveI32 = Refined<i32, Positive>;

fn process_user(name: NonEmptyString, age: PositiveI32) -> User {
    // name is GUARANTEED non-empty by construction
    // age is GUARANTEED positive by construction
    // No runtime checks needed!
    save_user(&name, age)
}

fn save_user(name: &NonEmptyString, age: &PositiveI32) {
    // Types guarantee validity - impossible to have invalid data here
    db.insert(name.get(), *age.get())
}
```

## Core API

### Creating Refined Values

```rust
use stillwater::refined::{Refined, NonEmpty, Positive};

type NonEmptyString = Refined<String, NonEmpty>;
type PositiveI32 = Refined<i32, Positive>;

// Validate at the boundary
let name = NonEmptyString::new("Alice".to_string());
assert!(name.is_ok());

let empty = NonEmptyString::new("".to_string());
assert!(empty.is_err());

// Access the inner value (zero-cost)
let name = NonEmptyString::new("Alice".to_string()).unwrap();
println!("Name: {}", name.get());      // Reference
println!("Length: {}", name.len());    // Deref allows direct access

// Consume the wrapper
let inner: String = name.into_inner();
```

### Transforming Refined Values

```rust
use stillwater::refined::{Refined, Positive};

type PositiveI32 = Refined<i32, Positive>;

let n = PositiveI32::new(42).unwrap();

// try_map re-checks the predicate after transformation
let doubled = n.try_map(|x| x * 2);
assert!(doubled.is_ok());

let negated = PositiveI32::new(5).unwrap().try_map(|x| -x);
assert!(negated.is_err()); // -5 is not positive
```

### Unsafe Construction (Use with Care)

```rust
use stillwater::refined::{Refined, Positive};

type PositiveI32 = Refined<i32, Positive>;

// Only use when you KNOW the value is valid
// No predicate check is performed!
let n = PositiveI32::new_unchecked(42);
```

## Built-in Predicates

### Numeric Predicates

```rust
use stillwater::refined::{Refined, Positive, NonNegative, Negative, NonZero, InRange};

// Positive: value > 0
type PositiveI32 = Refined<i32, Positive>;
assert!(PositiveI32::new(1).is_ok());
assert!(PositiveI32::new(0).is_err());

// NonNegative: value >= 0
type NonNegativeI32 = Refined<i32, NonNegative>;
assert!(NonNegativeI32::new(0).is_ok());
assert!(NonNegativeI32::new(-1).is_err());

// Negative: value < 0
type NegativeI32 = Refined<i32, Negative>;
assert!(NegativeI32::new(-1).is_ok());
assert!(NegativeI32::new(0).is_err());

// NonZero: value != 0
type NonZeroI32 = Refined<i32, NonZero>;
assert!(NonZeroI32::new(1).is_ok());
assert!(NonZeroI32::new(-1).is_ok());
assert!(NonZeroI32::new(0).is_err());

// InRange: MIN <= value <= MAX
type Percentage = Refined<i32, InRange<0, 100>>;
assert!(Percentage::new(50).is_ok());
assert!(Percentage::new(101).is_err());
```

### String Predicates

```rust
use stillwater::refined::{Refined, NonEmpty, Trimmed, MaxLength, MinLength};

// NonEmpty: string is not empty
type NonEmptyString = Refined<String, NonEmpty>;
assert!(NonEmptyString::new("hello".to_string()).is_ok());
assert!(NonEmptyString::new("".to_string()).is_err());

// Trimmed: no leading/trailing whitespace
type TrimmedString = Refined<String, Trimmed>;
assert!(TrimmedString::new("hello".to_string()).is_ok());
assert!(TrimmedString::new("  hello  ".to_string()).is_err());

// MaxLength<N>: length <= N
type ShortString = Refined<String, MaxLength<10>>;
assert!(ShortString::new("hello".to_string()).is_ok());
assert!(ShortString::new("this is too long".to_string()).is_err());

// MinLength<N>: length >= N
type Password = Refined<String, MinLength<8>>;
assert!(Password::new("secure_password".to_string()).is_ok());
assert!(Password::new("short".to_string()).is_err());
```

### Collection Predicates

```rust
use stillwater::refined::{Refined, NonEmpty, MaxSize, MinSize};

// NonEmpty for Vec
type NonEmptyList = Refined<Vec<i32>, NonEmpty>;
assert!(NonEmptyList::new(vec![1, 2, 3]).is_ok());
assert!(NonEmptyList::new(vec![]).is_err());

// MaxSize<N>: size <= N
type SmallVec = Refined<Vec<i32>, MaxSize<5>>;
assert!(SmallVec::new(vec![1, 2, 3]).is_ok());
assert!(SmallVec::new(vec![1, 2, 3, 4, 5, 6]).is_err());

// MinSize<N>: size >= N
type AtLeastTwo = Refined<Vec<i32>, MinSize<2>>;
assert!(AtLeastTwo::new(vec![1, 2]).is_ok());
assert!(AtLeastTwo::new(vec![1]).is_err());
```

## Predicate Combinators

### And: Both Must Hold

```rust
use stillwater::refined::{Refined, And, NonEmpty, Trimmed, MaxLength};

// String must be non-empty AND trimmed
type CleanString = Refined<String, And<NonEmpty, Trimmed>>;
assert!(CleanString::new("hello".to_string()).is_ok());
assert!(CleanString::new("".to_string()).is_err());
assert!(CleanString::new("  hello  ".to_string()).is_err());

// Chain multiple with nested And
type ValidUsername = Refined<String, And<And<NonEmpty, Trimmed>, MaxLength<20>>>;
```

### Or: At Least One Must Hold

```rust
use stillwater::refined::{Refined, Or, Positive, Negative};

// Value must be positive OR negative (i.e., non-zero)
type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;
assert!(NonZeroAlt::new(5).is_ok());
assert!(NonZeroAlt::new(-5).is_ok());
assert!(NonZeroAlt::new(0).is_err());
```

### Not: Must NOT Hold

```rust
use stillwater::refined::{Refined, Not, Positive};

// Value must NOT be positive (i.e., <= 0)
type NotPositive = Refined<i32, Not<Positive>>;
assert!(NotPositive::new(0).is_ok());
assert!(NotPositive::new(-5).is_ok());
assert!(NotPositive::new(5).is_err());
```

## Custom Predicates

Define your own predicates by implementing the `Predicate` trait:

```rust
use stillwater::refined::{Refined, Predicate};

// Custom predicate for even numbers
pub struct Even;

impl Predicate<i32> for Even {
    type Error = &'static str;

    fn check(value: &i32) -> Result<(), Self::Error> {
        if value % 2 == 0 {
            Ok(())
        } else {
            Err("value must be even")
        }
    }

    fn description() -> &'static str {
        "even number"
    }
}

type EvenI32 = Refined<i32, Even>;
assert!(EvenI32::new(42).is_ok());
assert!(EvenI32::new(41).is_err());
```

## Type Aliases

Stillwater provides convenient aliases for common patterns:

```rust
use stillwater::refined::{
    NonEmptyString, TrimmedString, NonEmptyTrimmedString,
    PositiveI32, NonNegativeI64, NonZeroU32,
    Port, Percentage,
    BoundedString, BoundedVec,
};

// String aliases
let name = NonEmptyString::new("Alice".to_string()).unwrap();
let clean = NonEmptyTrimmedString::new("hello".to_string()).unwrap();

// Numeric aliases
let age = PositiveI32::new(25).unwrap();
let count = NonZeroU32::new(100).unwrap();

// Domain-specific aliases
let port = Port::new(443).unwrap();           // 1-65535
let progress = Percentage::new(75).unwrap();  // 0-100

// Bounded types with const generics
type Username = BoundedString<20>;
type SmallList = BoundedVec<i32, 10>;
```

## Validation Integration

### Single Validation

```rust
use stillwater::refined::{Refined, Positive};
use stillwater::Validation;

type PositiveI32 = Refined<i32, Positive>;

// Returns Validation<PositiveI32, &'static str>
let result = PositiveI32::validate(42);
assert!(result.is_success());

let result = PositiveI32::validate(-5);
assert!(result.is_failure());
```

### Error Accumulation

```rust
use stillwater::refined::{NonEmptyString, PositiveI32};
use stillwater::Validation;

fn validate_user(
    name: String,
    age: i32,
) -> Validation<(NonEmptyString, PositiveI32), Vec<&'static str>> {
    let v1 = NonEmptyString::validate_vec(name);
    let v2 = PositiveI32::validate_vec(age);
    v1.and(v2)
}

// All errors collected
let result = validate_user("".to_string(), -5);
match result {
    Validation::Failure(errors) => {
        assert_eq!(errors.len(), 2); // Both errors collected
    }
    _ => panic!(),
}
```

### Field Context

```rust
use stillwater::refined::{NonEmptyString, FieldError, ValidationFieldExt};
use stillwater::Validation;

// Add field context to errors
let result = NonEmptyString::validate("".to_string())
    .with_field("username");

match result {
    Validation::Failure(err) => {
        assert_eq!(err.field, "username");
        println!("{}", err); // "username: string cannot be empty"
    }
    _ => panic!(),
}
```

## Effect Integration

Use refined types in effect chains:

```rust
use stillwater::effect::prelude::*;
use stillwater::refined::{refine, NonEmpty, Refined};

type NonEmptyString = Refined<String, NonEmpty>;

// Validate in effect chains
let effect = pure::<_, &str, ()>("hello".to_string())
    .and_then(|s| refine::<_, NonEmpty, ()>(s))
    .map(|refined| refined.get().len());

let result = effect.run(&()).await;
assert_eq!(result, Ok(5));
```

## Real-World Example

```rust
use stillwater::refined::{
    Refined, And, NonEmpty, Trimmed, MaxLength, MinLength, Port,
};
use stillwater::Validation;

// Domain types with encoded invariants
type Username = Refined<String, And<And<NonEmpty, Trimmed>, MaxLength<30>>>;
type Password = Refined<String, And<MinLength<8>, MaxLength<128>>>;
type PositiveI32 = Refined<i32, Positive>;

// User with guaranteed-valid fields
struct User {
    username: Username,
    password: Password,
    age: PositiveI32,
    port: Option<Port>,
}

fn validate_registration(
    username: String,
    password: String,
    age: i32,
    port: Option<u16>,
) -> Validation<User, Vec<String>> {
    let v_username = Username::validate(username)
        .map_err(|e| vec![format!("username: {}", e)]);
    let v_password = Password::validate(password)
        .map_err(|e| vec![format!("password: {}", e)]);
    let v_age = PositiveI32::validate(age)
        .map_err(|e| vec![format!("age: {}", e)]);
    let v_port = match port {
        Some(p) => Port::validate(p)
            .map(Some)
            .map_err(|e| vec![format!("port: {}", e)]),
        None => Validation::Success(None),
    };

    v_username
        .and(v_password)
        .and(v_age)
        .and(v_port)
        .map(|(((username, password), age), port)| User {
            username,
            password,
            age,
            port,
        })
}

// Functions that work with validated data need no checks
fn process_user(user: User) {
    // Types guarantee validity - no runtime checks needed!
    println!("Processing user: {}", user.username.get());
}
```

## Zero-Cost Abstraction

`Refined<T, P>` has the same memory layout as `T`. The predicate `P` exists only at compile time via `PhantomData`:

```rust
use std::mem::size_of;
use stillwater::refined::{Refined, Positive};

type PositiveI32 = Refined<i32, Positive>;

// Same size as the wrapped type
assert_eq!(size_of::<PositiveI32>(), size_of::<i32>());
```

## Best Practices

1. **Validate at the boundary**: Parse input into refined types as early as possible
2. **Use type aliases**: Create domain-specific aliases for readability
3. **Combine predicates**: Use `And`, `Or`, `Not` for complex constraints
4. **Accumulate errors**: Use `validate_vec()` with `Validation::and()` for all errors
5. **Add field context**: Use `with_field()` for user-friendly error messages

## Next Steps

- See [examples/refined.rs](../../examples/refined.rs) for more examples
- See the [Validation](02-validation.md) guide for error accumulation patterns
- See the [Effects](03-effects.md) guide for effect integration
