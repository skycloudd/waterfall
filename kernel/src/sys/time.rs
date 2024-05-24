use crate::sys::{cmos::Cmos, idt::Irq};
use crate::{log, sys};
use core::hint::spin_loop;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

pub const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0;
const PIT_DIVIDER: usize = 1 << 16;
#[allow(clippy::cast_precision_loss)]
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);
static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

#[must_use]
pub const fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

pub fn last_rtc_update() -> usize {
    LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_mm_lfence() };
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn sleep(seconds: f64) {
    let start = sys::clock::uptime();
    while sys::clock::uptime() - start < seconds {
        halt();
    }
}

pub fn nanowait(nanoseconds: u64) {
    let start = rdtsc();
    let delta = nanoseconds * CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
    while rdtsc() - start < delta {
        spin_loop();
    }
}

pub fn set_pit_frequency_divider(divider: u16, channel: u8) {
    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40 + u16::from(channel));
        let operating_mode = 6;
        let access_mode = 3;

        unsafe { cmd.write((channel << 6) | (access_mode << 4) | operating_mode) };
        unsafe { data.write(bytes[0]) };
        unsafe { data.write(bytes[1]) };
    });
}

pub fn pit_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn rtc_interrupt_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    Cmos::new().notify_end_of_interrupt();
}

pub fn init() {
    let divider = if PIT_DIVIDER < 65536 { PIT_DIVIDER } else { 0 };
    let channel = 0;
    set_pit_frequency_divider(u16::try_from(divider).unwrap(), channel);
    sys::idt::set_irq_handler(Irq::Timer as u8, pit_interrupt_handler);

    log!("pit initialized");

    sys::idt::set_irq_handler(Irq::Rtc as u8, rtc_interrupt_handler);
    Cmos::new().enable_update_interrupt();

    let calibration_time = 250_000;
    let a = rdtsc();
    #[allow(clippy::cast_precision_loss)]
    sleep(calibration_time as f64 / 1e6);
    let b = rdtsc();
    CLOCKS_PER_NANOSECOND.store((b - a) / calibration_time, Ordering::Relaxed);

    log!("clock initialized");
}
