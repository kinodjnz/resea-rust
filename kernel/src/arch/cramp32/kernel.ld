OUTPUT_ARCH("cramp")
ENTRY(boot)

MEMORY
{
    mem(rwx): ORIGIN = 0x20000000, LENGTH = 268435456
}

SECTIONS
{
    .text : {
        __kernel_image_start = .;
        . = ALIGN(4);
        *(.boot);
        *(.text);
        . = ALIGN(4);
    } >mem

    .rodata : {
        . = ALIGN(4);
        *(.rodata);
        *(.rodata.*);
        . = ALIGN(4);
    } >mem

    .sdata : {
        __sdata_start = .;
        . = ALIGN(4);
        *(.sdata);
        *(.sdata.*);
        . = ALIGN(4);
        __sdata_end = .;
    } >mem

    .data : {
        __data_start = .;
        . = ALIGN(4);
        *(.data);
        . = ALIGN(4);
        __data_end = .;
    } >mem

    .got : {
        . = ALIGN(4);
        *(.got);
        . = ALIGN(4);
    } >mem

    .bss : {
        . = ALIGN(4);
        __bss_start = .;
        *(.bss*);
        *(.sbss*);
        . = ALIGN(4);
        __bss_end = .;

        __boot_stack_start = .;
        . += 0x800;
        . = ALIGN(0x1000);
        __boot_stack_end = .;
        __kernel_image_end = .;
    } >mem

    /DISCARD/ : {
        *(.panic_info*);
    }
}
