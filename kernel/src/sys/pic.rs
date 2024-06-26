use crate::log;
use pic8259::ChainedPics;
use spin::Mutex;

pub fn init() {
    unsafe { PICS.lock().initialize() };

    log!("pic initialized");

    x86_64::instructions::interrupts::enable();

    log!("interrupts enabled");
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
