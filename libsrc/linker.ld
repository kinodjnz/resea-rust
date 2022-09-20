OUTPUT_ARCH( "cramp" )
ENTRY(_start)

MEMORY
{
    mem(rwx)  : ORIGIN = 0x20001000, LENGTH = 268431360
}

SECTIONS
{
  .isr_vector : {
      . = ALIGN(4);
      KEEP(*(.isr_vector))
      . = ALIGN(4);
  } >mem
  .text : { 
      . = ALIGN(4);
      *(.boot) 
      *(.text) 
      . = ALIGN(4);
  } >mem
  .rodata : { 
      . = ALIGN(4);
      *(.rodata) 
      . = ALIGN(4);
  } >mem
  .sdata : { 
      __sdata_start = .;
      . = ALIGN(4);
      *(.sdata) 
      . = ALIGN(4);
      __sdata_end = .;
  } >mem
  .data : {
      __data_start = .;
      . = ALIGN(4);
      *(.data)
      . = ALIGN(4);
      __data_end = .;
  } >mem AT>mem
  .bss : { 
      . = ALIGN(4);
      __bss_start = .;
      *(.bss*) 
      *(.sbss*);
      . = ALIGN(4);
      __bss_end = .;
  } >mem
  PROVIDE(stack_bottom = .);
  PROVIDE(_end = .);
  PROVIDE(end = .);
  PROVIDE(ramend = ORIGIN(mem) + LENGTH(mem));
}
