# dlhook
An rust proc-macro crate makes write LD_PRELOAD hooks easy.

## Usage
Just put `#[dlhook(origin = "origin_function_name")` on your hook function and receive the "real/origin" function pointer on the first argument. Note that you can(and must!) flag the type of the first function as `: _`(InferType) so you don't need to repeat yourself.

## Example
```rust
#[dlhook::dlhook(origin = "getuid")]
fn fake_root_uid(_: _) -> u32 {
    0
}
```
See more examples at [`examples/`](https://github.com/chlorodose/dlhook/tree/main/examples)
