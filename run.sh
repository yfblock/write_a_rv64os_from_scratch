cargo build
qemu-system-riscv64 --machine virt --kernel target/riscv64imac-unknown-none-elf/debug/my_os --nographic
