use crate::config;
use crate::mmio;
use crate::result::KResult;
use crate::task::{GetNoarchTask, KArchTask, NoarchTask};

const STACK_SIZE: usize = 4096;
const STACK_COUNT: usize = STACK_SIZE / 4;

#[repr(align(4096))]
struct ExceptionStack {
    stack: [[u32; STACK_COUNT]; config::NUM_TASKS as usize],
}

static mut EXCEPTION_STACKS: ExceptionStack = ExceptionStack {
    stack: [[0; STACK_COUNT]; config::NUM_TASKS as usize],
};

#[repr(align(16))]
pub struct Cramp32Task {
    pub stack: usize,
    noarch_task: NoarchTask,
}

fn init_stack(tid: u32, pc: usize) -> usize {
    unsafe {
        let stack: *mut u32 = EXCEPTION_STACKS.stack.get_unchecked_mut(tid as usize) as *mut u32;
        let sp = stack.add(STACK_COUNT).sub(16);
        extern "C" {
            fn cramp32_start_task();
        }
        mmio::writev(sp, pc as u32); // mepc
        for i in 0..7 {
            // gp, tp, s0-s11
            mmio::writev(sp.add(i * 2 + 1), 0);
            mmio::writev(sp.add(i * 2 + 2), 0);
        }
        mmio::writev(sp.add(15), cramp32_start_task as u32); // ra

        sp as usize
    }
}

impl KArchTask for Cramp32Task {
    fn arch_task_create(noarch_task: NoarchTask, pc: usize) -> KResult<Cramp32Task> {
        KResult::Ok(Cramp32Task {
            stack: init_stack(noarch_task.tid, pc),
            noarch_task,
        })
    }

    fn arch_task_switch(prev: &mut Cramp32Task, next: &mut Cramp32Task) {
        extern "C" {
            fn cramp32_task_switch(prev_sp: *mut usize, next_sp: usize);
        }
        unsafe {
            cramp32_task_switch(&mut prev.stack, next.stack);
        }
    }
}

impl GetNoarchTask for Cramp32Task {
    fn noarch(&self) -> &NoarchTask {
        &self.noarch_task
    }
    fn noarch_mut(&mut self) -> &mut NoarchTask {
        &mut self.noarch_task
    }
}
