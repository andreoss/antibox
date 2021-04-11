use crate::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;

static DPI: AtomicI32 = AtomicI32::new(96);

pub fn dpi() -> i32 {
    DPI.load(Ordering::Relaxed).max(96)
}

pub fn set_dpi(n: i32) {
    DPI.store(n.clamp(96, 384), Ordering::Relaxed);
}

pub fn scaled(base: i32) -> i32 {
    ((base as f32) * dpi() as f32 / 96.0).round() as i32
}

const COMFORT: f64 = 0.7;

fn comfort_dpi(ratio: f64) -> i32 {
    let factor = ((ratio * COMFORT * 10.0).round() / 10.0).clamp(1.0, 4.0);
    (96.0 * factor).round() as i32
}

pub fn resolution_dpi(width: u32, height: u32) -> i32 {
    let rw = width as f64 / 1920.0;
    let rh = height as f64 / 1080.0;
    let r = if rw > rh { rw } else { rh };
    if r <= 1.0 {
        96
    } else {
        comfort_dpi(r)
    }
}

pub fn physical_dpi(width: u32, height: u32, width_mm: u32, height_mm: u32) -> Option<i32> {
    if width_mm == 0 || height_mm == 0 {
        return None;
    }
    let dpi_x = width as f64 * 25.4 / width_mm as f64;
    let dpi_y = height as f64 * 25.4 / height_mm as f64;
    let real = dpi_x.max(dpi_y);
    if !(50.0..=400.0).contains(&real) {
        return None;
    }
    Some(comfort_dpi(real / 96.0))
}

pub fn detect(
    width: u32,
    height: u32,
    width_mm: u32,
    height_mm: u32,
    scale_hint: Option<f64>,
) -> i32 {
    if let Ok(v) = std::env::var("ANTIBOX_SCALE") {
        if let Ok(n) = v.trim().parse::<f32>() {
            if n >= 1.0 {
                return (96.0 * n).round() as i32;
            }
        }
    }
    if let Some(ratio) = scale_hint {
        if ratio > 1.0 {
            return comfort_dpi(ratio);
        }
    }
    physical_dpi(width, height, width_mm, height_mm).unwrap_or_else(|| resolution_dpi(width, height))
}

#[cfg(test)]
#[path = "scale_tests.rs"]
mod tests;
