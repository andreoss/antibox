use std::process::Command;

fn command(cmd: &str, args: &[&str]) -> Command {
    let mut c = Command::new(cmd);
    c.args(args).env("LC_ALL", "C").env("LANGUAGE", "C");
    c
}

pub fn capture(cmd: &str, args: &[&str]) -> Option<String> {
    command(cmd, args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

pub fn ok(cmd: &str, args: &[&str]) -> bool {
    command(cmd, args).status().is_ok_and(|s| s.success())
}
