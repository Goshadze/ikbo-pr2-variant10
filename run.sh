#!/usr/bin/env bash
# Собирает проект и печатает обход фикстуры тестового репозитория (этап 3, офлайн).
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

./target/release/depgraph \
    --package A \
    --repo tests/fixtures/diamond.txt \
    --test-mode file \
    --filter C
