#![allow(non_upper_case_globals, clippy::manual_range_contains)]

pub const KEY_BackSpace: u32 = 0xFF08;
pub const KEY_Tab: u32 = 0xFF09;
pub const KEY_Return: u32 = 0xFF0D;
pub const KEY_Escape: u32 = 0xFF1B;
pub const KEY_Delete: u32 = 0xFFFF;
pub const KEY_Home: u32 = 0xFF50;
pub const KEY_Left: u32 = 0xFF51;
pub const KEY_Up: u32 = 0xFF52;
pub const KEY_Right: u32 = 0xFF53;
pub const KEY_Down: u32 = 0xFF54;
pub const KEY_Prior: u32 = 0xFF55;
pub const KEY_Next: u32 = 0xFF56;
pub const KEY_End: u32 = 0xFF57;
pub const KEY_Begin: u32 = 0xFF58;
pub const KEY_Insert: u32 = 0xFF63;

pub const KEY_KP_0: u32 = 0xFFB0;
pub const KEY_KP_1: u32 = 0xFFB1;
pub const KEY_KP_2: u32 = 0xFFB2;
pub const KEY_KP_3: u32 = 0xFFB3;
pub const KEY_KP_4: u32 = 0xFFB4;
pub const KEY_KP_5: u32 = 0xFFB5;
pub const KEY_KP_6: u32 = 0xFFB6;
pub const KEY_KP_7: u32 = 0xFFB7;
pub const KEY_KP_8: u32 = 0xFFB8;
pub const KEY_KP_9: u32 = 0xFFB9;
pub const KEY_KP_Decimal: u32 = 0xFFAE;
pub const KEY_KP_Separator: u32 = 0xFFAC;
pub const KEY_KP_Divide: u32 = 0xFFAF;
pub const KEY_KP_Multiply: u32 = 0xFFAA;
pub const KEY_KP_Subtract: u32 = 0xFFAD;
pub const KEY_KP_Add: u32 = 0xFFAB;
pub const KEY_KP_Equal: u32 = 0xFFBD;
pub const KEY_KP_Enter: u32 = 0xFF8D;
pub const KEY_KP_Home: u32 = 0xFF95;
pub const KEY_KP_Insert: u32 = 0xFF9E;
pub const KEY_KP_Delete: u32 = 0xFF9F;
pub const KEY_KP_End: u32 = 0xFF9C;
pub const KEY_KP_Prior: u32 = 0xFF9A;
pub const KEY_KP_Next: u32 = 0xFF9B;
pub const KEY_KP_Left: u32 = 0xFF96;
pub const KEY_KP_Right: u32 = 0xFF98;
pub const KEY_KP_Up: u32 = 0xFF97;
pub const KEY_KP_Down: u32 = 0xFF99;
pub const KEY_KP_Begin: u32 = 0xFF9D;
pub const KEY_KP_F1: u32 = 0xFF91;
pub const KEY_KP_F2: u32 = 0xFF92;
pub const KEY_KP_F3: u32 = 0xFF93;
pub const KEY_KP_F4: u32 = 0xFF94;
pub const KEY_KP_Tab: u32 = 0xFF89;
pub const KEY_KP_Space: u32 = 0xFF80;

pub const KEY_0: u32 = 0x0030;
pub const KEY_1: u32 = 0x0031;
pub const KEY_2: u32 = 0x0032;
pub const KEY_3: u32 = 0x0033;
pub const KEY_4: u32 = 0x0034;
pub const KEY_5: u32 = 0x0035;
pub const KEY_6: u32 = 0x0036;
pub const KEY_7: u32 = 0x0037;
pub const KEY_8: u32 = 0x0038;
pub const KEY_9: u32 = 0x0039;
pub const KEY_period: u32 = 0x002E;
pub const KEY_comma: u32 = 0x002C;
pub const KEY_slash: u32 = 0x002F;
pub const KEY_asterisk: u32 = 0x002A;
pub const KEY_minus: u32 = 0x002D;
pub const KEY_plus: u32 = 0x002B;
pub const KEY_equal: u32 = 0x003D;
pub const KEY_space: u32 = 0x0020;

pub const KEY_F1: u32 = 0xFFBE;
pub const KEY_F2: u32 = 0xFFBF;
pub const KEY_F3: u32 = 0xFFC0;
pub const KEY_F4: u32 = 0xFFC1;
pub const KEY_F5: u32 = 0xFFC2;
pub const KEY_F6: u32 = 0xFFC3;
pub const KEY_F7: u32 = 0xFFC4;
pub const KEY_F8: u32 = 0xFFC5;
pub const KEY_F9: u32 = 0xFFC6;
pub const KEY_F10: u32 = 0xFFC7;
pub const KEY_F11: u32 = 0xFFC8;
pub const KEY_F12: u32 = 0xFFC9;

struct UcsKeysym {
    ucs: u16,
    keysym: u16,
}

