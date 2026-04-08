use crate::dbus::{Connection, Message, Value};
use antibox_core::sync::atomic::LazyRwLock;
use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

pub const BUS_PREFIX: &str = "org.mpris.MediaPlayer2.";
pub const ROOT_IFACE: &str = "org.mpris.MediaPlayer2";
pub const PLAYER_IFACE: &str = "org.mpris.MediaPlayer2.Player";
pub const OBJECT_PATH: &str = "/org/mpris/MediaPlayer2";
const PROPS_IFACE: &str = "org.freedesktop.DBus.Properties";
const BUS_NAME: &str = "org.freedesktop.DBus";
const BUS_PATH: &str = "/org/freedesktop/DBus";
const MAX_PENDING: usize = 64;
const POSITION_POLL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Playback {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl Playback {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "playing" => Self::Playing,
            "paused" => Self::Paused,
            _ => Self::Stopped,
        }
    }

    pub const fn text(self) -> &'static str {
        match self {
            Self::Playing => "Playing",
            Self::Paused => "Paused",
            Self::Stopped => "Stopped",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub length: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Player {
    pub bus: String,
    pub owner: String,
    pub identity: String,
    pub status: Playback,
    pub track: Track,
    pub can_next: bool,
    pub can_previous: bool,
    pub can_control: bool,
    pub can_seek: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    PlayPause,
    Next,
    Previous,
    Stop,
    Raise,
}

impl Command {
    pub const fn member(self) -> &'static str {
        match self {
            Self::PlayPause => "PlayPause",
            Self::Next => "Next",
            Self::Previous => "Previous",
            Self::Stop => "Stop",
            Self::Raise => "Raise",
        }
    }

    pub const fn interface(self) -> &'static str {
        match self {
            Self::Raise => ROOT_IFACE,
            _ => PLAYER_IFACE,
        }
    }
}

