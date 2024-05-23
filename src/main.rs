#![no_std]
#![no_main]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]

use bootloader::{entry_point, BootInfo};
use waterfall::{
    println,
    task::{executor::Executor, keyboard, Task},
};

extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    waterfall::init(boot_info);

    println!("-- welcome to waterfall os --");

    let mut executor = Executor::new();

    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.run();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);

    waterfall::hlt_loop();
}
