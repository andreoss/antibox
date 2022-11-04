use super::*;

#[test]
fn parse_percent_finds_level() {
    assert_eq!(
        parse_percent("Volume: front-left: 45000 / 69% / -9.30 dB"),
        Some(69)
    );
    assert_eq!(parse_percent("Mute: no"), None);
}

#[test]
fn parse_percent_takes_the_first_percentage() {
    assert_eq!(
        parse_percent("front-left: 10% / front-right: 90%"),
        Some(10)
    );
    assert_eq!(parse_percent("no numbers here"), None);
    assert_eq!(parse_percent(""), None);
}

#[test]
fn parse_readers_collects_recording_apps() {
    let list = "\
Source Output #12
\tapplication.name = \"Firefox\"
\tapplication.process.binary = \"firefox\"
Source Output #13
\tapplication.name = \"Zoom\"
";
    let r = parse_readers(list);
    assert_eq!(r, vec!["Firefox".to_string(), "Zoom".to_string()]);
}

#[test]
fn parse_readers_empty_when_nothing_recording() {
    assert!(parse_readers("").is_empty());
    assert!(parse_readers("Failure: no such object").is_empty());
}

#[test]
fn parse_readers_falls_back_to_binary_without_a_name() {
    let list = "\
Source Output #7
\tapplication.process.binary = \"obs\"
";
    assert_eq!(parse_readers(list), vec!["obs".to_string()]);
}

#[test]
fn parse_readers_dedups_the_same_app_across_streams() {
    let list = "\
Source Output #1
\tapplication.name = \"Zoom\"
Source Output #2
\tapplication.name = \"Zoom\"
";
    assert_eq!(parse_readers(list), vec!["Zoom".to_string()]);
}

#[cfg(target_os = "openbsd")]
#[test]
fn parse_kv_reads_sndioctl_lines() {
    let out = "output.level=0.750000\noutput.mute=0\ninput.mute=1\n";
    let kv = parse_kv(out);
    assert_eq!(kv.get("output.level").map(String::as_str), Some("0.750000"));
    assert_eq!(kv.get("output.mute").map(String::as_str), Some("0"));
    assert_eq!(kv.get("input.mute").map(String::as_str), Some("1"));
}

#[cfg(target_os = "openbsd")]
#[test]
fn parse_kv_ignores_malformed_lines() {
    let kv = parse_kv("garbage\noutput.level=0.5\n");
    assert_eq!(kv.len(), 1);
    assert_eq!(kv.get("output.level").map(String::as_str), Some("0.5"));
}
