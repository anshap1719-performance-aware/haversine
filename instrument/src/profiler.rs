use crate::cpu_timer::read_cpu_timer;

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

        for child in children {
            let prefix = if child.inner().parent_index.is_some() {
                "\t"
            } else {
                ""
            };

            println!(
                "{prefix}{} took {} cycles and is child of {:?}",
                child.identifier(),
                child.compute_runtime(),
                child.parent(),
            )
        }

        println!("program took {} cycles", end - start)
    }
}

impl ProfilerEntry {
    pub fn identifier(&self) -> &'static str {
        match self {
            ProfilerEntry::Function(ProfilerEntryData { identifier, .. }) => identifier,
            ProfilerEntry::CodeBlock(ProfilerEntryData { identifier, .. }) => identifier,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            ProfilerEntry::Function(ProfilerEntryData { index, .. }) => *index,
            ProfilerEntry::CodeBlock(ProfilerEntryData { index, .. }) => *index,
        }
    }

    pub fn inner(&self) -> &ProfilerEntryData {
        match self {
            ProfilerEntry::Function(data) => data,
            ProfilerEntry::CodeBlock(data) => data,
        }
    }

    pub fn inner_mut(&mut self) -> &mut ProfilerEntryData {
        match self {
            ProfilerEntry::Function(data) => data,
            ProfilerEntry::CodeBlock(data) => data,
        }
    }

    pub fn parent(&self) -> Option<usize> {
        match self {
            ProfilerEntry::Function(ProfilerEntryData { parent_index, .. }) => *parent_index,
            ProfilerEntry::CodeBlock(ProfilerEntryData { parent_index, .. }) => *parent_index,
        }
    }

    pub fn end(self) {
        match self {
            ProfilerEntry::Function(data) | ProfilerEntry::CodeBlock(data) => {
                let profiler = unsafe { &mut GLOBAL_PROFILER.0 };

                if let Some(entry) = profiler.children.get_mut(data.index) {
                    let entry = entry.inner_mut();
                    entry.end = Some(read_cpu_timer());
                } else {
                    panic!("Invalid entry: {:?}", data);
                }

                unsafe {
                    LAST_INDEX.pop();
                }
            }
        }
    }

    pub fn compute_runtime(&self) -> u64 {
        if let Some(end) = self.inner().end {
            end - self.inner().start
        } else {
            panic!("Profiler ended but entry didn't finish: {self:?}");
        }
    }
}

impl ProfilerEntryData {
    pub fn init(identifier: &'static str) -> Self {
        unsafe {
            Self {
                identifier,
                start: read_cpu_timer(),
                end: None,
                index: 0,
                parent_index: LAST_INDEX.last().copied(),
            }
        }
    }
}
