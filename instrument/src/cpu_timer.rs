use crate::os_timer::{os_timer_frequency, read_os_timer};

#[cfg(target_arch = "x86_64")]
pub fn read_cpu_timer() -> u64 {
    unsafe {
        _rdtsc();
    }
}

#[cfg(target_arch = "aarch64")]
#[must_use]
pub fn read_cpu_timer() -> u64 {
    use mach2::mach_time::mach_absolute_time;

    unsafe { mach_absolute_time() }
}

#[must_use]
pub fn estimate_cpu_frequency() -> u64 {
    let millis_to_wait = 100_u64;
    let os_timer_frequency = os_timer_frequency();

    let cpu_timer_start = read_cpu_timer();
    let os_timer_start = read_os_timer();

    let mut os_timer_end;
    let mut os_timer_elapsed = 0;

    let os_wait_time = os_timer_frequency * millis_to_wait / 1000;

    while os_timer_elapsed < os_wait_time {
        os_timer_end = read_os_timer();
        os_timer_elapsed = os_timer_end - os_timer_start;
    }

    let cpu_timer_end = read_cpu_timer();
    let cpu_timer_elapsed = cpu_timer_end - cpu_timer_start;

    let mut cpu_frequency = 0;

    if os_timer_elapsed > 0 {
        cpu_frequency = os_timer_frequency * cpu_timer_elapsed / os_timer_elapsed;
    }

    cpu_frequency
}
