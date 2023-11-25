use crate::arch::task::ArchTask;
use crate::config;
use crate::task::{GetNoarchTask, NoarchTask};
use core::arch::asm;
use core::cell::Cell;
use core::{mem, slice};
use klib::local_address_of;
use klib::result::KResult;

const STACK_SIZE: usize = 512;
const STACK_COUNT: usize = STACK_SIZE / 4;

struct KernelStack {
    stack: [[u32; STACK_COUNT]; config::NUM_TASKS as usize],
}

#[link_section = ".ubss"]
static mut KERNEL_STACKS: KernelStack = KernelStack {
    stack: [[0; STACK_COUNT]; config::NUM_TASKS as usize],
};

#[repr(C, align(128))]
pub struct Task {
    stack: Cell<u32>,
    user_sp: Cell<u32>,
    user_tp: Cell<u32>,
    noarch_task: NoarchTask,
}

fn init_stack(tid: u32, pc: u32) -> u32 {
    unsafe {
        let stack: *mut u32 = KERNEL_STACKS.stack.get_unchecked_mut(tid as usize) as *mut u32;
        let sp = stack.add(STACK_COUNT).sub(16);
        let prep = slice::from_raw_parts_mut(sp, 16);
        let cramp32_start_task_ptr = local_address_of!("cramp32_start_task");
        prep[0] = pc as u32; // mepc
        for i in 1..15 {
            // gp, tp, s0-s11
            prep[i] = 0;
        }
        prep[15] = cramp32_start_task_ptr; // ra

        sp as u32
    }
}

#[no_mangle]
pub extern "C" fn idle_task() {
    loop {}
}

impl ArchTask for Task {
    fn arch_task_init(tid: u32, task: &Task, pc: u32, sp: u32) -> KResult<()> {
        task.stack.set(init_stack(tid, pc));
        task.user_sp.set(sp);
        task.user_tp.set(0);
        KResult::Ok(())
    }

    fn arch_idle_task_entry_point() -> u32 {
        local_address_of!("idle_task")
    }

    fn arch_task_switch(prev: &Task, next: &Task) {
        extern "C" {
            #[allow(improper_ctypes)]
            fn cramp32_task_switch(prev_sp: *mut u32, next_sp: u32, next_task: *const Task);
        }
        unsafe {
            cramp32_task_switch(prev.stack.as_ptr(), next.stack.get(), next);
        }
    }

    fn arch_switch_idle_task(idle_task: &Task) {
        extern "C" {
            #[allow(improper_ctypes)]
            fn cramp32_switch_idle_task(dummy: u32, next_sp: u32, next_task: *const Task);
        }
        unsafe {
            cramp32_switch_idle_task(0, idle_task.stack.get(), idle_task);
        }
    }

    fn current() -> &'static Task {
        unsafe {
            let mut task_ptr: u32;
            asm!("mv {0}, tp", out(reg) task_ptr);
            &*mem::transmute::<u32, *const Task>(task_ptr)
        }
    }
}

impl GetNoarchTask for Task {
    fn noarch(&self) -> &NoarchTask {
        &self.noarch_task
    }
}
