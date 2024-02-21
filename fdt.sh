qemu-system-riscv64 --machine virt,dumpdtb=virt.out --kernel target/riscv64imac-unknown-none-elf/debug/kernel --nographic -smp 1 -m 128M
fdtdump virt.out
