use crate::config;
use crate::task::{GetNoarchTask, KArchTask, NoarchTask};
use crate::macros::*;
use core::cell::Cell;
use core::slice;
use klib::result::KResult;

const STACK_SIZE: usize = 4096;
const STACK_COUNT: usize = STACK_SIZE / 4;

#[repr(align(4096))]
struct ExceptionStack {
    stack: [[u32; STACK_COUNT]; config::NUM_TASKS as usize],
}

static mut EXCEPTION_STACKS: ExceptionStack = ExceptionStack {
    stack: [[0; STACK_COUNT]; config::NUM_TASKS as usize],
};

#[repr(align(128))]
pub struct Cramp32Task {
    stack: Cell<usize>,
    noarch_task: NoarchTask,
}

fn init_stack(tid: u32, pc: usize) -> usize {
    unsafe {
        let stack: *mut u32 = EXCEPTION_STACKS.stack.get_unchecked_mut(tid as usize) as *mut u32;
        let sp = stack.add(STACK_COUNT).sub(16);
        let prep = slice::from_raw_parts_mut(sp, 16);
        let cramp32_start_task_ptr = local_address_of!("cramp32_start_task");
        prep[0] = pc as u32; // mepc
        for i in 1..15 {
            // gp, tp, s0-s11
            prep[i] = 0;
        }
        prep[15] = cramp32_start_task_ptr as u32; // ra

        sp as usize
    }
}

impl KArchTask for Cramp32Task {
    fn arch_task_init(tid: u32, task: &Cramp32Task, pc: usize) -> KResult<()> {
        task.stack.set(init_stack(tid, pc));
        KResult::Ok(())
    }

    fn arch_task_switch(prev: &Cramp32Task, next: &Cramp32Task) {
        extern "C" {
            fn cramp32_task_switch(prev_sp: *mut usize, next_sp: usize);
        }
        unsafe {
            cramp32_task_switch(prev.stack.as_ptr(), next.stack.get());
        }
    }
}

impl GetNoarchTask for Cramp32Task {
    fn noarch(&self) -> &NoarchTask {
        &self.noarch_task
    }
}
