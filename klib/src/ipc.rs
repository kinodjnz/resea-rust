use core::mem;
use core::ops::BitOr;

#[derive(Clone, Copy)]
pub struct IpcFlags(u8);

#[allow(unused)]
impl IpcFlags {
    const NOBLOCK: u8 = 1 << 0;
    // const KERNEL: u8 = 1 << 1; // Internally used by kernel.

    pub fn from_u32(flags: u32) -> IpcFlags {
        IpcFlags(flags as u8)
    }
    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }

    pub fn block() -> IpcFlags {
        IpcFlags(0)
    }
    pub fn is_noblock(&self) -> bool {
        self.0 & Self::NOBLOCK != 0
    }
}

#[derive(Clone, Copy)]
pub struct MessageType(pub u32);

impl MessageType {
    pub const NOTIFICATIONS: MessageType = MessageType(1);
}

#[derive(Clone, Copy)]
pub struct Message {
    pub message_type: MessageType,
    pub src_tid: u32,
    pub raw: [u8; 24],
}

impl Message {
    pub fn set_payload<T: Copy>(&mut self, data: &T) {
        unsafe {
            *mem::transmute::<_, &mut _>(&mut self.raw) = *data;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Notifications(u8);

#[allow(unused)]
impl Notifications {
    const TIMER: u8 = 1 << 0;
    const IRQ: u8 = 1 << 1;
    const ABORTED: u8 = 1 << 2;
    const ASYNC: u8 = 1 << 3;

    pub fn from_u32(n: u32) -> Notifications {
        Notifications(n as u8)
    }

    pub fn timer() -> Notifications {
        Notifications(Self::TIMER)
    }
    pub fn aborted() -> Notifications {
        Notifications(Self::ABORTED)
    }
    pub fn clear(&self, notifications: Notifications) -> Notifications {
        Notifications(self.0 & !notifications.0)
    }
    pub fn none() -> Notifications {
        Notifications(0)
    }
    pub fn is_aborted(&self) -> bool {
        self.0 & Self::ABORTED != 0
    }
    pub fn exists(&self) -> bool {
        self.0 != 0
    }
}

impl BitOr for Notifications {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
