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
    draw_ops_on(g, def, ops, ox, oy, w, h, s, None)
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ops_on(
    g: &dyn GraphicsContext,
    def: &ThemeDef,
    ops: &[Op],
    ox: i16,
    oy: i16,
    w: i16,
    h: i16,
    s: i16,
    bg: Option<antibox_gfx::colour::Colour>,
) -> bool {
    let resolve_colour = |r: &_| resolve_colour_on(def, r, bg);
    let mut painted = false;
    for op in ops {
        match op {
            Op::Fill {
                colour,
                x,
                y,
                w: fw,
                h: fh,
            } => {
                if let Some(c) = resolve_colour(colour) {
                    let _ = g.set_foreground(c);
                    let rx = clamp_i16(x.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let ry = clamp_i16(y.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let rw = clamp_u16(fw.eval(i32::from(w), i32::from(h), i32::from(s)));
                    let rh = clamp_u16(fh.eval(i32::from(w), i32::from(h), i32::from(s)));
                    if g.fill_rect(ox + rx, oy + ry, rw, rh).is_ok() {
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
                if let Some(c) = resolve_colour(colour) {
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
                if let Some(c) = resolve_colour(colour) {
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
                kind,
                x,
                y,
                w: gw,
                h: gh,
            } => {
                let Some(fc) = resolve_colour(from) else { continue };
                let Some(tc) = resolve_colour(to) else { continue };
                let gx = ox + clamp_i16(x.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gy = oy + clamp_i16(y.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gw = clamp_u16(gw.eval(i32::from(w), i32::from(h), i32::from(s)));
                let gh = clamp_u16(gh.eval(i32::from(w), i32::from(h), i32::from(s)));
                if gw == 0 || gh == 0 {
                    continue;
                }
                let (nx, ny) = match kind {
                    GradKind::Vertical => (1, gh),
                    GradKind::Horizontal => (gw, 1),
                    GradKind::Diagonal => (gw.min(48), gh.min(48)),
                };
                let denx = f32::from(nx.saturating_sub(1).max(1));
                let deny = f32::from(ny.saturating_sub(1).max(1));
                for j in 0..ny {
                    let cy = gy + (i32::from(j) * i32::from(gh) / i32::from(ny)) as i16;
                    let ch = (i32::from(j + 1) * i32::from(gh) / i32::from(ny)
                        - i32::from(j) * i32::from(gh) / i32::from(ny))
                    .max(1) as u16;
                    let fy = if ny == 1 { 0.5 } else { f32::from(j) / deny };
                    for i in 0..nx {
                        let cx = gx + (i32::from(i) * i32::from(gw) / i32::from(nx)) as i16;
                        let cw = (i32::from(i + 1) * i32::from(gw) / i32::from(nx)
                            - i32::from(i) * i32::from(gw) / i32::from(nx))
                        .max(1) as u16;
                        let fx = if nx == 1 { 0.5 } else { f32::from(i) / denx };
                        let _ = g.set_foreground(lerp(fc, tc, kind.at(fx, fy)));
                        if g.fill_rect(cx, cy, cw, ch).is_ok() {
                            painted = true;
                        }
                    }
                }
            }
            Op::Polygon { colour, points } => {
                if let Some(c) = resolve_colour(colour) {
                    let _ = g.set_foreground(c);
                    let p: Vec<(i16, i16)> = points
                        .iter()
                        .map(|(px, py)| {
                            (
                                ox + clamp_i16(px.eval(i32::from(w), i32::from(h), i32::from(s))),
                                oy + clamp_i16(py.eval(i32::from(w), i32::from(h), i32::from(s))),
                            )
                        })
                        .collect();
                    if g.fill_polygon(&p).is_ok() {
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
                    if let Some(c) = resolve_colour(hi) {
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
                    if let Some(c) = resolve_colour(lo) {
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
