#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioState {
    pub sink_muted: bool,
    pub volume: i32,
    pub source_muted: bool,
    pub readers: Vec<String>,
}

pub trait AudioSystem {
    fn read(&self) -> Option<AudioState>;
    fn toggle_sink_mute(&self);
    fn toggle_source_mute(&self);
    fn nudge_sink_volume(&self, delta_pct: i32);
    fn nudge_source_volume(&self, delta_pct: i32);
}

pub fn detect() -> Box<dyn AudioSystem> {
    #[cfg(target_os = "openbsd")]
    {
        Box::new(SndioAudio)
    }
    #[cfg(not(target_os = "openbsd"))]
    {
        Box::new(PulseAudio)
    }
}

fn parse_percent(s: &str) -> Option<i32> {
    for tok in s.split(&[' ', '/', ',', '\t'][..]) {
        if let Some(num) = tok.trim().strip_suffix('%') {
            if let Ok(v) = num.trim().parse::<i32>() {
                return Some(v);
            }
        }
    }
    None
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let rest = line.trim().strip_prefix(key)?;
    let end = rest.find('"')?;
    let v = rest[..end].to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

fn parse_readers(list: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for block in list.split("Source Output") {
        let name = block
            .lines()
            .find_map(|l| quoted_value(l, "application.name = \""))
            .or_else(|| {
                block
                    .lines()
                    .find_map(|l| quoted_value(l, "application.process.binary = \""))
            });
        if let Some(n) = name {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

pub struct PulseAudio;

impl PulseAudio {
    fn pactl(args: &[&str]) -> Option<String> {
        crate::run::capture("pactl", args)
    }

    fn toggle(target: &str) {
        let sink_or_source = if target.contains("sink") {
            "@DEFAULT_SINK@"
        } else {
            "@DEFAULT_SOURCE@"
        };
        let _ = crate::run::ok("pactl", &[target, sink_or_source, "toggle"]);
    }

    fn set_volume(cmd: &str, target: &str, delta_pct: i32) {
        let arg = if delta_pct >= 0 {
            format!("+{delta_pct}%")
        } else {
            format!("{delta_pct}%")
        };
        let _ = crate::run::ok("pactl", &[cmd, target, &arg]);
    }
}

impl AudioSystem for PulseAudio {
    fn read(&self) -> Option<AudioState> {
        let sink_mute = Self::pactl(&["get-sink-mute", "@DEFAULT_SINK@"])?;
        let volume = Self::pactl(&["get-sink-volume", "@DEFAULT_SINK@"])
            .as_deref()
            .and_then(parse_percent)
            .unwrap_or(-1);
        let source_muted = Self::pactl(&["get-source-mute", "@DEFAULT_SOURCE@"])
            .is_some_and(|s| s.contains("yes"));
        let readers = Self::pactl(&["list", "source-outputs"])
            .as_deref()
            .map(parse_readers)
            .unwrap_or_default();
        Some(AudioState {
            sink_muted: sink_mute.contains("yes"),
            volume,
            source_muted,
            readers,
        })
    }

    fn toggle_sink_mute(&self) {
        Self::toggle("set-sink-mute");
    }

    fn toggle_source_mute(&self) {
        Self::toggle("set-source-mute");
    }

    fn nudge_sink_volume(&self, delta_pct: i32) {
        Self::set_volume("set-sink-volume", "@DEFAULT_SINK@", delta_pct);
    }

    fn nudge_source_volume(&self, delta_pct: i32) {
        Self::set_volume("set-source-volume", "@DEFAULT_SOURCE@", delta_pct);
    }
}

#[cfg(target_os = "openbsd")]
fn parse_kv(out: &str) -> std::collections::HashMap<String, String> {
    out.lines()
        .filter_map(|l| {
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

#[cfg(target_os = "openbsd")]
pub struct SndioAudio;

#[cfg(target_os = "openbsd")]
impl SndioAudio {
    fn nudge(control: &str, delta_pct: i32) {
        let frac = delta_pct.abs() as f64 / 100.0;
        let arg = if delta_pct >= 0 {
            format!("{}=+{:.2}", control, frac)
        } else {
            format!("{}=-{:.2}", control, frac)
        };
        let _ = crate::run::ok("sndioctl", &[&arg]);
    }
}

#[cfg(target_os = "openbsd")]
impl AudioSystem for SndioAudio {
    fn read(&self) -> Option<AudioState> {
        let out = crate::run::capture(
            "sndioctl",
            &["output.level", "output.mute", "input.mute"],
        )?;
        let kv = parse_kv(&out);
        let level: f64 = kv.get("output.level")?.parse().ok()?;
        let sink_muted = kv
            .get("output.mute")
            .and_then(|v| v.parse::<f64>().ok())
            .is_some_and(|v| v >= 0.5);
        let source_muted = kv
            .get("input.mute")
            .and_then(|v| v.parse::<f64>().ok())
            .is_some_and(|v| v >= 0.5);
        Some(AudioState {
            sink_muted,
            volume: (level * 100.0).round() as i32,
            source_muted,
            readers: Vec::new(),
        })
    }

    fn toggle_sink_mute(&self) {
        let _ = crate::run::ok("sndioctl", &["output.mute=!"]);
    }

    fn toggle_source_mute(&self) {
        let _ = crate::run::ok("sndioctl", &["input.mute=!"]);
    }

    fn nudge_sink_volume(&self, delta_pct: i32) {
        Self::nudge("output.level", delta_pct);
    }

    fn nudge_source_volume(&self, delta_pct: i32) {
        Self::nudge("input.level", delta_pct);
    }
}

#[cfg(test)]
#[path = "audio_tests.rs"]
mod tests;
