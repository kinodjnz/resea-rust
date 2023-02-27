.option rvc
.section .boot, "ax", @progbits
.global boot

boot:
	auipc	sp, %pcrel_hi(__boot_stack_end)
	jump	cramp32_init, t0
