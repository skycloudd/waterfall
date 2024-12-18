use crate::log;
use lazy_static::lazy_static;
use x86_64::{
    instructions::{
        segmentation::{Segment, CS},
        tables::load_tss,
    },
    registers::segmentation::{DS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

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

#[derive(Debug)]
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
            VirtAddr::from_ptr(&raw const STACK) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            VirtAddr::from_ptr(&raw const STACK) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            VirtAddr::from_ptr(&raw const STACK) + STACK_SIZE as u64
        };

        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(&raw const STACK) + STACK_SIZE as u64
        };

        tss
    };
}
