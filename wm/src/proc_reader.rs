use antibox_core::libc;

pub fn read_proc(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

pub fn pid_command(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    let raw = std::fs::read(format!("/proc/{}/cmdline", pid)).ok()?;
    let text = String::from_utf8_lossy(&raw).replace('\0', " ");
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
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

#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
const CTL_KERN: libc::c_int = 1;
#[cfg(not(target_os = "linux"))]
const CTL_VM: libc::c_int = 2;
#[cfg(not(target_os = "linux"))]
const KERN_CPTIME: libc::c_int = 40;
#[cfg(not(target_os = "linux"))]
const VM_UVMEXP: libc::c_int = 4;

#[cfg(not(target_os = "linux"))]
fn sysctl_bytes(mib: &[libc::c_int]) -> Option<Vec<u8>> {
    let mut len: usize = 0;
    let rc = unsafe {
        libc::sysctl(
            mib.as_ptr(),
            mib.len() as libc::c_uint,
            std::ptr::null_mut(),
            &mut len,
            std::ptr::null(),
            0,
        )
    };
    if rc != 0 || len == 0 {
        return None;
    }
    let mut buf = vec![0u8; len];
    let mut len2 = len;
    let rc = unsafe {
        libc::sysctl(
            mib.as_ptr(),
            mib.len() as libc::c_uint,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut len2,
            std::ptr::null(),
            0,
        )
    };
    if rc != 0 {
        return None;
    }
    buf.truncate(len2);
    Some(buf)
}

#[cfg(not(target_os = "linux"))]
fn i32_at(buf: &[u8], off: usize) -> Option<i32> {
    buf.get(off..off + 4)
        .map(|s| i32::from_ne_bytes(s.try_into().unwrap()))
}

#[cfg(not(target_os = "linux"))]
pub struct UvmMem {
    pub total: u64,
    pub free: u64,
    pub active: u64,
    pub inactive: u64,
    pub wired: u64,
}

#[cfg(not(target_os = "linux"))]
pub fn read_uvmexp() -> Option<UvmMem> {
    let buf = sysctl_bytes(&[CTL_VM, VM_UVMEXP])?;
    let pagesize = i32_at(&buf, 0)? as u64;
    let npages = i32_at(&buf, 12)? as u64;
    let free = i32_at(&buf, 16)? as u64;
    let active = i32_at(&buf, 20)? as u64;
    let inactive = i32_at(&buf, 24)? as u64;
    let wired = i32_at(&buf, 32)? as u64;
    if pagesize == 0 || npages == 0 {
        return None;
    }
    Some(UvmMem {
        total: npages * pagesize,
        free: free * pagesize,
        active: active * pagesize,
        inactive: inactive * pagesize,
        wired: wired * pagesize,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn read_proc_meminfo() -> Option<(u64, u64)> {
    let m = read_uvmexp()?;
    Some((m.total, m.free))
}

#[cfg(not(target_os = "linux"))]
pub fn read_cptime() -> Option<[u64; 6]> {
    let buf = sysctl_bytes(&[CTL_KERN, KERN_CPTIME])?;
    let n = buf.len() / 8;
    let mut out = [0u64; 6];
    for (i, slot) in out.iter_mut().enumerate() {
        if i >= n {
            break;
        }
        let bytes: [u8; 8] = buf[i * 8..i * 8 + 8].try_into().ok()?;
        *slot = i64::from_ne_bytes(bytes) as u64;
    }
    Some(out)
}

#[cfg(test)]
#[path = "proc_reader_tests.rs"]
mod tests;
