#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig};
use x86_64::instructions::port::Port;

extern crate alloc;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        Port::new(0xf4).write(exit_code as u32);
    }
}
