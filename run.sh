#!/usr/bin/env bash
# Собирает проект и выводит прямые зависимости пакета curl (этап 2).
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

./target/release/depgraph \
    --package curl \
    --repo https://archive.ubuntu.com/ubuntu/
