use super::*;

#[test]
fn paints_into_the_bound_buffer() {
    let store = BufferStore::new();
    let id = store.create(8, 8);
    let g = WaylandGraphics::new(Arc::clone(&store), id);
    g.set_foreground(0x00FF_0000).unwrap();
    g.fill_rect(0, 0, 8, 8).unwrap();
    g.set_foreground(0x0000_00FF).unwrap();
    g.draw_line(0, 0, 7, 0).unwrap();
    let pm = store.snapshot(id).unwrap();
    assert_eq!(&pm.data[0..4], &[0x00, 0x00, 0xFF, 0xFF]);
    let mid = (4 * 8 + 4) * 4;
    assert_eq!(&pm.data[mid..mid + 4], &[0xFF, 0x00, 0x00, 0xFF]);
}

#[test]
fn drawable_id_is_reported() {
    let store = BufferStore::new();
    let id = store.create(2, 2);
    let g = WaylandGraphics::new(store, id);
    assert_eq!(g.drawable(), id);
}