pub fn format_time(micros: i64) -> String {
    let secs = (micros / 1_000_000).max(0);
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn parse_track(meta: &Value) -> Track {
    let text = |key: &str| {
        meta.lookup(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let list = |key: &str| {
        meta.lookup(key).map_or_else(Vec::new, Value::strings).join(", ")
    };
    let mut artist = list("xesam:artist");
    if artist.is_empty() {
        artist = list("xesam:albumArtist");
    }
    if artist.is_empty() {
        artist = text("xesam:artist");
    }
    Track {
        title: text("xesam:title"),
        artist: artist.trim().to_string(),
        album: text("xesam:album"),
        length: meta
            .lookup("mpris:length")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(0),
    }
}

impl Player {
    pub fn new(bus: &str) -> Self {
        Self {
            bus: bus.to_string(),
            ..Self::default()
        }
    }

    pub fn short_name(&self) -> &str {
        self.bus.strip_prefix(BUS_PREFIX).unwrap_or(&self.bus)
    }

    pub fn name(&self) -> String {
        if self.identity.is_empty() {
            let short = self.short_name();
            let head = short.split('.').next().unwrap_or(short);
            let mut chars = head.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        } else {
            self.identity.clone()
        }
    }

    pub fn label(&self) -> String {
        let title = self.track.title.trim();
        let artist = self.track.artist.trim();
        if title.is_empty() && artist.is_empty() {
            self.name()
        } else if artist.is_empty() {
            title.to_string()
        } else if title.is_empty() {
            artist.to_string()
        } else {
            format!("{artist} \u{2014} {title}")
        }
    }

    pub fn tooltip(&self) -> String {
        let mut lines = vec![self.name()];
        if !self.track.title.is_empty() {
            lines.push(self.track.title.clone());
        }
        if !self.track.artist.is_empty() {
            lines.push(self.track.artist.clone());
        }
        if !self.track.album.is_empty() {
            lines.push(self.track.album.clone());
        }
        if self.track.length > 0 {
            lines.push(format!(
                "{}  {}",
                self.status.text(),
                format_time(self.track.length)
            ));
        } else {
            lines.push(self.status.text().to_string());
        }
        lines.join("\n")
    }

    pub fn apply(&mut self, key: &str, value: &Value) -> bool {
        let before = self.clone();
        match key {
            "PlaybackStatus" => self.status = Playback::parse(value.as_str().unwrap_or("")),
            "Metadata" => self.track = parse_track(value),
            "Identity" => self.identity = value.as_str().unwrap_or("").to_string(),
            "CanGoNext" => self.can_next = value.as_bool().unwrap_or(false),
            "CanGoPrevious" => self.can_previous = value.as_bool().unwrap_or(false),
            "CanControl" => self.can_control = value.as_bool().unwrap_or(false),
            "CanSeek" => self.can_seek = value.as_bool().unwrap_or(false),
            _ => {}
        }
        before != *self
    }

    pub fn apply_all(&mut self, props: &Value) -> bool {
        let mut changed = false;
        for item in props.items() {
            if let Value::Entry(k, v) = item {
                if let Some(key) = k.as_str() {
                    changed |= self.apply(key, v);
                }
            }
        }
        changed
    }
}

pub fn seek_target(position: i64, length: i64, delta: i64) -> i64 {
    let target = position.saturating_add(delta).max(0);
    if length > 0 {
        target.min(length)
    } else {
        target
    }
}

pub fn pick(players: &[Player], current: &str) -> String {
    let playing_current = players
        .iter()
        .any(|p| p.bus == current && p.status == Playback::Playing);
    if playing_current {
        return current.to_string();
    }
    if let Some(p) = players.iter().find(|p| p.status == Playback::Playing) {
        return p.bus.clone();
    }
    if players.iter().any(|p| p.bus == current) {
        return current.to_string();
    }
    players.first().map_or_else(String::new, |p| p.bus.clone())
}

pub fn next_bus(players: &[Player], current: &str) -> String {
    if players.is_empty() {
        return String::new();
    }
    let at = players.iter().position(|p| p.bus == current);
    let idx = at.map_or(0, |i| (i + 1) % players.len());
    players[idx].bus.clone()
}

enum Ask {
    Names,
    Props(String),
    Identity(String),
    Owner(String),
    Position(String),
}

pub struct Client {
    conn: Connection,
    players: Vec<Player>,
    current: String,
    pending: Vec<(u32, Ask)>,
    changed: bool,
    position: i64,
    sampled: Instant,
    polled: Instant,
    timed: bool,
}

impl Client {
    pub fn connect() -> Option<Self> {
        let mut conn = Connection::session()?;
        conn.add_match(&format!(
            "type='signal',interface='{PROPS_IFACE}',member='PropertiesChanged',path='{OBJECT_PATH}'"
        ));
        conn.add_match(&format!(
            "type='signal',interface='{BUS_NAME}',member='NameOwnerChanged',arg0namespace='{ROOT_IFACE}'"
        ));
        conn.add_match(&format!(
            "type='signal',interface='{PLAYER_IFACE}',member='Seeked',path='{OBJECT_PATH}'"
        ));
        let mut client = Self {
            conn,
            players: Vec::new(),
            current: String::new(),
            pending: Vec::new(),
            changed: false,
            position: 0,
            sampled: Instant::now(),
            polled: Instant::now(),
            timed: false,
        };
        client.list_names();
        Some(client)
    }

    pub fn fd(&self) -> RawFd {
        self.conn.fd()
    }

    pub const fn dead(&self) -> bool {
        self.conn.dead()
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn current(&self) -> Option<&Player> {
        self.players.iter().find(|p| p.bus == self.current)
    }

    pub fn take_changed(&mut self) -> bool {
        std::mem::replace(&mut self.changed, false)
    }

    fn expect(&mut self, serial: u32, ask: Ask) {
        if serial == 0 {
            return;
        }
        if self.pending.len() >= MAX_PENDING {
            self.pending.remove(0);
        }
        self.pending.push((serial, ask));
    }

    pub fn list_names(&mut self) {
        let serial = self
            .conn
            .send(&Message::call(BUS_NAME, BUS_PATH, BUS_NAME, "ListNames"));
        self.expect(serial, Ask::Names);
    }

    fn get_all(&mut self, bus: &str, interface: &str, ask: Ask) {
        let msg = Message::call(bus, OBJECT_PATH, PROPS_IFACE, "GetAll").arg(Value::str(interface));
        let serial = self.conn.send(&msg);
        self.expect(serial, ask);
    }

    fn get_owner(&mut self, bus: &str) {
        let msg = Message::call(BUS_NAME, BUS_PATH, BUS_NAME, "GetNameOwner").arg(Value::str(bus));
        let serial = self.conn.send(&msg);
        self.expect(serial, Ask::Owner(bus.to_string()));
    }

    fn poll_position(&mut self) {
        if self.current.is_empty() {
            return;
        }
        let bus = self.current.clone();
        let msg = Message::call(&bus, OBJECT_PATH, PROPS_IFACE, "Get")
            .arg(Value::str(PLAYER_IFACE))
            .arg(Value::str("Position"));
        let serial = self.conn.send(&msg);
        self.polled = Instant::now();
        self.expect(serial, Ask::Position(bus));
    }

    fn sample_position(&mut self, micros: i64) {
        self.position = micros.max(0);
        self.sampled = Instant::now();
        self.timed = true;
    }

    fn forget_position(&mut self) {
        self.position = 0;
        self.sampled = Instant::now();
        self.timed = false;
    }

    pub fn position(&self) -> i64 {
        if !self.timed {
            return 0;
        }
        let Some(player) = self.current() else { return 0 };
        let mut pos = self.position;
        if player.status == Playback::Playing {
            pos += self.sampled.elapsed().as_micros() as i64;
        }
        if player.track.length > 0 {
            pos = pos.min(player.track.length);
        }
        pos.max(0)
    }

    fn apply_props(&mut self, bus: &str, props: &Value) {
        let before = self.players.iter().find(|p| p.bus == bus).map(|p| p.track.clone());
        if self.player_mut(bus).is_some_and(|p| p.apply_all(props)) {
            self.changed = true;
        }
        let after = self.players.iter().find(|p| p.bus == bus).map(|p| p.track.clone());
        if before != after && bus == self.current {
            self.forget_position();
            self.poll_position();
        }
    }

    fn query(&mut self, bus: &str) {
        let name = bus.to_string();
        self.get_all(&name, PLAYER_IFACE, Ask::Props(name.clone()));
        self.get_all(&name, ROOT_IFACE, Ask::Identity(name.clone()));
        self.get_owner(&name);
    }

    fn add_player(&mut self, bus: &str) {
        if self.players.iter().any(|p| p.bus == bus) {
            return;
        }
        self.players.push(Player::new(bus));
        self.players.sort_by(|a, b| a.bus.cmp(&b.bus));
        self.query(bus);
        self.changed = true;
    }

    fn drop_player(&mut self, bus: &str) {
        let before = self.players.len();
        self.players.retain(|p| p.bus != bus);
        if self.players.len() != before {
            self.changed = true;
        }
    }

    fn reselect(&mut self) {
        let chosen = pick(&self.players, &self.current);
        if chosen != self.current {
            self.current = chosen;
            self.changed = true;
            self.forget_position();
            self.poll_position();
        }
    }

    pub fn cycle(&mut self) {
        let next = next_bus(&self.players, &self.current);
        if next != self.current {
            self.current = next;
            self.changed = true;
            self.forget_position();
            self.poll_position();
        }
    }

    pub fn command(&mut self, cmd: Command) {
        if self.current.is_empty() {
            return;
        }
        let msg = Message::call(
            &self.current,
            OBJECT_PATH,
            cmd.interface(),
            cmd.member(),
        )
        .no_reply();
        self.conn.send(&msg);
        if matches!(cmd, Command::PlayPause | Command::Stop) {
            self.get_all(
                &self.current.clone(),
                PLAYER_IFACE,
                Ask::Props(self.current.clone()),
            );
        }
    }

    pub fn seek(&mut self, delta: i64) {
        let Some((can_seek, length)) = self
            .current()
            .map(|p| (p.can_seek, p.track.length)) else { return };
        if !can_seek || self.current.is_empty() {
            return;
        }
        let (offset, target) = if self.timed {
            let now = self.position();
            let target = seek_target(now, length, delta);
            (target - now, Some(target))
        } else {
            (delta, None)
        };
        if offset == 0 {
            return;
        }
        let msg = Message::call(&self.current, OBJECT_PATH, PLAYER_IFACE, "Seek")
            .arg(Value::I64(offset))
            .no_reply();
        self.conn.send(&msg);
        if let Some(target) = target {
            self.sample_position(target);
            self.changed = true;
        }
        self.poll_position();
    }

    fn take_pending(&mut self, serial: u32) -> Option<Ask> {
        let at = self.pending.iter().position(|(s, _)| *s == serial)?;
        Some(self.pending.remove(at).1)
    }

    fn player_mut(&mut self, bus: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.bus == bus)
    }

    fn handle_names(&mut self, msg: &Message) {
        let names: Vec<String> = msg
            .body
            .first()
            .map_or_else(Vec::new, Value::strings)
            .into_iter()
            .filter(|n| n.starts_with(BUS_PREFIX))
            .collect();
        let gone: Vec<String> = self
            .players
            .iter()
            .map(|p| p.bus.clone())
            .filter(|b| !names.contains(b))
            .collect();
        for bus in gone {
            self.drop_player(&bus);
        }
        for name in names {
            self.add_player(&name);
        }
    }

    fn handle_signal(&mut self, msg: &Message) {
        if msg.member == "PropertiesChanged" {
            if msg.arg_str(0) != PLAYER_IFACE {
                return;
            }
            let owner = msg.sender.clone();
            let bus = self
                .players
                .iter()
                .find(|p| p.owner == owner)
                .map(|p| p.bus.clone());
            let Some(bus) = bus else { return };
            let Some(props) = msg.body.get(1).cloned() else { return };
            self.apply_props(&bus, &props);
            self.reselect();
        } else if msg.member == "NameOwnerChanged" {
            let name = msg.arg_str(0).to_string();
            if !name.starts_with(BUS_PREFIX) {
                return;
            }
            let new_owner = msg.arg_str(2).to_string();
            if new_owner.is_empty() {
                self.drop_player(&name);
            } else {
                self.add_player(&name);
                if let Some(player) = self.player_mut(&name) {
                    player.owner = new_owner;
                }
            }
            self.reselect();
        } else if msg.member == "Seeked" {
            let owner = msg.sender.clone();
            let current_owner = self
                .current()
                .map_or_else(String::new, |p| p.owner.clone());
            if owner == current_owner {
                if let Some(micros) = msg.body.first().and_then(Value::as_i64) {
                    self.sample_position(micros);
                    self.changed = true;
                }
            }
        }
    }

    pub fn poll(&mut self) {
        self.pump();
        if self.polled.elapsed() >= POSITION_POLL {
            self.poll_position();
        }
    }

    pub fn pump(&mut self) {
        for msg in self.conn.pump() {
            match msg.kind {
                crate::dbus::RETURN => {
                    let Some(ask) = self.take_pending(msg.reply_serial) else { continue };
                    match ask {
                        Ask::Names => self.handle_names(&msg),
                        Ask::Props(bus) | Ask::Identity(bus) => {
                            let Some(props) = msg.body.first().cloned() else { continue };
                            self.apply_props(&bus, &props);
                        }
                        Ask::Position(bus) => {
                            if bus == self.current {
                                let micros = msg.body.first().and_then(Value::as_i64).unwrap_or(0);
                                self.sample_position(micros);
                            }
                        }
                        Ask::Owner(bus) => {
                            let owner = msg.arg_str(0).to_string();
                            if let Some(player) = self.player_mut(&bus) {
                                player.owner = owner;
                            }
                        }
                    }
                    self.reselect();
                }
                crate::dbus::ERROR => {
                    self.take_pending(msg.reply_serial);
                }
                crate::dbus::SIGNAL => self.handle_signal(&msg),
                _ => {}
            }
        }
    }
}

static CLIENT: LazyRwLock<Option<Client>> = LazyRwLock::new();
static CHANGED: AtomicBool = AtomicBool::new(false);
static ALIVE: AtomicBool = AtomicBool::new(false);

pub fn init() -> Option<RawFd> {
    let client = Client::connect()?;
    let fd = client.fd();
    if let Ok(mut g) = CLIENT.write() {
        *g = Some(client);
    }
    ALIVE.store(true, Ordering::Relaxed);
    Some(fd)
}

pub fn active() -> bool {
    ALIVE.load(Ordering::Relaxed)
}

fn with_client<R, F: FnOnce(&mut Client) -> R>(f: F) -> Option<R> {
    let mut guard = CLIENT.write().ok()?;
    let client = guard.as_mut()?;
    let out = f(client);
    if client.take_changed() {
        CHANGED.store(true, Ordering::Relaxed);
    }
    if client.dead() {
        ALIVE.store(false, Ordering::Relaxed);
    }
    Some(out)
}

pub fn drain() {
    let _ = with_client(Client::pump);
}

pub fn refresh() {
    let _ = with_client(|c| {
        c.pump();
        c.list_names();
    });
}

pub fn poll() {
    let _ = with_client(Client::poll);
}

pub fn position() -> i64 {
    with_client(|c| c.position()).unwrap_or(0)
}

pub fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

pub fn current() -> Option<Player> {
    with_client(|c| c.current().cloned()).flatten()
}

pub fn players() -> Vec<Player> {
    with_client(|c| c.players().to_vec()).unwrap_or_default()
}

pub fn send(cmd: Command) {
    let _ = with_client(|c| c.command(cmd));
}

pub fn seek(delta: i64) {
    let _ = with_client(|c| c.seek(delta));
}

pub fn cycle_player() {
    let _ = with_client(Client::cycle);
}

#[cfg(test)]
#[path = "mpris_tests.rs"]
mod tests;
