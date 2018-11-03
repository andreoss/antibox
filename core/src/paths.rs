use std::path::PathBuf;

pub fn system_config_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/etc/antibox"),
        PathBuf::from("/usr/local/etc/antibox"),
        PathBuf::from("/usr/share/antibox"),
        PathBuf::from("/usr/local/share/antibox"),
    ]
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
