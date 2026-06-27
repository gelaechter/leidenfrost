#!/bin/sh
RUSTFLAGS="-Awarnings" \
RUST_LOG="leidenfrost=debug" \
cargo run --release