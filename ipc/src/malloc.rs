use klib::ipc::{Message, MessageType};
use syscall::payload::{MessageAdapter, PayloadForMessageType};

pub struct AllocPayload {
    pub size: usize,
    pub align: usize,
}

pub struct AllocResponsePayload(pub *mut u8);

impl PayloadForMessageType for AllocPayload {
    const MESSAGE_TYPE: MessageType = MessageType(2);
}

impl PayloadForMessageType for AllocResponsePayload {
    const MESSAGE_TYPE: MessageType = MessageType(3);
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
