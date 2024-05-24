#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::sys::task::executor::Executor;
use kernel::sys::task::{keyboard, Task};
use kernel::{println, BOOTLOADER_CONFIG};

extern crate alloc;

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    println!("welcome to waterfall v{}", env!("CARGO_PKG_VERSION"));
    println!("written in Rust");
    println!("by skycloudd");
    println!();

    let mut executor = Executor::new();

    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");

    kernel::hlt_loop();
}
