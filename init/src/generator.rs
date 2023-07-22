use crate::init::ConsoleMessage;
use ::syscall::error::print_error;
use alloc::vec::Vec;
use core::mem;
use core::ops::{Generator, GeneratorState};
use core::pin::Pin;
use klib::ipc::{self, MessageType, Notifications};
use klib::result::KResult;
use syscall::syscall;

#[repr(u32)]
enum GeneratorCommand {
    Continue,
    Complete,
    Sleep(u32),
}

#[repr(u32, align(4))]
enum GeneratorResponse {
    None,
    Message(ipc::Message),
}

fn delayed_writer() -> impl Generator<GeneratorResponse, Yield = GeneratorCommand> {
    |response: GeneratorResponse| {
        let message = if let GeneratorResponse::Message(message) = response {
            message
        } else {
            unreachable!()
        };
        let src_text = ConsoleMessage::text_of(&message);
        syscall::console_write(src_text);
        let text: Vec<u8> = src_text.iter().cloned().collect();
        print_error!(b"text: {}\n", unsafe {
            mem::transmute::<*const u8, u32>(text.as_ptr())
        });
        let text2: Vec<u8> = src_text.iter().cloned().collect();
        print_error!(b"text2: {}\n", unsafe {
            mem::transmute::<*const u8, u32>(text2.as_ptr())
        });
        yield GeneratorCommand::Sleep(100);
        syscall::console_write(&text);
    }
}

fn run_generator<G>(generator: &mut Option<G>, mut response: GeneratorResponse)
where
    G: Generator<GeneratorResponse, Yield = GeneratorCommand> + core::marker::Unpin,
{
    let mut command = GeneratorCommand::Continue;
    loop {
        (command, response) = match command {
            GeneratorCommand::Continue => {
                match Pin::new(&mut *generator)
                    .as_pin_mut()
                    .unwrap()
                    .resume(response)
                {
                    GeneratorState::Yielded(next_command) => {
                        (next_command, GeneratorResponse::None)
                    }
                    GeneratorState::Complete(_) => {
                        (GeneratorCommand::Complete, GeneratorResponse::None)
                    }
                }
            }
            GeneratorCommand::Complete => break,
            GeneratorCommand::Sleep(sleep_ms) => {
                syscall::set_timer(sleep_ms);
                break;
            }
        }
    }
}

pub fn console_task() {
    syscall::console_write(b"generator console task started\n");
    let mut generator = None;

    loop {
        match syscall::ipc_recv(0) {
            KResult::Ok(message) => match message.message_type {
                ConsoleMessage::CONSOLE_OUT => {
                    generator = Some(delayed_writer());
                    run_generator(&mut generator, GeneratorResponse::Message(message));
                }
                MessageType::NOTIFICATIONS => {
                    let notifications =
                        unsafe { *<*const _>::from(&message.raw).cast::<Notifications>() };
                    if notifications.is_timer() {
                        run_generator(&mut generator, GeneratorResponse::None);
                    }
                }
                _ => (),
            },
            err => print_error!(b"ipc_recv failed: {}\n", err.err_as_u32()),
        };
    }
}
