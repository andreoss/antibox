use antibox_core::backend::{IconData, PixmapData};

pub fn select_icon(icons: &[IconData], target: u32) -> Option<&IconData> {
    icons
        .iter()
        .filter(|i| i.width >= target && i.height >= target && i.width > 0 && i.height > 0)
        .min_by_key(|i| i.width)
        .or_else(|| {
            icons
                .iter()
                .filter(|i| i.width > 0 && i.height > 0)
                .max_by_key(|i| i.width * i.height)
        })
}

pub fn icon_to_pixmap(
    icon: &IconData,
    tw: u16,
    th: u16,
    bg: antibox_core::colour::Colour,
) -> PixmapData {
    let sw = icon.width.max(1);
    let sh = icon.height.max(1);
    let (br, bgc, bb) = ((bg >> 16) & 0xff, (bg >> 8) & 0xff, bg & 0xff);
    let mut data = Vec::with_capacity(tw as usize * th as usize * 4);
    for ty in 0..th as u32 {
        let sy = (ty * sh / th.max(1) as u32).min(sh - 1);
        for tx in 0..tw as u32 {
            let sx = (tx * sw / tw.max(1) as u32).min(sw - 1);
            let px = icon
                .pixels
                .get((sy * sw + sx) as usize)
                .copied()
                .unwrap_or(0);
            let a = (px >> 24) & 0xff;
            let r = (px >> 16) & 0xff;
            let g = (px >> 8) & 0xff;
            let b = px & 0xff;
            let blend = |fg: antibox_core::colour::Colour, bgv: u32| {
                ((fg * a + bgv * (255 - a)) / 255) as u8
            };
            data.push(blend(r, br));
            data.push(blend(g, bgc));
            data.push(blend(b, bb));
            data.push(255);
        }
    }
    PixmapData::new(tw, th, data)
}

pub fn default_icon(size: u32, bg: antibox_core::colour::Colour) -> PixmapData {
    let s = size.max(4);
    let (br, bgc, bb) = (
        ((bg >> 16) & 0xff) as u8,
        ((bg >> 8) & 0xff) as u8,
        (bg & 0xff) as u8,
    );
    let split = |c: u32| {
        (
            ((c >> 16) & 0xff) as u8,
            ((c >> 8) & 0xff) as u8,
            (c & 0xff) as u8,
        )
    };
    let (frame, title, body) = (
        split(antibox_ui::theme::shadow()),
        split(antibox_ui::theme::title_active()),
        split(antibox_ui::theme::field()),
    );
    let m = (s / 8).max(1);
    let title_h = (s / 4).max(2);
    let mut data = Vec::with_capacity((s * s * 4) as usize);
    for y in 0..s {
        for x in 0..s {
            let inside = x >= m && x < s - m && y >= m && y < s - m;
            let (r, g, b) = if !inside {
                (br, bgc, bb)
            } else if x == m || x == s - m - 1 || y == m || y == s - m - 1 {
                frame
            } else if y < m + title_h {
                title
            } else {
                body
            };
            data.extend_from_slice(&[r, g, b, 255]);
        }
    }
    PixmapData::new(s as u16, s as u16, data)
}

pub fn resolve_client_icon(
    icons: &[IconData],
    size: u16,
    bg: antibox_core::colour::Colour,
) -> PixmapData {
    match select_icon(icons, size as u32) {
        Some(ic) => icon_to_pixmap(ic, size, size, bg),
        None => default_icon(size as u32, bg),
    }
}

#[cfg(test)]
#[path = "icon_render_tests.rs"]
mod tests;
