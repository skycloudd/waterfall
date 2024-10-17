use crate::log;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

#[macro_export]
macro_rules! print_serial {
    ($($arg:tt)*) => {
        $crate::sys::serial::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println_serial {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::print_serial!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print_serial!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        write!(SERIAL.lock(), "{args}").expect("printing to serial failed");
    });
}

lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });
}

pub fn init() {
    SERIAL.lock().init();

    log!("serial initialized");
}
