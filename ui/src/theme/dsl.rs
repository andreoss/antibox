use antibox_gfx::colour::Colour;
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub enum Coord {
    Lit(i32),
    W,
    H,
    S,
    Add(Box<Coord>, Box<Coord>),
    Sub(Box<Coord>, Box<Coord>),
    Mul(Box<Coord>, Box<Coord>),
}

impl Coord {
    pub fn eval(&self, w: i32, h: i32, s: i32) -> i32 {
        match self {
            Self::Lit(v) => *v,
            Self::W => w,
            Self::H => h,
            Self::S => s,
            Self::Add(a, b) => a.eval(w, h, s) + b.eval(w, h, s),
            Self::Sub(a, b) => a.eval(w, h, s) - b.eval(w, h, s),
            Self::Mul(a, b) => a.eval(w, h, s) * b.eval(w, h, s),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ColourRef {
    Named(String),
    Scale(String, f32),
    Blend(String, String, f32),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    Fill {
        colour: ColourRef,
    },
    Outline {
        colour: ColourRef,
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
    },
    Line {
        x1: Coord,
        y1: Coord,
        x2: Coord,
        y2: Coord,
        colour: ColourRef,
    },
    Gradient {
        from: ColourRef,
        to: ColourRef,
        vertical: bool,
        x: Coord,
        y: Coord,
        w: Coord,
        h: Coord,
    },
    Bevel {
        raised: bool,
        light: ColourRef,
        shadow: ColourRef,
        depth: Coord,
    },
}

#[derive(Clone, Default, Debug)]
pub struct ThemeDef {
    pub name: String,
    pub colours: HashMap<String, u32>,
    pub metrics: HashMap<String, i64>,
    pub strings: HashMap<String, String>,
    pub elements: HashMap<String, Vec<Op>>,
}

impl ThemeDef {
    pub fn colour(&self, name: &str) -> Option<u32> {
        self.colours.get(name).copied()
    }

    pub fn metric(&self, name: &str) -> Option<i64> {
        self.metrics.get(name).copied()
    }

    pub fn string(&self, name: &str) -> Option<&str> {
        self.strings.get(name).map(String::as_str)
    }

    pub fn element(&self, name: &str) -> Option<&[Op]> {
        self.elements.get(name).map(Vec::as_slice)
    }
}

const fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn parse_primary(src: &str, pos: &mut usize) -> Option<Coord> {
    let bytes = src.as_bytes();
    while *pos < bytes.len() && bytes[*pos] == b' ' {
        *pos += 1;
    }
    if *pos >= bytes.len() {
        return None;
    }
    match bytes[*pos] {
        b'w' => {
            *pos += 1;
            Some(Coord::W)
        }
        b'h' => {
            *pos += 1;
            Some(Coord::H)
        }
        b's' => {
            *pos += 1;
            Some(Coord::S)
        }
        b'(' => {
            *pos += 1;
            let inner = parse_sum(src, pos)?;
            while *pos < bytes.len() && bytes[*pos] == b' ' {
                *pos += 1;
            }
            if *pos >= bytes.len() || bytes[*pos] != b')' {
                return None;
            }
            *pos += 1;
            Some(inner)
        }
        c if c.is_ascii_digit() => {
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos].is_ascii_digit() {
                *pos += 1;
            }
            let v: i32 = src[start..*pos].parse().ok()?;
            Some(Coord::Lit(v))
        }
        _ => None,
    }
}

fn parse_mul(src: &str, pos: &mut usize) -> Option<Coord> {
    let mut left = parse_primary(src, pos)?;
    loop {
        let save = *pos;
        let bytes = src.as_bytes();
        while *pos < bytes.len() && bytes[*pos] == b' ' {
            *pos += 1;
        }
        if *pos < bytes.len() && bytes[*pos] == b'*' {
            *pos += 1;
            let right = parse_primary(src, pos)?;
            left = Coord::Mul(Box::new(left), Box::new(right));
        } else {
            *pos = save;
            return Some(left);
        }
    }
}

fn parse_sum(src: &str, pos: &mut usize) -> Option<Coord> {
    let mut left = parse_mul(src, pos)?;
    loop {
        let save = *pos;
        let bytes = src.as_bytes();
        while *pos < bytes.len() && bytes[*pos] == b' ' {
            *pos += 1;
        }
        if *pos >= bytes.len() {
            *pos = save;
            return Some(left);
        }
        match bytes[*pos] {
            b'+' => {
                *pos += 1;
                let right = parse_mul(src, pos)?;
                left = Coord::Add(Box::new(left), Box::new(right));
            }
            b'-' => {
                *pos += 1;
                let right = parse_mul(src, pos)?;
                left = Coord::Sub(Box::new(left), Box::new(right));
            }
            _ => {
                *pos = save;
                return Some(left);
            }
        }
    }
}

pub fn parse_coord(src: &str) -> Option<Coord> {
    let mut pos = 0usize;
    let c = parse_sum(src, &mut pos)?;
    let bytes = src.as_bytes();
    while pos < bytes.len() && bytes[pos] == b' ' {
        pos += 1;
    }
    if pos != bytes.len() {
        return None;
    }
    Some(c)
}

pub fn parse_colour_ref(src: &str) -> Option<ColourRef> {
    let s = src.trim();
    if let Some(idx) = s.find('~') {
        let (a, rest) = s.split_at(idx);
        let rest = &rest[1..];
        let (b, f) = rest.split_at(rest.find('*').unwrap_or(rest.len()));
        let f = if f.is_empty() {
            0.5
        } else {
            f.trim_start_matches('*').trim().parse::<f32>().ok()?
        };
        return Some(ColourRef::Blend(a.trim().to_string(), b.trim().to_string(), f));
    }
    if let Some(idx) = s.find('*') {
        let (name, f) = s.split_at(idx);
        let f = f[1..].trim().parse::<f32>().ok()?;
        return Some(ColourRef::Scale(name.trim().to_string(), f));
    }
    if s.is_empty() || !s.bytes().all(is_ident) {
        return None;
    }
    Some(ColourRef::Named(s.to_string()))
}

pub fn parse_hex(src: &str) -> Option<u32> {
    let s = src.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    u32::from_str_radix(s, 16).ok()
}

pub fn resolve_colour(def: &ThemeDef, r: &ColourRef) -> Option<Colour> {
    match r {
        ColourRef::Named(n) => def.colour(n),
        ColourRef::Scale(n, f) => {
            let base = def.colour(n)?;
            Some(scale_rgb(base, *f))
        }
        ColourRef::Blend(a, b, f) => {
            let ca = def.colour(a)?;
            let cb = def.colour(b)?;
            Some(mix_rgb(ca, cb, *f))
        }
    }
}

fn scale_rgb(c: u32, f: f32) -> u32 {
    let ch = |shift: u32| {
        let v = ((c >> shift) & 0xFF) as f32 * f;
        (v.clamp(0.0, 255.0) as u32) << shift
    };
    ch(16) | ch(8) | ch(0)
}

fn mix_rgb(a: u32, b: u32, t: f32) -> u32 {
    let ch = |shift: u32| {
        let av = ((a >> shift) & 0xFF) as f32;
        let bv = ((b >> shift) & 0xFF) as f32;
        (((av + (bv - av) * t).clamp(0.0, 255.0)) as u32) << shift
    };
    ch(16) | ch(8) | ch(0)
}

#[cfg(test)]
#[path = "dsl_tests.rs"]
mod tests;
