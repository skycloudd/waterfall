use super::{
    gdt,
    pic::{PICS, PIC_1_OFFSET},
    syscall,
};
use crate::{log, println};
use core::arch::naked_asm;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    instructions::{interrupts, port::Port},
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

const PIC1: u16 = 0x21;
const PIC2: u16 = 0xA1;

const fn default_irq_handler() {}

lazy_static! {
    pub static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = Mutex::new([default_irq_handler; 16]);
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler);

        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        }

        unsafe {
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault_handler)
                .set_stack_index(gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);
        }

        unsafe {
            idt[0x80]
                .set_handler_fn(core::mem::transmute::<
                    *mut fn(),
                    extern "x86-interrupt" fn(x86_64::structures::idt::InterruptStackFrame),
                >(wrapped_syscall_handler as *mut fn()))
                .set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }

        idt[interrupt_index(0)].set_handler_fn(irq0_handler);
        idt[interrupt_index(1)].set_handler_fn(irq1_handler);
        idt[interrupt_index(2)].set_handler_fn(irq2_handler);
        idt[interrupt_index(3)].set_handler_fn(irq3_handler);
        idt[interrupt_index(4)].set_handler_fn(irq4_handler);
        idt[interrupt_index(5)].set_handler_fn(irq5_handler);
        idt[interrupt_index(6)].set_handler_fn(irq6_handler);
        idt[interrupt_index(7)].set_handler_fn(irq7_handler);
        idt[interrupt_index(8)].set_handler_fn(irq8_handler);
        idt[interrupt_index(9)].set_handler_fn(irq9_handler);
        idt[interrupt_index(10)].set_handler_fn(irq10_handler);
        idt[interrupt_index(11)].set_handler_fn(irq11_handler);
        idt[interrupt_index(12)].set_handler_fn(irq12_handler);
        idt[interrupt_index(13)].set_handler_fn(irq13_handler);
        idt[interrupt_index(14)].set_handler_fn(irq14_handler);
        idt[interrupt_index(15)].set_handler_fn(irq15_handler);

        idt
    };
}

#[derive(Debug)]
#[repr(u8)]
pub enum Irq {
    Timer = 0,
    Keyboard = 1,
    Rtc = 8,

    Error = 12,
    Spurious = 13,
}

#[must_use]
pub const fn interrupt_index(irq: u8) -> u8 {
    PIC_1_OFFSET + irq
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            unsafe {
                PICS.lock().notify_end_of_interrupt(interrupt_index($irq));
            }
        }
    };
}

irq_handler!(irq0_handler, 0);
irq_handler!(irq1_handler, 1);
irq_handler!(irq2_handler, 2);
irq_handler!(irq3_handler, 3);
irq_handler!(irq4_handler, 4);
irq_handler!(irq5_handler, 5);
irq_handler!(irq6_handler, 6);
irq_handler!(irq7_handler, 7);
irq_handler!(irq8_handler, 8);
irq_handler!(irq9_handler, 9);
irq_handler!(irq10_handler, 10);
irq_handler!(irq11_handler, 11);
irq_handler!(irq12_handler, 12);
irq_handler!(irq13_handler, 13);
irq_handler!(irq14_handler, 14);
irq_handler!(irq15_handler, 15);

pub fn init() {
    IDT.load();

    log!("idt loaded");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("Stack Frame: {:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    println!("Stack Frame: {:#?}", stack_frame);
    println!("Error Code: {}", error_code);
    panic!();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Stack Frame: {:#?}", stack_frame);
    println!("Error Code: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT");
    println!("Stack Frame: {:#?}", stack_frame);
    println!("Error Code: {}", error_code);
    panic!();
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: STACK SEGMENT FAULT");
    println!("Stack Frame: {:#?}", stack_frame);
    println!("Error Code: {}", error_code);
    panic!();
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!("EXCEPTION: SEGMENT NOT PRESENT");
    println!("Stack Frame: {:#?}", stack_frame);
    println!("Error Code: {}", error_code);
    panic!();
}

macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        unsafe extern "sysv64" fn $w() {
            unsafe{
                naked_asm!(
                    "push rax",
                    "push rcx",
                    "push rdx",
                    "push rsi",
                    "push rdi",
                    "push r8",
                    "push r9",
                    "push r10",
                    "push r11",
                    "mov rsi, rsp", // Arg #2: register list
                    "mov rdi, rsp", // Arg #1: interupt frame
                    "add rdi, 9 * 8",
                    "call {}",
                    "pop r11",
                    "pop r10",
                    "pop r9",
                    "pop r8",
                    "pop rdi",
                    "pop rsi",
                    "pop rdx",
                    "pop rcx",
                    "pop rax",
                    "iretq",
                    sym $fn
                );
            }
        }
    };
}

wrap!(syscall_handler => wrapped_syscall_handler);

extern "sysv64" fn syscall_handler(_stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    let n = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    let result = syscall::dispatcher(n, arg1, arg2, arg3, arg4);

    regs.rax = result;

    unsafe { PICS.lock().notify_end_of_interrupt(0x80) };
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        clear_irq_mask(irq);
    });
}

pub fn set_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });

    let value = unsafe { port.read() } | (1 << (if irq < 8 { irq } else { irq - 8 }));
    unsafe { port.write(value) };
}

pub fn clear_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });

    let value = unsafe { port.read() } & !(1 << if irq < 8 { irq } else { irq - 8 });
    unsafe { port.write(value) };
}

#[repr(align(8), C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}
