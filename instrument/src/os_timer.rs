use mach2::mach_time::{mach_absolute_time, mach_timebase_info};

pub fn os_timer_frequency() -> u64 {
    let mut mach_timebase_info_t = mach_timebase_info { numer: 0, denom: 0 };

    unsafe {
        mach_timebase_info(&mut mach_timebase_info_t);
    }

    let result = 1. / (mach_timebase_info_t.numer as f64 / mach_timebase_info_t.denom as f64)
        * (10_u64.pow(9) as f64);

    result as u64
}

pub fn read_os_timer() -> u64 {
    let mut time = 0;

    unsafe {
        time = mach_absolute_time();
    }

    time
}
