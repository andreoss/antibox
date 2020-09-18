use antibox_core::libc;

pub fn read_proc(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

pub fn load_average() -> Option<[f64; 3]> {
    let mut avgs = [0f64; 3];
    let n = unsafe { libc::getloadavg(avgs.as_mut_ptr(), 3) };
    if n == 3 {
        Some(avgs)
    } else {
        None
    }
}

pub fn read_proc_meminfo() -> Option<(u64, u64)> {
    let data = read_proc("/proc/meminfo")?;
    let mut total = 0u64;
    let mut free = 0u64;
    for line in data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        match parts[0] {
            "MemTotal:" => total = parts[1].parse().ok()?,
            "MemFree:" => free = parts[1].parse().ok()?,
            "MemAvailable:" => {
                free = parts[1].parse().ok()?;
                break;
            }
            _ => {}
        }
    }
    if total > 0 {
        Some((total, free))
    } else {
        None
    }
}

#[cfg(test)]
#[path = "proc_reader_tests.rs"]
mod tests;
