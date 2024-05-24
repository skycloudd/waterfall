use acpi::platform::interrupt::Apic;
use core::alloc::Allocator;
use x86_64::instructions::interrupts;

pub fn init<A: Allocator>(apic: Apic<A>) {
    interrupts::without_interrupts(|| {});
}
