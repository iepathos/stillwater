# Running the Examples

Run Stillwater examples locally with Cargo. Browser Run buttons are disabled:
the public Rust Playground does not include Stillwater, even for examples using
the 1.x API. Publishing 2.0 does not automatically install it in that service.
Code remains selectable and copyable, and Rust examples remain covered by Cargo
doctests.

## Run examples from this checkout

Install Rust 1.89 or newer, then clone the repository:

```sh
git clone https://github.com/iepathos/stillwater.git
cd stillwater
cargo test --doc --all-features
```

This tests the Rust examples against the checked-out library and its declared
dependencies, including before 2.0 is published. To test just the Semigroup chapter:

```sh
cargo test --doc --all-features book_doctests::semigroup
```

To run a complete application example:

```sh
cargo run --example user_registration --all-features
```

Other complete programs are in the repository's `examples/` directory. Use
`--all-features` when running them so feature-gated examples are available.

## Copy a snippet into a standalone project

From the directory containing your `stillwater` checkout, create a sibling project:

```sh
cargo new stillwater-book-example
cd stillwater-book-example
```

Replace its `Cargo.toml` with:

```toml
[package]
name = "stillwater-book-example"
version = "0.1.0"
edition = "2021"

[dependencies]
stillwater = { path = "../stillwater" }
```

Replace `src/main.rs` with this complete version of the Semigroup vector example:

```rust
use stillwater::Semigroup;

fn main() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];
    assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);

    let empty: Vec<i32> = vec![];
    let values = vec![1, 2, 3];
    assert_eq!(empty.combine(values), vec![1, 2, 3]);
    println!("Semigroup examples passed.");
}
```

Then run:

```sh
cargo run
```

After 2.0 is published, you can replace the path dependency with
`stillwater = "2.0"` to use crates.io. The path-based instructions above work now
and are exercised automatically in a fresh project by CI.

## Understand the different kinds of examples

- Most chapter snippets are function bodies. Put them inside `fn main()` when
  copying into a binary project, unless they already define it.
- Use the eye button to reveal hidden setup lines before copying examples that
  need them. Async examples may use `tokio_test::block_on`; a standalone project
  must add `tokio-test = "0.4"` as a dependency for that helper. The checkout's
  doctest commands already provide this and other test dependencies.
- Feature-specific APIs need their Cargo features enabled. See
  [API tiers](guide/17-api-tiers.md) and the relevant chapter.
- `compile_fail` examples intentionally demonstrate compiler rejection;
  `no_run` examples are compile-checked but not executed by doctests.
- Plain-text sketches and historical examples illustrate ideas, not complete
  programs. They are not presented as runnable applications.

The chapter doctests and complete application examples are the supported way to
run broader examples with the correct dependencies and setup.
