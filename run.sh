cargo build
qemu-system-riscv64 --machine virt --kernel target/riscv64imac-unknown-none-elf/debug/kernel --nographic -smp 1 -m 128M
