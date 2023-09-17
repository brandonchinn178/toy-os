# toy-os

An OS I'm building from scratch.

https://os.phil-opp.com/

## Quickstart

```bash
rustup override set nightly
rustup component add rust-src --toolchain nightly-aarch64-apple-darwin

cargo install bootimage
rustup component add llvm-tools-preview

brew install qemu

cargo run
```
