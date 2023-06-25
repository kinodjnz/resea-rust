#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Syscall {
    Nop,
    Kdebug,
    IpcSend,
    IpcRecv,
    IpcCall,
    Notify,
    SetTimer,
    ConsoleWrite,
    CreateTask,
    DestroyTask,
    ExitTask,
    TaskSelf,
    ScheduleTask,
    IrqAquire,
    IrqRelease,
}

impl Syscall {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
