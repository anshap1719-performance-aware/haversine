use std::io;
use std::process::Command;
use std::str::from_utf8;

pub fn get_absolute_page_faults_count() -> Result<u64, io::Error> {
    let pid = std::process::id();
    let output = Command::new("top")
        .arg("-e")
        .arg("-ncols")
        .arg("1")
        .arg("-stats")
        .arg("faults")
        .arg("-l")
        .arg("1")
        .arg("-pid")
        .arg(format!("{pid}"))
        .output()?
        .stdout;

    let output_string =
        from_utf8(&output).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    let output_vec = output_string
        .lines()
        .skip(11)
        .take(2)
        .collect::<Vec<&str>>();

    if let [key, value] = output_vec.as_slice() {
        if key.trim() == "FAULTS" {
            return value
                .trim()
                .parse()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
        }
    }

    Err(io::Error::from(io::ErrorKind::InvalidInput))
}

pub static mut PAGE_SIZE: u64 = 0;

pub fn get_page_size() -> u64 {
    unsafe {
        if PAGE_SIZE != 0 {
            return PAGE_SIZE;
        }

        let output = Command::new("pagesize").output().unwrap().stdout;
        let output_str = from_utf8(&output).unwrap().trim();
        PAGE_SIZE = output_str.parse().unwrap();

        PAGE_SIZE
    }
}
