#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(naked_functions)]

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};

extern crate alloc;

pub mod api;
pub mod sys;

pub fn init(boot_info: &'static mut BootInfo) {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        sys::framebuffer::init(framebuffer);

        log!("framebuffer initialized");
    }

    sys::gdt::init();
    sys::idt::init();

    sys::pic::init();

    sys::time::init();
    sys::serial::init();
    sys::task::keyboard::init();

    sys::memory::init(
        boot_info.physical_memory_offset.into_option().unwrap(),
        &boot_info.memory_regions,
    );
    sys::clock::init();
    sys::cpu::init();

    sys::ata::init();

    log!("kernel initialized\n");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};
