# Niko OS

## Troubleshooting
riscv64-unknown-elf-nm target/riscv64gc-unknown-none-elf/debug/niko-os | grep _start
This checks the start address which should be exactly: 0000000080000000
