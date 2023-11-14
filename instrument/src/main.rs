use instrument::cpu_timer::{estimate_cpu_timer_frequency, read_cpu_timer};
use instrument::os_timer::{os_timer_frequency, read_os_timer};

fn main() {
    println!("Frequency: {}", os_timer_frequency());
    println!("Ticks: {}", read_os_timer());
    println!("CPU Ticks: {}", read_cpu_timer());
    println!("CPU Timer Frequency: {}", estimate_cpu_timer_frequency());
}
