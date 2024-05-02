 use antibox_gfx::error::Result;
use crate::backend::DisplayBackend;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotspot {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone)]
pub struct XpmCursor {
    pub pixels: Vec<u8>,
    pub width: u16,
    pub height: u16,
    pub hotspot: Hotspot,
    pub foreground: [u8; 3],
    pub background: [u8; 3],
}

impl XpmCursor {
    pub fn load(path: &Path) -> Option<Self> {
        use crate::xpm;

        let pixmap = xpm::load_xpm_file(path)?;

        let (foreground, background) = Self::extract_colours(path)?;

        let hotspot = Self::read_hotspot(path, pixmap.width, pixmap.height);

        Some(Self {
            pixels: pixmap.data,
            width: pixmap.width,
            height: pixmap.height,
            hotspot,
            foreground,
            background,
        })
    }

    fn extract_colours(path: &Path) -> Option<([u8; 3], [u8; 3])> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let mut colours_found = 0;
        let mut foreground = [255u8, 255u8, 255u8];
        let mut background = [0u8, 0u8, 0u8];

        for line in reader.lines() {
            let line = line.ok()?;
            let line = line.trim();

            if line.starts_with("/*") || line.starts_with("//") {
                continue;
            }

            if line.len() >= 3 && line.starts_with('"') {
                let content = line
                    .trim_start_matches('"')
                    .trim_end_matches(&['"', ',', ';'] as &[_]);

                if let Some(c_pos) = content.find(" c ") {
                    let rest = c_pos + 3;
                    let colour_str = content[rest..].trim();
                    if colour_str.starts_with('#') && colour_str.len() >= 7 {
                        if let Ok(rgb) = u32::from_str_radix(&colour_str[1..7], 16) {
                            let r = ((rgb >> 16) & 0xFF) as u8;
                            let g = ((rgb >> 8) & 0xFF) as u8;
                            let b = (rgb & 0xFF) as u8;
                            if colours_found == 0 {
                                foreground = [r, g, b];
                            } else if colours_found == 1 {
                                background = [r, g, b];
                                break;
                            }
                            colours_found += 1;
                        }
                    }
                }
            }

            if line.starts_with('"') && line.ends_with("};") {
                break;
            }
        }

        Some((foreground, background))
    }

    fn read_hotspot(path: &Path, width: u16, height: u16) -> Hotspot {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(std::result::Result::ok) {
                let line = line.trim();
                if line.starts_with("/*")
                    || line.starts_with("//")
                    || line.starts_with('!')
                    || line.is_empty()
                {
                    continue;
                }

                if line.starts_with('"') {
                    let content = line
                        .trim_start_matches('"')
                        .trim_end_matches(&['"', ',', ';'] as &[_]);
                    let parts: Vec<&str> = content.split_whitespace().collect();

                    if parts.len() >= 6 {
                        if let (Ok(x), Ok(y)) = (parts[4].parse::<i32>(), parts[5].parse::<i32>()) {
                            if x >= 0 && y >= 0 {
                                return Hotspot {
                                    x: (x as u16).min(width - 1),
                                    y: (y as u16).min(height - 1),
                                };
                            }
                        }
                    }
                    break;
                }
            }
        }

        Self::guess_hotspot(path, width, height)
    }

    fn guess_hotspot(path: &Path, width: u16, height: u16) -> Hotspot {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let cx = width / 2;
        let cy = height / 2;

        if name.contains("size_") {
            if name.contains("bottom") {
                if name.contains("left") {
                    return Hotspot {
                        x: 0,
                        y: height - 1,
                    };
                } else if name.contains("right") {
                    return Hotspot {
                        x: width - 1,
                        y: height - 1,
                    };
                }
                return Hotspot {
                    x: cx,
                    y: height - 1,
                };
            } else if name.contains("top") {
                if name.contains("left") {
                    return Hotspot { x: 0, y: 0 };
                } else if name.contains("right") {
                    return Hotspot { x: width - 1, y: 0 };
                }
                return Hotspot { x: cx, y: 0 };
            } else if name.contains("left") {
                return Hotspot { x: 0, y: cy };
            } else if name.contains("right") {
                return Hotspot {
                    x: width - 1,
                    y: cy,
                };
            } else if name.contains('t') {
                return Hotspot { x: cx, y: 0 };
            } else if name.contains('b') {
                return Hotspot {
                    x: cx,
                    y: height - 1,
                };
            } else if name.contains('l') {
                return Hotspot { x: 0, y: cy };
            } else if name.contains('r') {
                return Hotspot {
                    x: width - 1,
                    y: cy,
                };
            }
        } else if name.contains("scroll") {
            if name.contains("left") || name.contains('l') {
                return Hotspot { x: 0, y: cy };
            } else if name.contains("right") || name.contains('r') {
                return Hotspot {
                    x: width - 1,
                    y: cy,
                };
            } else if name.contains("up") || name.contains('u') {
                return Hotspot { x: cx, y: 0 };
            } else if name.contains("down") || name.contains('d') {
                return Hotspot {
                    x: cx,
                    y: height - 1,
                };
            }
        } else if name.contains("left") {
            return Hotspot { x: 0, y: 0 };
        } else if name.contains("move") {
            return Hotspot { x: cx, y: cy };
        } else if name.contains("right") {
            return Hotspot { x: width - 1, y: 0 };
        }

        Hotspot { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    path: Option<String>,
    glyph: Option<u32>,
    xname: Option<String>,
}

impl Cursor {
    pub fn new(path: Option<&str>, glyph: Option<u32>, xname: Option<&str>) -> Self {
        Self {
            path: path.map(ToString::to_string),
            glyph,
            xname: xname.map(ToString::to_string),
        }
    }

    pub fn from_path(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
            glyph: None,
            xname: None,
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_ref().map(AsRef::as_ref)
    }

    pub const fn glyph(&self) -> Option<u32> {
        self.glyph
    }

    pub fn xname(&self) -> Option<&str> {
        self.xname.as_ref().map(AsRef::as_ref)
    }

    pub fn load_xpm(&self) -> Option<XpmCursor> {
        let path = self.path.as_ref()?;
        XpmCursor::load(Path::new(path))
    }

    pub fn load(&self, backend: &dyn DisplayBackend) -> Result<u32> {
        if self.path().is_some() {
            if let Ok(c) = self.load_xpm_to_cursor(backend) {
                return Ok(c);
            }
        }

        if let Some(xname) = self.xname() {
            if let Ok(c) = backend.create_named_cursor(xname) {
                return Ok(c);
            }
        }

        if let Some(glyph) = self.glyph() {
            return backend.create_font_cursor(glyph);
        }

        Err("Cursor: no cursor source available".into())
    }

    fn load_xpm_to_cursor(
        &self,
        backend: &dyn DisplayBackend,
    ) -> Result<u32> {
        let xpm = self.load_xpm().ok_or("Cursor: failed to load XPM")?;
        backend.create_cursor_from_rgba(
            &xpm.pixels,
            crate::point::Dimension::px(xpm.width, xpm.height),
            crate::point::Point::new(xpm.hotspot.x as i32, xpm.hotspot.y as i32),
            xpm.foreground,
            xpm.background,
        )
    }
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
