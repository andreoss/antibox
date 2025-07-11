use super::dsl::*;
use antibox_gfx::error::Error;
use antibox_gfx::error::Result;

fn as_str(v: &toml::Value) -> Option<&str> {
    v.as_str()
}

fn as_coord(v: &toml::Value) -> Option<Coord> {
    match v {
        toml::Value::Integer(n) => i32::try_from(*n).ok().map(Coord::Lit),
        _ => v.as_str().and_then(parse_coord),
    }
}

fn coord(v: &toml::Value, field: &str) -> Result<Coord> {
    let raw = v
        .get(field)
        .ok_or_else(|| Error::message(format!("missing coord {field}")))?;
    as_coord(raw).ok_or_else(|| Error::message(format!("bad coord {field}: {raw}")))
}

fn coord_or(v: &toml::Value, field: &str, fallback: Coord) -> Result<Coord> {
    match v.get(field) {
        Some(raw) => {
            as_coord(raw).ok_or_else(|| Error::message(format!("bad coord {field}: {raw}")))
        }
        None => Ok(fallback),
    }
}

fn colour(v: &toml::Value, field: &str) -> Result<ColourRef> {
    let raw = v
        .get(field)
        .and_then(as_str)
        .ok_or_else(|| Error::message(format!("missing colour {field}")))?;
    parse_colour_ref(raw)
        .ok_or_else(|| Error::message(format!("bad colour {field}: {raw}")))
}

fn parse_op(v: &toml::Value) -> Result<Op> {
    let kind = v
        .get("op")
        .and_then(as_str)
        .ok_or_else(|| Error::message("op without a kind"))?;
    match kind {
        "fill" => Ok(Op::Fill {
            colour: colour(v, "colour")?,
            x: coord_or(v, "x", Coord::Lit(0))?,
            y: coord_or(v, "y", Coord::Lit(0))?,
            w: coord_or(v, "w", Coord::W)?,
            h: coord_or(v, "h", Coord::H)?,
        }),
        "outline" => Ok(Op::Outline {
            colour: colour(v, "colour")?,
            x: coord(v, "x")?,
            y: coord(v, "y")?,
            w: coord(v, "w")?,
            h: coord(v, "h")?,
        }),
        "line" => Ok(Op::Line {
            x1: coord(v, "x1")?,
            y1: coord(v, "y1")?,
            x2: coord(v, "x2")?,
            y2: coord(v, "y2")?,
            colour: colour(v, "colour")?,
        }),
        "gradient" => Ok(Op::Gradient {
            from: colour(v, "from")?,
            to: colour(v, "to")?,
            kind: match v.get("kind").and_then(as_str) {
                Some(k) => GradKind::parse(k),
                None if v.get("vertical").and_then(toml::Value::as_bool) == Some(false) => {
                    GradKind::Horizontal
                }
                None => GradKind::Vertical,
            },
            x: coord(v, "x")?,
            y: coord(v, "y")?,
            w: coord(v, "w")?,
            h: coord(v, "h")?,
        }),
        "polygon" => {
            let pts = v
                .get("points")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| Error::message("polygon without points"))?;
            let mut points = Vec::with_capacity(pts.len());
            for p in pts {
                let pair = p
                    .as_array()
                    .filter(|a| a.len() == 2)
                    .ok_or_else(|| Error::message("polygon point must be a pair"))?;
                let one = |i: usize| {
                    pair[i]
                        .as_str()
                        .and_then(parse_coord)
                        .ok_or_else(|| Error::message("bad polygon coord"))
                };
                points.push((one(0)?, one(1)?));
            }
            Ok(Op::Polygon {
                colour: colour(v, "colour")?,
                points,
            })
        }
        "bevel" => Ok(Op::Bevel {
            raised: v.get("raised").and_then(toml::Value::as_bool).unwrap_or(true),
            light: colour(v, "light")?,
            shadow: colour(v, "shadow")?,
            depth: coord(v, "depth")?,
        }),
        other => Err(Error::message(format!(
            "unknown op {other}"
        ))),
    }
}

pub fn from_toml(text: &str) -> Result<ThemeDef> {
    let doc = match text.parse::<toml::Value>() {
        Ok(d) => d,
        Err(e) => return Err(Error::message(e.to_string())),
    };
    let mut def = ThemeDef {
        name: doc
            .get("name")
            .and_then(as_str)
            .unwrap_or("untitled")
            .to_string(),
        ..ThemeDef::default()
    };

    if let Some(t) = doc.get("colors").and_then(toml::Value::as_table) {
        for (k, v) in t {
            if let Some(s) = v.as_str() {
                if let Some(c) = parse_hex(s) {
                    def.colours.insert(k.clone(), c);
                }
            }
        }
    }
    if let Some(t) = doc.get("metrics").and_then(toml::Value::as_table) {
        for (k, v) in t {
            if let Some(n) = v.as_integer() {
                def.metrics.insert(k.clone(), n);
            }
        }
    }
    if let Some(t) = doc.get("strings").and_then(toml::Value::as_table) {
        for (k, v) in t {
            if let Some(s) = v.as_str() {
                def.strings.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(t) = doc.get("flags").and_then(toml::Value::as_table) {
        for (k, v) in t {
            if let Some(b) = v.as_bool() {
                def.flags.insert(k.clone(), b);
            }
        }
    }
    if let Some(t) = doc.get("glyphs").and_then(toml::Value::as_table) {
        for (k, v) in t {
            if let Some(rows) = v.as_array() {
                let rows: Vec<u16> = rows
                    .iter()
                    .filter_map(toml::Value::as_integer)
                    .map(|n| n as u16)
                    .collect();
                def.glyphs.insert(k.clone(), rows);
            }
        }
    }
    if let Some(t) = doc.get("elements").and_then(toml::Value::as_table) {
        for (name, v) in t {
            let mut ops = Vec::new();
            for o in v.get("ops").and_then(toml::Value::as_array).unwrap_or(&Vec::new()) {
                ops.push(parse_op(o).map_err(|e| {
                    Error::message(format!("element {name}: {e}"))
                })?);
            }
            def.elements.insert(name.clone(), ops);
        }
    }
    Ok(def)
}
