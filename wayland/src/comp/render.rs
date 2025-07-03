use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::{
    MemoryRenderBuffer, MemoryRenderBufferRenderElement,
};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::{render_elements, AsRenderElements, Kind};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::{ImportAll, ImportMem};
use smithay::utils::{Logical, Point, Scale, Transform};

use crate::shared::WinKind;

use super::state::Compositor;

render_elements! {
    pub RiceElement<R> where R: ImportAll + ImportMem;
    Surface = WaylandSurfaceRenderElement<R>,
    Memory = MemoryRenderBufferRenderElement<R>,
}

pub(crate) fn output_elements(
    state: &Compositor,
    renderer: &mut GlesRenderer,
    scale: f64,
) -> Vec<RiceElement<GlesRenderer>> {
    let scale = Scale::from(scale);
    let mut rendered: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut elements: Vec<RiceElement<GlesRenderer>> = Vec::new();

    let order: Vec<u32> = {
        let s = state.shared.lock();
        s.stack.iter().rev().copied().collect()
    };

    for id in &order {
        if let Some(window) = state.clients.get(id) {
            let Some(loc) = state.space.element_location(window) else {
                continue;
            };
            let phys = loc.to_physical_precise_round(scale);
            let mut els =
                window.render_elements::<RiceElement<GlesRenderer>>(renderer, phys, scale, 1.0);
            elements.append(&mut els);
            rendered.insert(*id);
        } else {
            let kids: Vec<u32> = {
                let s = state.shared.lock();
                s.windows
                    .iter()
                    .filter(|(_, r)| {
                        r.parent == *id && matches!(r.kind, WinKind::Client) && r.mapped
                    })
                    .map(|(kid, _)| *kid)
                    .collect()
            };
            for kid in kids {
                if let Some(window) = state.clients.get(&kid) {
                    let Some(loc) = state.space.element_location(window) else {
                        continue;
                    };
                    let phys = loc.to_physical_precise_round(scale);
                    let mut els = window
                        .render_elements::<RiceElement<GlesRenderer>>(renderer, phys, scale, 1.0);
                    elements.append(&mut els);
                    rendered.insert(kid);
                }
            }
            if let Some(el) = decoration_element(state, renderer, *id, scale) {
                elements.push(el);
                rendered.insert(*id);
            }
        }
    }

    let mut head: Vec<RiceElement<GlesRenderer>> = Vec::new();
    for (id, window) in &state.clients {
        if rendered.contains(id) {
            continue;
        }
        let Some(loc) = state.space.element_location(window) else {
            continue;
        };
        let phys = loc.to_physical_precise_round(scale);
        let mut els =
            window.render_elements::<RiceElement<GlesRenderer>>(renderer, phys, scale, 1.0);
        head.append(&mut els);
        rendered.insert(*id);
    }
    head.append(&mut elements);

    let mut server: Vec<(u32, usize)> = {
        let s = state.shared.lock();
        s.windows
            .iter()
            .filter(|(id, r)| {
                matches!(r.kind, WinKind::Server) && r.mapped && !rendered.contains(id)
            })
            .map(|(id, _)| (*id, window_depth(&s, *id)))
            .collect()
    };
    server.sort_by_key(|b| std::cmp::Reverse(b.1));
    let mut top: Vec<RiceElement<GlesRenderer>> = Vec::new();
    for (id, _) in server {
        if let Some(el) = decoration_element(state, renderer, id, scale) {
            top.push(el);
        }
    }
    top.append(&mut head);
    top
}

fn window_depth(s: &crate::shared::SharedState, id: u32) -> usize {
    let mut depth = 0;
    let mut cur = id;
    let mut guard = 0;
    while let Some(rec) = s.windows.get(&cur) {
        if rec.parent == crate::shared::ROOT_WINDOW || rec.parent == cur || guard > 32 {
            break;
        }
        cur = rec.parent;
        depth += 1;
        guard += 1;
    }
    depth
}

fn decoration_element(
    state: &Compositor,
    renderer: &mut GlesRenderer,
    id: u32,
    scale: Scale<f64>,
) -> Option<RiceElement<GlesRenderer>> {
    let (ax, ay, mapped, kind) = {
        let s = state.shared.lock();
        let rec = s.windows.get(&id)?;
        let (mut mapped, kind) = (rec.mapped, rec.kind);
        let (mut x, mut y) = (rec.rect.x, rec.rect.y);
        let mut parent = rec.parent;
        let mut guard = 0;
        while parent != crate::shared::ROOT_WINDOW && guard < 32 {
            let Some(pr) = s.windows.get(&parent) else {
                break;
            };
            mapped = mapped && pr.mapped;
            x += pr.rect.x;
            y += pr.rect.y;
            parent = pr.parent;
            guard += 1;
        }
        (x, y, mapped, kind)
    };
    if !mapped || !matches!(kind, WinKind::Server) {
        return None;
    }
    let pd = state.buffers.snapshot(id)?;
    if pd.width == 0 || pd.height == 0 {
        return None;
    }
    let buffer = MemoryRenderBuffer::from_slice(
        &pd.data,
        Fourcc::Abgr8888,
        (pd.width as i32, pd.height as i32),
        1,
        Transform::Normal,
        None,
    );
    let loc = Point::<i32, Logical>::from((ax, ay)).to_physical_precise_round::<f64, f64>(scale);
    MemoryRenderBufferRenderElement::from_buffer(
        renderer,
        loc,
        &buffer,
        None,
        None,
        None,
        Kind::Unspecified,
    )
    .ok()
    .map(RiceElement::Memory)
}
