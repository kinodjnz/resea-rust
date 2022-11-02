.option rvc
.section .boot, "ax", @progbits
.global boot

boot:
	la	sp, __boot_stack_end
	jump	cramp32_init, t0
