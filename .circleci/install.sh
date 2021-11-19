#! /usr/bin/env sh

echo "Install toolchain"
rustup toolchain install nightly

echo "Install grcov"
cargo install grcov
