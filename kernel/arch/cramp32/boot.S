.option rvc
.section .boot, "ax", @progbits
.global boot

boot:
	la	sp, __boot_stack_end
	j	cramp32_init
