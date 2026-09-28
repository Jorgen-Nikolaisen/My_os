#!/usr/bin/env bash
set -u
cd "$(dirname "$0")/.."
cargo build -p niko-os || exit 1
for t in test/t_*.sh; do
    [ -x "$t" ] || chmod +x "$t"
    "$t" || exit 1
done
echo "ALL TESTS PASS"
