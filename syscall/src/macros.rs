pub use crate::error::print_error;

#[macro_export]
macro_rules! print_error {
    ($message:expr, $err:expr) => {
        print_error::<{ $message.len() + 8 }>($message, $err)
    };
}
