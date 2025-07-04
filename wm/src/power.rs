#![allow(unsafe_code)]
pub const MAX_BATTERIES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryInfo {
    pub percent: i32,
    pub charging: bool,
    pub discharging: bool,
    pub energy_now: Option<u64>,
    pub energy_full: Option<u64>,
    pub power_now: Option<u64>,
    pub minutes_left: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub batteries: Vec<BatteryInfo>,
}

pub fn read_status() -> PowerStatus {
    #[cfg(target_os = "linux")]
    {
        linux::read()
    }
    #[cfg(target_os = "openbsd")]
    {
        openbsd::read()
    }
    #[cfg(not(any(target_os = "linux", target_os = "openbsd")))]
    {
        PowerStatus::default()
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{BatteryInfo, PowerStatus, MAX_BATTERIES};

    const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

    pub(crate) fn read() -> PowerStatus {
        PowerStatus {
            ac_online: detect_ac(),
            batteries: scan_batteries(),
        }
    }

    fn read_num<T: std::str::FromStr>(path: &str) -> Option<T> {
        std::fs::read_to_string(path).ok()?.trim().parse().ok()
    }

    fn scan_batteries() -> Vec<BatteryInfo> {
        let Ok(entries) = std::fs::read_dir(POWER_SUPPLY_DIR) else { return Vec::new() };
        let mut names: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") || name.starts_with("battery") || name.starts_with("bat") {
                names.push(name);
            }
        }
        names.sort();
        names.truncate(MAX_BATTERIES);
        let mut batteries: Vec<BatteryInfo> = Vec::new();
        for name in names {
            let base = format!("{POWER_SUPPLY_DIR}/{name}");
            let capacity = read_num::<i32>(&format!("{base}/capacity"));
            let status = std::fs::read_to_string(format!("{base}/status"))
                .ok()
                .map(|s| s.trim().to_string());
            let energy_now = read_num::<u64>(&format!("{base}/energy_now"))
                .or_else(|| read_num::<u64>(&format!("{base}/charge_now")));
            let energy_full = read_num::<u64>(&format!("{base}/energy_full"))
                .or_else(|| read_num::<u64>(&format!("{base}/charge_full")));
            let power_now = read_num::<u64>(&format!("{base}/power_now"))
                .or_else(|| read_num::<u64>(&format!("{base}/current_now")));
            let charging = status.as_deref() == Some("Charging");
            let discharging = status.as_deref() == Some("Discharging");
            if let Some(cap) = capacity {
                batteries.push(BatteryInfo {
                    percent: if discharging && cap == 0 { 0 } else { cap },
                    charging,
                    discharging,
                    energy_now,
                    energy_full,
                    power_now,
                    minutes_left: None,
                });
            } else if energy_now.is_some() && energy_full.is_some() {
                let pct = energy_full
                    .and_then(|f| energy_now.map(|n| (n * 100).checked_div(f).unwrap_or(0) as i32));
                batteries.push(BatteryInfo {
                    percent: pct.unwrap_or(0),
                    charging,
                    discharging,
                    energy_now,
                    energy_full,
                    power_now,
                    minutes_left: None,
                });
            }
        }
        batteries
    }

    fn detect_ac() -> bool {
        let Ok(entries) = std::fs::read_dir(POWER_SUPPLY_DIR) else { return false };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("AC") || name.starts_with("ADP") || name.starts_with("ACAD") {
                if let Ok(online) = std::fs::read_to_string(entry.path().join("online")) {
                    if online.trim() == "1" {
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[cfg(target_os = "openbsd")]
mod openbsd {
    use super::{BatteryInfo, PowerStatus};
    use antibox_core::libc::{self, c_ulong, c_void};

    #[repr(C)]
    struct ApmPowerInfo {
        battery_state: u8,
        ac_state: u8,
        battery_life: u8,
        spare1: u8,
        minutes_left: u32,
        spare2: [u32; 6],
    }

    const APM_IOC_GETPOWER: c_ulong = 0x4020_4103;
    const APM_BATT_CHARGING: u8 = 3;
    const APM_BATT_LIFE_UNKNOWN: u8 = 0xff;
    const APM_BATTERY_ABSENT: u8 = 4;
    const APM_AC_ON: u8 = 1;
    const MINUTES_LEFT_UNKNOWN: u32 = 0xffff_ffff;

    pub(crate) fn read() -> PowerStatus {
        let path = match std::ffi::CString::new("/dev/apm") {
            Ok(p) => p,
            Err(_) => return PowerStatus::default(),
        };
        let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
        if fd < 0 {
            return PowerStatus::default();
        }
        let mut info = ApmPowerInfo {
            battery_state: 0,
            ac_state: 0,
            battery_life: 0,
            spare1: 0,
            minutes_left: 0,
            spare2: [0; 6],
        };
        let rc = unsafe {
            libc::ioctl(
                fd,
                APM_IOC_GETPOWER,
                &mut info as *mut ApmPowerInfo as *mut c_void,
            )
        };
        unsafe { libc::close(fd) };
        if rc < 0 {
            return PowerStatus::default();
        }
        let ac_online = info.ac_state == APM_AC_ON;
        let mut batteries = Vec::new();
        if info.battery_state != APM_BATTERY_ABSENT && info.battery_life != APM_BATT_LIFE_UNKNOWN {
            let charging = info.battery_state == APM_BATT_CHARGING;
            batteries.push(BatteryInfo {
                percent: info.battery_life as i32,
                charging,
                discharging: !charging && !ac_online,
                energy_now: None,
                energy_full: None,
                power_now: None,
                minutes_left: if info.minutes_left == MINUTES_LEFT_UNKNOWN {
                    None
                } else {
                    Some(info.minutes_left)
                },
            });
        }
        PowerStatus {
            ac_online,
            batteries,
        }
    }
}

#[cfg(test)]
#[path = "power_tests.rs"]
mod tests;
