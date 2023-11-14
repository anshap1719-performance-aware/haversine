use crate::os_timer::{os_timer_frequency, read_os_timer};
use libc::CLOCK_PROCESS_CPUTIME_ID;
use libc::{clock_gettime, timespec};
use std::io::Error;

pub fn read_cpu_timer() -> u64 {
    let mut time = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    if unsafe { clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut time) } == -1 {
        panic!("{:?}", Error::last_os_error());
    }

    time.tv_sec as u64 * 10_u64.pow(9) + time.tv_nsec as u64
}

pub fn estimate_cpu_timer_frequency() -> u64 {
    let millis_to_wait = 100_u64;
    let os_timer_frequency = os_timer_frequency();

    let cpu_timer_start = read_cpu_timer();
    let os_timer_start = read_os_timer();

    let mut os_timer_end = 0;
    let mut os_timer_elapsed = 0;

    let os_wait_time = os_timer_frequency * millis_to_wait / 1000;

    while os_timer_elapsed < os_wait_time {
        os_timer_end = read_os_timer();
        os_timer_elapsed = os_timer_end - os_timer_start
    }

    let cpu_timer_end = read_cpu_timer();
    let cpu_timer_elapsed = cpu_timer_end - cpu_timer_start;

    let mut cpu_frequency = 0;

    if os_timer_elapsed > 0 {
        cpu_frequency = os_timer_frequency * cpu_timer_elapsed / os_timer_elapsed;
    }

    cpu_frequency
}
