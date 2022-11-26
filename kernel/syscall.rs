use core::slice;
use core::arch::asm;
use crate::result::KResult;
use crate::task;
use crate::console::Console;

pub struct Syscall;

impl Syscall {
    pub const NOP: u32 = 0;
    pub const SET_TIMER: u32 = 5;
    pub const CONSOLE_WRITE: u32 = 6;
}

fn handle_set_timer(timeout: u32) -> KResult<()> {
    let task_pool = task::get_task_pool();
    task_pool.set_current_timeout(timeout)
}

fn handle_console_write(s: &[u8]) -> KResult<()> {
    if s.len() > 1024 {
        KResult::TooLarge
    } else {
        Console::puts(s);
        KResult::Ok(())
    }
}

#[no_mangle]
pub extern "C" fn handle_syscall(
    a0: u32,
    a1: u32,
    _a2: u32,
    _a3: u32,
    _a4: u32,
    _a5: u32,
    _syscall_subid: u32,
    syscall_id: u32,
) -> u32 {
    let r = match syscall_id {
        Syscall::NOP => KResult::Ok(()),
        Syscall::SET_TIMER => handle_set_timer(a0),
        Syscall::CONSOLE_WRITE => handle_console_write(unsafe { slice::from_raw_parts(a0 as *const u8, a1 as usize) }),
        _ => KResult::InvalidArg,
    };
    match r {
        KResult::Ok(()) => 0,
        e => e.err_as_u32(),
    }
}

#[allow(unused)]
fn to_u32_result(a0: u32, a1: u32) -> KResult<u32> {
    if a0 == 0 {
        KResult::Ok(a1.into())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(unused)]
fn to_unit_result(a0: u32) -> KResult<()> {
    if a0 == 0 {
        KResult::Ok(Default::default())
    } else {
        KResult::err_from_u32(a0)
    }
}

#[allow(unused)]
pub fn syscall0r(syscall_id: u32) -> KResult<u32> {
    unsafe {
        let mut a0: u32;
        let mut a1: u32;
        asm!("ecall", in("a7") syscall_id, out("a0") a0, out("a1") a1);
        to_u32_result(a0, a1)
    }
}

#[allow(unused)]
pub fn syscall0(syscall_id: u32) -> KResult<()> {
    unsafe {
        let mut a0: u32;
        asm!("ecall", in("a7") syscall_id, out("a0") a0);
        to_unit_result(a0)
    }
}

#[allow(unused)]
pub fn syscall2r(syscall_id: u32, mut a0: u32, mut a1: u32) -> KResult<u32> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") syscall_id);
        to_u32_result(a0, a1)
    }
}

#[allow(unused)]
pub fn syscall2(syscall_id: u32, mut a0: u32, mut a1: u32) -> KResult<()> {
    unsafe {
        asm!("ecall", inout("a0") a0, inout("a1") a1, in("a7") syscall_id);
        to_unit_result(a0)
    }
}

#[allow(unused)]
pub fn nop() -> KResult<()> {
    syscall0(Syscall::NOP)
}

#[allow(unused)]
pub fn console_write(s: &[u8]) -> KResult<()> {
    syscall2(Syscall::CONSOLE_WRITE, s.as_ptr() as u32, s.len() as u32)
}
