use crate::backend::PixmapData;
use std::path::Path;

pub fn load_file(path: &Path) -> Option<PixmapData> {
    if path.extension().and_then(|e| e.to_str()) == Some("xpm") {
        return crate::xpm::load_xpm_file(path);
    }
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 || w > u32::from(u16::MAX) || h > u32::from(u16::MAX) {
        return None;
    }
    Some(PixmapData::new(w as u16, h as u16, rgba.into_raw()))
}

pub fn scale_rgba(src: &PixmapData, tw: u16, th: u16) -> PixmapData {
    let (sw, sh) = (u32::from(src.width.max(1)), u32::from(src.height.max(1)));
    let mut data = Vec::with_capacity(tw as usize * th as usize * 4);
    for ty in 0..u32::from(th) {
        let sy = (ty * sh / u32::from(th.max(1))).min(sh - 1);
        for tx in 0..u32::from(tw) {
            let sx = (tx * sw / u32::from(tw.max(1))).min(sw - 1);
            let off = ((sy * sw + sx) * 4) as usize;
            data.extend_from_slice(src.data.get(off..off + 4).unwrap_or(&[0, 0, 0, 0]));
        }
    }
    PixmapData::new(tw, th, data)
}
