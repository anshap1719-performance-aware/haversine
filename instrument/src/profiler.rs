use crate::cpu_timer::{estimate_cpu_frequency, read_cpu_timer};
use std::time::Duration;

pub struct GlobalProfiler {
    start: u64,
    end: Option<u64>,
    children: Vec<ProfilerEntry>,
}

#[derive(Debug, Clone)]
pub enum ProfilerEntry {
    Function(ProfilerEntryData),
    CodeBlock(ProfilerEntryData),
}

#[derive(Debug, Clone)]
pub struct ProfilerEntryData {
    identifier: &'static str,
    start: u64,
    end: Option<u64>,
    index: usize,
    parent_index: Option<usize>,
    ancestors: usize,
}

pub struct GlobalProfilerWrapper(pub GlobalProfiler);

pub static mut GLOBAL_PROFILER: GlobalProfilerWrapper = GlobalProfilerWrapper(GlobalProfiler {
    start: 0,
    end: None,
    children: Vec::new(),
});

pub static mut LAST_INDEX: Vec<usize> = vec![];

impl GlobalProfilerWrapper {
    pub fn start() {
        let profiler = unsafe { &mut GLOBAL_PROFILER.0 };

        profiler.start = read_cpu_timer();
    }

    pub fn end() {
        let profiler = unsafe { &mut GLOBAL_PROFILER.0 };

        profiler.end = Some(read_cpu_timer());

        GlobalProfilerWrapper::print_results();
    }

    pub fn push(entry: &mut ProfilerEntry) {
        let profiler = unsafe { &mut GLOBAL_PROFILER.0 };

        profiler.children.push(entry.clone());

        let index = profiler.children.len() - 1;
        entry.inner_mut().index = index;

        unsafe { LAST_INDEX.push(index) }
    }

    pub fn print_results() {
        let profiler = unsafe { &GLOBAL_PROFILER.0 };

        let start = profiler.start;
        let end = profiler
            .end
            .expect("Didn't finish profiling before trying to print results");
        let children = &profiler.children;

        let total = end - start;
        let ratio = 100.0 / total as f64;

        let cpu_frequency = estimate_cpu_frequency();

        for child in children {
            let tab = "\t";
            let prefix = tab.repeat(child.inner().ancestors);

            let runtime = child.compute_runtime();

            let time = Duration::from_secs_f64(runtime as f64 / cpu_frequency as f64);
            let percentage = ratio * runtime as f64;

            println!(
                "{prefix}{} took {time:.2?} ({percentage:.4}%)",
                child.identifier(),
            );
        }

        println!("program took {} cycles", end - start);
    }
}

use ProfilerEntry::*;

impl ProfilerEntry {
    #[must_use]
    pub fn identifier(&self) -> &'static str {
        match self {
            CodeBlock(ProfilerEntryData { identifier, .. })
            | Function(ProfilerEntryData { identifier, .. }) => identifier,
        }
    }

    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            CodeBlock(ProfilerEntryData { index, .. })
            | Function(ProfilerEntryData { index, .. }) => *index,
        }
    }

    #[must_use]
    pub fn inner(&self) -> &ProfilerEntryData {
        match self {
            CodeBlock(data) | Function(data) => data,
        }
    }

    pub fn inner_mut(&mut self) -> &mut ProfilerEntryData {
        match self {
            CodeBlock(data) | Function(data) => data,
        }
    }

    #[must_use]
    pub fn parent(&self) -> Option<usize> {
        match self {
            CodeBlock(ProfilerEntryData { parent_index, .. })
            | Function(ProfilerEntryData { parent_index, .. }) => *parent_index,
        }
    }

    pub fn end(self) {
        match self {
            Function(data) | CodeBlock(data) => {
                let profiler = unsafe { &mut GLOBAL_PROFILER.0 };

                if let Some(entry) = profiler.children.get_mut(data.index) {
                    let entry = entry.inner_mut();
                    entry.end = Some(read_cpu_timer());
                } else {
                    panic!("Invalid entry: {data:?}");
                }

                unsafe {
                    LAST_INDEX.pop();
                }
            }
        }
    }

    #[must_use]
    pub fn compute_runtime(&self) -> u64 {
        if let Some(end) = self.inner().end {
            end - self.inner().start
        } else {
            panic!("Profiler ended but entry didn't finish: {self:?}");
        }
    }
}

impl ProfilerEntryData {
    #[must_use]
    pub fn init(identifier: &'static str) -> Self {
        unsafe {
            Self {
                identifier,
                start: read_cpu_timer(),
                end: None,
                index: 0,
                parent_index: LAST_INDEX.last().copied(),
                ancestors: LAST_INDEX.len(),
            }
        }
    }
}
