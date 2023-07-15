use core::marker::PhantomData;
use klib::ipc::{Message, MessageType};

pub struct MessageAdapter<Payload: PayloadForMessageType>(PhantomData<Payload>);

pub trait PayloadForMessageType {
    const MESSAGE_TYPE: MessageType;
}

impl<Payload: PayloadForMessageType> MessageAdapter<Payload> {
    pub fn payload(message: &Message) -> &Payload {
        unsafe { &*(message.raw.as_ptr() as *const Payload) }
    }

    pub fn message(payload: &Payload) -> Message {
        Message {
            message_type: Payload::MESSAGE_TYPE,
            src_tid: 0,
            raw: unsafe { *(payload as *const Payload as *const [u8; 24]) },
        }
    }
}
