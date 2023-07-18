use klib::ipc::{Message, MessageType};
use syscall::payload::{MessageAdapter, PayloadForMessageType};

pub struct AllocPayload {
    pub size: usize,
    pub align: usize,
}

pub struct AllocResponsePayload(pub *mut u8);

pub struct DeallocPayload(pub *mut u8);

pub struct DeallocResponsePayload();

pub const ALLOC_MESSAGE: MessageType = MessageType(2);
pub const DEALLOC_MESSAGE: MessageType = MessageType(3);

impl PayloadForMessageType for AllocPayload {
    const MESSAGE_TYPE: MessageType = ALLOC_MESSAGE;
}

impl PayloadForMessageType for AllocResponsePayload {
    const MESSAGE_TYPE: MessageType = ALLOC_MESSAGE;
}

impl PayloadForMessageType for DeallocPayload {
    const MESSAGE_TYPE: MessageType = DEALLOC_MESSAGE;
}

impl PayloadForMessageType for DeallocResponsePayload {
    const MESSAGE_TYPE: MessageType = DEALLOC_MESSAGE;
}

pub struct AllocMessage;

impl AllocMessage {
    pub fn request(size: usize, align: usize) -> Message {
        MessageAdapter::<AllocPayload>::message(&AllocPayload { size, align })
    }

    pub fn response(ptr: *mut u8) -> Message {
        MessageAdapter::<AllocResponsePayload>::message(&AllocResponsePayload(ptr))
    }

    pub fn parse_request(message: &Message) -> &AllocPayload {
        MessageAdapter::<AllocPayload>::payload(message)
    }

    pub fn parse_response(message: &Message) -> *mut u8 {
        MessageAdapter::<AllocResponsePayload>::payload(message).0
    }
}

pub struct DeallocMessage;

impl DeallocMessage {
    pub fn request(ptr: *mut u8) -> Message {
        MessageAdapter::<DeallocPayload>::message(&DeallocPayload(ptr))
    }

    pub fn response() -> Message {
        MessageAdapter::<DeallocResponsePayload>::message(&DeallocResponsePayload())
    }

    pub fn parse_request(message: &Message) -> *mut u8 {
        MessageAdapter::<DeallocPayload>::payload(message).0
    }
}
