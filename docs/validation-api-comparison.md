# Validation::all() - Tuples vs Alternatives

## Current Design (Tuples)

```rust
Validation::all((
    validate_email(&input.email),
    validate_password(&input.password),
    validate_age(input.age),
))
.map(|(email, password, age)| User { email, password, age })
```

## Trade-off Analysis

### Option 1: Tuples (Current Design)

**How it works:**
```rust
impl<T1, T2, E: Semigroup> Validation<(T1, T2), E> {
    fn all(validations: (Validation<T1, E>, Validation<T2, E>)) -> Validation<(T1, T2), E>
}

// Implement for (T1, T2), (T1, T2, T3), (T1, T2, T3, T4), etc.
```

**Pros:**
- ✅ Native Rust, no dependencies
- ✅ Type inference works perfectly
- ✅ Pattern matching is clean: `|(email, age, name)|`
- ✅ Each validation can have different type
- ✅ Compiler catches arity mismatches
- ✅ Zero runtime overhead

**Cons:**
- ❌ Limited to tuple size (typically 12-16)
- ❌ Need to implement for each tuple size (macro helps)
- ❌ Larger tuples get unwieldy: `(a, b, c, d, e, f, g, h, i, j, k, l)`

**Real-world impact:**
- Forms rarely have >12 fields that validate independently
- If you do, you're probably doing something wrong (split the form)
- For rare cases, can nest: `Validation::all((group1, group2))`

**Example of edge case:**
```rust
// 15 fields? Probably indicates poor UX
let personal = Validation::all((name, email, phone, address));
let payment = Validation::all((card, cvv, expiry, billing));
let shipping = Validation::all((addr, method, preference));

Validation::all((personal, payment, shipping))
    .map(|((p1, p2, p3, p4), (pay1, pay2, pay3, pay4), (s1, s2, s3))| {
        // Awkward, but you shouldn't be here anyway
    })
```

---

### Option 2: Vec/Slice (Homogeneous)

```rust
fn all_same<T, E>(validations: Vec<Validation<T, E>>) -> Validation<Vec<T>, E>
```

**Pros:**
- ✅ No size limit
- ✅ Works with dynamic number of validations
- ✅ Simple implementation