static UCS_KEYSYMS: &[UcsKeysym] = &[
    UcsKeysym {
        ucs: 0x0100,
        keysym: 0x03c0,
    },
    UcsKeysym {
        ucs: 0x0101,
        keysym: 0x03e0,
    },
    UcsKeysym {
        ucs: 0x0102,
        keysym: 0x01c3,
    },
    UcsKeysym {
        ucs: 0x0103,
        keysym: 0x01e3,
    },
    UcsKeysym {
        ucs: 0x0104,
        keysym: 0x01a1,
    },
    UcsKeysym {
        ucs: 0x0105,
        keysym: 0x01b1,
    },
    UcsKeysym {
        ucs: 0x0106,
        keysym: 0x01c6,
    },
    UcsKeysym {
        ucs: 0x0107,
        keysym: 0x01e6,
    },
    UcsKeysym {
        ucs: 0x0108,
        keysym: 0x02c6,
    },
    UcsKeysym {
        ucs: 0x0109,
        keysym: 0x02e6,
    },
    UcsKeysym {
        ucs: 0x010a,
        keysym: 0x02c5,
    },
    UcsKeysym {
        ucs: 0x010b,
        keysym: 0x02e5,
    },
    UcsKeysym {
        ucs: 0x010c,
        keysym: 0x01c8,
    },
    UcsKeysym {
        ucs: 0x010d,
        keysym: 0x01e8,
    },
    UcsKeysym {
        ucs: 0x010e,
        keysym: 0x01cf,
    },
    UcsKeysym {
        ucs: 0x010f,
        keysym: 0x01ef,
    },
    UcsKeysym {
        ucs: 0x0110,
        keysym: 0x01d0,
    },
    UcsKeysym {
        ucs: 0x0111,
        keysym: 0x01f0,
    },
    UcsKeysym {
        ucs: 0x0112,
        keysym: 0x03aa,
    },
    UcsKeysym {
        ucs: 0x0113,
        keysym: 0x03ba,
    },
    UcsKeysym {
        ucs: 0x0116,
        keysym: 0x03cc,
    },
    UcsKeysym {
        ucs: 0x0117,
        keysym: 0x03ec,
    },
    UcsKeysym {
        ucs: 0x0118,
        keysym: 0x01ca,
    },
    UcsKeysym {
        ucs: 0x0119,
        keysym: 0x01ea,
    },
    UcsKeysym {
        ucs: 0x011a,
        keysym: 0x01cc,
    },
    UcsKeysym {
        ucs: 0x011b,
        keysym: 0x01ec,
    },
    UcsKeysym {
        ucs: 0x011c,
        keysym: 0x02d8,
    },
    UcsKeysym {
        ucs: 0x011d,
        keysym: 0x02f8,
    },
    UcsKeysym {
        ucs: 0x011e,
        keysym: 0x02ab,
    },
    UcsKeysym {
        ucs: 0x011f,
        keysym: 0x02bb,
    },
    UcsKeysym {
        ucs: 0x0120,
        keysym: 0x02d5,
    },
    UcsKeysym {
        ucs: 0x0121,
        keysym: 0x02f5,
    },
    UcsKeysym {
        ucs: 0x0122,
        keysym: 0x03ab,
    },
    UcsKeysym {
        ucs: 0x0123,
        keysym: 0x03bb,
    },
    UcsKeysym {
        ucs: 0x0124,
        keysym: 0x02a6,
    },
    UcsKeysym {
        ucs: 0x0125,
        keysym: 0x02b6,
    },
    UcsKeysym {
        ucs: 0x0126,
        keysym: 0x02a1,
    },
    UcsKeysym {
        ucs: 0x0127,
        keysym: 0x02b1,
    },
    UcsKeysym {
        ucs: 0x0128,
        keysym: 0x03a5,
    },
    UcsKeysym {
        ucs: 0x0129,
        keysym: 0x03b5,
    },
    UcsKeysym {
        ucs: 0x012a,
        keysym: 0x03cf,
    },
    UcsKeysym {
        ucs: 0x012b,
        keysym: 0x03ef,
    },
    UcsKeysym {
        ucs: 0x012e,
        keysym: 0x03c7,
    },
    UcsKeysym {
        ucs: 0x012f,
        keysym: 0x03e7,
    },
    UcsKeysym {
        ucs: 0x0130,
        keysym: 0x02a9,
    },
    UcsKeysym {
        ucs: 0x0131,
        keysym: 0x02b9,
    },
    UcsKeysym {
        ucs: 0x0134,
        keysym: 0x02ac,
    },
    UcsKeysym {
        ucs: 0x0135,
        keysym: 0x02bc,
    },
    UcsKeysym {
        ucs: 0x0136,
        keysym: 0x03d3,
    },
    UcsKeysym {
        ucs: 0x0137,
        keysym: 0x03f3,
    },
    UcsKeysym {
        ucs: 0x0138,
        keysym: 0x03a2,
    },
    UcsKeysym {
        ucs: 0x0139,
        keysym: 0x01c5,
    },
    UcsKeysym {
        ucs: 0x013a,
        keysym: 0x01e5,
    },
    UcsKeysym {
        ucs: 0x013b,
        keysym: 0x03a6,
    },
    UcsKeysym {
        ucs: 0x013c,
        keysym: 0x03b6,
    },
    UcsKeysym {
        ucs: 0x013d,
        keysym: 0x01a5,
    },
    UcsKeysym {
        ucs: 0x013e,
        keysym: 0x01b5,
    },
    UcsKeysym {
        ucs: 0x0141,
        keysym: 0x01a3,
    },
    UcsKeysym {
        ucs: 0x0142,
        keysym: 0x01b3,
    },
    UcsKeysym {
        ucs: 0x0143,
        keysym: 0x01d1,
    },
    UcsKeysym {
        ucs: 0x0144,
        keysym: 0x01f1,
    },
    UcsKeysym {
        ucs: 0x0145,
        keysym: 0x03d1,
    },
    UcsKeysym {
        ucs: 0x0146,
        keysym: 0x03f1,
    },
    UcsKeysym {
        ucs: 0x0147,
        keysym: 0x01d2,
    },
    UcsKeysym {
        ucs: 0x0148,
        keysym: 0x01f2,
    },
    UcsKeysym {
        ucs: 0x014a,
        keysym: 0x03bd,
    },
    UcsKeysym {
        ucs: 0x014b,
        keysym: 0x03bf,
    },
    UcsKeysym {
        ucs: 0x014c,
        keysym: 0x03d2,
    },
    UcsKeysym {
        ucs: 0x014d,
        keysym: 0x03f2,
    },
    UcsKeysym {
        ucs: 0x0150,
        keysym: 0x01d5,
    },
    UcsKeysym {
        ucs: 0x0151,
        keysym: 0x01f5,
    },
    UcsKeysym {
        ucs: 0x0152,
        keysym: 0x13bc,
    },
    UcsKeysym {
        ucs: 0x0153,
        keysym: 0x13bd,
    },
    UcsKeysym {
        ucs: 0x0154,
        keysym: 0x01c0,
    },
    UcsKeysym {
        ucs: 0x0155,
        keysym: 0x01e0,
    },
    UcsKeysym {
        ucs: 0x0156,
        keysym: 0x03a3,
    },
    UcsKeysym {
        ucs: 0x0157,
        keysym: 0x03b3,
    },
    UcsKeysym {
        ucs: 0x0158,
        keysym: 0x01d8,
    },
    UcsKeysym {
        ucs: 0x0159,
        keysym: 0x01f8,
    },
    UcsKeysym {
        ucs: 0x015a,
        keysym: 0x01a6,
    },
    UcsKeysym {
        ucs: 0x015b,
        keysym: 0x01b6,
    },
    UcsKeysym {
        ucs: 0x015c,
        keysym: 0x02de,
    },
    UcsKeysym {
        ucs: 0x015d,
        keysym: 0x02fe,
    },
    UcsKeysym {
        ucs: 0x015e,
        keysym: 0x01aa,
    },
    UcsKeysym {
        ucs: 0x015f,
        keysym: 0x01ba,
    },
    UcsKeysym {
        ucs: 0x0160,
        keysym: 0x01a9,
    },
    UcsKeysym {
        ucs: 0x0161,
        keysym: 0x01b9,
    },
    UcsKeysym {
        ucs: 0x0162,
        keysym: 0x01de,
    },
    UcsKeysym {
        ucs: 0x0163,
        keysym: 0x01fe,
    },
    UcsKeysym {
        ucs: 0x0164,
        keysym: 0x01ab,
    },
    UcsKeysym {
        ucs: 0x0165,
        keysym: 0x01bb,
    },
    UcsKeysym {
        ucs: 0x0166,
        keysym: 0x03ac,
    },
    UcsKeysym {
        ucs: 0x0167,
        keysym: 0x03bc,
    },
    UcsKeysym {
        ucs: 0x0168,
        keysym: 0x03dd,
    },
    UcsKeysym {
        ucs: 0x0169,
        keysym: 0x03fd,
    },
    UcsKeysym {
        ucs: 0x016a,
        keysym: 0x03de,
    },
    UcsKeysym {
        ucs: 0x016b,
        keysym: 0x03fe,
    },
    UcsKeysym {
        ucs: 0x016c,
        keysym: 0x02dd,
    },
    UcsKeysym {
        ucs: 0x016d,
        keysym: 0x02fd,
    },
    UcsKeysym {
        ucs: 0x016e,
        keysym: 0x01d9,
    },
    UcsKeysym {
        ucs: 0x016f,
        keysym: 0x01f9,
    },
    UcsKeysym {
        ucs: 0x0170,
        keysym: 0x01db,
    },
    UcsKeysym {
        ucs: 0x0171,
        keysym: 0x01fb,
    },
    UcsKeysym {
        ucs: 0x0172,
        keysym: 0x03d9,
    },
    UcsKeysym {
        ucs: 0x0173,
        keysym: 0x03f9,
    },
    UcsKeysym {
        ucs: 0x0178,
        keysym: 0x13be,
    },
    UcsKeysym {
        ucs: 0x0179,
        keysym: 0x01ac,
    },
    UcsKeysym {
        ucs: 0x017a,
        keysym: 0x01bc,
    },
    UcsKeysym {
        ucs: 0x017b,
        keysym: 0x01af,
    },
    UcsKeysym {
        ucs: 0x017c,
        keysym: 0x01bf,
    },
    UcsKeysym {
        ucs: 0x017d,
        keysym: 0x01ae,
    },
    UcsKeysym {
        ucs: 0x017e,
        keysym: 0x01be,
    },
    UcsKeysym {
        ucs: 0x0192,
        keysym: 0x08f6,
    },
    UcsKeysym {
        ucs: 0x02c7,
        keysym: 0x01b7,
    },
    UcsKeysym {
        ucs: 0x02d8,
        keysym: 0x01a2,
    },
    UcsKeysym {
        ucs: 0x02d9,
        keysym: 0x01ff,
    },
    UcsKeysym {
        ucs: 0x02db,
        keysym: 0x01b2,
    },
    UcsKeysym {
        ucs: 0x02dd,
        keysym: 0x01bd,
    },
    UcsKeysym {
        ucs: 0x0385,
        keysym: 0x07ae,
    },
    UcsKeysym {
        ucs: 0x0386,
        keysym: 0x07a1,
    },
    UcsKeysym {
        ucs: 0x0388,
        keysym: 0x07a2,
    },
    UcsKeysym {
        ucs: 0x0389,
        keysym: 0x07a3,
    },
    UcsKeysym {
        ucs: 0x038a,
        keysym: 0x07a4,
    },
    UcsKeysym {
        ucs: 0x038c,
        keysym: 0x07a7,
    },
    UcsKeysym {
        ucs: 0x038e,
        keysym: 0x07a8,
    },
    UcsKeysym {
        ucs: 0x038f,
        keysym: 0x07ab,
    },
    UcsKeysym {
        ucs: 0x0390,
        keysym: 0x07b6,
    },
    UcsKeysym {
        ucs: 0x0391,
        keysym: 0x07c1,
    },
    UcsKeysym {
        ucs: 0x0392,
        keysym: 0x07c2,
    },
    UcsKeysym {
        ucs: 0x0393,
        keysym: 0x07c3,
    },
    UcsKeysym {
        ucs: 0x0394,
        keysym: 0x07c4,
    },
    UcsKeysym {
        ucs: 0x0395,
        keysym: 0x07c5,
    },
    UcsKeysym {
        ucs: 0x0396,
        keysym: 0x07c6,
    },
    UcsKeysym {
        ucs: 0x0397,
        keysym: 0x07c7,
    },
    UcsKeysym {
        ucs: 0x0398,
        keysym: 0x07c8,
    },
    UcsKeysym {
        ucs: 0x0399,
        keysym: 0x07c9,
    },
    UcsKeysym {
        ucs: 0x039a,
        keysym: 0x07ca,
    },
    UcsKeysym {
        ucs: 0x039b,
        keysym: 0x07cb,
    },
    UcsKeysym {
        ucs: 0x039c,
        keysym: 0x07cc,
    },
    UcsKeysym {
        ucs: 0x039d,
        keysym: 0x07cd,
    },
    UcsKeysym {
        ucs: 0x039e,
        keysym: 0x07ce,
    },
    UcsKeysym {
        ucs: 0x039f,
        keysym: 0x07cf,
    },
    UcsKeysym {
        ucs: 0x03a0,
        keysym: 0x07d0,
    },
    UcsKeysym {
        ucs: 0x03a1,
        keysym: 0x07d1,
    },
    UcsKeysym {
        ucs: 0x03a3,
        keysym: 0x07d2,
    },
    UcsKeysym {
        ucs: 0x03a4,
        keysym: 0x07d4,
    },
    UcsKeysym {
        ucs: 0x03a5,
        keysym: 0x07d5,
    },
    UcsKeysym {
        ucs: 0x03a6,
        keysym: 0x07d6,
    },
    UcsKeysym {
        ucs: 0x03a7,
        keysym: 0x07d7,
    },
    UcsKeysym {
        ucs: 0x03a8,
        keysym: 0x07d8,
    },
    UcsKeysym {
        ucs: 0x03a9,
        keysym: 0x07d9,
    },
    UcsKeysym {
        ucs: 0x03aa,
        keysym: 0x07a5,
    },
    UcsKeysym {
        ucs: 0x03ab,
        keysym: 0x07a9,
    },
    UcsKeysym {
        ucs: 0x03ac,
        keysym: 0x07b1,
    },
    UcsKeysym {
        ucs: 0x03ad,
        keysym: 0x07b2,
    },
    UcsKeysym {
        ucs: 0x03ae,
        keysym: 0x07b3,
    },
    UcsKeysym {
        ucs: 0x03af,
        keysym: 0x07b4,
    },
    UcsKeysym {
        ucs: 0x03b0,
        keysym: 0x07ba,
    },
    UcsKeysym {
        ucs: 0x03b1,
        keysym: 0x07e1,
    },
    UcsKeysym {
        ucs: 0x03b2,
        keysym: 0x07e2,
    },
    UcsKeysym {
        ucs: 0x03b3,
        keysym: 0x07e3,
    },
    UcsKeysym {
        ucs: 0x03b4,
        keysym: 0x07e4,
    },
    UcsKeysym {
        ucs: 0x03b5,
        keysym: 0x07e5,
    },
    UcsKeysym {
        ucs: 0x03b6,
        keysym: 0x07e6,
    },
    UcsKeysym {
        ucs: 0x03b7,
        keysym: 0x07e7,
    },
    UcsKeysym {
        ucs: 0x03b8,
        keysym: 0x07e8,
    },
    UcsKeysym {
        ucs: 0x03b9,
        keysym: 0x07e9,
    },
    UcsKeysym {
        ucs: 0x03ba,
        keysym: 0x07ea,
    },
    UcsKeysym {
        ucs: 0x03bb,
        keysym: 0x07eb,
    },
    UcsKeysym {
        ucs: 0x03bc,
        keysym: 0x07ec,
    },
    UcsKeysym {
        ucs: 0x03bd,
        keysym: 0x07ed,
    },
    UcsKeysym {
        ucs: 0x03be,
        keysym: 0x07ee,
    },
    UcsKeysym {
        ucs: 0x03bf,
        keysym: 0x07ef,
    },
    UcsKeysym {
        ucs: 0x03c0,
        keysym: 0x07f0,
    },
    UcsKeysym {
        ucs: 0x03c1,
        keysym: 0x07f1,
    },
    UcsKeysym {
        ucs: 0x03c2,
        keysym: 0x07f3,
    },
    UcsKeysym {
        ucs: 0x03c3,
        keysym: 0x07f2,
    },
    UcsKeysym {
        ucs: 0x03c4,
        keysym: 0x07f4,
    },
    UcsKeysym {
        ucs: 0x03c5,
        keysym: 0x07f5,
    },
    UcsKeysym {
        ucs: 0x03c6,
        keysym: 0x07f6,
    },
    UcsKeysym {
        ucs: 0x03c7,
        keysym: 0x07f7,
    },
    UcsKeysym {
        ucs: 0x03c8,
        keysym: 0x07f8,
    },
    UcsKeysym {
        ucs: 0x03c9,
        keysym: 0x07f9,
    },
    UcsKeysym {
        ucs: 0x03ca,
        keysym: 0x07b5,
    },
    UcsKeysym {
        ucs: 0x03cb,
        keysym: 0x07b9,
    },
    UcsKeysym {
        ucs: 0x03cc,
        keysym: 0x07b7,
    },
    UcsKeysym {
        ucs: 0x03cd,
        keysym: 0x07b8,
    },
    UcsKeysym {
        ucs: 0x03ce,
        keysym: 0x07bb,
    },
    UcsKeysym {
        ucs: 0x0401,
        keysym: 0x06b3,
    },
    UcsKeysym {
        ucs: 0x0402,
        keysym: 0x06b1,
    },
    UcsKeysym {
        ucs: 0x0403,
        keysym: 0x06b2,
    },
    UcsKeysym {
        ucs: 0x0404,
        keysym: 0x06b4,
    },
    UcsKeysym {
        ucs: 0x0405,
        keysym: 0x06b5,
    },
    UcsKeysym {
        ucs: 0x0406,
        keysym: 0x06b6,
    },
    UcsKeysym {
        ucs: 0x0407,
        keysym: 0x06b7,
    },
    UcsKeysym {
        ucs: 0x0408,
        keysym: 0x06b8,
    },
    UcsKeysym {
        ucs: 0x0409,
        keysym: 0x06b9,
    },
    UcsKeysym {
        ucs: 0x040a,
        keysym: 0x06ba,
    },
    UcsKeysym {
        ucs: 0x040b,
        keysym: 0x06bb,
    },
    UcsKeysym {
        ucs: 0x040c,
        keysym: 0x06bc,
    },
    UcsKeysym {
        ucs: 0x040e,
        keysym: 0x06be,
    },
    UcsKeysym {
        ucs: 0x040f,
        keysym: 0x06bf,
    },
    UcsKeysym {
        ucs: 0x0410,
        keysym: 0x06e1,
    },
    UcsKeysym {
        ucs: 0x0411,
        keysym: 0x06e2,
    },
    UcsKeysym {
        ucs: 0x0412,
        keysym: 0x06f7,
    },
    UcsKeysym {
        ucs: 0x0413,
        keysym: 0x06e7,
    },
    UcsKeysym {
        ucs: 0x0414,
        keysym: 0x06e4,
    },
    UcsKeysym {
        ucs: 0x0415,
        keysym: 0x06e5,
    },
    UcsKeysym {
        ucs: 0x0416,
        keysym: 0x06f6,
    },
    UcsKeysym {
        ucs: 0x0417,
        keysym: 0x06fa,
    },
    UcsKeysym {
        ucs: 0x0418,
        keysym: 0x06e9,
    },
    UcsKeysym {
        ucs: 0x0419,
        keysym: 0x06ea,
    },
    UcsKeysym {
        ucs: 0x041a,
        keysym: 0x06eb,
    },
    UcsKeysym {
        ucs: 0x041b,
        keysym: 0x06ec,
    },
    UcsKeysym {
        ucs: 0x041c,
        keysym: 0x06ed,
    },
    UcsKeysym {
        ucs: 0x041d,
        keysym: 0x06ee,
    },
    UcsKeysym {
        ucs: 0x041e,
        keysym: 0x06ef,
    },
    UcsKeysym {
        ucs: 0x041f,
        keysym: 0x06f0,
    },
    UcsKeysym {
        ucs: 0x0420,
        keysym: 0x06f2,
    },
    UcsKeysym {
        ucs: 0x0421,
        keysym: 0x06f3,
    },
    UcsKeysym {
        ucs: 0x0422,
        keysym: 0x06f4,
    },
    UcsKeysym {
        ucs: 0x0423,
        keysym: 0x06f5,
    },
    UcsKeysym {
        ucs: 0x0424,
        keysym: 0x06e6,
    },
    UcsKeysym {
        ucs: 0x0425,
        keysym: 0x06e8,
    },
    UcsKeysym {
        ucs: 0x0426,
        keysym: 0x06e3,
    },
    UcsKeysym {
        ucs: 0x0427,
        keysym: 0x06fe,
    },
    UcsKeysym {
        ucs: 0x0428,
        keysym: 0x06fb,
    },
    UcsKeysym {
        ucs: 0x0429,
        keysym: 0x06fd,
    },
    UcsKeysym {
        ucs: 0x042a,
        keysym: 0x06ff,
    },
    UcsKeysym {
        ucs: 0x042b,
        keysym: 0x06f9,
    },
    UcsKeysym {
        ucs: 0x042c,
        keysym: 0x06f8,
    },
    UcsKeysym {
        ucs: 0x042d,
        keysym: 0x06fc,
    },
    UcsKeysym {
        ucs: 0x042e,
        keysym: 0x06e0,
    },
    UcsKeysym {
        ucs: 0x042f,
        keysym: 0x06f1,
    },
    UcsKeysym {
        ucs: 0x0430,
        keysym: 0x06c1,
    },
    UcsKeysym {
        ucs: 0x0431,
        keysym: 0x06c2,
    },
    UcsKeysym {
        ucs: 0x0432,
        keysym: 0x06d7,
    },
    UcsKeysym {
        ucs: 0x0433,
        keysym: 0x06c7,
    },
    UcsKeysym {
        ucs: 0x0434,
        keysym: 0x06c4,
    },
    UcsKeysym {
        ucs: 0x0435,
        keysym: 0x06c5,
    },
    UcsKeysym {
        ucs: 0x0436,
        keysym: 0x06d6,
    },
    UcsKeysym {
        ucs: 0x0437,
        keysym: 0x06da,
    },
    UcsKeysym {
        ucs: 0x0438,
        keysym: 0x06c9,
    },
    UcsKeysym {
        ucs: 0x0439,
        keysym: 0x06ca,
    },
    UcsKeysym {
        ucs: 0x043a,
        keysym: 0x06cb,
    },
    UcsKeysym {
        ucs: 0x043b,
        keysym: 0x06cc,
    },
    UcsKeysym {
        ucs: 0x043c,
        keysym: 0x06cd,
    },
    UcsKeysym {
        ucs: 0x043d,
        keysym: 0x06ce,
    },
    UcsKeysym {
        ucs: 0x043e,
        keysym: 0x06cf,
    },
    UcsKeysym {
        ucs: 0x043f,
        keysym: 0x06d0,
    },
    UcsKeysym {
        ucs: 0x0440,
        keysym: 0x06d2,
    },
    UcsKeysym {
        ucs: 0x0441,
        keysym: 0x06d3,
    },
    UcsKeysym {
        ucs: 0x0442,
        keysym: 0x06d4,
    },
    UcsKeysym {
        ucs: 0x0443,
        keysym: 0x06d5,
    },
    UcsKeysym {
        ucs: 0x0444,
        keysym: 0x06c6,
    },
    UcsKeysym {
        ucs: 0x0445,
        keysym: 0x06c8,
    },
    UcsKeysym {
        ucs: 0x0446,
        keysym: 0x06c3,
    },
    UcsKeysym {
        ucs: 0x0447,
        keysym: 0x06de,
    },
    UcsKeysym {
        ucs: 0x0448,
        keysym: 0x06db,
    },
    UcsKeysym {
        ucs: 0x0449,
        keysym: 0x06dd,
    },
    UcsKeysym {
        ucs: 0x044a,
        keysym: 0x06df,
    },
    UcsKeysym {
        ucs: 0x044b,
        keysym: 0x06d9,
    },
    UcsKeysym {
        ucs: 0x044c,
        keysym: 0x06d8,
    },
    UcsKeysym {
        ucs: 0x044d,
        keysym: 0x06dc,
    },
    UcsKeysym {
        ucs: 0x044e,
        keysym: 0x06c0,
    },
    UcsKeysym {
        ucs: 0x044f,
        keysym: 0x06d1,
    },
    UcsKeysym {
        ucs: 0x0451,
        keysym: 0x06a3,
    },
    UcsKeysym {
        ucs: 0x0452,
        keysym: 0x06a1,
    },
    UcsKeysym {
        ucs: 0x0453,
        keysym: 0x06a2,
    },
    UcsKeysym {
        ucs: 0x0454,
        keysym: 0x06a4,
    },
    UcsKeysym {
        ucs: 0x0455,
        keysym: 0x06a5,
    },
    UcsKeysym {
        ucs: 0x0456,
        keysym: 0x06a6,
    },
    UcsKeysym {
        ucs: 0x0457,
        keysym: 0x06a7,
    },
    UcsKeysym {
        ucs: 0x0458,
        keysym: 0x06a8,
    },
    UcsKeysym {
        ucs: 0x0459,
        keysym: 0x06a9,
    },
    UcsKeysym {
        ucs: 0x045a,
        keysym: 0x06aa,
    },
    UcsKeysym {
        ucs: 0x045b,
        keysym: 0x06ab,
    },
    UcsKeysym {
        ucs: 0x045c,
        keysym: 0x06ac,
    },
    UcsKeysym {
        ucs: 0x045e,
        keysym: 0x06ae,
    },
    UcsKeysym {
        ucs: 0x045f,
        keysym: 0x06af,
    },
    UcsKeysym {
        ucs: 0x0490,
        keysym: 0x06bd,
    },
    UcsKeysym {
        ucs: 0x0491,
        keysym: 0x06ad,
    },
    UcsKeysym {
        ucs: 0x05d0,
        keysym: 0x0ce0,
    },
    UcsKeysym {
        ucs: 0x05d1,
        keysym: 0x0ce1,
    },
    UcsKeysym {
        ucs: 0x05d2,
        keysym: 0x0ce2,
    },
    UcsKeysym {
        ucs: 0x05d3,
        keysym: 0x0ce3,
    },
    UcsKeysym {
        ucs: 0x05d4,
        keysym: 0x0ce4,
    },
    UcsKeysym {
        ucs: 0x05d5,
        keysym: 0x0ce5,
    },
    UcsKeysym {
        ucs: 0x05d6,
        keysym: 0x0ce6,
    },
    UcsKeysym {
        ucs: 0x05d7,
        keysym: 0x0ce7,
    },
    UcsKeysym {
        ucs: 0x05d8,
        keysym: 0x0ce8,
    },
    UcsKeysym {
        ucs: 0x05d9,
        keysym: 0x0ce9,
    },
    UcsKeysym {
        ucs: 0x05da,
        keysym: 0x0cea,
    },
    UcsKeysym {
        ucs: 0x05db,
        keysym: 0x0ceb,
    },
    UcsKeysym {
        ucs: 0x05dc,
        keysym: 0x0cec,
    },
    UcsKeysym {
        ucs: 0x05dd,
        keysym: 0x0ced,
    },
    UcsKeysym {
        ucs: 0x05de,
        keysym: 0x0cee,
    },
    UcsKeysym {
        ucs: 0x05df,
        keysym: 0x0cef,
    },
    UcsKeysym {
        ucs: 0x05e0,
        keysym: 0x0cf0,
    },
    UcsKeysym {
        ucs: 0x05e1,
        keysym: 0x0cf1,
    },
    UcsKeysym {
        ucs: 0x05e2,
        keysym: 0x0cf2,
    },
    UcsKeysym {
        ucs: 0x05e3,
        keysym: 0x0cf3,
    },
    UcsKeysym {
        ucs: 0x05e4,
        keysym: 0x0cf4,
    },
    UcsKeysym {
        ucs: 0x05e5,
        keysym: 0x0cf5,
    },
    UcsKeysym {
        ucs: 0x05e6,
        keysym: 0x0cf6,
    },
    UcsKeysym {
        ucs: 0x05e7,
        keysym: 0x0cf7,
    },
    UcsKeysym {
        ucs: 0x05e8,
        keysym: 0x0cf8,
    },
    UcsKeysym {
        ucs: 0x05e9,
        keysym: 0x0cf9,
    },
    UcsKeysym {
        ucs: 0x05ea,
        keysym: 0x0cfa,
    },
    UcsKeysym {
        ucs: 0x060c,
        keysym: 0x05ac,
    },
    UcsKeysym {
        ucs: 0x061b,
        keysym: 0x05bb,
    },
    UcsKeysym {
        ucs: 0x061f,
        keysym: 0x05bf,
    },
    UcsKeysym {
        ucs: 0x0621,
        keysym: 0x05c1,
    },
    UcsKeysym {
        ucs: 0x0622,
        keysym: 0x05c2,
    },
    UcsKeysym {
        ucs: 0x0623,
        keysym: 0x05c3,
    },
    UcsKeysym {
        ucs: 0x0624,
        keysym: 0x05c4,
    },
    UcsKeysym {
        ucs: 0x0625,
        keysym: 0x05c5,
    },
    UcsKeysym {
        ucs: 0x0626,
        keysym: 0x05c6,
    },
    UcsKeysym {
        ucs: 0x0627,
        keysym: 0x05c7,
    },
    UcsKeysym {
        ucs: 0x0628,
        keysym: 0x05c8,
    },
    UcsKeysym {
        ucs: 0x0629,
        keysym: 0x05c9,
    },
    UcsKeysym {
        ucs: 0x062a,
        keysym: 0x05ca,
    },
    UcsKeysym {
        ucs: 0x062b,
        keysym: 0x05cb,
    },
    UcsKeysym {
        ucs: 0x062c,
        keysym: 0x05cc,
    },
    UcsKeysym {
        ucs: 0x062d,
        keysym: 0x05cd,
    },
    UcsKeysym {
        ucs: 0x062e,
        keysym: 0x05ce,
    },
    UcsKeysym {
        ucs: 0x062f,
        keysym: 0x05cf,
    },
    UcsKeysym {
        ucs: 0x0630,
        keysym: 0x05d0,
    },
    UcsKeysym {
        ucs: 0x0631,
        keysym: 0x05d1,
    },
    UcsKeysym {
        ucs: 0x0632,
        keysym: 0x05d2,
    },
    UcsKeysym {
        ucs: 0x0633,
        keysym: 0x05d3,
    },
    UcsKeysym {
        ucs: 0x0634,
        keysym: 0x05d4,
    },
    UcsKeysym {
        ucs: 0x0635,
        keysym: 0x05d5,
    },
    UcsKeysym {
        ucs: 0x0636,
        keysym: 0x05d6,
    },
    UcsKeysym {
        ucs: 0x0637,
        keysym: 0x05d7,
    },
    UcsKeysym {
        ucs: 0x0638,
        keysym: 0x05d8,
    },
    UcsKeysym {
        ucs: 0x0639,
        keysym: 0x05d9,
    },
    UcsKeysym {
        ucs: 0x063a,
        keysym: 0x05da,
    },
    UcsKeysym {
        ucs: 0x0640,
        keysym: 0x05e0,
    },
    UcsKeysym {
        ucs: 0x0641,
        keysym: 0x05e1,
    },
    UcsKeysym {
        ucs: 0x0642,
        keysym: 0x05e2,
    },
    UcsKeysym {
        ucs: 0x0643,
        keysym: 0x05e3,
    },
    UcsKeysym {
        ucs: 0x0644,
        keysym: 0x05e4,
    },
    UcsKeysym {
        ucs: 0x0645,
        keysym: 0x05e5,
    },
    UcsKeysym {
        ucs: 0x0646,
        keysym: 0x05e6,
    },
    UcsKeysym {
        ucs: 0x0647,
        keysym: 0x05e7,
    },
    UcsKeysym {
        ucs: 0x0648,
        keysym: 0x05e8,
    },
    UcsKeysym {
        ucs: 0x0649,
        keysym: 0x05e9,
    },
    UcsKeysym {
        ucs: 0x064a,
        keysym: 0x05ea,
    },
    UcsKeysym {
        ucs: 0x064b,
        keysym: 0x05eb,
    },
    UcsKeysym {
        ucs: 0x064c,
        keysym: 0x05ec,
    },
    UcsKeysym {
        ucs: 0x064d,
        keysym: 0x05ed,
    },
    UcsKeysym {
        ucs: 0x064e,
        keysym: 0x05ee,
    },
    UcsKeysym {
        ucs: 0x064f,
        keysym: 0x05ef,
    },
    UcsKeysym {
        ucs: 0x0650,
        keysym: 0x05f0,
    },
    UcsKeysym {
        ucs: 0x0651,
        keysym: 0x05f1,
    },
    UcsKeysym {
        ucs: 0x0652,
        keysym: 0x05f2,
    },
    UcsKeysym {
        ucs: 0x0e01,
        keysym: 0x0da1,
    },
    UcsKeysym {
        ucs: 0x0e02,
        keysym: 0x0da2,
    },
    UcsKeysym {
        ucs: 0x0e03,
        keysym: 0x0da3,
    },
    UcsKeysym {
        ucs: 0x0e04,
        keysym: 0x0da4,
    },
    UcsKeysym {
        ucs: 0x0e05,
        keysym: 0x0da5,
    },
    UcsKeysym {
        ucs: 0x0e06,
        keysym: 0x0da6,
    },
    UcsKeysym {
        ucs: 0x0e07,
        keysym: 0x0da7,
    },
    UcsKeysym {
        ucs: 0x0e08,
        keysym: 0x0da8,
    },
    UcsKeysym {
        ucs: 0x0e09,
        keysym: 0x0da9,
    },
    UcsKeysym {
        ucs: 0x0e0a,
        keysym: 0x0daa,
    },
    UcsKeysym {
        ucs: 0x0e0b,
        keysym: 0x0dab,
    },
    UcsKeysym {
        ucs: 0x0e0c,
        keysym: 0x0dac,
    },
    UcsKeysym {
        ucs: 0x0e0d,
        keysym: 0x0dad,
    },
    UcsKeysym {
        ucs: 0x0e0e,
        keysym: 0x0dae,
    },
    UcsKeysym {
        ucs: 0x0e0f,
        keysym: 0x0daf,
    },
    UcsKeysym {
        ucs: 0x0e10,
        keysym: 0x0db0,
    },
    UcsKeysym {
        ucs: 0x0e11,
        keysym: 0x0db1,
    },
    UcsKeysym {
        ucs: 0x0e12,
        keysym: 0x0db2,
    },
    UcsKeysym {
        ucs: 0x0e13,
        keysym: 0x0db3,
    },
    UcsKeysym {
        ucs: 0x0e14,
        keysym: 0x0db4,
    },
    UcsKeysym {
        ucs: 0x0e15,
        keysym: 0x0db5,
    },
    UcsKeysym {
        ucs: 0x0e16,
        keysym: 0x0db6,
    },
    UcsKeysym {
        ucs: 0x0e17,
        keysym: 0x0db7,
    },
    UcsKeysym {
        ucs: 0x0e18,
        keysym: 0x0db8,
    },
    UcsKeysym {
        ucs: 0x0e19,
        keysym: 0x0db9,
    },
    UcsKeysym {
        ucs: 0x0e1a,
        keysym: 0x0dba,
    },
    UcsKeysym {
        ucs: 0x0e1b,
        keysym: 0x0dbb,
    },
    UcsKeysym {
        ucs: 0x0e1c,
        keysym: 0x0dbc,
    },
    UcsKeysym {
        ucs: 0x0e1d,
        keysym: 0x0dbd,
    },
    UcsKeysym {
        ucs: 0x0e1e,
        keysym: 0x0dbe,
    },
    UcsKeysym {
        ucs: 0x0e1f,
        keysym: 0x0dbf,
    },
    UcsKeysym {
        ucs: 0x0e20,
        keysym: 0x0dc0,
    },
    UcsKeysym {
        ucs: 0x0e21,
        keysym: 0x0dc1,
    },
    UcsKeysym {
        ucs: 0x0e22,
        keysym: 0x0dc2,
    },
    UcsKeysym {
        ucs: 0x0e23,
        keysym: 0x0dc3,
    },
    UcsKeysym {
        ucs: 0x0e24,
        keysym: 0x0dc4,
    },
    UcsKeysym {
        ucs: 0x0e25,
        keysym: 0x0dc5,
    },
    UcsKeysym {
        ucs: 0x0e26,
        keysym: 0x0dc6,
    },
    UcsKeysym {
        ucs: 0x0e27,
        keysym: 0x0dc7,
    },
    UcsKeysym {
        ucs: 0x0e28,
        keysym: 0x0dc8,
    },
    UcsKeysym {
        ucs: 0x0e29,
        keysym: 0x0dc9,
    },
    UcsKeysym {
        ucs: 0x0e2a,
        keysym: 0x0dca,
    },
    UcsKeysym {
        ucs: 0x0e2b,
        keysym: 0x0dcb,
    },
    UcsKeysym {
        ucs: 0x0e2c,
        keysym: 0x0dcc,
    },
    UcsKeysym {
        ucs: 0x0e2d,
        keysym: 0x0dcd,
    },
    UcsKeysym {
        ucs: 0x0e2e,
        keysym: 0x0dce,
    },
    UcsKeysym {
        ucs: 0x0e2f,
        keysym: 0x0dcf,
    },
    UcsKeysym {
        ucs: 0x0e30,
        keysym: 0x0dd0,
    },
    UcsKeysym {
        ucs: 0x0e31,
        keysym: 0x0dd1,
    },
    UcsKeysym {
        ucs: 0x0e32,
        keysym: 0x0dd2,
    },
    UcsKeysym {
        ucs: 0x0e33,
        keysym: 0x0dd3,
    },
    UcsKeysym {
        ucs: 0x0e34,
        keysym: 0x0dd4,
    },
    UcsKeysym {
        ucs: 0x0e35,
        keysym: 0x0dd5,
    },
    UcsKeysym {
        ucs: 0x0e36,
        keysym: 0x0dd6,
    },
    UcsKeysym {
        ucs: 0x0e37,
        keysym: 0x0dd7,
    },
    UcsKeysym {
        ucs: 0x0e38,
        keysym: 0x0dd8,
    },
    UcsKeysym {
        ucs: 0x0e39,
        keysym: 0x0dd9,
    },
    UcsKeysym {
        ucs: 0x0e3a,
        keysym: 0x0dda,
    },
    UcsKeysym {
        ucs: 0x0e3e,
        keysym: 0x0dde,
    },
    UcsKeysym {
        ucs: 0x0e3f,
        keysym: 0x0ddf,
    },
    UcsKeysym {
        ucs: 0x0e40,
        keysym: 0x0de0,
    },
    UcsKeysym {
        ucs: 0x0e41,
        keysym: 0x0de1,
    },
    UcsKeysym {
        ucs: 0x0e42,
        keysym: 0x0de2,
    },
    UcsKeysym {
        ucs: 0x0e43,
        keysym: 0x0de3,
    },
    UcsKeysym {
        ucs: 0x0e44,
        keysym: 0x0de4,
    },
    UcsKeysym {
        ucs: 0x0e45,
        keysym: 0x0de5,
    },
    UcsKeysym {
        ucs: 0x0e46,
        keysym: 0x0de6,
    },
    UcsKeysym {
        ucs: 0x0e47,
        keysym: 0x0de7,
    },
    UcsKeysym {
        ucs: 0x0e48,
        keysym: 0x0de8,
    },
    UcsKeysym {
        ucs: 0x0e49,
        keysym: 0x0de9,
    },
    UcsKeysym {
        ucs: 0x0e4a,
        keysym: 0x0dea,
    },
    UcsKeysym {
        ucs: 0x0e4b,
        keysym: 0x0deb,
    },
    UcsKeysym {
        ucs: 0x0e4c,
        keysym: 0x0dec,
    },
    UcsKeysym {
        ucs: 0x0e4d,
        keysym: 0x0ded,
    },
    UcsKeysym {
        ucs: 0x0e50,
        keysym: 0x0df0,
    },
    UcsKeysym {
        ucs: 0x0e51,
        keysym: 0x0df1,
    },
    UcsKeysym {
        ucs: 0x0e52,
        keysym: 0x0df2,
    },
    UcsKeysym {
        ucs: 0x0e53,
        keysym: 0x0df3,
    },
    UcsKeysym {
        ucs: 0x0e54,
        keysym: 0x0df4,
    },
    UcsKeysym {
        ucs: 0x0e55,
        keysym: 0x0df5,
    },
    UcsKeysym {
        ucs: 0x0e56,
        keysym: 0x0df6,
    },
    UcsKeysym {
        ucs: 0x0e57,
        keysym: 0x0df7,
    },
    UcsKeysym {
        ucs: 0x0e58,
        keysym: 0x0df8,
    },
    UcsKeysym {
        ucs: 0x0e59,
        keysym: 0x0df9,
    },
    UcsKeysym {
        ucs: 0x11a8,
        keysym: 0x0ed4,
    },
    UcsKeysym {
        ucs: 0x11a9,
        keysym: 0x0ed5,
    },
    UcsKeysym {
        ucs: 0x11aa,
        keysym: 0x0ed6,
    },
    UcsKeysym {
        ucs: 0x11ab,
        keysym: 0x0ed7,
    },
    UcsKeysym {
        ucs: 0x11ac,
        keysym: 0x0ed8,
    },
    UcsKeysym {
        ucs: 0x11ad,
        keysym: 0x0ed9,
    },
    UcsKeysym {
        ucs: 0x11ae,
        keysym: 0x0eda,
    },
    UcsKeysym {
        ucs: 0x11af,
        keysym: 0x0edb,
    },
    UcsKeysym {
        ucs: 0x11b0,
        keysym: 0x0edc,
    },
    UcsKeysym {
        ucs: 0x11b1,
        keysym: 0x0edd,
    },
    UcsKeysym {
        ucs: 0x11b2,
        keysym: 0x0ede,
    },
    UcsKeysym {
        ucs: 0x11b3,
        keysym: 0x0edf,
    },
    UcsKeysym {
        ucs: 0x11b4,
        keysym: 0x0ee0,
    },
    UcsKeysym {
        ucs: 0x11b5,
        keysym: 0x0ee1,
    },
    UcsKeysym {
        ucs: 0x11b6,
        keysym: 0x0ee2,
    },
    UcsKeysym {
        ucs: 0x11b7,
        keysym: 0x0ee3,
    },
    UcsKeysym {
        ucs: 0x11b8,
        keysym: 0x0ee4,
    },
    UcsKeysym {
        ucs: 0x11b9,
        keysym: 0x0ee5,
    },
    UcsKeysym {
        ucs: 0x11ba,
        keysym: 0x0ee6,
    },
    UcsKeysym {
        ucs: 0x11bb,
        keysym: 0x0ee7,
    },
    UcsKeysym {
        ucs: 0x11bc,
        keysym: 0x0ee8,
    },
    UcsKeysym {
        ucs: 0x11bd,
        keysym: 0x0ee9,
    },
    UcsKeysym {
        ucs: 0x11be,
        keysym: 0x0eea,
    },
    UcsKeysym {
        ucs: 0x11bf,
        keysym: 0x0eeb,
    },
    UcsKeysym {
        ucs: 0x11c0,
        keysym: 0x0eec,
    },
    UcsKeysym {
        ucs: 0x11c1,
        keysym: 0x0eed,
    },
    UcsKeysym {
        ucs: 0x11c2,
        keysym: 0x0eee,
    },
    UcsKeysym {
        ucs: 0x11eb,
        keysym: 0x0ef8,
    },
    UcsKeysym {
        ucs: 0x11f9,
        keysym: 0x0efa,
    },
    UcsKeysym {
        ucs: 0x2002,
        keysym: 0x0aa2,
    },
    UcsKeysym {
        ucs: 0x2003,
        keysym: 0x0aa1,
    },
    UcsKeysym {
        ucs: 0x2004,
        keysym: 0x0aa3,
    },
    UcsKeysym {
        ucs: 0x2005,
        keysym: 0x0aa4,
    },
    UcsKeysym {
        ucs: 0x2007,
        keysym: 0x0aa5,
    },
    UcsKeysym {
        ucs: 0x2008,
        keysym: 0x0aa6,
    },
    UcsKeysym {
        ucs: 0x2009,
        keysym: 0x0aa7,
    },
    UcsKeysym {
        ucs: 0x200a,
        keysym: 0x0aa8,
    },
    UcsKeysym {
        ucs: 0x2012,
        keysym: 0x0abb,
    },
    UcsKeysym {
        ucs: 0x2013,
        keysym: 0x0aaa,
    },
    UcsKeysym {
        ucs: 0x2014,
        keysym: 0x0aa9,
    },
    UcsKeysym {
        ucs: 0x2015,
        keysym: 0x07af,
    },
    UcsKeysym {
        ucs: 0x2017,
        keysym: 0x0cdf,
    },
    UcsKeysym {
        ucs: 0x2018,
        keysym: 0x0ad0,
    },
    UcsKeysym {
        ucs: 0x2019,
        keysym: 0x0ad1,
    },
    UcsKeysym {
        ucs: 0x201a,
        keysym: 0x0afd,
    },
    UcsKeysym {
        ucs: 0x201c,
        keysym: 0x0ad2,
    },
    UcsKeysym {
        ucs: 0x201d,
        keysym: 0x0ad3,
    },
    UcsKeysym {
        ucs: 0x201e,
        keysym: 0x0afe,
    },
    UcsKeysym {
        ucs: 0x2020,
        keysym: 0x0af1,
    },
    UcsKeysym {
        ucs: 0x2021,
        keysym: 0x0af2,
    },
    UcsKeysym {
        ucs: 0x2022,
        keysym: 0x0ae6,
    },
    UcsKeysym {
        ucs: 0x2025,
        keysym: 0x0aaf,
    },
    UcsKeysym {
        ucs: 0x2026,
        keysym: 0x0aae,
    },
    UcsKeysym {
        ucs: 0x2030,
        keysym: 0x0ad5,
    },
    UcsKeysym {
        ucs: 0x2032,
        keysym: 0x0ad6,
    },
    UcsKeysym {
        ucs: 0x2033,
        keysym: 0x0ad7,
    },
    UcsKeysym {
        ucs: 0x2038,
        keysym: 0x0afc,
    },
    UcsKeysym {
        ucs: 0x203e,
        keysym: 0x047e,
    },
    UcsKeysym {
        ucs: 0x20a0,
        keysym: 0x20a0,
    },
    UcsKeysym {
        ucs: 0x20a1,
        keysym: 0x20a1,
    },
    UcsKeysym {
        ucs: 0x20a2,
        keysym: 0x20a2,
    },
    UcsKeysym {
        ucs: 0x20a3,
        keysym: 0x20a3,
    },
    UcsKeysym {
        ucs: 0x20a4,
        keysym: 0x20a4,
    },
    UcsKeysym {
        ucs: 0x20a5,
        keysym: 0x20a5,
    },
    UcsKeysym {
        ucs: 0x20a6,
        keysym: 0x20a6,
    },
    UcsKeysym {
        ucs: 0x20a7,
        keysym: 0x20a7,
    },
    UcsKeysym {
        ucs: 0x20a8,
        keysym: 0x20a8,
    },
    UcsKeysym {
        ucs: 0x20a9,
        keysym: 0x0eff,
    },
    UcsKeysym {
        ucs: 0x20aa,
        keysym: 0x20aa,
    },
    UcsKeysym {
        ucs: 0x20ab,
        keysym: 0x20ab,
    },
    UcsKeysym {
        ucs: 0x20ac,
        keysym: 0x20ac,
    },
    UcsKeysym {
        ucs: 0x2105,
        keysym: 0x0ab8,
    },
    UcsKeysym {
        ucs: 0x2116,
        keysym: 0x06b0,
    },
    UcsKeysym {
        ucs: 0x2117,
        keysym: 0x0afb,
    },
    UcsKeysym {
        ucs: 0x211e,
        keysym: 0x0ad4,
    },
    UcsKeysym {
        ucs: 0x2122,
        keysym: 0x0ac9,
    },
    UcsKeysym {
        ucs: 0x2153,
        keysym: 0x0ab0,
    },
    UcsKeysym {
        ucs: 0x2154,
        keysym: 0x0ab1,
    },
    UcsKeysym {
        ucs: 0x2155,
        keysym: 0x0ab2,
    },
    UcsKeysym {
        ucs: 0x2156,
        keysym: 0x0ab3,
    },
    UcsKeysym {
        ucs: 0x2157,
        keysym: 0x0ab4,
    },
    UcsKeysym {
        ucs: 0x2158,
        keysym: 0x0ab5,
    },
    UcsKeysym {
        ucs: 0x2159,
        keysym: 0x0ab6,
    },
    UcsKeysym {
        ucs: 0x215a,
        keysym: 0x0ab7,
    },
    UcsKeysym {
        ucs: 0x215b,
        keysym: 0x0ac3,
    },
    UcsKeysym {
        ucs: 0x215c,
        keysym: 0x0ac4,
    },
    UcsKeysym {
        ucs: 0x215d,
        keysym: 0x0ac5,
    },
    UcsKeysym {
        ucs: 0x215e,
        keysym: 0x0ac6,
    },
    UcsKeysym {
        ucs: 0x2190,
        keysym: 0x08fb,
    },
    UcsKeysym {
        ucs: 0x2191,
        keysym: 0x08fc,
    },
    UcsKeysym {
        ucs: 0x2192,
        keysym: 0x08fd,
    },
    UcsKeysym {
        ucs: 0x2193,
        keysym: 0x08fe,
    },
    UcsKeysym {
        ucs: 0x21d2,
        keysym: 0x08ce,
    },
    UcsKeysym {
        ucs: 0x21d4,
        keysym: 0x08cd,
    },
    UcsKeysym {
        ucs: 0x2202,
        keysym: 0x08ef,
    },
    UcsKeysym {
        ucs: 0x2207,
        keysym: 0x08c5,
    },
    UcsKeysym {
        ucs: 0x2218,
        keysym: 0x0bca,
    },
    UcsKeysym {
        ucs: 0x221a,
        keysym: 0x08d6,
    },
    UcsKeysym {
        ucs: 0x221d,
        keysym: 0x08c1,
    },
    UcsKeysym {
        ucs: 0x221e,
        keysym: 0x08c2,
    },
    UcsKeysym {
        ucs: 0x2227,
        keysym: 0x08de,
    },
    UcsKeysym {
        ucs: 0x2228,
        keysym: 0x08df,
    },
    UcsKeysym {
        ucs: 0x2229,
        keysym: 0x08dc,
    },
    UcsKeysym {
        ucs: 0x222a,
        keysym: 0x08dd,
    },
    UcsKeysym {
        ucs: 0x222b,
        keysym: 0x08bf,
    },
    UcsKeysym {
        ucs: 0x2234,
        keysym: 0x08c0,
    },
    UcsKeysym {
        ucs: 0x223c,
        keysym: 0x08c8,
    },
    UcsKeysym {
        ucs: 0x2243,
        keysym: 0x08c9,
    },
    UcsKeysym {
        ucs: 0x2245,
        keysym: 0x08c8,
    },
    UcsKeysym {
        ucs: 0x2260,
        keysym: 0x08bd,
    },
    UcsKeysym {
        ucs: 0x2261,
        keysym: 0x08cf,
    },
    UcsKeysym {
        ucs: 0x2264,
        keysym: 0x08bc,
    },
    UcsKeysym {
        ucs: 0x2265,
        keysym: 0x08be,
    },
    UcsKeysym {
        ucs: 0x2282,
        keysym: 0x08da,
    },
    UcsKeysym {
        ucs: 0x2283,
        keysym: 0x08db,
    },
    UcsKeysym {
        ucs: 0x22a2,
        keysym: 0x0bfc,
    },
    UcsKeysym {
        ucs: 0x22a3,
        keysym: 0x0bdc,
    },
    UcsKeysym {
        ucs: 0x22a4,
        keysym: 0x0bc2,
    },
    UcsKeysym {
        ucs: 0x22a5,
        keysym: 0x0bce,
    },
    UcsKeysym {
        ucs: 0x2308,
        keysym: 0x0bd3,
    },
    UcsKeysym {
        ucs: 0x230a,
        keysym: 0x0bc4,
    },
    UcsKeysym {
        ucs: 0x2315,
        keysym: 0x0afa,
    },
    UcsKeysym {
        ucs: 0x2320,
        keysym: 0x08a4,
    },
    UcsKeysym {
        ucs: 0x2321,
        keysym: 0x08a5,
    },
    UcsKeysym {
        ucs: 0x2329,
        keysym: 0x0abc,
    },
    UcsKeysym {
        ucs: 0x232a,
        keysym: 0x0abe,
    },
    UcsKeysym {
        ucs: 0x2395,
        keysym: 0x0bcc,
    },
    UcsKeysym {
        ucs: 0x239b,
        keysym: 0x08ab,
    },
    UcsKeysym {
        ucs: 0x239d,
        keysym: 0x08ac,
    },
    UcsKeysym {
        ucs: 0x239e,
        keysym: 0x08ad,
    },
    UcsKeysym {
        ucs: 0x23a0,
        keysym: 0x08ae,
    },
    UcsKeysym {
        ucs: 0x23a1,
        keysym: 0x08a7,
    },
    UcsKeysym {
        ucs: 0x23a3,
        keysym: 0x08a8,
    },
    UcsKeysym {
        ucs: 0x23a4,
        keysym: 0x08a9,
    },
    UcsKeysym {
        ucs: 0x23a6,
        keysym: 0x08aa,
    },
    UcsKeysym {
        ucs: 0x23a8,
        keysym: 0x08af,
    },
    UcsKeysym {
        ucs: 0x23ac,
        keysym: 0x08b0,
    },
    UcsKeysym {
        ucs: 0x23b7,
        keysym: 0x08a1,
    },
    UcsKeysym {
        ucs: 0x23ba,
        keysym: 0x09ef,
    },
    UcsKeysym {
        ucs: 0x23bb,
        keysym: 0x09f0,
    },
    UcsKeysym {
        ucs: 0x23bc,
        keysym: 0x09f2,
    },
    UcsKeysym {
        ucs: 0x23bd,
        keysym: 0x09f3,
    },
    UcsKeysym {
        ucs: 0x2409,
        keysym: 0x09e2,
    },
    UcsKeysym {
        ucs: 0x240a,
        keysym: 0x09e5,
    },
    UcsKeysym {
        ucs: 0x240b,
        keysym: 0x09e9,
    },
    UcsKeysym {
        ucs: 0x240c,
        keysym: 0x09e3,
    },
    UcsKeysym {
        ucs: 0x240d,
        keysym: 0x09e4,
    },
    UcsKeysym {
        ucs: 0x2422,
        keysym: 0x09df,
    },
    UcsKeysym {
        ucs: 0x2424,
        keysym: 0x09e8,
    },
    UcsKeysym {
        ucs: 0x2500,
        keysym: 0x09f1,
    },
    UcsKeysym {
        ucs: 0x2502,
        keysym: 0x08a6,
    },
    UcsKeysym {
        ucs: 0x250c,
        keysym: 0x09ec,
    },
    UcsKeysym {
        ucs: 0x2510,
        keysym: 0x09eb,
    },
    UcsKeysym {
        ucs: 0x2514,
        keysym: 0x09ed,
    },
    UcsKeysym {
        ucs: 0x2518,
        keysym: 0x09ea,
    },
    UcsKeysym {
        ucs: 0x251c,
        keysym: 0x09f4,
    },
    UcsKeysym {
        ucs: 0x2524,
        keysym: 0x09f5,
    },
    UcsKeysym {
        ucs: 0x252c,
        keysym: 0x09f7,
    },
    UcsKeysym {
        ucs: 0x2534,
        keysym: 0x09f6,
    },
    UcsKeysym {
        ucs: 0x253c,
        keysym: 0x09ee,
    },
    UcsKeysym {
        ucs: 0x2592,
        keysym: 0x09e1,
    },
    UcsKeysym {
        ucs: 0x25a0,
        keysym: 0x0adf,
    },
    UcsKeysym {
        ucs: 0x25a1,
        keysym: 0x0acf,
    },
    UcsKeysym {
        ucs: 0x25aa,
        keysym: 0x0ae7,
    },
    UcsKeysym {
        ucs: 0x25ab,
        keysym: 0x0ae1,
    },
    UcsKeysym {
        ucs: 0x25ac,
        keysym: 0x0adb,
    },
    UcsKeysym {
        ucs: 0x25ad,
        keysym: 0x0ae2,
    },
    UcsKeysym {
        ucs: 0x25b2,
        keysym: 0x0ae8,
    },
    UcsKeysym {
        ucs: 0x25b3,
        keysym: 0x0ae3,
    },
    UcsKeysym {
        ucs: 0x25b6,
        keysym: 0x0add,
    },
    UcsKeysym {
        ucs: 0x25b7,
        keysym: 0x0acd,
    },
    UcsKeysym {
        ucs: 0x25bc,
        keysym: 0x0ae9,
    },
    UcsKeysym {
        ucs: 0x25bd,
        keysym: 0x0ae4,
    },
    UcsKeysym {
        ucs: 0x25c0,
        keysym: 0x0adc,
    },
    UcsKeysym {
        ucs: 0x25c1,
        keysym: 0x0acc,
    },
    UcsKeysym {
        ucs: 0x25c6,
        keysym: 0x09e0,
    },
    UcsKeysym {
        ucs: 0x25cb,
        keysym: 0x0ace,
    },
    UcsKeysym {
        ucs: 0x25cf,
        keysym: 0x0ade,
    },
    UcsKeysym {
        ucs: 0x25e6,
        keysym: 0x0ae0,
    },
    UcsKeysym {
        ucs: 0x2606,
        keysym: 0x0ae5,
    },
    UcsKeysym {
        ucs: 0x260e,
        keysym: 0x0af9,
    },
    UcsKeysym {
        ucs: 0x2613,
        keysym: 0x0aca,
    },
    UcsKeysym {
        ucs: 0x261c,
        keysym: 0x0aea,
    },
    UcsKeysym {
        ucs: 0x261e,
        keysym: 0x0aeb,
    },
    UcsKeysym {
        ucs: 0x2640,
        keysym: 0x0af8,
    },
    UcsKeysym {
        ucs: 0x2642,
        keysym: 0x0af7,
    },
    UcsKeysym {
        ucs: 0x2663,
        keysym: 0x0aec,
    },
    UcsKeysym {
        ucs: 0x2665,
        keysym: 0x0aee,
    },
    UcsKeysym {
        ucs: 0x2666,
        keysym: 0x0aed,
    },
    UcsKeysym {
        ucs: 0x266d,
        keysym: 0x0af6,
    },
    UcsKeysym {
        ucs: 0x266f,
        keysym: 0x0af5,
    },
    UcsKeysym {
        ucs: 0x2713,
        keysym: 0x0af3,
    },
    UcsKeysym {
        ucs: 0x2717,
        keysym: 0x0af4,
    },
    UcsKeysym {
        ucs: 0x271d,
        keysym: 0x0ad9,
    },
    UcsKeysym {
        ucs: 0x2720,
        keysym: 0x0af0,
    },
    UcsKeysym {
        ucs: 0x3001,
        keysym: 0x04a4,
    },
    UcsKeysym {
        ucs: 0x3002,
        keysym: 0x04a1,
    },
    UcsKeysym {
        ucs: 0x300c,
        keysym: 0x04a2,
    },
    UcsKeysym {
        ucs: 0x300d,
        keysym: 0x04a3,
    },
    UcsKeysym {
        ucs: 0x309b,
        keysym: 0x04de,
    },
    UcsKeysym {
        ucs: 0x309c,
        keysym: 0x04df,
    },
    UcsKeysym {
        ucs: 0x30a1,
        keysym: 0x04a7,
    },
    UcsKeysym {
        ucs: 0x30a2,
        keysym: 0x04b1,
    },
    UcsKeysym {
        ucs: 0x30a3,
        keysym: 0x04a8,
    },
    UcsKeysym {
        ucs: 0x30a4,
        keysym: 0x04b2,
    },
    UcsKeysym {
        ucs: 0x30a5,
        keysym: 0x04a9,
    },
    UcsKeysym {
        ucs: 0x30a6,
        keysym: 0x04b3,
    },
    UcsKeysym {
        ucs: 0x30a7,
        keysym: 0x04aa,
    },
    UcsKeysym {
        ucs: 0x30a8,
        keysym: 0x04b4,
    },
    UcsKeysym {
        ucs: 0x30a9,
        keysym: 0x04ab,
    },
    UcsKeysym {
        ucs: 0x30aa,
        keysym: 0x04b5,
    },
    UcsKeysym {
        ucs: 0x30ab,
        keysym: 0x04b6,
    },
    UcsKeysym {
        ucs: 0x30ad,
        keysym: 0x04b7,
    },
    UcsKeysym {
        ucs: 0x30af,
        keysym: 0x04b8,
    },
    UcsKeysym {
        ucs: 0x30b1,
        keysym: 0x04b9,
    },
    UcsKeysym {
        ucs: 0x30b3,
        keysym: 0x04ba,
    },
    UcsKeysym {
        ucs: 0x30b5,
        keysym: 0x04bb,
    },
    UcsKeysym {
        ucs: 0x30b7,
        keysym: 0x04bc,
    },
    UcsKeysym {
        ucs: 0x30b9,
        keysym: 0x04bd,
    },
    UcsKeysym {
        ucs: 0x30bb,
        keysym: 0x04be,
    },
    UcsKeysym {
        ucs: 0x30bd,
        keysym: 0x04bf,
    },
    UcsKeysym {
        ucs: 0x30bf,
        keysym: 0x04c0,
    },
    UcsKeysym {
        ucs: 0x30c1,
        keysym: 0x04c1,
    },
    UcsKeysym {
        ucs: 0x30c3,
        keysym: 0x04af,
    },
    UcsKeysym {
        ucs: 0x30c4,
        keysym: 0x04c2,
    },
    UcsKeysym {
        ucs: 0x30c6,
        keysym: 0x04c3,
    },
    UcsKeysym {
        ucs: 0x30c8,
        keysym: 0x04c4,
    },
    UcsKeysym {
        ucs: 0x30ca,
        keysym: 0x04c5,
    },
    UcsKeysym {
        ucs: 0x30cb,
        keysym: 0x04c6,
    },
    UcsKeysym {
        ucs: 0x30cc,
        keysym: 0x04c7,
    },
    UcsKeysym {
        ucs: 0x30cd,
        keysym: 0x04c8,
    },
    UcsKeysym {
        ucs: 0x30ce,
        keysym: 0x04c9,
    },
    UcsKeysym {
        ucs: 0x30cf,
        keysym: 0x04ca,
    },
    UcsKeysym {
        ucs: 0x30d2,
        keysym: 0x04cb,
    },
    UcsKeysym {
        ucs: 0x30d5,
        keysym: 0x04cc,
    },
    UcsKeysym {
        ucs: 0x30d8,
        keysym: 0x04cd,
    },
    UcsKeysym {
        ucs: 0x30db,
        keysym: 0x04ce,
    },
    UcsKeysym {
        ucs: 0x30de,
        keysym: 0x04cf,
    },
    UcsKeysym {
        ucs: 0x30df,
        keysym: 0x04d0,
    },
    UcsKeysym {
        ucs: 0x30e0,
        keysym: 0x04d1,
    },
    UcsKeysym {
        ucs: 0x30e1,
        keysym: 0x04d2,
    },
    UcsKeysym {
        ucs: 0x30e2,
        keysym: 0x04d3,
    },
    UcsKeysym {
        ucs: 0x30e3,
        keysym: 0x04ac,
    },
    UcsKeysym {
        ucs: 0x30e4,
        keysym: 0x04d4,
    },
    UcsKeysym {
        ucs: 0x30e5,
        keysym: 0x04ad,
    },
    UcsKeysym {
        ucs: 0x30e6,
        keysym: 0x04d5,
    },
    UcsKeysym {
        ucs: 0x30e7,
        keysym: 0x04ae,
    },
    UcsKeysym {
        ucs: 0x30e8,
        keysym: 0x04d6,
    },
    UcsKeysym {
        ucs: 0x30e9,
        keysym: 0x04d7,
    },
    UcsKeysym {
        ucs: 0x30ea,
        keysym: 0x04d8,
    },
    UcsKeysym {
        ucs: 0x30eb,
        keysym: 0x04d9,
    },
    UcsKeysym {
        ucs: 0x30ec,
        keysym: 0x04da,
    },
    UcsKeysym {
        ucs: 0x30ed,
        keysym: 0x04db,
    },
    UcsKeysym {
        ucs: 0x30ef,
        keysym: 0x04dc,
    },
    UcsKeysym {
        ucs: 0x30f2,
        keysym: 0x04a6,
    },
    UcsKeysym {
        ucs: 0x30f3,
        keysym: 0x04dd,
    },
    UcsKeysym {
        ucs: 0x30fb,
        keysym: 0x04a5,
    },
    UcsKeysym {
        ucs: 0x30fc,
        keysym: 0x04b0,
    },
    UcsKeysym {
        ucs: 0x3131,
        keysym: 0x0ea1,
    },
    UcsKeysym {
        ucs: 0x3132,
        keysym: 0x0ea2,
    },
    UcsKeysym {
        ucs: 0x3133,
        keysym: 0x0ea3,
    },
    UcsKeysym {
        ucs: 0x3134,
        keysym: 0x0ea4,
    },
    UcsKeysym {
        ucs: 0x3135,
        keysym: 0x0ea5,
    },
    UcsKeysym {
        ucs: 0x3136,
        keysym: 0x0ea6,
    },
    UcsKeysym {
        ucs: 0x3137,
        keysym: 0x0ea7,
    },
    UcsKeysym {
        ucs: 0x3138,
        keysym: 0x0ea8,
    },
    UcsKeysym {
        ucs: 0x3139,
        keysym: 0x0ea9,
    },
    UcsKeysym {
        ucs: 0x313a,
        keysym: 0x0eaa,
    },
    UcsKeysym {
        ucs: 0x313b,
        keysym: 0x0eab,
    },
    UcsKeysym {
        ucs: 0x313c,
        keysym: 0x0eac,
    },
    UcsKeysym {
        ucs: 0x313d,
        keysym: 0x0ead,
    },
    UcsKeysym {
        ucs: 0x313e,
        keysym: 0x0eae,
    },
    UcsKeysym {
        ucs: 0x313f,
        keysym: 0x0eaf,
    },
    UcsKeysym {
        ucs: 0x3140,
        keysym: 0x0eb0,
    },
    UcsKeysym {
        ucs: 0x3141,
        keysym: 0x0eb1,
    },
    UcsKeysym {
        ucs: 0x3142,
        keysym: 0x0eb2,
    },
    UcsKeysym {
        ucs: 0x3143,
        keysym: 0x0eb3,
    },
    UcsKeysym {
        ucs: 0x3144,
        keysym: 0x0eb4,
    },
    UcsKeysym {
        ucs: 0x3145,
        keysym: 0x0eb5,
    },
    UcsKeysym {
        ucs: 0x3146,
        keysym: 0x0eb6,
    },
    UcsKeysym {
        ucs: 0x3147,
        keysym: 0x0eb7,
    },
    UcsKeysym {
        ucs: 0x3148,
        keysym: 0x0eb8,
    },
    UcsKeysym {
        ucs: 0x3149,
        keysym: 0x0eb9,
    },
    UcsKeysym {
        ucs: 0x314a,
        keysym: 0x0eba,
    },
    UcsKeysym {
        ucs: 0x314b,
        keysym: 0x0ebb,
    },
    UcsKeysym {
        ucs: 0x314c,
        keysym: 0x0ebc,
    },
    UcsKeysym {
        ucs: 0x314d,
        keysym: 0x0ebd,
    },
    UcsKeysym {
        ucs: 0x314e,
        keysym: 0x0ebe,
    },
    UcsKeysym {
        ucs: 0x314f,
        keysym: 0x0ebf,
    },
    UcsKeysym {
        ucs: 0x3150,
        keysym: 0x0ec0,
    },
    UcsKeysym {
        ucs: 0x3151,
        keysym: 0x0ec1,
    },
    UcsKeysym {
        ucs: 0x3152,
        keysym: 0x0ec2,
    },
    UcsKeysym {
        ucs: 0x3153,
        keysym: 0x0ec3,
    },
    UcsKeysym {
        ucs: 0x3154,
        keysym: 0x0ec4,
    },
    UcsKeysym {
        ucs: 0x3155,
        keysym: 0x0ec5,
    },
    UcsKeysym {
        ucs: 0x3156,
        keysym: 0x0ec6,
    },
    UcsKeysym {
        ucs: 0x3157,
        keysym: 0x0ec7,
    },
    UcsKeysym {
        ucs: 0x3158,
        keysym: 0x0ec8,
    },
    UcsKeysym {
        ucs: 0x3159,
        keysym: 0x0ec9,
    },
    UcsKeysym {
        ucs: 0x315a,
        keysym: 0x0eca,
    },
    UcsKeysym {
        ucs: 0x315b,
        keysym: 0x0ecb,
    },
    UcsKeysym {
        ucs: 0x315c,
        keysym: 0x0ecc,
    },
    UcsKeysym {
        ucs: 0x315d,
        keysym: 0x0ecd,
    },
    UcsKeysym {
        ucs: 0x315e,
        keysym: 0x0ece,
    },
    UcsKeysym {
        ucs: 0x315f,
        keysym: 0x0ecf,
    },
    UcsKeysym {
        ucs: 0x3160,
        keysym: 0x0ed0,
    },
    UcsKeysym {
        ucs: 0x3161,
        keysym: 0x0ed1,
    },
    UcsKeysym {
        ucs: 0x3162,
        keysym: 0x0ed2,
    },
    UcsKeysym {
        ucs: 0x3163,
        keysym: 0x0ed3,
    },
    UcsKeysym {
        ucs: 0x316d,
        keysym: 0x0eef,
    },
    UcsKeysym {
        ucs: 0x3171,
        keysym: 0x0ef0,
    },
    UcsKeysym {
        ucs: 0x3178,
        keysym: 0x0ef1,
    },
    UcsKeysym {
        ucs: 0x317f,
        keysym: 0x0ef2,
    },
    UcsKeysym {
        ucs: 0x3184,
        keysym: 0x0ef4,
    },
    UcsKeysym {
        ucs: 0x3186,
        keysym: 0x0ef5,
    },
    UcsKeysym {
        ucs: 0x318d,
        keysym: 0x0ef6,
    },
    UcsKeysym {
        ucs: 0x318e,
        keysym: 0x0ef7,
    },
];

