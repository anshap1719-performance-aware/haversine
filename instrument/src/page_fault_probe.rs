use instrument::page_faults::{get_absolute_page_faults_count, get_page_size};

#[allow(clippy::cast_possible_truncation)]
fn main() {
    let page_size = get_page_size();
    let page_count = 2048_u64;

    let memory_size = page_size * page_count;

    println!("Page Count,Touch Count,Fault Count,Extra Count");

    let mut memory = memmap::MmapMut::map_anon(usize::try_from(memory_size).unwrap()).unwrap();

    for i in 0..page_count {
        let touch_count = i + 1;
        let touch_size = page_size * touch_count;

        let start_fault_count = get_absolute_page_faults_count().unwrap();

        for index in 0..=touch_size {
            memory[index as usize] = index as u8;
        }

        let end_fault_count = get_absolute_page_faults_count().unwrap();
        let fault_count = end_fault_count - start_fault_count;

        println!(
            "{page_count},{touch_count},{fault_count},{}",
            fault_count - touch_count
        );
    }
}
