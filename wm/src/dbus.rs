#![allow(unsafe_code)]
use antibox_core::libc;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

pub const CALL: u8 = 1;
pub const RETURN: u8 = 2;
pub const ERROR: u8 = 3;
pub const SIGNAL: u8 = 4;

pub const NO_REPLY: u8 = 1;

const MAX_MESSAGE: usize = 64 * 1024 * 1024;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(1500);

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Byte(u8),
    Bool(bool),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F64(f64),
    Str(String),
    Path(String),
    Sig(String),
    Array(Vec<Value>),
    Struct(Vec<Value>),
    Entry(Box<Value>, Box<Value>),
    Variant(Box<Value>),
}

impl Value {
    pub fn str(s: &str) -> Self {
        Self::Str(s.to_string())
    }

    pub fn path(s: &str) -> Self {
        Self::Path(s.to_string())
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) | Self::Path(s) | Self::Sig(s) => Some(s),
            Self::Variant(v) => v.as_str(),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Byte(v) => Some(i64::from(*v)),
            Self::I16(v) => Some(i64::from(*v)),
            Self::U16(v) => Some(i64::from(*v)),
            Self::I32(v) => Some(i64::from(*v)),
            Self::U32(v) => Some(i64::from(*v)),
            Self::I64(v) => Some(*v),
            Self::U64(v) => Some(*v as i64),
            Self::F64(v) => Some(*v as i64),
            Self::Variant(v) => v.as_i64(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(*v),
            Self::Variant(v) => v.as_f64(),
            _ => self.as_i64().map(|v| v as f64),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            Self::Variant(v) => v.as_bool(),
            _ => None,
        }
    }

    pub fn items(&self) -> &[Self] {
        match self {
            Self::Array(v) | Self::Struct(v) => v,
            Self::Variant(v) => v.items(),
            _ => &[],
        }
    }

    pub fn strings(&self) -> Vec<String> {
        self.items()
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    }

    pub fn lookup(&self, key: &str) -> Option<&Self> {
        for item in self.items() {
            if let Self::Entry(k, v) = item {
                if k.as_str() == Some(key) {
                    return Some(v);
                }
            }
        }
        None
    }
}

pub fn signature(v: &Value) -> String {
    match v {
        Value::Byte(_) => "y".to_string(),
        Value::Bool(_) => "b".to_string(),
        Value::I16(_) => "n".to_string(),
        Value::U16(_) => "q".to_string(),
        Value::I32(_) => "i".to_string(),
        Value::U32(_) => "u".to_string(),
        Value::I64(_) => "x".to_string(),
        Value::U64(_) => "t".to_string(),
        Value::F64(_) => "d".to_string(),
        Value::Str(_) => "s".to_string(),
        Value::Path(_) => "o".to_string(),
        Value::Sig(_) => "g".to_string(),
        Value::Variant(_) => "v".to_string(),
        Value::Array(items) => {
            let inner = items.first().map_or_else(|| "v".to_string(), signature);
            format!("a{inner}")
        }
        Value::Struct(fields) => {
            let inner: String = fields.iter().map(signature).collect();
            format!("({inner})")
        }
        Value::Entry(k, v) => format!("{{{}{}}}", signature(k), signature(v)),
    }
}

pub const fn alignment(sig: &str) -> usize {
    match sig.as_bytes() {
        [b'y' | b'g' | b'v', ..] => 1,
        [b'n' | b'q', ..] => 2,
        [b'x' | b't' | b'd' | b'(' | b'{' | b')' | b'}', ..] => 8,
        _ => 4,
    }
}

pub fn split_signature(sig: &str) -> Option<Vec<String>> {
    let bytes = sig.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i] == b'a' {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        match bytes[i] {
            b'(' | b'{' => {
                let mut depth = 0i32;
                while i < bytes.len() {
                    match bytes[i] {
                        b'(' | b'{' => depth += 1,
                        b')' | b'}' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                }
                if depth != 0 {
                    return None;
                }
            }
            _ => i += 1,
        }
        out.push(sig[start..i].to_string());
    }
    Some(out)
}

fn element_signature(sig: &str) -> &str {
    &sig[1..]
}

