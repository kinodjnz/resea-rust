use crate::config;
use crate::macros::*;
use crate::task::{GetNoarchTask, KArchTask, NoarchTask};
use core::cell::Cell;
use core::slice;
use klib::result::KResult;

const STACK_SIZE: usize = 4096;
const STACK_COUNT: usize = STACK_SIZE / 4;

#[repr(align(4096))]
struct ExceptionStack {
    stack: [[u32; STACK_COUNT]; config::NUM_TASKS as usize],
}

#[link_section = ".ubss"]
static mut EXCEPTION_STACKS: ExceptionStack = ExceptionStack {
    stack: [[0; STACK_COUNT]; config::NUM_TASKS as usize],
};

#[repr(C, align(128))]
pub struct Cramp32Task {
    stack: Cell<usize>,
    user_sp: Cell<u32>,
    user_tp: Cell<u32>,
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
            #[allow(improper_ctypes)]
            fn cramp32_task_switch(
                prev_sp: *mut usize,
                next_sp: usize,
                next_task: *const Cramp32Task,
            );
        }
        unsafe {
            cramp32_task_switch(prev.stack.as_ptr(), next.stack.get(), next);
        }
    }

    fn init_current(task: &Cramp32Task) {
        unsafe {
            let task_ptr: u32 = mem::transmute(<*const Cramp32Task>::from(task));
            asm!("csrw mscratch, {0}", in(reg) task_ptr);
            asm!("mv   tp, {0}", in(reg) task_ptr);
        }
    }

    fn current() -> &'static Cramp32Task {
        unsafe {
            let mut task_ptr: u32;
            asm!("mv {0}, tp", out(reg) task_ptr);
            &*mem::transmute::<u32, *const Cramp32Task>(task_ptr)
        }
    }
}

impl GetNoarchTask for Cramp32Task {
    fn noarch(&self) -> &NoarchTask {
        &self.noarch_task
    }
}
