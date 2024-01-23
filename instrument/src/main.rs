use instrument::cpu_timer::estimate_cpu_frequency;
use instrument::repetition::RepetitionTester;
use std::fs::File;
use std::io::{Read, Seek};
use std::os::unix::fs::MetadataExt;

fn main() {
    let mut file = File::open("test.json").unwrap();

    let mut repetition_tester = RepetitionTester::new(
        file.metadata().unwrap().size(),
        estimate_cpu_frequency(),
        Some(10),
    );

    while repetition_tester.loop_test() {
        repetition_tester.begin();

        let mut container = Vec::with_capacity(
            file.metadata()
                .ok()
                .map_or(0, |file| usize::try_from(file.len()).unwrap()),
        );

        let bytes_read = file.read_to_end(&mut container).unwrap();
        repetition_tester.count_bytes(bytes_read as u64);

        repetition_tester.end();

        file.rewind().unwrap();
    }
}
