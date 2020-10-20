# Working DPDK API (partial)

This work is basically slight modification of the code found [here](https://github.com/capsule-rs/capsule) (in the core dir). This work developed a nice interface for using DPDK in Rust and I have just exposed some internal and crate only function.

The main reason for doing so is to be able to program multi-process DPDK apps in Rust. As of now, there are no Rust libs/frameworks that do so reliably.

Capsule comes closest to providing this [support](https://github.com/capsule-rs/capsule/issues/74); however, it is still some distance off.

## Workspaces

There are two workspaces (nested) here. The first one is the one that contains the FFI and the APIs/wrappers around the FFI:
- capsule
- capsule-ffi
- capsule-macros

These do not contain versioning yet!

The second (nested) workspace is in `examples` which contains the `toy-onvm` project.

## Running

Running `cargo build | run` inside `examples`, should also run the equivalent `cargo build` in the top level workspace.

```bash
cd examples
cargo build
sudo ./target/debug/basic-demo
```

This will create mempools, zones and rings and print out information and status of the same.
