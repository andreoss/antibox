use super::dsl::*;
use antibox_gfx::error::Error;
use antibox_gfx::error::Result;

fn as_str(v: &toml::Value) -> Option<&str> {
    v.as_str()
}

fn coord(v: &toml::Value, field: &str) -> Result<Coord> {
    let raw = v
        .get(field)
        .and_then(as_str)
        .ok_or_else(|| Error::message(format!("missing coord {}", field)))?;
    parse_coord(raw)
        .ok_or_else(|| Error::message(format!("bad coord {}: {}", field, raw)))
}

fn colour(v: &toml::Value, field: &str) -> Result<ColourRef> {
    let raw = v
        .get(field)
        .and_then(as_str)
        .ok_or_else(|| Error::message(format!("missing colour {}", field)))?;
    parse_colour_ref(raw)
        .ok_or_else(|| Error::message(format!("bad colour {}: {}", field, raw)))
}

fn parse_op(v: &toml::Value) -> Result<Op> {
    let kind = v
        .get("op")
        .and_then(as_str)
        .ok_or_else(|| Error::message("op without a kind"))?;
    match kind {
        "fill" => Ok(Op::Fill {
            colour: colour(v, "colour")?,
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
            vertical: v.get("vertical").and_then(|v| v.as_bool()).unwrap_or(true),
            x: coord(v, "x")?,
            y: coord(v, "y")?,
            w: coord(v, "w")?,
            h: coord(v, "h")?,
        }),
        "bevel" => Ok(Op::Bevel {
            raised: v.get("raised").and_then(|v| v.as_bool()).unwrap_or(true),
            light: colour(v, "light")?,
            shadow: colour(v, "shadow")?,
            depth: coord(v, "depth")?,
        }),
        other => Err(Error::message(format!(
            "unknown op {}",
            other
        ))),
    }
}

pub fn from_toml(text: &str) -> Result<ThemeDef> {
    let doc = match text.parse::<toml::Value>() {
        Ok(d) => d,
        Err(e) => return Err(Error::message(e.to_string())),
    };
    let mut def = ThemeDef::default();
    def.name = doc
        .get("name")
        .and_then(as_str)
        .unwrap_or("untitled")
        .to_string();

    if let Some(t) = doc.get("colors").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(s) = v.as_str() {
                if let Some(c) = parse_hex(s) {
                    def.colours.insert(k.clone(), c);
                }
            }
        }
    }
    if let Some(t) = doc.get("metrics").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(n) = v.as_integer() {
                def.metrics.insert(k.clone(), n);
            }
        }
    }
    if let Some(t) = doc.get("strings").and_then(|v| v.as_table()) {
        for (k, v) in t {
            if let Some(s) = v.as_str() {
                def.strings.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(t) = doc.get("elements").and_then(|v| v.as_table()) {
        for (name, v) in t {
            let ops = v
                .get("ops")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|o| parse_op(o).ok()).collect())
                .unwrap_or_default();
            def.elements.insert(name.clone(), ops);
        }
    }
    Ok(def)
}
