OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    _skernel = .;
    /*0x80200000*/
    .text ALIGN(4K): {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        etext = .;
    }

    .sigtrx ALIGN(4K): {
        *(.sigtrx .sigtrx.*)
    }

    .rodata ALIGN(4K): {
        srodata = .;
        *(.rodata .rodata.*)
        . = ALIGN(4K);
        erodata = .;
    }

    .data ALIGN(4K): {
        . = ALIGN(4K);
        *(.data.prepage .data.prepage.*)
        . = ALIGN(4K);
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        edata = .;
    }

    .bss ALIGN(4K): {
        *(.bss.stack)
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        _ebss = .;
    }
    . = ALIGN(4K);
    _ekernel = .;

    PROVIDE(end = .);
}