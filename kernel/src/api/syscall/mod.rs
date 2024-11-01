use crate::{sys::syscall::number::Syscall, syscall};

pub fn sleep(seconds: f64) {
    let _ = unsafe { syscall!(Syscall::Sleep, usize::try_from(seconds.to_bits()).unwrap()) };
}
