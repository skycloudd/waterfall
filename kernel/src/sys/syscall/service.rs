use crate::log;

pub fn sleep(seconds: f64) {
    log!("syscall: sleep {}", seconds);
    crate::sys::clock::sleep(seconds);
}
