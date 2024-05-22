#![no_std]
#![no_main]
#![warn(clippy::multiple_unsafe_ops_per_block)]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        let first_byte = unsafe { vga_buffer.offset(i as isize * 2) };
        let second_byte = unsafe { vga_buffer.offset(i as isize * 2 + 1) };

        unsafe {
            *first_byte = byte;
        }
        unsafe {
            *second_byte = 0xb;
        }
    }

    panic!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
