use crate::cpu_timer::{estimate_cpu_frequency, read_cpu_timer};
use std::collections::HashMap;
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
    children_elapsed: u64,
}

pub struct ProfilerMetricEntry {
    identifier: &'static str,
    elapsed_inclusive: u64,
    elapsed_exclusive: u64,
    hit_count: u64,
    ancestors_count: usize,
    insert_index: usize,
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

        profiler.children = Vec::with_capacity(2048);
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

        let mut child_map = HashMap::<&str, ProfilerMetricEntry>::new();
        let mut insert_index = 0;

        for child in children {
            let total_runtime = child.compute_runtime();
            let children_runtime = child.get_child_elapsed();

            if let Some(child_entry) = child_map.get_mut(child.identifier()) {
                child_entry.hit_count += 1;
                child_entry.elapsed_inclusive += total_runtime;
                child_entry.elapsed_exclusive += total_runtime - children_runtime;
            } else {
                child_map.insert(
                    child.identifier(),
                    ProfilerMetricEntry {
                        identifier: child.identifier(),
                        hit_count: 1,
                        elapsed_inclusive: total_runtime,
                        elapsed_exclusive: total_runtime - children_runtime,
                        ancestors_count: child.inner().ancestors,
                        insert_index,
                    },
                );

                insert_index += 1;
            }
        }

        let mut entries = child_map.values().collect::<Vec<_>>();
        entries.sort_by(|a, b| a.insert_index.cmp(&b.insert_index));

        for value in entries {
            let tab = "\t";
            let prefix = tab.repeat(value.ancestors_count);

            let time =
                Duration::from_secs_f64(value.elapsed_inclusive as f64 / cpu_frequency as f64);

            let percentage = ratio * value.elapsed_exclusive as f64;

            if value.elapsed_exclusive.abs_diff(value.elapsed_inclusive) < 100 {
                println!(
                    "{prefix}{}[{}] took {time:.2?} ({percentage:.4}%)",
                    value.identifier, value.hit_count
                );
            } else {
                let percentage_with_children = ratio * value.elapsed_inclusive as f64;

                println!(
                    "{prefix}{}[{}] took {time:.2?} ({percentage:.4}% | {percentage_with_children:.4}% w/ children)",
                    value.identifier,
                    value.hit_count
                );
            }
        }

        let program_runtime = Duration::from_secs_f64((end - start) as f64 / cpu_frequency as f64);
        println!(
            "program took {program_runtime:.2?} ({} cycles)",
            end - start
        );
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
                    let end = read_cpu_timer();
                    entry.end = Some(end);

                    let elapsed = end - entry.start;

                    unsafe {
                        LAST_INDEX.pop();
                    }

                    if let Some(parent_index) = entry.parent_index {
                        if let Some(parent) = profiler.children.get_mut(parent_index) {
                            parent.add_child_elapsed(elapsed);
                        }
                    }
                } else {
                    panic!("Invalid entry: {data:?}");
                }
            }
        }
    }

    pub fn add_child_elapsed(&mut self, elapsed: u64) {
        match self {
            CodeBlock(data) | Function(data) => data.children_elapsed += elapsed,
        }
    }

    #[must_use]
    pub fn get_child_elapsed(&self) -> u64 {
        match self {
            CodeBlock(data) | Function(data) => data.children_elapsed,
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
                children_elapsed: 0,
            }
        }
    }
}
