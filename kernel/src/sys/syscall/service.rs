use crate::sys::{clock, time};

#[allow(clippy::cast_precision_loss)]
pub fn sleep(ms: u64) {
    time::sleep(ms as f64 / 1000.0);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn uptime() -> u64 {
    clock::uptime() as u64
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn realtime() -> u64 {
    clock::realtime() as u64
}
