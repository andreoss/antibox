use std::fmt;
use std::str::FromStr;

pub type Colour = u32;

pub fn scale(c: Colour, f: f32) -> Colour {
    let r = (((c >> 16) & 0xFF) as f32 * f) as u32;
    let g = (((c >> 8) & 0xFF) as f32 * f) as u32;
    let b = ((c & 0xFF) as f32 * f) as u32;
    (clamp255(r) << 16) | (clamp255(g) << 8) | clamp255(b)
}

pub fn tint(c: Colour, f: f32) -> Colour {
    let r = (((c >> 16) & 0xFF) as f32 + (255.0 - ((c >> 16) & 0xFF) as f32) * f) as u32;
    let g = (((c >> 8) & 0xFF) as f32 + (255.0 - ((c >> 8) & 0xFF) as f32) * f) as u32;
    let b = ((c & 0xFF) as f32 + (255.0 - (c & 0xFF) as f32) * f) as u32;
    (clamp255(r) << 16) | (clamp255(g) << 8) | clamp255(b)
}

fn clamp255(v: u32) -> u32 {
    if v > 255 {
        255
    } else {
        v
    }
}

pub fn lerp(a: Colour, b: Colour, t: f32) -> Colour {
    let ca = ((a >> 16) & 0xFF) as f32;
    let cb = ((b >> 16) & 0xFF) as f32;
    let r = clamp255((ca + (cb - ca) * t) as u32);
    let ca = ((a >> 8) & 0xFF) as f32;
    let cb = ((b >> 8) & 0xFF) as f32;
    let g = clamp255((ca + (cb - ca) * t) as u32);
    let ca = (a & 0xFF) as f32;
    let cb = (b & 0xFF) as f32;
    let b = clamp255((ca + (cb - ca) * t) as u32);
    (r << 16) | (g << 8) | b
}

fn clamp_i32(v: i32) -> u32 {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v as u32
    }
}

pub fn shift(c: Colour, d: i32) -> Colour {
    (clamp_i32(((c >> 16) & 0xFF) as i32 + d) << 16)
        | (clamp_i32(((c >> 8) & 0xFF) as i32 + d) << 8)
        | clamp_i32((c & 0xFF) as i32 + d)
}

fn luma(c: Colour) -> u32 {
    (((c >> 16) & 0xFF) * 299 + ((c >> 8) & 0xFF) * 587 + (c & 0xFF) * 114) / 1000
}

pub fn is_dark(c: Colour) -> bool {
    luma(c) < 128
}

pub fn contrast(fg: Colour, bg: Colour) -> Colour {
    let lb = luma(bg);
    let lf = luma(fg);
    let d = if lf >= lb { lf - lb } else { lb - lf };
    if d * 4 >= 255 {
        fg
    } else if lb * 2 >= 255 {
        0x000000
    } else {
        0xFFFFFF
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColour {
    pub const fn new(r: u8, g: u8, b: u8) -> RgbColour {
        RgbColour { r, g, b }
    }

    pub const fn to_pixel(&self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn from_pixel(p: Colour) -> RgbColour {
        RgbColour {
            r: ((p >> 16) & 0xFF) as u8,
            g: ((p >> 8) & 0xFF) as u8,
            b: (p & 0xFF) as u8,
        }
    }

    #[must_use]
    pub fn lerp(&self, other: &Self, t: f32) -> RgbColour {
        Self::from_pixel(lerp(self.to_pixel(), other.to_pixel(), t))
    }
}

impl FromStr for RgbColour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.starts_with('#') {
            let hex = &s[1..];
            if hex.len() == 6 {
                let r =
                    u8::from_str_radix(&hex[0..2], 16).map_err(|e| format!("Invalid hex: {}", e))?;
                let g =
                    u8::from_str_radix(&hex[2..4], 16).map_err(|e| format!("Invalid hex: {}", e))?;
                let b =
                    u8::from_str_radix(&hex[4..6], 16).map_err(|e| format!("Invalid hex: {}", e))?;
                return Ok(RgbColour { r, g, b });
            }
        }
        if s.starts_with("rgb:") {
            let rest = &s[4..];
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() == 3 {
                let parse_hex = |s: &str| {
                    let s = s.trim();
                    if s.len() == 2 {
                        u8::from_str_radix(s, 16)
                    } else {
                        u8::from_str_radix(s, 16).map(|v| v * 17)
                    }
                };
                let r = parse_hex(parts[0]).map_err(|e| format!("Invalid R: {}", e))?;
                let g = parse_hex(parts[1]).map_err(|e| format!("Invalid G: {}", e))?;
                let b = parse_hex(parts[2]).map_err(|e| format!("Invalid B: {}", e))?;
                return Ok(RgbColour { r, g, b });
            }
        }
        Err(format!("Invalid colour format: {}", s))
    }
}

impl fmt::Display for RgbColour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
#[path = "colour_tests.rs"]
mod tests;
