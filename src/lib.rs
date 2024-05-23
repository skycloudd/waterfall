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

pub mod allocator;
mod gdt;
mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

extern crate alloc;

pub fn init() {
    gdt::init();

    interrupts::init_idt();

    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
