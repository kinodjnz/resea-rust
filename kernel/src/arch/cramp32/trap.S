.option rvc
.global intr_handler
intr_handler:
        csrrw   tp, mscratch, tp
        sw      sp, 4(tp)
        lw      sp, 0(tp)
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
        csrrw   t0, mscratch, tp
        sw      t0, 8(tp)

        csrr    t0, mcause
        bltz    t0, 1f          // if interrupt bit is set
        addi    t0, t0, -11
        bnez    t0, 3f          // if not an environment call from m-mode

        // In case of ecall, mepc points to the ecall address.
        // Fix mepc to next to the ecall address.
        csrr    t0, mepc
        addi    t0, t0, 4
        csrw    mepc, t0

        call    handle_syscall
        j       2f

1:
        sw      a0, 4(sp)
        sw      a1, 0(sp)
        mv      a2, t0
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

.global cramp32_start_task
cramp32_start_task:
        sw      sp, 0(tp)
        lw      sp, 4(tp)
        lw      tp, 8(tp)
        mret

3:
        lw      t0, 32(sp)
        sw      a0, 4(sp)
        sw      a1, 0(sp)
        call    cramp32_handle_exception
        j       4b
