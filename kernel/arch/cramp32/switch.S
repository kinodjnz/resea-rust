.option rvc
.global cramp32_task_switch
cramp32_task_switch:
        addi    sp, sp, -60
        sw      ra, 0(sp)
        sw      gp, 4(sp)
        sw      tp, 8(sp)
        sw      s0, 12(sp)
        sw      s1, 16(sp)
        sw      s2, 20(sp)
        sw      s3, 24(sp)
        sw      s4, 28(sp)
        sw      s5, 32(sp)
        sw      s6, 36(sp)
        sw      s7, 40(sp)
        sw      s8, 44(sp)
        sw      s9, 48(sp)
        sw      s10, 52(sp)
        sw      s11, 56(sp)

        sw      sp, 0(a0)
        mv      sp, a1

        lw      s11, 56(sp)
        lw      s10, 52(sp)
        lw      s9, 48(sp)
        lw      s8, 44(sp)
        lw      s7, 40(sp)
        lw      s6, 36(sp)
        lw      s5, 32(sp)
        lw      s4, 28(sp)
        lw      s3, 24(sp)
        lw      s2, 20(sp)
        lw      s1, 16(sp)
        lw      s0, 12(sp)
        lw      tp, 8(sp)
        lw      gp, 4(sp)
        lw      ra, 0(sp)
        addi    sp, sp, 60
        ret

.global cramp32_start_task
cramp32_start_task:
        //call    stack_set_canary

        li      a0, 0x80
        csrs    mstatus, a0
        lw      a0, 0(sp)
        csrw    mepc, a0
        addi    sp, sp, 4

        mret
