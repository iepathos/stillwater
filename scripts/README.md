# Documentation checks

Run `just book-check-playground` with Python 3.11+ and mdBook installed.
It builds the book, tests the checkers, enforces local-only execution on the
rendered HTML, and runs the documented standalone quickstart. CI and documentation
deployment run the same checks.

## Rendered-book policy

```sh
python3 -B scripts/check_book_playground.py book/book
```

This offline check rejects every element with a `playground` class in every HTML
page, including print output, and reports file names and line numbers. It does
not depend on the element's tag, code contents, or `no_run`, `ignore`, and
`compile_fail` attributes: mdBook's JavaScript selects `.playground` containers.
Keep `output.html.playground.runnable = false` and avoid custom playground markup.
Missing or empty book output fails the check.

There is no live inventory mode or network dependency. Publishing to crates.io
does not automatically add Stillwater to the public Rust Playground. This is a
structural guard for the local-only policy, not a browser execution test.

## Standalone quickstart

```sh
python3 -B scripts/check_book_quickstart.py
```

The smoke test runs the exact TOML and Rust snippets in `docs/running-examples.md`
as a fresh standalone Cargo project with the documented relative dependency path.
This test needs Cargo and may fetch dependencies, just like the reader's
`cargo run`.

Keep the existing Cargo doctests and documentation-contract tests. Do not change
Rust examples to `text`, `ignore`, or `no_run` merely to pass the rendered check.
Deployment runs documentation-contract tests and all-feature doctests before
publishing. Any future custom browser runner also needs an end-to-end execution
test for its exact dependencies and runtime.