fn entry_parts(sig: &str) -> Option<(String, String)> {
    let inner = sig.strip_prefix('{')?.strip_suffix('}')?;
    let parts = split_signature(inner)?;
    if parts.len() == 2 {
        Some((parts[0].clone(), parts[1].clone()))
    } else {
        None
    }
}

fn struct_parts(sig: &str) -> Option<Vec<String>> {
    let inner = sig.strip_prefix('(')?.strip_suffix(')')?;
    split_signature(inner)
}

#[derive(Default)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn take(self) -> Vec<u8> {
        self.buf
    }

    fn align(&mut self, n: usize) {
        while self.buf.len() % n != 0 {
            self.buf.push(0);
        }
    }

    fn byte(&mut self, v: u8) {
        self.buf.push(v);
    }

    fn u16(&mut self, v: u16) {
        self.align(2);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn u32(&mut self, v: u32) {
        self.align(4);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn u64(&mut self, v: u64) {
        self.align(8);
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    fn text(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    fn sig_text(&mut self, s: &str) {
        self.buf.push(s.len() as u8);
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    pub fn value(&mut self, v: &Value) {
        match v {
            Value::Byte(x) => self.byte(*x),
            Value::Bool(x) => self.u32(u32::from(*x)),
            Value::I16(x) => self.u16(*x as u16),
            Value::U16(x) => self.u16(*x),
            Value::I32(x) => self.u32(*x as u32),
            Value::U32(x) => self.u32(*x),
            Value::I64(x) => self.u64(*x as u64),
            Value::U64(x) => self.u64(*x),
            Value::F64(x) => self.u64(x.to_bits()),
            Value::Str(x) | Value::Path(x) => self.text(x),
            Value::Sig(x) => self.sig_text(x),
            Value::Variant(inner) => {
                self.sig_text(&signature(inner));
                self.value(inner);
            }
            Value::Struct(fields) => {
                self.align(8);
                for f in fields {
                    self.value(f);
                }
            }
            Value::Entry(k, val) => {
                self.align(8);
                self.value(k);
                self.value(val);
            }
            Value::Array(items) => {
                let elem = signature(v);
                self.u32(0);
                let len_at = self.buf.len() - 4;
                self.align(alignment(element_signature(&elem)));
                let start = self.buf.len();
                for item in items {
                    self.value(item);
                }
                let len = (self.buf.len() - start) as u32;
                self.buf[len_at..len_at + 4].copy_from_slice(&len.to_le_bytes());
            }
        }
    }
}

pub struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
    le: bool,
}

impl<'a> Decoder<'a> {
    pub const fn new(buf: &'a [u8], le: bool) -> Self {
        Self { buf, pos: 0, le }
    }

    pub const fn at(buf: &'a [u8], pos: usize, le: bool) -> Self {
        Self { buf, pos, le }
    }

    pub const fn pos(&self) -> usize {
        self.pos
    }

    fn align(&mut self, n: usize) -> Option<()> {
        while self.pos % n != 0 {
            self.pos = self.pos.checked_add(1)?;
        }
        if self.pos <= self.buf.len() {
            Some(())
        } else {
            None
        }
    }

    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        if end > self.buf.len() {
            return None;
        }
        let out = &self.buf[self.pos..end];
        self.pos = end;
        Some(out)
    }

    fn byte(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }

    fn u16(&mut self) -> Option<u16> {
        self.align(2)?;
        let b = self.take(2)?;
        let raw = [b[0], b[1]];
        Some(if self.le {
            u16::from_le_bytes(raw)
        } else {
            u16::from_be_bytes(raw)
        })
    }

    fn u32(&mut self) -> Option<u32> {
        self.align(4)?;
        let b = self.take(4)?;
        let raw = [b[0], b[1], b[2], b[3]];
        Some(if self.le {
            u32::from_le_bytes(raw)
        } else {
            u32::from_be_bytes(raw)
        })
    }

    fn u64(&mut self) -> Option<u64> {
        self.align(8)?;
        let b = self.take(8)?;
        let raw = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
        Some(if self.le {
            u64::from_le_bytes(raw)
        } else {
            u64::from_be_bytes(raw)
        })
    }

    fn text(&mut self) -> Option<String> {
        let len = self.u32()? as usize;
        let bytes = self.take(len)?;
        let s = String::from_utf8_lossy(bytes).into_owned();
        self.take(1)?;
        Some(s)
    }

    fn sig_text(&mut self) -> Option<String> {
        let len = self.byte()? as usize;
        let bytes = self.take(len)?;
        let s = String::from_utf8_lossy(bytes).into_owned();
        self.take(1)?;
        Some(s)
    }

    pub fn value(&mut self, sig: &str) -> Option<Value> {
        match sig.as_bytes().first()? {
            b'y' => self.byte().map(Value::Byte),
            b'b' => self.u32().map(|v| Value::Bool(v != 0)),
            b'n' => self.u16().map(|v| Value::I16(v as i16)),
            b'q' => self.u16().map(Value::U16),
            b'i' => self.u32().map(|v| Value::I32(v as i32)),
            b'u' | b'h' => self.u32().map(Value::U32),
            b'x' => self.u64().map(|v| Value::I64(v as i64)),
            b't' => self.u64().map(Value::U64),
            b'd' => self.u64().map(|v| Value::F64(f64::from_bits(v))),
            b's' => self.text().map(Value::Str),
            b'o' => self.text().map(Value::Path),
            b'g' => self.sig_text().map(Value::Sig),
            b'v' => {
                let inner = self.sig_text()?;
                let single = split_signature(&inner)?;
                let one = single.first()?;
                let v = self.value(one)?;
                Some(Value::Variant(Box::new(v)))
            }
            b'a' => {
                let len = self.u32()? as usize;
                let elem = element_signature(sig).to_string();
                self.align(alignment(&elem))?;
                let end = self.pos.checked_add(len)?;
                if end > self.buf.len() {
                    return None;
                }
                let mut items = Vec::new();
                while self.pos < end {
                    items.push(self.value(&elem)?);
                }
                self.pos = end;
                Some(Value::Array(items))
            }
            b'(' => {
                let parts = struct_parts(sig)?;
                self.align(8)?;
                let mut fields = Vec::new();
                for p in &parts {
                    fields.push(self.value(p)?);
                }
                Some(Value::Struct(fields))
            }
            b'{' => {
                let (ks, vs) = entry_parts(sig)?;
                self.align(8)?;
                let k = self.value(&ks)?;
                let v = self.value(&vs)?;
                Some(Value::Entry(Box::new(k), Box::new(v)))
            }
            _ => None,
        }
    }

    pub fn values(&mut self, sig: &str) -> Option<Vec<Value>> {
        let mut out = Vec::new();
        for part in split_signature(sig)? {
            out.push(self.value(&part)?);
        }
        Some(out)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Message {
    pub kind: u8,
    pub flags: u8,
    pub serial: u32,
    pub path: String,
    pub interface: String,
    pub member: String,
    pub error_name: String,
    pub reply_serial: u32,
    pub destination: String,
    pub sender: String,
    pub body: Vec<Value>,
}

#[derive(Debug, PartialEq)]
pub enum Decoded {
    Message(Message, usize),
    Incomplete,
    Bad,
}

impl Message {
    pub fn call(destination: &str, path: &str, interface: &str, member: &str) -> Self {
        Self {
            kind: CALL,
            destination: destination.to_string(),
            path: path.to_string(),
            interface: interface.to_string(),
            member: member.to_string(),
            ..Self::default()
        }
    }

    pub fn arg(mut self, v: Value) -> Self {
        self.body.push(v);
        self
    }

    pub const fn no_reply(mut self) -> Self {
        self.flags |= NO_REPLY;
        self
    }

    pub fn body_signature(&self) -> String {
        self.body.iter().map(signature).collect()
    }

    pub fn arg_str(&self, index: usize) -> &str {
        self.body.get(index).and_then(Value::as_str).unwrap_or("")
    }

    pub fn encode(&self, serial: u32) -> Vec<u8> {
        let mut body = Encoder::new();
        for v in &self.body {
            body.value(v);
        }
        let body = body.take();

        let mut fields: Vec<Value> = Vec::new();
        let field = |code: u8, v: Value| {
            Value::Struct(vec![Value::Byte(code), Value::Variant(Box::new(v))])
        };
        if !self.path.is_empty() {
            fields.push(field(1, Value::path(&self.path)));
        }
        if !self.interface.is_empty() {
            fields.push(field(2, Value::str(&self.interface)));
        }
        if !self.member.is_empty() {
            fields.push(field(3, Value::str(&self.member)));
        }
        if !self.error_name.is_empty() {
            fields.push(field(4, Value::str(&self.error_name)));
        }
        if self.reply_serial != 0 {
            fields.push(field(5, Value::U32(self.reply_serial)));
        }
        if !self.destination.is_empty() {
            fields.push(field(6, Value::str(&self.destination)));
        }
        if !self.body.is_empty() {
            fields.push(field(8, Value::Sig(self.body_signature())));
        }

        let mut head = Encoder::new();
        head.byte(b'l');
        head.byte(self.kind);
        head.byte(self.flags);
        head.byte(1);
        head.u32(body.len() as u32);
        head.u32(serial);
        head.value(&Value::Array(fields));
        head.align(8);
        let mut out = head.take();
        out.extend_from_slice(&body);
        out
    }

    pub fn decode(buf: &[u8]) -> Decoded {
        if buf.len() < 16 {
            return Decoded::Incomplete;
        }
        let le = match buf[0] {
            b'l' => true,
            b'B' => false,
            _ => return Decoded::Bad,
        };
        if buf[3] != 1 {
            return Decoded::Bad;
        }
        let word = |at: usize| -> u32 {
            let raw = [buf[at], buf[at + 1], buf[at + 2], buf[at + 3]];
            if le {
                u32::from_le_bytes(raw)
            } else {
                u32::from_be_bytes(raw)
            }
        };
        let body_len = word(4) as usize;
        let serial = word(8);
        let fields_len = word(12) as usize;
        let head_end = match 16usize.checked_add(fields_len) {
            Some(v) => (v + 7) / 8 * 8,
            None => return Decoded::Bad,
        };
        let Some(total) = head_end.checked_add(body_len) else { return Decoded::Bad };
        if total > MAX_MESSAGE {
            return Decoded::Bad;
        }
        if buf.len() < total {
            return Decoded::Incomplete;
        }
        let mut msg = Self {
            kind: buf[1],
            flags: buf[2],
            serial,
            ..Self::default()
        };
        let mut dec = Decoder::at(buf, 12, le);
        let Some(fields) = dec.value("a(yv)") else { return Decoded::Bad };
        let mut body_sig = String::new();
        for f in fields.items() {
            let parts = f.items();
            let (Some(code), Some(v)) = (parts.first().and_then(Value::as_i64), parts.get(1)) else {
                continue;
            };
            match code {
                1 => msg.path = v.as_str().unwrap_or("").to_string(),
                2 => msg.interface = v.as_str().unwrap_or("").to_string(),
                3 => msg.member = v.as_str().unwrap_or("").to_string(),
                4 => msg.error_name = v.as_str().unwrap_or("").to_string(),
                5 => msg.reply_serial = v.as_i64().unwrap_or(0) as u32,
                6 => msg.destination = v.as_str().unwrap_or("").to_string(),
                7 => msg.sender = v.as_str().unwrap_or("").to_string(),
                8 => body_sig = v.as_str().unwrap_or("").to_string(),
                _ => {}
            }
        }
        if !body_sig.is_empty() {
            let mut body = Decoder::new(&buf[head_end..total], le);
            let Some(values) = body.values(&body_sig) else { return Decoded::Bad };
            msg.body = values;
        }
        Decoded::Message(msg, total)
    }
}

pub fn unescape(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn socket_path(address: &str) -> Option<String> {
    for entry in address.split(';') {
        let Some(rest) = entry.trim().strip_prefix("unix:") else { continue };
        for kv in rest.split(',') {
            if let Some(v) = kv.strip_prefix("path=") {
                return Some(unescape(v));
            }
        }
    }
    None
}

pub fn session_address() -> Option<String> {
    if let Ok(v) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        if !v.is_empty() {
            return Some(v);
        }
    }
    let uid = unsafe { libc::getuid() };
    let path = format!("/run/user/{uid}/bus");
    if std::path::Path::new(&path).exists() {
        Some(format!("unix:path={path}"))
    } else {
        None
    }
}

pub fn hex_ascii(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

pub fn auth_command(uid: u32) -> String {
    format!("AUTH EXTERNAL {}\r\n", hex_ascii(&uid.to_string()))
}

fn read_line(sock: &mut UnixStream, deadline: Instant) -> Option<String> {
    let mut line: Vec<u8> = Vec::new();
    while Instant::now() < deadline {
        let mut b = [0u8; 1];
        match sock.read(&mut b) {
            Ok(0) => return None,
            Ok(_) => {
                if b[0] == b'\n' {
                    while line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    return String::from_utf8(line).ok();
                }
                line.push(b[0]);
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
    None
}

fn authenticate(sock: &mut UnixStream) -> bool {
    let uid = unsafe { libc::getuid() };
    let deadline = Instant::now() + HANDSHAKE_TIMEOUT;
    if sock.write_all(&[0]).is_err() || sock.write_all(auth_command(uid).as_bytes()).is_err() {
        return false;
    }
    loop {
        let Some(line) = read_line(sock, deadline) else { return false };
        if line.starts_with("OK") {
            break;
        }
        if line.starts_with("DATA") {
            if sock.write_all(b"DATA\r\n").is_err() {
                return false;
            }
            continue;
        }
        if line.starts_with("REJECTED") || line.starts_with("ERROR") {
            return false;
        }
    }
    sock.write_all(b"BEGIN\r\n").is_ok()
}

pub struct Connection {
    sock: UnixStream,
    serial: u32,
    inbuf: Vec<u8>,
    outbuf: Vec<u8>,
    dead: bool,
    pub unique_name: String,
}

impl Connection {
    pub fn session() -> Option<Self> {
        let path = socket_path(&session_address()?)?;
        let mut sock = UnixStream::connect(path).ok()?;
        sock.set_read_timeout(Some(HANDSHAKE_TIMEOUT)).ok()?;
        sock.set_write_timeout(Some(HANDSHAKE_TIMEOUT)).ok()?;
        if !authenticate(&mut sock) {
            return None;
        }
        sock.set_nonblocking(true).ok()?;
        let mut conn = Self {
            sock,
            serial: 0,
            inbuf: Vec::new(),
            outbuf: Vec::new(),
            dead: false,
            unique_name: String::new(),
        };
        conn.send(&Message::call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "Hello",
        ));
        Some(conn)
    }

    pub fn fd(&self) -> RawFd {
        self.sock.as_raw_fd()
    }

    pub const fn dead(&self) -> bool {
        self.dead
    }

    pub fn send(&mut self, msg: &Message) -> u32 {
        if self.dead {
            return 0;
        }
        self.serial = self.serial.wrapping_add(1).max(1);
        let serial = self.serial;
        self.outbuf.extend_from_slice(&msg.encode(serial));
        self.flush();
        serial
    }

    pub fn add_match(&mut self, rule: &str) {
        let msg = Message::call(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "AddMatch",
        )
        .arg(Value::str(rule))
        .no_reply();
        self.send(&msg);
    }

    fn flush(&mut self) {
        while !self.outbuf.is_empty() {
            match self.sock.write(&self.outbuf) {
                Ok(0) => {
                    self.dead = true;
                    return;
                }
                Ok(n) => {
                    self.outbuf.drain(..n);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => {
                    self.dead = true;
                    return;
                }
            }
        }
    }

    pub fn pump(&mut self) -> Vec<Message> {
        if self.dead {
            return Vec::new();
        }
        self.flush();
        let mut buf = [0u8; 8192];
        loop {
            match self.sock.read(&mut buf) {
                Ok(0) => {
                    self.dead = true;
                    break;
                }
                Ok(n) => self.inbuf.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => {
                    self.dead = true;
                    break;
                }
            }
        }
        let mut out = Vec::new();
        loop {
            match Message::decode(&self.inbuf) {
                Decoded::Message(m, used) => {
                    self.inbuf.drain(..used);
                    if m.kind == RETURN && m.member.is_empty() && self.unique_name.is_empty() {
                        if let Some(name) = m.body.first().and_then(Value::as_str) {
                            if name.starts_with(':') {
                                self.unique_name = name.to_string();
                            }
                        }
                    }
                    out.push(m);
                }
                Decoded::Incomplete => break,
                Decoded::Bad => {
                    self.dead = true;
                    self.inbuf.clear();
                    break;
                }
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "dbus_tests.rs"]
mod tests;
