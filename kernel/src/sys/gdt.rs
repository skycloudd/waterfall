use crate::log;
use core::ptr::addr_of;
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{DS, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        let code = gdt.append(Descriptor::kernel_code_segment());

        let data = gdt.append(Descriptor::kernel_data_segment());

        let stack = gdt.append(Descriptor::kernel_data_segment());

        let user_code = gdt.append(Descriptor::user_code_segment());

        let user_data = gdt.append(Descriptor::user_data_segment());

        (
            gdt,
            Selectors {
                code,
                data,
                stack,
                tss,
                user_code,
                user_data,
            },
        )
    };
}

pub struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    stack: SegmentSelector,
    tss: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
}

pub fn init() {
    GDT.0.load();

    unsafe { CS::set_reg(GDT.1.code) };
    unsafe { DS::set_reg(GDT.1.data) };
    unsafe { SS::set_reg(GDT.1.stack) };

    unsafe { load_tss(GDT.1.tss) };

    log!("gdt loaded");
}

const STACK_SIZE: usize = 4096 * 5;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.privilege_stack_table[0] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(addr_of!(STACK)) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            VirtAddr::from_ptr(addr_of!(STACK)) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            VirtAddr::from_ptr(addr_of!(STACK)) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(addr_of!(STACK)) + STACK_SIZE as u64
        };

        tss
    };
}
