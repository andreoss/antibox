use super::*;

#[test]
fn plain_utf8_passes_through() {
    assert_eq!(decode_property("hello".as_bytes()), "hello");
    assert_eq!(decode_property("Привет".as_bytes()), "Привет");
}

#[test]
fn trailing_nul_is_dropped() {
    assert_eq!(decode_property(b"name\0\0"), "name");
}

#[test]
fn latin1_bytes_decode_without_escapes() {
    assert_eq!(decode_property(&[0x68, 0xE9, 0x6C]), "hél");
}

#[test]
fn compound_text_cyrillic_decodes() {
    let data = [
        0x1B, 0x2D, 0x4C, 0xBF, 0xE0, 0xD8, 0xD2, 0xD5, 0xE2, 0x20, 0xDC, 0xD8, 0xE0,
    ];
    assert_eq!(decode_property(&data), "Привет мир");
}

#[test]
fn compound_text_greek_decodes() {
    let data = [0x1B, 0x2D, 0x46, 0xC1, 0xC2];
    assert_eq!(decode_property(&data), "ΑΒ");
}

#[test]
fn compound_text_utf8_segment_decodes() {
    let mut data = vec![0x1B, b'%', b'G'];
    data.extend_from_slice("Привет".as_bytes());
    assert_eq!(decode_property(&data), "Привет");
}

#[test]
fn ascii_designation_is_skipped_not_printed() {
    let data = [0x1B, 0x28, 0x42, b'o', b'k'];
    assert_eq!(decode_property(&data), "ok");
}

#[test]
fn unsupported_multibyte_set_yields_no_garbage() {
    let data = [
        0x1B, 0x24, 0x28, 0x42, 0x27, 0x31, 0x27, 0x62, 0x1B, 0x28, 0x42, b'!',
    ];
    assert_eq!(decode_property(&data), "!");
}

#[test]
fn cyrillic_edges_map_to_their_symbols() {
    assert_eq!(decode_property(&[0x1B, 0x2D, 0x4C, 0xF0]), "№");
    assert_eq!(decode_property(&[0x1B, 0x2D, 0x4C, 0xFD]), "§");
    assert_eq!(decode_property(&[0x1B, 0x2D, 0x4C, 0xF1]), "ё");
    assert_eq!(decode_property(&[0x1B, 0x2D, 0x4C, 0xB0]), "А");
    assert_eq!(decode_property(&[0x1B, 0x2D, 0x4C, 0xFF]), "џ");
}

#[test]
fn utf8_segment_returns_to_iso2022_afterwards() {
    let data = [
        0x1B, 0x2D, 0x4C, 0xBF, 0xE0, 0xD8, 0xD2, 0xD5, 0xE2, 0x20, 0xDC, 0xD8, 0xE0, 0x20,
        0x1B, 0x25, 0x47, 0xE2, 0x80, 0x94, 0x1B, 0x25, 0x40, 0x20, b'C', b'y', b'r', b'i',
        b'l', b'l', b'i', b'c',
    ];
    assert_eq!(decode_property(&data), "Привет мир — Cyrillic");
}
