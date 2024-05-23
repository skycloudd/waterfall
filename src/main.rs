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
    memory::BootInfoFrameAllocator,
    println,
    task::{executor::Executor, keyboard, Task},
};

extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    waterfall::init();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { waterfall::memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    waterfall::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

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
