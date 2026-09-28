#!/usr/bin/env bash
set -u
cd "$(dirname "$0")/.."     # always run from workspace root — fixes the earlier bug too
KERNEL=target/riscv64gc-unknown-none-elf/debug/niko-os
LOG=/tmp/t_procs.log

timeout 15 qemu-system-riscv64 -machine virt -nographic -bios none \
    -kernel "$KERNEL" > "$LOG" 2>&1

fail=0
check() { grep -q "$1" "$LOG" || { echo "  missing: $1"; fail=1; }; }
check "hello A"
check "hello B"
check "no processes left"
grep -q "KERNEL PANIC" "$LOG" && { echo "  kernel panicked"; fail=1; }
[ "$(grep -c 'hello A' "$LOG")" -ge 3 ] || { echo "  expected 3x hello A"; fail=1; }
[ "$(grep -c 'hello B' "$LOG")" -ge 3 ] || { echo "  expected 3x hello B"; fail=1; }

[ $fail -eq 0 ] && echo "PASS: t_procs" || { echo "FAIL: t_procs"; sed -n '1,40p' "$LOG"; exit 1; }