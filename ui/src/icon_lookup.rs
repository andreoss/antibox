use antibox_core::backend::PixmapData;
use antibox_gfx::sync::atomic::LazyRwLock;
use std::collections::HashMap;
use std::path::PathBuf;

type ThemeEntry = (String, Vec<(u32, String)>);
type ScaledCache = HashMap<(String, u32), Option<PixmapData>>;

struct DirIndex {
    bases: Vec<String>,
    themes: Vec<ThemeEntry>,
}

static NAME_INDEX: LazyRwLock<Option<HashMap<String, PathBuf>>> = LazyRwLock::new();
static SCALED_CACHE: LazyRwLock<Option<ScaledCache>> = LazyRwLock::new();

fn icon_base_dirs() -> Vec<String> {
    let mut bases: Vec<String> = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        bases.push(format!("{home}/.local/share/icons"));
        bases.push(format!("{home}/.icons"));
    }
    if let Ok(dirs) = std::env::var("XDG_DATA_DIRS") {
        for d in dirs.split(':').filter(|s| !s.is_empty()) {
            bases.push(format!("{d}/icons"));
        }
    }
    bases.push("/usr/local/share/icons".into());
    bases.push("/usr/share/icons".into());
    bases.dedup();
    bases
}

fn subdirs(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.path().is_dir() {
                if let Some(n) = e.file_name().to_str() {
                    out.push(n.to_string());
                }
            }
        }
    }
    out
}

fn size_dir_px(name: &str) -> Option<u32> {
    name.split(['x', '@']).next()?.parse::<u32>().ok()
}

fn dir_index() -> DirIndex {
    let bases = icon_base_dirs();
    let mut themes = Vec::new();
    for base in &bases {
        for theme in subdirs(base) {
            let theme_dir = format!("{base}/{theme}");
            let mut sized: Vec<(u32, String)> = Vec::new();
            for d in subdirs(&theme_dir) {
                if let Some(px) = size_dir_px(&d) {
                    sized.push((px, d));
                }
            }
            if !sized.is_empty() {
                themes.push((theme_dir, sized));
            }
        }
    }
    DirIndex { bases, themes }
}

fn build_name_index() -> HashMap<String, PathBuf> {
    let cats = [
        "apps",
        "actions",
        "places",
        "devices",
        "categories",
        "mimetypes",
        "status",
    ];
    let mut best: HashMap<String, (i64, PathBuf)> = HashMap::new();
    let scan = |best: &mut HashMap<String, (i64, PathBuf)>, dir: &str, size_score: i64| {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let fname = e.file_name();
            let Some(fname) = fname.to_str() else { continue };
            let Some(dot) = fname.rfind('.') else { continue };
            let ext_score = match &fname[dot + 1..] {
                "png" => 1,
                "xpm" => 0,
                _ => continue,
            };
            let score = size_score * 4 + ext_score;
            let stem = &fname[..dot];
            if best.get(stem).map_or(true, |(s, _)| score > *s) {
                best.insert(stem.to_string(), (score, e.path()));
            }
        }
    };
    let idx = dir_index();
    for (theme_dir, sized) in &idx.themes {
        for (px, d) in sized {
            for cat in &cats {
                scan(&mut best, &format!("{theme_dir}/{d}/{cat}"), i64::from(*px));
            }
        }
    }
    for base in &idx.bases {
        scan(&mut best, base, 1);
    }
    for dir in ["/usr/share/pixmaps", "/usr/local/share/pixmaps"] {
        scan(&mut best, dir, 1);
    }
    best.into_iter().map(|(k, (_, p))| (k, p)).collect()
}

fn indexed_path(name: &str) -> Option<PathBuf> {
    if let Ok(g) = NAME_INDEX.read() {
        if let Some(idx) = g.as_ref() {
            return idx.get(name).cloned();
        }
    }
    let idx = build_name_index();
    let path = idx.get(name).cloned();
    if let Ok(mut g) = NAME_INDEX.write() {
        *g = Some(idx);
    }
    path
}

pub fn fit_to_size(src: &PixmapData, size: u32) -> Option<PixmapData> {
    if size == 0 || src.width == 0 || src.height == 0 {
        return None;
    }
    let (sw, sh) = (u32::from(src.width), u32::from(src.height));
    if sw == size && sh == size {
        return Some(src.clone());
    }
    let (nw, nh) = if sw >= sh {
        (size, (size * sh / sw).max(1))
    } else {
        ((size * sw / sh).max(1), size)
    };
    let scaled = antibox_gfx::raster::scale_rgba(src, nw as u16, nh as u16);
    if nw == size && nh == size {
        return Some(scaled);
    }
    let mut data = vec![0u8; (size * size * 4) as usize];
    let ox = ((size - nw) / 2) as usize * 4;
    let oy = ((size - nh) / 2) as usize;
    let src_stride = nw as usize * 4;
    let dst_stride = size as usize * 4;
    for row in 0..nh as usize {
        let s0 = row * src_stride;
        let d0 = (row + oy) * dst_stride + ox;
        data[d0..d0 + src_stride].copy_from_slice(&scaled.data[s0..s0 + src_stride]);
    }
    Some(PixmapData::new(size as u16, size as u16, data))
}

pub fn find_icon(name: &str, size: u32) -> Option<PixmapData> {
    if name.is_empty() {
        return None;
    }
    let key = (name.to_string(), size);
    if let Ok(g) = SCALED_CACHE.read() {
        if let Some(hit) = g.as_ref().and_then(|c| c.get(&key)) {
            return hit.clone();
        }
    }
    let path = if name.contains('/') {
        Some(PathBuf::from(name)).filter(|p| p.exists())
    } else {
        indexed_path(name)
    };
    let loaded = path
        .and_then(|p| antibox_gfx::raster::load_file(&p))
        .and_then(|pm| fit_to_size(&pm, size));
    if let Ok(mut g) = SCALED_CACHE.write() {
        g.get_or_insert_with(HashMap::new)
            .insert(key, loaded.clone());
    }
    loaded
}
