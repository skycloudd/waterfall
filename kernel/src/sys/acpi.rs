use crate::log;

use super::memory;
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::PhysAddr;

#[must_use]
pub fn init(rsdp_addr: u64) -> AcpiTables<Handler> {
    let handler = Handler;

    let res =
        unsafe { AcpiTables::from_rsdp(handler, usize::try_from(rsdp_addr).unwrap()) }.unwrap();

    log!("ACPI initialized");

    res
}

#[derive(Clone, Copy, Debug)]
pub struct Handler;

impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = memory::phys_to_virt(PhysAddr::new(physical_address as u64));

        unsafe {
            PhysicalMapping::new(
                physical_address,
                NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
                size,
                size,
                Self,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}
