#!/bin/sh
RUSTFLAGS="-Awarnings" \
RUST_LOG="leidenfrost" \
cargo run --release