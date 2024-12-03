pub fn sleep(seconds: f64) {
    crate::sys::clock::sleep(seconds);
}
