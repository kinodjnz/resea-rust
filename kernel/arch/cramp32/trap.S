.option rvc
.global intr_handler
intr_handler:
        addi    sp, sp, -64
        sw      ra, 60(sp)
        sw      a0, 56(sp)
        sw      a1, 52(sp)
        sw      a2, 48(sp)
        sw      a3, 44(sp)
        sw      a4, 40(sp)
        sw      a5, 36(sp)
        sw      a6, 32(sp)
        sw      a7, 28(sp)
        sw      t0, 24(sp)
        sw      t1, 20(sp)
        sw      t2, 16(sp)
        sw      t3, 12(sp)
        sw      t4, 8(sp)
        sw      t5, 4(sp)
        sw      t6, 0(sp)

        csrr    a0, mcause
        bltz    a0, 1f          // if intrrupt bit set
        addi    a0, a0, -11
        bnez    a0, 3f          // if not environment call from m-mode

        lw      a0, 4(sp)
        call    handle_syscall
        j       2f
1:
        call    cramp32_handle_interrupt

2:
        lw      t6, 0(sp)
        lw      t5, 4(sp)
        lw      t4, 8(sp)
        lw      t3, 12(sp)
        lw      t2, 16(sp)
        lw      t1, 20(sp)
        lw      t0, 24(sp)
        lw      a7, 28(sp)
        lw      a6, 32(sp)
        lw      a5, 36(sp)
        lw      a4, 40(sp)
        lw      a3, 44(sp)
        lw      a2, 48(sp)
        lw      a1, 52(sp)
        lw      a0, 56(sp)
        lw      ra, 60(sp)
        addi    sp, sp, 64
        mret

3:
        lw      a0, 4(sp)
        call    cramp32_handle_exception
        j       2b
