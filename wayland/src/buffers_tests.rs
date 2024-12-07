use super::*;

#[test]
fn ids_are_unique_and_above_reserved() {
    let s = BufferStore::new();
    let a = s.alloc_id();
    let b = s.alloc_id();
    assert!(a >= FIRST_DYNAMIC_ID && b > a);
}

#[test]
fn create_with_and_snapshot_roundtrip() {
    let s = BufferStore::new();
    let id = s.create(4, 4);
    s.with(id, |c| c.fill_rect(0, 0, 4, 4, 0x00AA_BBCC));
    let pm = s.snapshot(id).expect("snapshot");
    assert_eq!(&pm.data[0..4], &[0xAA, 0xBB, 0xCC, 0xFF]);
}

#[test]
fn copy_region_between_buffers() {
    let s = BufferStore::new();
    let src = s.create(2, 2);
    let dst = s.create(2, 2);
    s.with(src, |c| c.fill_rect(0, 0, 2, 2, 0x0000_FF00));
    s.copy_region(src, Rect::px(0, 0, 2, 2), dst, Point::ZERO);
    let pm = s.snapshot(dst).unwrap();
    assert_eq!(&pm.data[0..4], &[0x00, 0xFF, 0x00, 0xFF]);
}

#[test]
fn with_missing_id_is_none() {
    let s = BufferStore::new();
    assert!(s.with(999, |_| ()).is_none());
}
