pub mod allocator;
pub mod clock;
pub mod cmos;
pub mod cpu;
pub mod framebuffer;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod pic;
pub mod serial;
pub mod syscall;
pub mod task;
pub mod time;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        {
            let uptime = $crate::sys::clock::uptime();

            $crate::println!("[{:.4}] {}", uptime, format_args!($($arg)*));
            $crate::println_serial!("[{:.4}] {}", uptime, format_args!($($arg)*));
        }
    }
}