pub fn ucs_to_keysym(ucs: u32) -> u32 {
    if ucs >= 0x0100 && ucs <= 0x318e {
        let mut lo = 0;
        let mut hi = UCS_KEYSYMS.len();

        while hi > lo {
            let pv = lo + (hi - lo) / 2;
            let entry = &UCS_KEYSYMS[pv];
            match (entry.ucs as u32).cmp(&ucs) {
                std::cmp::Ordering::Less => lo = pv + 1,
                std::cmp::Ordering::Greater => hi = pv,
                std::cmp::Ordering::Equal => return entry.keysym as u32,
            }
        }
    }
    0
}

pub fn map_keypad(keysym: u32) -> u32 {
    match keysym {
        KEY_KP_0 => KEY_0,
        KEY_KP_1 => KEY_1,
        KEY_KP_2 => KEY_2,
        KEY_KP_3 => KEY_3,
        KEY_KP_4 => KEY_4,
        KEY_KP_5 => KEY_5,
        KEY_KP_6 => KEY_6,
        KEY_KP_7 => KEY_7,
        KEY_KP_8 => KEY_8,
        KEY_KP_9 => KEY_9,
        KEY_KP_Decimal => KEY_period,
        KEY_KP_Separator => KEY_comma,
        KEY_KP_Divide => KEY_slash,
        KEY_KP_Multiply => KEY_asterisk,
        KEY_KP_Subtract => KEY_minus,
        KEY_KP_Add => KEY_plus,
        KEY_KP_Equal => KEY_equal,
        KEY_KP_Enter => KEY_Return,
        KEY_KP_Home => KEY_Home,
        KEY_KP_Insert | KEY_KP_Delete => KEY_Delete,
        KEY_KP_End => KEY_End,
        KEY_KP_Prior => KEY_Prior,
        KEY_KP_Next => KEY_Next,
        KEY_KP_Left => KEY_Left,
        KEY_KP_Right => KEY_Right,
        KEY_KP_Up => KEY_Up,
        KEY_KP_Down => KEY_Down,
        KEY_KP_Begin => KEY_Begin,
        KEY_KP_F1 => KEY_F1,
        KEY_KP_F2 => KEY_F2,
        KEY_KP_F3 => KEY_F3,
        KEY_KP_F4 => KEY_F4,
        KEY_KP_Tab => KEY_Tab,
        KEY_KP_Space => KEY_space,
        _ => keysym,
    }
}

#[cfg(test)]
#[path = "keysyms_tests.rs"]
mod tests;
