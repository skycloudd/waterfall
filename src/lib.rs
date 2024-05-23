#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::multiple_unsafe_ops_per_block)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]

use memory::BootInfoFrameAllocator;

pub mod allocator;
mod gdt;
mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

extern crate alloc;

/// Initializes the kernel.
///
/// # Panics
///
/// This function will panic if the heap initialization fails.
pub fn init(boot_info: &'static bootloader::BootInfo) {
    gdt::init();

    interrupts::init_idt();

    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
