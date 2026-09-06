# Documentation checks

Run `just book-check-playground` with Python 3.11+ and mdBook installed.
It builds the book, tests the checker, and enforces the book's local-only execution
policy on the rendered HTML. CI and documentation deployment use `--local-only`,
which rejects reintroduced browser Run blocks without any network calls. It also
runs the exact TOML and Rust snippets in `docs/running-examples.md` as a fresh
standalone Cargo project with the documented relative dependency path. This smoke
test needs Cargo and may fetch dependencies, just like the reader's `cargo run`.
Deployment additionally runs Cargo documentation-contract tests and all-feature
doctests before publishing the book.

The optional inventory mode (omit `--local-only`) checks against the public Rust
Playground's live `/meta/crates` inventory. This remains available for diagnosing
the original unsupported-dependency failure; it is not needed by the local-only
book's CI checks.

The check reports HTML file names and line numbers when a playground block refers
to Stillwater or a declared dependency that the Playground does not provide.
Stillwater's available version must match the version in `Cargo.toml`: testing
against a different published version does not verify the documented candidate.
Publishing to crates.io does not automatically add a crate to the Playground.
Inventory network failures and malformed responses fail the check, not pass it.

For an offline check, save the inventory JSON and supply it explicitly:

```sh
python3 -B scripts/check_book_playground.py book/book --inventory inventory.json
```

This is a conservative dependency guard, not a browser execution test or a Rust
parser. It recognizes namespace references and `extern crate` declarations for
the package and its direct/dev dependencies, including hidden code lines. It
does not prove that other dependency versions, features, runtime behavior, or
undeclared dependencies are compatible. It can conservatively flag references
inside comments or strings. HTML marked `no_run`, `ignore`, or `compile_fail`, and
blocks without mdBook's `playground` class, are outside this check; they do not
advertise normal Playground execution. This checker targets mdBook's standard
public Playground integration, not a custom execution backend.

Keep the existing Cargo doctests and documentation-contract tests. Do not change
Rust examples to `text`, `ignore`, or `no_run` merely to pass the Playground check.
The documented local Cargo workflow supplies the execution path. Any future
custom browser runner also needs an end-to-end execution test for its exact
dependencies and runtime; this inventory check alone is not sufficient.
