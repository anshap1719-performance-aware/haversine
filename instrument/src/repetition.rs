use crate::cpu_timer::read_cpu_timer;
use crate::page_faults::{get_absolute_page_faults_count, get_page_size};
use crate::stats::{RunTime, Throughput};
use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, QueueableCommand};
use std::io::{stdout, Write};

#[derive(Default, Copy, Clone, Eq, PartialEq)]
enum TestState {
    #[default]
    Uninitialised,
    Testing,
    Error,
    Completed,
}

pub struct TestResult {
    pub test_count: u64,
    pub total_time: u64,
    pub max_time: u64,
    pub min_time: u64,
    pub page_faults: u64,
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            test_count: 0,
            total_time: 0,
            max_time: 0,
            min_time: u64::MAX,
            page_faults: 0,
        }
    }
}

pub struct RepetitionTester {
    target_byte_count: u64,
    cpu_timer_frequency: u64,
    try_for_time: u64,
    tests_started_at: u64,
    open_block_count: u64,
    closed_block_count: u64,
    time_accumulated_this_test: i128,
    bytes_accumulated_this_test: u64,
    faults_accumulated_this_test: i128,
    state: TestState,
    results: TestResult,
}

impl RepetitionTester {
    #[must_use]
    pub fn new(
        target_byte_count: u64,
        cpu_timer_frequency: u64,
        seconds_to_try: Option<u64>,
    ) -> Self {
        Self {
            target_byte_count,
            cpu_timer_frequency,
            try_for_time: cpu_timer_frequency * seconds_to_try.unwrap_or(10),
            tests_started_at: read_cpu_timer(),
            open_block_count: 0,
            closed_block_count: 0,
            time_accumulated_this_test: 0,
            bytes_accumulated_this_test: 0,
            faults_accumulated_this_test: 0,
            state: TestState::Testing,
            results: TestResult::default(),
        }
    }

    pub fn new_wave(
        &mut self,
        target_byte_count: u64,
        cpu_timer_frequency: u64,
        seconds_to_try: Option<u64>,
    ) {
        self.state = TestState::Testing;
        if self.target_byte_count != target_byte_count {
            self.error("Target bytes count changed");
        }

        if self.cpu_timer_frequency != cpu_timer_frequency {
            self.error("CPU timer frequency changed");
        }

        self.try_for_time = cpu_timer_frequency * seconds_to_try.unwrap_or(10);
        self.tests_started_at = read_cpu_timer();
    }

    pub fn begin(&mut self) {
        self.open_block_count += 1;
        self.time_accumulated_this_test -= i128::from(read_cpu_timer());

        let page_faults = get_absolute_page_faults_count().unwrap();
        self.faults_accumulated_this_test -= i128::from(page_faults);
    }

    pub fn end(&mut self) {
        self.closed_block_count += 1;
        self.time_accumulated_this_test += i128::from(read_cpu_timer());

        let page_faults = get_absolute_page_faults_count().unwrap();
        self.faults_accumulated_this_test += i128::from(page_faults);
    }

    pub fn count_bytes(&mut self, bytes: u64) {
        self.bytes_accumulated_this_test += bytes;
    }

    fn error(&mut self, error: &str) {
        self.state = TestState::Error;
        eprintln!("{error}");
    }

    #[must_use]
    pub fn loop_test(&mut self) -> bool {
        if self.state != TestState::Testing {
            return false;
        }

        let current_time = read_cpu_timer();
        if self.open_block_count > 0 {
            if self.open_block_count != self.closed_block_count {
                self.error("Unbalanced begin & end encountered");
            }

            if self.bytes_accumulated_this_test != self.target_byte_count {
                self.error(&format!(
                    "Processed byte count mismatch: {} vs {}",
                    self.bytes_accumulated_this_test, self.target_byte_count,
                ));
            }

            if self.state == TestState::Testing {
                let results = &mut self.results;

                // first iteration
                if self.open_block_count == 1 {
                    results.page_faults = u64::try_from(self.faults_accumulated_this_test).unwrap();
                }

                let elapsed =
                    u64::try_from(self.time_accumulated_this_test).expect("Negative time spent");
                results.test_count += 1;
                results.total_time += elapsed;
                results.max_time = results.max_time.max(elapsed);

                if results.min_time > elapsed {
                    results.min_time = elapsed;
                    self.tests_started_at = current_time;

                    self.print_new_stats();
                }

                self.reset_after_iteration();
            }
        }

        if current_time - self.tests_started_at > self.try_for_time {
            self.state = TestState::Completed;
            self.print_results();
        }

        self.state == TestState::Testing
    }

    fn reset_after_iteration(&mut self) {
        self.open_block_count = 0;
        self.closed_block_count = 0;
        self.time_accumulated_this_test = 0;
        self.bytes_accumulated_this_test = 0;
        self.faults_accumulated_this_test = 0;
    }

    fn print_new_stats(&self) {
        let mut stdout = stdout();

        let run_time =
            RunTime::with_timer_frequency(self.results.min_time, self.cpu_timer_frequency);
        let throughput = Throughput::new(self.bytes_accumulated_this_test, run_time);

        stdout
            .queue(terminal::Clear(ClearType::CurrentLine))
            .unwrap();
        stdout.queue(cursor::MoveToColumn(0)).unwrap();
        stdout.queue(cursor::SavePosition).unwrap();
        stdout
            .write_all(format!("Min: Took {run_time} at {throughput}").as_bytes())
            .unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(ClearType::FromCursorDown))
            .unwrap();
    }

    fn print_results(&self) {
        let min_run_time =
            RunTime::with_timer_frequency(self.results.min_time, self.cpu_timer_frequency);
        let max_run_time =
            RunTime::with_timer_frequency(self.results.max_time, self.cpu_timer_frequency);
        let average_run_time = RunTime::with_timer_frequency(
            self.results.total_time / self.results.test_count,
            self.cpu_timer_frequency,
        );

        let max_throughput = Throughput::new(self.target_byte_count, min_run_time);
        let min_throughput = Throughput::new(self.target_byte_count, max_run_time);
        let average_throughput = Throughput::new(self.target_byte_count, average_run_time);
        let page_faults = self.results.page_faults;
        let page_size = get_page_size();

        let page_fault_memory = (page_size * page_faults) as f64 / 1024. / 1024.;

        println!("Min: {min_run_time} at {max_throughput}");
        println!("Max: {max_run_time} at {min_throughput}");
        println!("Avg: {average_run_time} at {average_throughput}");
        println!("Page faults: {page_faults} ({page_fault_memory:.2}MB)");
    }

    #[must_use]
    pub fn results(&self) -> &TestResult {
        &self.results
    }
}
