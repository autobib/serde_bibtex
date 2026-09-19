#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --all --check
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings" cargo doc --all-features --no-deps
