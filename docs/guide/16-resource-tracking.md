# Compile-Time Resource Tracking

Stillwater's resource tracking layer lets you describe resource acquisition and release in types. It is useful when an effect opens something that must be closed, such as a file handle, database transaction, lock, socket, or custom resource.

This feature is opt-in. Existing `Effect` code continues to work without resource annotations.

## When to Use It

Use resource tracking when:

- a workflow has explicit acquire/use/release phases
- a missing cleanup step would be a correctness bug
- the resource protocol matters, such as begin/commit/rollback
- you want function signatures to document resource behavior

For ordinary scoped cleanup, the runtime bracket helpers are often enough. Resource tracking adds type-level documentation and compile-time neutrality checks on top.

## Core Types

Resource tracking is built from marker types and type-level sets:

```rust,ignore
use stillwater::effect::resource::*;

// Built-in resource markers
FileRes;
DbRes;
LockRes;
SocketRes;
TxRes;

// Type-level resource sets
Empty;        // no resources
Has<FileRes>; // one tracked resource
```

Tracked effects implement `ResourceEffect`, which extends `Effect` with two associated resource sets:

```rust,ignore
trait ResourceEffect {
    type Acquires;
    type Releases;
}
```

These sets describe what an effect acquires and releases at the type level. The tracking wrapper has no runtime behavior beyond running the original effect.

## Annotating Effects

Use extension methods to mark effects as acquiring, releasing, or neutral with respect to resources:

```rust,ignore
use stillwater::effect::resource::*;
use stillwater::{pure, Effect};

fn open_file(path: &str) -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Has<FileRes>,
    Releases = Empty,
> {
    pure::<_, String, ()>(format!("FileHandle({path})"))
        .acquires::<FileRes>()
}

fn close_file(handle: String) -> impl ResourceEffect<
    Output = (),
    Error = String,
    Env = (),
    Acquires = Empty,
    Releases = Has<FileRes>,
> {
    let _ = handle;
    pure::<_, String, ()>(()).releases::<FileRes>()
}

fn read_contents(handle: &str) -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Empty,
    Releases = Empty,
> {
    pure::<_, String, ()>(format!("Contents of {handle}")).neutral()
}
```

The function signatures make the resource contract explicit:

- `open_file` acquires `FileRes`
- `close_file` releases `FileRes`
- `read_contents` is resource-neutral

## Prefer Bracket for Resource Safety

The most ergonomic way to acquire and release a resource safely is the bracket builder:

```rust,ignore
use stillwater::effect::resource::*;
use stillwater::{pure, Effect};

let effect = bracket::<FileRes>()
    .acquire(pure::<_, String, ()>("file_handle".to_string()))
    .release(|handle: String| async move {
        let _ = handle;
        Ok(())
    })
    .use_fn(|handle: &String| {
        pure::<_, String, ()>(format!("Processed: {handle}"))
    });

let result = effect.run(&()).await;
```

The bracket shape is:

1. acquire the resource
2. use the resource
3. release the resource even if use fails

The bracketed effect is resource-neutral as a whole: it does not leak its acquired resource through the type signature.

## Protocol Enforcement

Resource tracking is also useful for protocols such as transactions:

```rust,ignore
use stillwater::effect::resource::*;
use stillwater::{pure, Effect};

fn begin_transaction() -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Has<TxRes>,
    Releases = Empty,
> {
    pure::<_, String, ()>("tx_12345".to_string()).acquires::<TxRes>()
}

fn commit_transaction(tx: String) -> impl ResourceEffect<
    Output = (),
    Error = String,
    Env = (),
    Acquires = Empty,
    Releases = Has<TxRes>,
> {
    let _ = tx;
    pure::<_, String, ()>(()).releases::<TxRes>()
}

fn run_in_transaction() -> impl ResourceEffect<
    Output = String,
    Error = String,
    Env = (),
    Acquires = Empty,
    Releases = Empty,
> {
    bracket::<TxRes>()
        .acquire(begin_transaction())
        .release(|tx| async move { commit_transaction(tx).run(&()).await })
        .use_fn(|tx| pure::<_, String, ()>(format!("used {tx}")))
}
```

The return type says `run_in_transaction` is neutral: it begins and ends the transaction internally.

## Compile-Time Neutrality Checks

Use `assert_resource_neutral` when an API must not expose outstanding resources:

```rust,ignore
use stillwater::effect::resource::*;

fn must_be_neutral<E>(effect: E) -> E
where
    E: ResourceEffect<Acquires = Empty, Releases = Empty>,
{
    assert_resource_neutral(effect)
}
```

If the effect still acquires or releases a resource, the code will fail to compile.

## Custom Resource Markers

Define a marker when the built-in resource kinds do not fit your domain:

```rust,ignore
use stillwater::effect::resource::ResourceKind;

pub struct PoolCheckout;

impl ResourceKind for PoolCheckout {
    const NAME: &'static str = "PoolCheckout";
}
```

You can then use `.acquires::<PoolCheckout>()`, `.releases::<PoolCheckout>()`, and `bracket::<PoolCheckout>()` the same way as built-in markers.

## Example

Run the full example:

```bash
cargo run --example resource_tracking
```

The example covers basic annotations, bracket usage, transaction protocol enforcement, multiple resource types, custom resource markers, and compile-time neutrality assertions.

For API-level details, see `stillwater::effect::resource`.
