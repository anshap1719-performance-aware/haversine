use instrument::cpu_timer::estimate_cpu_frequency;
use instrument::repetition::RepetitionTester;

#[allow(clippy::cast_possible_truncation)]
fn main() {
    let gigabyte = 1024 * 1024 * 1024;
    let mut memory = vec![0; 1024 * 1024 * 1024].into_boxed_slice();

    for i in 0..gigabyte {
        memory[i] = i as u8;
    }

    // println!(
    //     "Cycles per loop: {}",
    //     estimate_cpu_frequency() as f64 / repetition_tester.results().min_time as f64
    // );
}
