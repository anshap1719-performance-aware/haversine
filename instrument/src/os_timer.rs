use mach2::mach_time::{mach_absolute_time, mach_timebase_info};

#[must_use]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub fn os_timer_frequency() -> u64 {
    let mut mach_timebase_info_t = mach_timebase_info { numer: 0, denom: 0 };

    unsafe {
        mach_timebase_info(&mut mach_timebase_info_t);
    }

    let result = 1.
        / (f64::from(mach_timebase_info_t.numer) / f64::from(mach_timebase_info_t.denom))
        * (10_u64.pow(9) as f64);

    result as u64
}

#[must_use]
pub fn read_os_timer() -> u64 {
    unsafe { mach_absolute_time() }
}
