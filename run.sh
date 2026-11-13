#!/usr/bin/env bash
# Собирает проект и запускает его с примером параметров (этап 1).
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

./target/release/depgraph \
    --package curl \
    --repo https://archive.ubuntu.com/ubuntu/ \
    --test-mode url \
    --output graph.png \
    --ascii-tree on \
    --filter lib