**Cons:**
- ❌ All validations must return SAME type
- ❌ Loses type information (can't distinguish email from name)
- ❌ Can't build heterogeneous structs easily

**When it's useful:**
```rust
// Validating a list of records (all same type)
let validated_records = Validation::all_vec(
    records.into_iter().map(validate_record).collect()
);

// Good for: bulk data validation
// Bad for: form field validation
```

**Verdict:** Useful as **separate method**, not replacement for tuple version.

---

### Option 3: HList (Like Frunk)

```rust
// Heterogeneous list (compile-time linked list)
Validation::all(HCons(
    validate_email(input),
    HCons(
        validate_password(input),
        HCons(
            validate_age(input),
            HNil
        )
    )
))
```

**Pros:**
- ✅ No size limit
- ✅ Each element can be different type
- ✅ Type-safe composition

**Cons:**
- ❌ Horrible syntax
- ❌ Requires complex type-level programming
- ❌ Hard for users to understand
- ❌ Error messages are cryptic
- ❌ Defeats our "simplicity" goal

**Example error:**
```
error[E0271]: type mismatch resolving `<HCons<Validation<Email, Vec<Error>>,
  HCons<Validation<Password, Vec<Error>>, HCons<Validation<Age, Vec<Error>>,
  HNil>>> as ValidateAll>::Output == Validation<HCons<Email, HCons<Password,
  HCons<Age, HNil>>>, Vec<Error>>`
```

**Verdict:** Too complex. Against our philosophy of "Rust-first, not Haskell-in-Rust."

---

### Option 4: Macro

```rust
validate_all![
    email: validate_email(&input.email),
    password: validate_password(&input.password),
    age: validate_age(input.age),
]
// Returns: Validation<NamedFields, E>
//   where you can access .email, .password, .age
```

**Pros:**
- ✅ Clean syntax
- ✅ No size limit
- ✅ Named fields (self-documenting)
- ✅ Could generate struct automatically

**Cons:**
- ❌ Macro complexity
- ❌ Magic / non-obvious
- ❌ Debugging is harder
- ❌ We said we want to avoid heavy macros
- ❌ Generated types have weird names

**Verdict:** Nice ergonomics, but against our "no magic" principle.

---

### Option 5: Builder Pattern

```rust
Validation::builder()
    .add(validate_email(&input.email))
    .add(validate_password(&input.password))
    .add(validate_age(input.age))
    .build()
    .map(|(email, password, age)| User { email, password, age })
```

**Pros:**
- ✅ No size limit
- ✅ Fluent API

**Cons:**
- ❌ More verbose than tuples
- ❌ Still limited by tuple size at the end
- ❌ Doesn't solve the actual problem
- ❌ More API surface area
- ❌ Less clear than direct tuple

**Verdict:** Adds complexity without solving the core issue.

---

## Real-World Data

Let's look at actual forms in popular apps:

**Simple forms (90% of cases):**
- Login: 2 fields (email, password)
- Signup: 3-5 fields (email, password, name, age, terms)
- Contact: 4-5 fields (name, email, subject, message, consent)
- Payment: 6-8 fields (card, cvv, expiry, name, address, zip, country)

**Complex forms (9% of cases):**
- User profile: 10-12 fields
- Shipping info: 8-10 fields
- Advanced settings: 12-15 fields

**Insanely complex (1% of cases):**
- Tax forms: 50+ fields
- Medical intake: 30+ fields
- Government applications: 100+ fields

**For the 1% edge case:**
- You should probably split into multiple steps/pages anyway (UX best practice)
- Or validate in groups (more meaningful error grouping)
- Tuple limit isn't the real problem

---

## Hybrid Approach

**Recommendation:** Support BOTH

```rust
// 1. Tuple version (for most cases)
impl Validation<T, E> {
    fn all<Tuple>(validations: Tuple) -> Validation<TupleOutput, E>
    where
        Tuple: ValidateAll<E>,  // Implemented for tuples 1-12
    {
        // ...
    }
}

// 2. Vec version (for homogeneous collections)
impl Validation<T, E> {
    fn all_vec(validations: Vec<Validation<T, E>>) -> Validation<Vec<T>, E> {
        // ...
    }
}

// 3. Iterator version (for lazy evaluation)
impl Validation<T, E> {
    fn all_iter<I>(validations: I) -> Validation<Vec<T>, E>
    where
        I: IntoIterator<Item = Validation<T, E>>,
    {
        // ...
    }
}
```

**Usage examples:**

```rust
// Case 1: Form validation (different types)
Validation::all((
    validate_email(input),
    validate_password(input),
    validate_age(input),
))  // Returns: Validation<(Email, Password, Age), E>

// Case 2: Bulk validation (same type)
let records: Vec<RawRecord> = ...;
Validation::all_vec(
    records.into_iter().map(validate_record).collect()
)  // Returns: Validation<Vec<ValidRecord>, E>

// Case 3: Lazy validation (iterator)
Validation::all_iter(
    csv_lines.iter().map(|line| validate_line(line))
)  // Returns: Validation<Vec<ValidLine>, E>
```

---

## Comparison Matrix

| Approach | Syntax | Type Safety | Size Limit | Complexity | Verdict |
|----------|--------|-------------|------------|------------|---------|
| **Tuples** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ (12-16) | ⭐⭐⭐⭐⭐ | ✅ **USE** |
| **Vec** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ **ADD** |
| **HList** | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ❌ SKIP |
| **Macro** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ❌ SKIP |
| **Builder** | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ❌ SKIP |

---

## Recommended Implementation

```rust
// Core trait for tuple validation
pub trait ValidateAll<E: Semigroup> {
    type Output;
    fn validate_all(self) -> Validation<Self::Output, E>;
}

// Implement for tuples 1-12 (via macro)
macro_rules! impl_validate_all {
    ($($T:ident),+) => {
        impl<E: Semigroup, $($T),+> ValidateAll<E> for ($(Validation<$T, E>),+) {
            type Output = ($($T),+);

            fn validate_all(self) -> Validation<Self::Output, E> {
                // Implementation that accumulates errors
            }
        }
    }
}

impl_validate_all!(T1);
impl_validate_all!(T1, T2);
impl_validate_all!(T1, T2, T3);
// ... up to 12

// Public API
impl<T, E: Semigroup> Validation<T, E> {
    pub fn all<V: ValidateAll<E>>(validations: V) -> Validation<V::Output, E> {
        validations.validate_all()
    }

    pub fn all_vec(validations: Vec<Validation<T, E>>) -> Validation<Vec<T>, E> {
        // Fold over vec, accumulating errors
    }

    pub fn all_iter<I>(iter: I) -> Validation<Vec<T>, E>
    where
        I: IntoIterator<Item = Validation<T, E>>,
    {
        Self::all_vec(iter.into_iter().collect())
    }
}
```

---

## Decision

**YES, tuples are fine!**

**Reasons:**
1. ✅ **Covers 99% of use cases** (forms rarely >12 independent fields)
2. ✅ **Zero magic** (native Rust, obvious behavior)
3. ✅ **Type safe** (different types for different fields)
4. ✅ **Clean syntax** (readable, no boilerplate)
5. ✅ **Easy to implement** (macro generates impls)

**Edge cases handled by:**
- `all_vec()` for homogeneous collections (bulk validation)
- Nesting tuples for rare >12 field cases (split into logical groups anyway)

**Not worth the complexity:**
- ❌ HList (too complex, cryptic errors)
- ❌ Macro (magic, debugging pain)
- ❌ Builder (verbose, doesn't solve size limit)

---

## Real Advantage of Alternatives?

**Short answer: No.**

**Long answer:**

The only "advantage" alternatives offer is **no size limit**, but:

1. **Size limit isn't a real problem in practice**
   - 99% of validations fit in 12 fields
   - The 1% should be split anyway (UX best practice)

2. **Size limit is a feature, not a bug**
   - Forces you to think about grouping
   - Prevents monster validation functions
   - Encourages better UX (multi-step forms)

3. **Alternative costs outweigh benefits**
   - HList: Too complex, scary errors
   - Macro: Magic, non-obvious
   - Vec: Loses type safety

**If you truly have 50 validations:**
```rust
// Good: Logical grouping
let personal = Validation::all((name, email, phone, dob));
let address = Validation::all((street, city, state, zip));
let payment = Validation::all((card, cvv, expiry));

Validation::all((personal, address, payment))
    .map(|(personal, address, payment)| {
        CompleteForm { personal, address, payment }
    })

// This is better UX AND better code!
```

---

## Final Recommendation

**Use tuples for `Validation::all()`**

**Also provide:**
- `all_vec()` for homogeneous collections
- `all_iter()` for iterator compatibility

**Document:**
- Tuple limit (12) and why it's not a problem
- How to group validations for complex forms
- When to use `all_vec()` vs `all()`

**Skip:**
- HList (too complex)
- Macros (too magical)
- Builder (adds no value)

---

*Tuples are not just "fine for now" - they're the right long-term choice.*
