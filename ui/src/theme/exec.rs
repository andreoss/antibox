use super::dsl::*;
use antibox_gfx::backend::GraphicsContext;
use antibox_gfx::colour::lerp;

fn clamp_i16(v: i32) -> i16 {
    v.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn clamp_u16(v: i32) -> u16 {
    v.clamp(0, i32::from(u16::MAX)) as u16
}

pub fn draw_ops(
    g: &dyn GraphicsContext,
    def: &ThemeDef,
    ops: &[Op],
    ox: i16,
    oy: i16,
    w: i16,
    h: i16,
    s: i16,
) -> bool {
    let mut painted = false;
    for op in ops {
        match op {
            Op::Fill { colour } => {
                if let Some(c) = resolve_colour(def, colour) {
                    let _ = g.set_foreground(c);
                    if g.fill_rect(ox, oy, clamp_u16(i32::from(w)), clamp_u16(i32::from(h))).is_ok()
                    {
                        painted = true;
                    }
                }
            }
            Op::Outline {
                colour,
                x,
                y,
                w: ow,
                h: oh,
            } => {
                if let Some(c) = resolve_colour(def, colour) {
                    let _ = g.set_foreground(c);
                    let rx = clamp_i16(x.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let ry = clamp_i16(y.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let rw = clamp_u16(ow.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let rh = clamp_u16(oh.eval(i32::from(w), i32::from(h), i32::from(s)));
                    if g.draw_rect(ox + rx, oy + ry, rw, rh).is_ok() {
                        painted = true;
                    }
                }
            }
            Op::Line {
                x1,
                y1,
                x2,
                y2,
                colour,
            } => {
                if let Some(c) = resolve_colour(def, colour) {
                    let _ = g.set_foreground(c);
                    let p = |c: &Coord| c.eval(i32::from(w), i32::from(h), i32::from(s));
                    let a = (ox + clamp_i16(p(x1)), oy + clamp_i16(p(y1)));
                    let b = (ox + clamp_i16(p(x2)), oy + clamp_i16(p(y2)));
                    if g.draw_line(a.0, a.1, b.0, b.1).is_ok() {
                        painted = true;
                    }
                }
            }
            Op::Gradient {
                from,
                to,
                vertical,
                x,
                y,
                w: gw,
                h: gh,
            } => {
                let Some(fc) = resolve_colour(def, from) else { continue };
                let Some(tc) = resolve_colour(def, to) else { continue };
                let gx = ox + clamp_i16(x.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gy = oy + clamp_i16(y.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gw = clamp_u16(gw.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gh = clamp_u16(gh.eval(i32::from(w), i32::from(h), i32::from(s)));
                let span = if *vertical { gh } else { gw };
                for i in 0..span {
                    let t = if span <= 1 {
                        0.0
                    } else {
                        i as f32 / (span - 1) as f32
                    };
                    let _ = g.set_foreground(lerp(fc, tc, t));
                    let ok = if *vertical {
                        g.fill_rect(gx, gy + i as i16, gw, 1)
                    } else {
                        g.fill_rect(gx + i as i16, gy, 1, gh)
                    };
                    if ok.is_ok() {
                        painted = true;
                    }
                }
            }
            Op::Bevel {
                raised,
                light,
                shadow,
                depth,
            } => {
                let d = depth
                    .eval(i32::from(w), i32::from(h), i32::from(s))
                    .clamp(0, i32::from(h.min(w)));
                let (hi, lo) = if *raised {
                    (light, shadow)
                } else {
                    (shadow, light)
                };
                for i in 0..d {
                    if let Some(c) = resolve_colour(def, hi) {
                        let _ = g.set_foreground(c);
                        let _ = g.draw_line(
                            ox + i as i16,
                            oy + i as i16,
                            ox + w - 1 - i as i16,
                            oy + i as i16,
                        );
                        let _ = g.draw_line(
                            ox + i as i16,
                            oy + i as i16,
                            ox + i as i16,
                            oy + h - 1 - i as i16,
                        );
                    }
                    if let Some(c) = resolve_colour(def, lo) {
                        let _ = g.set_foreground(c);
                        let _ = g.draw_line(
                            ox + i as i16,
                            oy + h - 1 - i as i16,
                            ox + w - 1 - i as i16,
                            oy + h - 1 - i as i16,
                        );
                        let _ = g.draw_line(
                            ox + w - 1 - i as i16,
                            oy + i as i16,
                            ox + w - 1 - i as i16,
                            oy + h - 1 - i as i16,
                        );
                    }
                    painted = true;
                }
            }
        }
    }
    painted
}
