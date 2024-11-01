use crate::{
    log,
    sys::{self, cmos::Cmos},
};
use alloc::string::{String, ToString};
use chrono::DateTime;
use num_traits::float::FloatCore;
use x86_64::instructions::interrupts;

const DAYS_BEFORE_MONTH: [u64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

pub fn sleep(seconds: f64) {
    let start = uptime();

    #[allow(clippy::while_float)]
    while uptime() - start < seconds {
        halt();
    }
}

pub fn halt() {
    let disabled = !interrupts::are_enabled();

    interrupts::enable_and_hlt();

    if disabled {
        interrupts::disable();
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn uptime() -> f64 {
    sys::time::time_between_ticks() * sys::time::ticks() as f64
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn realtime() -> f64 {
    let rtc = Cmos::new().rtc();

    let timestamp = 86400 * days_before_year(u64::from(rtc.year))
        + 86400 * days_before_month(u64::from(rtc.year), u64::from(rtc.month))
        + 86400 * u64::from(rtc.day - 1)
        + 3600 * u64::from(rtc.hour)
        + 60 * u64::from(rtc.minute)
        + u64::from(rtc.second);

    let fract = sys::time::time_between_ticks()
        * (sys::time::ticks() - sys::time::last_rtc_update()) as f64;

    (timestamp as f64) + fract
}

fn days_before_year(year: u64) -> u64 {
    (1970..year).fold(0, |days, y| days + if is_leap_year(y) { 366 } else { 365 })
}

fn days_before_month(year: u64, month: u64) -> u64 {
    let leap_day = is_leap_year(year) && month > 2;
    DAYS_BEFORE_MONTH[(usize::try_from(month).unwrap()) - 1] + u64::from(leap_day)
}

const fn is_leap_year(year: u64) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else {
        year % 400 == 0
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn format(dt: f64) -> String {
    let dt = DateTime::from_timestamp(dt.trunc() as i64, (dt.fract() * 1e9) as u32).unwrap();

    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}

pub fn init() {
    let rtc = format(realtime());
    log!("RTC {}", rtc);
}
