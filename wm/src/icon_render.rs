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
            data.push(a as u8);
        }
    }
    let mut pm = PixmapData::new(tw, th, data);
    pm.mask_from_alpha();
    pm
}

pub fn default_icon(size: u32, bg: antibox_core::colour::Colour) -> PixmapData {
    let s = size.max(4).min(u16::MAX as u32) as u16;
    let colours = [
        antibox_ui::theme::shadow(),
        antibox_ui::theme::title_active(),
        antibox_ui::theme::field(),
    ];
    let data = crate::icon_dsl::WINDOW_ICON.rasterize(s, bg, &colours);
    let mut pm = PixmapData::new(s, s, data);
    pm.mask_from_alpha();
    pm
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
