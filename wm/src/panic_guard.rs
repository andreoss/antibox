use antibox_core::sync::atomic::LazyLock;
use std::path::{Path, PathBuf};

static LAST_PANIC: LazyLock<Option<String>> = LazyLock::new();

pub fn install_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut g) = LAST_PANIC.lock() {
            *g = Some(format!("{}", info));
        }
        prev(info);
    }));
}

pub fn take_last() -> Option<String> {
    LAST_PANIC.lock().ok().and_then(|mut g| g.take())
}

pub fn breadcrumb_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".antibox/last-panic.log"))
}

pub fn record(event: &str, detail: &str, path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, format!("event: {}\n{}\n", event, detail))
}

pub fn report_previous() {
    let path = match breadcrumb_path() {
        Some(p) => p,
        None => return,
    };
    if !path.is_file() {
        return;
    }
    eprintln!(
        "previous run recorded a panic; details preserved in {}",
        path.display()
    );
    let prev = path.with_extension("prev.log");
    let _ = std::fs::rename(&path, prev);
}

#[cfg(test)]
#[path = "panic_guard_tests.rs"]
mod tests;
