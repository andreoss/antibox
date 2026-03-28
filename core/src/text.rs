const ESC: u8 = 0x1B;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Charset {
    Latin1,
    Cyrillic,
    Greek,
    Unsupported,
}

impl Charset {
    const fn from_final(f: u8) -> Self {
        match f {
            b'A' => Self::Latin1,
            b'L' => Self::Cyrillic,
            b'F' => Self::Greek,
            _ => Self::Unsupported,
        }
    }

    fn decode(self, b: u8) -> Option<char> {
        let high = u32::from(b) - 0xA0;
        match self {
            Self::Latin1 => char::from_u32(u32::from(b)),
            Self::Cyrillic => match b {
                0xA0 => Some('\u{00A0}'),
                0xAD => Some('\u{00AD}'),
                0xF0 => Some('\u{2116}'),
                0xFD => Some('\u{00A7}'),
                _ => char::from_u32(0x0400 + high),
            },
            Self::Greek => match b {
                0xA0 => Some('\u{00A0}'),
                0xA1 => Some('\u{2018}'),
                0xA2 => Some('\u{2019}'),
                0xAF => Some('\u{2015}'),
                0xA3..=0xAD | 0xB0..=0xFF => char::from_u32(0x0370 + high),
                _ => None,
            },
            Self::Unsupported => None,
        }
    }
}

fn decode_iso2022(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len());
    let mut right = Charset::Latin1;
    let mut left_ascii = true;
    let mut i = 0;
    while i < data.len() {
        let b = data[i];
        if b == ESC {
            let rest = data.len() - i;
            if rest >= 3 && data[i + 1] == b'%' {
                i += 3;
                if data[i - 1] != b'G' {
                    continue;
                }
                let start = i;
                while i < data.len() && data[i] != ESC {
                    i += 1;
                }
                out.push_str(&String::from_utf8_lossy(&data[start..i]));
                continue;
            }
            if rest >= 4 && data[i + 1] == b'$' && matches!(data[i + 2], b'(' | b')' | b'-') {
                left_ascii = false;
                i += 4;
                continue;
            }
            if rest >= 3 && data[i + 1] == b'$' {
                left_ascii = false;
                i += 3;
                continue;
            }
            if rest >= 3 && matches!(data[i + 1], b'-' | b')') {
                right = Charset::from_final(data[i + 2]);
                i += 3;
                continue;
            }
            if rest >= 3 && data[i + 1] == b'(' {
                left_ascii = data[i + 2] == b'B';
                i += 3;
                continue;
            }
            i = data.len();
            continue;
        }
        if b < 0x80 {
            if left_ascii {
                out.push(char::from(b));
            }
            i += 1;
            continue;
        }
        if let Some(c) = right.decode(b) {
            out.push(c);
        }
        i += 1;
    }
    out
}

pub fn decode_property(data: &[u8]) -> String {
    let trimmed = match data.iter().position(|b| *b == 0) {
        Some(end) => &data[..end],
        None => data,
    };
    if trimmed.contains(&ESC) {
        return decode_iso2022(trimmed);
    }
    match std::str::from_utf8(trimmed) {
        Ok(s) => s.to_string(),
        Err(_) => trimmed.iter().map(|b| char::from(*b)).collect(),
    }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
