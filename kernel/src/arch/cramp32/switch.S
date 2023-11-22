.option rvc
.global cramp32_task_switch
cramp32_task_switch:
        addi    sp, sp, -64
        sw      ra, 60(sp)
        sw      gp, 56(sp)
        # sw      tp, 52(sp)
        sw      s0, 48(sp)
        sw      s1, 44(sp)
        sw      s2, 40(sp)
        sw      s3, 36(sp)
        sw      s4, 32(sp)
        sw      s5, 28(sp)
        sw      s6, 24(sp)
        sw      s7, 20(sp)
        sw      s8, 16(sp)
        sw      s9, 12(sp)
        sw      s10, 8(sp)
        sw      s11, 4(sp)
        csrr    a3, mepc
        sw      a3, 0(sp)

        sw      sp, 0(a0)

.global cramp32_switch_idle_task
cramp32_switch_idle_task:
        mv      sp, a1

        csrw    mscratch, a2
        mv      tp, a2

        lw      a3, 0(sp)
        csrw    mepc, a3
        lw      s11, 4(sp)
        lw      s10, 8(sp)
        lw      s9, 12(sp)
        lw      s8, 16(sp)
        lw      s7, 20(sp)
        lw      s6, 24(sp)
        lw      s5, 28(sp)
        lw      s4, 32(sp)
        lw      s3, 36(sp)
        lw      s2, 40(sp)
        lw      s1, 44(sp)
        lw      s0, 48(sp)
        # lw      tp, 52(sp)
        lw      gp, 56(sp)
        lw      ra, 60(sp)
        addi    sp, sp, 64
        ret

.global cramp32_start_task
cramp32_start_task:
        //call    stack_set_canary

        sw      sp, 0(tp)

        mret
