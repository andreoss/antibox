use antibox_wm::dbus::{Connection, Message, Value, CALL, RETURN};
use std::time::{Duration, Instant};

fn variant(v: Value) -> Value {
    Value::Variant(Box::new(v))
}

fn entry(key: &str, v: Value) -> Value {
    Value::Entry(Box::new(Value::str(key)), Box::new(variant(v)))
}

fn metadata(track: u32) -> Value {
    Value::Array(vec![
        entry("mpris:trackid", Value::path(&format!("/fake/track/{track}"))),
        entry("xesam:title", Value::str(&format!("Track {track}"))),
        entry("xesam:artist", Value::Array(vec![Value::str("Arvo Pärt")])),
        entry("xesam:album", Value::str("Tabula Rasa")),
        entry("mpris:length", Value::I64(672_000_000)),
    ])
}

fn player_props(playing: bool, track: u32, position: i64) -> Value {
    Value::Array(vec![
        entry(
            "PlaybackStatus",
            Value::str(if playing { "Playing" } else { "Paused" }),
        ),
        entry("Metadata", metadata(track)),
        entry("CanGoNext", Value::Bool(true)),
        entry("CanGoPrevious", Value::Bool(true)),
        entry("CanControl", Value::Bool(true)),
        entry("CanSeek", Value::Bool(true)),
        entry("Position", Value::I64(position)),
    ])
}

fn root_props() -> Value {
    Value::Array(vec![
        entry("Identity", Value::str("Fake Player")),
        entry("CanRaise", Value::Bool(true)),
    ])
}

fn main() {
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "org.mpris.MediaPlayer2.fake".to_string());
    let Some(mut conn) = Connection::session() else {
        eprintln!("no session bus");
        return;
    };
    conn.send(
        &Message::call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "RequestName",
        )
        .arg(Value::str(&name))
        .arg(Value::U32(4)),
    );
    eprintln!("fake player on {name}");
    let mut playing = true;
    let mut track = 1u32;
    let mut position = 0i64;
    let started = Instant::now();
    let mut ticked = Instant::now();
    while started.elapsed() < Duration::from_secs(120) {
        for msg in conn.pump() {
            if msg.kind != CALL {
                continue;
            }
            let mut reply = Message {
                kind: RETURN,
                reply_serial: msg.serial,
                destination: msg.sender.clone(),
                ..Message::default()
            };
            match (msg.interface.as_str(), msg.member.as_str()) {
                ("org.freedesktop.DBus.Properties", "GetAll") => {
                    reply.body = vec![if msg.arg_str(0).ends_with("Player") {
                        player_props(playing, track, position)
                    } else {
                        root_props()
                    }];
                }
                ("org.freedesktop.DBus.Properties", "Get") => {
                    let props = if msg.arg_str(0).ends_with("Player") {
                        player_props(playing, track, position)
                    } else {
                        root_props()
                    };
                    let Some(v) = props.lookup(msg.arg_str(1)).cloned() else { continue };
                    reply.body = vec![v];
                }
                ("org.mpris.MediaPlayer2.Player", member) => {
                    match member {
                        "PlayPause" => playing = !playing,
                        "Seek" => {
                            let offset = msg.body.first().and_then(Value::as_i64).unwrap_or(0);
                            position = (position + offset).clamp(0, 672_000_000);
                        }
                        "Next" => {
                            track += 1;
                            position = 0;
                        }
                        "Previous" => {
                            track = track.saturating_sub(1).max(1);
                            position = 0;
                        }
                        "Stop" => playing = false,
                        _ => {}
                    }
                    eprintln!("fake: {member} -> playing={playing} track={track}");
                    conn.send(&reply);
                    if member == "Seek" {
                        let seeked = Message {
                            kind: antibox_wm::dbus::SIGNAL,
                            path: "/org/mpris/MediaPlayer2".to_string(),
                            interface: "org.mpris.MediaPlayer2.Player".to_string(),
                            member: "Seeked".to_string(),
                            body: vec![Value::I64(position)],
                            ..Message::default()
                        };
                        conn.send(&seeked);
                        continue;
                    }
                    let signal = Message {
                        kind: antibox_wm::dbus::SIGNAL,
                        path: "/org/mpris/MediaPlayer2".to_string(),
                        interface: "org.freedesktop.DBus.Properties".to_string(),
                        member: "PropertiesChanged".to_string(),
                        body: vec![
                            Value::str("org.mpris.MediaPlayer2.Player"),
                            player_props(playing, track, position),
                            Value::Array(Vec::new()),
                        ],
                        ..Message::default()
                    };
                    conn.send(&signal);
                    continue;
                }
                _ => continue,
            }
            conn.send(&reply);
        }
        if playing {
            position += ticked.elapsed().as_micros() as i64;
        }
        ticked = Instant::now();
        std::thread::sleep(Duration::from_millis(30));
    }
}
