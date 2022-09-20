.option rvc
.section .boot, "ax", @progbits
.global _start

_start:
	la	sp, ramend
	j	__start_rust
