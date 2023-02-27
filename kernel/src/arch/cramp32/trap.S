.option rvc
.global intr_handler
intr_handler:
        addi    sp, sp, -64
        sw      ra, 60(sp)
        sw      a2, 56(sp)
        sw      a3, 52(sp)
        sw      a4, 48(sp)
        sw      a5, 44(sp)
        sw      a6, 40(sp)
        sw      a7, 36(sp)
        sw      t0, 32(sp)
        sw      t1, 28(sp)
        sw      t2, 24(sp)
        sw      t3, 20(sp)
        sw      t4, 16(sp)
        sw      t5, 12(sp)
        sw      t6, 8(sp)

        csrr    a2, mcause
        bltz    a2, 1f          // if interrupt bit is set
        addi    a2, a2, -11
        bnez    a2, 3f          // if not an environment call from m-mode

        lw      a2, 56(sp)
        call    handle_syscall

        // In case of ecall, mepc points to the ecall address.
        // Fix mepc to next to the ecall address.
        csrr    a2, mepc
        addi    a2, a2, 4
        csrw    mepc, a2
        j       2f

1:
        sw      a0, 4(sp)
        sw      a1, 0(sp)
        call    cramp32_handle_interrupt
4:
        lw      a1, 0(sp)
        lw      a0, 4(sp)

2:
        lw      t6, 8(sp)
        lw      t5, 12(sp)
        lw      t4, 16(sp)
        lw      t3, 20(sp)
        lw      t2, 24(sp)
        lw      t1, 28(sp)
        lw      t0, 32(sp)
        lw      a7, 36(sp)
        lw      a6, 40(sp)
        lw      a5, 44(sp)
        lw      a4, 48(sp)
        lw      a3, 52(sp)
        lw      a2, 56(sp)
        lw      ra, 60(sp)
        addi    sp, sp, 64
        mret

3:
        lw      a2, 56(sp)
        sw      a0, 4(sp)
        sw      a1, 0(sp)
        call    cramp32_handle_exception
        j       4b
