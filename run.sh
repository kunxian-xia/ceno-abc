#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$ROOT_DIR/program"
cargo ceno build --release

cd "$ROOT_DIR/host"
cargo run --release
